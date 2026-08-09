//! Source-backed CSS parser context contracts (#166).
//!
//! Establishes the parser-domain vocabulary required before any nested
//! qualified-rule producer execution (#167): context identity, parent
//! relationships, direct-item ordering, and honest termination. This module
//! defines contracts only; the current producer never allocates a
//! [`CssParserContextId`] or constructs a [`CssParserContextRecord`], and
//! [`super::result::CssParserRunResult::context_records`] is always empty for
//! every existing execution.
//!
//! # Evidence table, not an AST
//!
//! A [`CssParserContextRecord`] is one row of retained evidence, not a
//! recursive tree node: it owns no child vector. The stylesheet root remains
//! implicit and is never given a [`CssParserContextId`] or a fabricated
//! source anchor; a context with `parent = None` is a direct child of that
//! implicit root. Parent/child relationships are read by scanning the
//! retained table (parent ID lookup, source-containment check), never by
//! re-deriving structure from raw source. Every record — including a
//! top-level one with `parent = None` — carries its own
//! [`CssParserDirectItemOrdinal`], scoped to the real parent when present or
//! to the implicit root otherwise; the root is a genuine ordinal scope, not
//! an unordered special case.
//!
//! # Run-local identity
//!
//! [`CssParserContextId`] is scoped to exactly one parser run: allocation is
//! monotonic from zero, contiguous, and never reused, so a valid result's
//! `context_records()[i].id() == CssParserContextId(i)`. This is parser-run
//! analysis identity, never a persistent, global, or authored-source
//! identity.
//!
//! # Derived extent
//!
//! A record does not retain a redundant "complete" anchor duplicating
//! `header.start()` through the termination endpoint: [`Self::extent_start`]
//! and [`Self::extent_end`] derive the authored/partial extent from already-
//! retained evidence without searching source.
//!
//! # Partial contexts are honest evidence
//!
//! [`CssParserContextTermination`] never fabricates a closing `}`. A context
//! still active when the parser run stops is retained with
//! [`CssParserContextTermination::EndOfInput`],
//! [`CssParserContextTermination::UpstreamTokenizerIncomplete`], or
//! [`CssParserContextTermination::ParserResourceLimit`] evidence instead, per
//! whichever boundary actually stopped it; [`super::result`] reconciles that
//! evidence against the run's own lifecycle.
//!
//! # Deferred to later Leaves
//!
//! [`CssParserContextKind`] contains only `QualifiedRuleBlock`, the first
//! kind #167 needs. Group-rule (#168), descriptor (#169), page (#170), and
//! keyframe (#171) context meanings are deliberately absent rather than
//! pre-populated speculatively. Context nesting depth
//! ([`super::resource::CssParserResourceKind::PeakContextDepth`]) is tracked
//! independently of component-value nesting
//! ([`super::resource::CssParserResourceKind::PeakComponentDepth`]), and
//! [`super::resource::MAX_ACTIVE_SPECULATIVE_CHECKPOINT_DEPTH`] remains
//! exactly one: no #166 change widens the checkpoint model.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

/// A parser-run-local context identity.
///
/// Scoped to exactly one parser run, allocated monotonically from zero,
/// contiguous, and never reused: for a valid result, the record at vector
/// index `i` has `id() == CssParserContextId(i)`. Never a UUID, source hash,
/// or persistent/public identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssParserContextId(usize);

impl CssParserContextId {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

/// The finite, deliberately minimal production context-kind vocabulary for
/// the #166 contract foundation.
///
/// Only the first kind #167 needs is present. Later approved Leaves extend
/// this enum explicitly (#168 group-rule kinds, #169 descriptor contexts,
/// #170 page/page-margin contexts, #171 keyframe contexts); this Leaf does
/// not pre-populate them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserContextKind {
    QualifiedRuleBlock,
}

/// The position of a materialized direct block-content item within its
/// scope: the explicit parent context when `parent = Some(_)`, or the
/// implicit stylesheet root when `parent = None`.
///
/// Represents the ordinal among materialized structural block-content items,
/// never a raw lexical-token index or byte offset. Gaps are permitted:
/// declarations and other direct items occupy some ordinals without a
/// retained context record itself needing every intervening value. Every
/// retained context carries exactly one such ordinal, including top-level
/// contexts: the implicit stylesheet root is a real ordinal scope, not a
/// special case that goes unordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssParserDirectItemOrdinal(usize);

impl CssParserDirectItemOrdinal {
    pub(crate) const fn new(ordinal: usize) -> Self {
        Self(ordinal)
    }

    pub(crate) const fn value(self) -> usize {
        self.0
    }
}

/// The exhaustive, honest set of structural context-termination meanings.
///
/// Never fabricates an authored closing `}`: a context stopped by anything
/// other than an authored right curly retains empty-point terminal evidence
/// at the exact boundary where the run actually stopped. The
/// [`Self::ParserResourceLimit`] variant does not duplicate the detailed
/// resource kind/limit/attempted evidence already owned by
/// [`super::result::CssParserTermination::ParserResourceLimit`]; it retains
/// only the source-backed terminal boundary needed to reconcile this
/// context's own extent.
#[derive(Clone)]
pub(crate) enum CssParserContextTermination {
    AuthoredRightCurly { right_curly: SourceAnchor },
    EndOfInput { terminal: SourceAnchor },
    UpstreamTokenizerIncomplete { terminal: SourceAnchor },
    ParserResourceLimit { terminal: SourceAnchor },
}

impl CssParserContextTermination {
    /// The exclusive end offset of this context's authored/partial extent,
    /// derived from already-retained termination evidence without searching
    /// source.
    pub(crate) fn extent_end(&self) -> usize {
        match self {
            Self::AuthoredRightCurly { right_curly } => right_curly.range().end(),
            Self::EndOfInput { terminal }
            | Self::UpstreamTokenizerIncomplete { terminal }
            | Self::ParserResourceLimit { terminal } => terminal.range().end(),
        }
    }
}

impl PartialEq for CssParserContextTermination {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::AuthoredRightCurly { right_curly: left },
                Self::AuthoredRightCurly { right_curly: right },
            ) => same_anchor(left, right),
            (Self::EndOfInput { terminal: left }, Self::EndOfInput { terminal: right }) => {
                same_anchor(left, right)
            }
            (
                Self::UpstreamTokenizerIncomplete { terminal: left },
                Self::UpstreamTokenizerIncomplete { terminal: right },
            ) => same_anchor(left, right),
            (
                Self::ParserResourceLimit { terminal: left },
                Self::ParserResourceLimit { terminal: right },
            ) => same_anchor(left, right),
            _ => false,
        }
    }
}

impl Eq for CssParserContextTermination {}

impl fmt::Debug for CssParserContextTermination {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthoredRightCurly { right_curly } => formatter
                .debug_struct("AuthoredRightCurly")
                .field("source_id", &right_curly.source_id())
                .field("right_curly", &right_curly.range())
                .finish(),
            Self::EndOfInput { terminal } => formatter
                .debug_struct("EndOfInput")
                .field("source_id", &terminal.source_id())
                .field("terminal", &terminal.range())
                .finish(),
            Self::UpstreamTokenizerIncomplete { terminal } => formatter
                .debug_struct("UpstreamTokenizerIncomplete")
                .field("source_id", &terminal.source_id())
                .field("terminal", &terminal.range())
                .finish(),
            Self::ParserResourceLimit { terminal } => formatter
                .debug_struct("ParserResourceLimit")
                .field("source_id", &terminal.source_id())
                .field("terminal", &terminal.range())
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserContextEvidenceRole {
    Header,
    BlockOpener,
    Body,
    AuthoredRightCurly,
    EndOfInputTerminal,
    UpstreamIncompleteTerminal,
    ParserResourceLimitTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssParserContextContractError {
    SourceIdentityMismatch {
        role: CssParserContextEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    EmptyEvidence {
        role: CssParserContextEvidenceRole,
    },
    FixedSpellingMismatch {
        role: CssParserContextEvidenceRole,
        expected: &'static str,
    },
    /// `header.end()` must equal `block_opener.start()`.
    HeaderOpenerBoundaryMismatch,
    /// `body.start()` must equal `block_opener.end()`.
    BodyOpenerBoundaryMismatch,
    /// An authored `right_curly` must start exactly at `body.end()`.
    AuthoredRightCurlyBoundaryMismatch,
    /// A partial/EOF terminal must be an empty point anchor, never authored
    /// content.
    TerminalMustBeEmpty {
        role: CssParserContextEvidenceRole,
    },
    /// A partial/EOF terminal must start exactly at `body.end()`.
    TerminalBoundaryMismatch,
    /// An `EndOfInput` terminal must sit at the retained source's true end.
    TerminalNotAtSourceEnd,
}

impl fmt::Display for CssParserContextContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS parser context contract violation: {self:?}")
    }
}

impl Error for CssParserContextContractError {}

/// One retained context's source-backed evidence: identity, parent id,
/// parent-local (or implicit-root-local) direct-item ordinal, kind,
/// header/opener/body anchors, and honest termination.
///
/// Evidence, not an AST node: owns no child vector and no decoded
/// selector/prelude string. Every record carries exactly one
/// [`CssParserDirectItemOrdinal`], scoped to `parent` when `Some`, or to the
/// implicit stylesheet root when `parent` is `None`; the root is never given
/// a [`CssParserContextId`] or a fabricated source anchor, but it is still a
/// real ordinal scope. The constructor validates only relationships
/// available without another record (anchor source identity, exact `{`/`}`
/// spelling, header/opener/body/termination ordering); parent existence,
/// parent-ID ordering, sibling-ordinal uniqueness/order (for both real
/// parents and the implicit root), and source containment against another
/// record are [`super::result::CssParserRunResult`]'s responsibility.
#[derive(Clone)]
pub(crate) struct CssParserContextRecord {
    id: CssParserContextId,
    parent: Option<CssParserContextId>,
    item_ordinal: CssParserDirectItemOrdinal,
    kind: CssParserContextKind,
    header: SourceAnchor,
    block_opener: SourceAnchor,
    body: SourceAnchor,
    termination: CssParserContextTermination,
}

impl CssParserContextRecord {
    /// `header` is the exact retained raw interval from the authored
    /// construct start up to, but excluding, `block_opener`; it may be empty
    /// where CSS Syntax permits an empty qualified-rule prelude. `body` is
    /// the exact retained raw interval after `block_opener` and before the
    /// termination boundary; it never includes a closing `}`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        id: CssParserContextId,
        parent: Option<CssParserContextId>,
        item_ordinal: CssParserDirectItemOrdinal,
        kind: CssParserContextKind,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        let expected = source_text.id();
        require_source(expected, &header, CssParserContextEvidenceRole::Header)?;
        require_source(
            expected,
            &block_opener,
            CssParserContextEvidenceRole::BlockOpener,
        )?;
        require_source(expected, &body, CssParserContextEvidenceRole::Body)?;

        non_empty(&block_opener, CssParserContextEvidenceRole::BlockOpener)?;
        exact(
            &block_opener,
            CssParserContextEvidenceRole::BlockOpener,
            "{",
        )?;

        if header.range().end() != block_opener.range().start() {
            return Err(CssParserContextContractError::HeaderOpenerBoundaryMismatch);
        }
        if body.range().start() != block_opener.range().end() {
            return Err(CssParserContextContractError::BodyOpenerBoundaryMismatch);
        }

        validate_termination(source_text, &body, &termination)?;

        Ok(Self {
            id,
            parent,
            item_ordinal,
            kind,
            header,
            block_opener,
            body,
            termination,
        })
    }

    pub(crate) const fn id(&self) -> CssParserContextId {
        self.id
    }

    pub(crate) const fn parent(&self) -> Option<CssParserContextId> {
        self.parent
    }

    pub(crate) const fn item_ordinal(&self) -> CssParserDirectItemOrdinal {
        self.item_ordinal
    }

    pub(crate) const fn kind(&self) -> CssParserContextKind {
        self.kind
    }

    pub(crate) const fn header(&self) -> &SourceAnchor {
        &self.header
    }

    pub(crate) const fn block_opener(&self) -> &SourceAnchor {
        &self.block_opener
    }

    pub(crate) const fn body(&self) -> &SourceAnchor {
        &self.body
    }

    pub(crate) const fn termination(&self) -> &CssParserContextTermination {
        &self.termination
    }

    /// The authored/partial extent's start offset, derived from already-
    /// retained evidence: the construct start whether or not `header` is
    /// empty.
    pub(crate) fn extent_start(&self) -> usize {
        self.header.range().start()
    }

    /// The authored/partial extent's end offset, derived from already-
    /// retained termination evidence without searching source.
    pub(crate) fn extent_end(&self) -> usize {
        self.termination.extent_end()
    }
}

impl PartialEq for CssParserContextRecord {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.item_ordinal == other.item_ordinal
            && self.kind == other.kind
            && same_anchor(&self.header, &other.header)
            && same_anchor(&self.block_opener, &other.block_opener)
            && same_anchor(&self.body, &other.body)
            && self.termination == other.termination
    }
}

impl Eq for CssParserContextRecord {}

impl fmt::Debug for CssParserContextRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssParserContextRecord")
            .field("id", &self.id)
            .field("parent", &self.parent)
            .field("item_ordinal", &self.item_ordinal)
            .field("kind", &self.kind)
            .field("source_id", &self.header.source_id())
            .field("header", &self.header.range())
            .field("block_opener", &self.block_opener.range())
            .field("body", &self.body.range())
            .field("termination", &self.termination)
            .finish()
    }
}

fn validate_termination(
    source_text: &SourceText,
    body: &SourceAnchor,
    termination: &CssParserContextTermination,
) -> Result<(), CssParserContextContractError> {
    let expected = source_text.id();
    match termination {
        CssParserContextTermination::AuthoredRightCurly { right_curly } => {
            require_source(
                expected,
                right_curly,
                CssParserContextEvidenceRole::AuthoredRightCurly,
            )?;
            non_empty(
                right_curly,
                CssParserContextEvidenceRole::AuthoredRightCurly,
            )?;
            exact(
                right_curly,
                CssParserContextEvidenceRole::AuthoredRightCurly,
                "}",
            )?;
            if right_curly.range().start() != body.range().end() {
                return Err(CssParserContextContractError::AuthoredRightCurlyBoundaryMismatch);
            }
        }
        CssParserContextTermination::EndOfInput { terminal } => {
            require_source(
                expected,
                terminal,
                CssParserContextEvidenceRole::EndOfInputTerminal,
            )?;
            must_be_empty(terminal, CssParserContextEvidenceRole::EndOfInputTerminal)?;
            if terminal.range().start() != body.range().end() {
                return Err(CssParserContextContractError::TerminalBoundaryMismatch);
            }
            if terminal.range().start() != source_text.as_str().len() {
                return Err(CssParserContextContractError::TerminalNotAtSourceEnd);
            }
        }
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            require_source(
                expected,
                terminal,
                CssParserContextEvidenceRole::UpstreamIncompleteTerminal,
            )?;
            must_be_empty(
                terminal,
                CssParserContextEvidenceRole::UpstreamIncompleteTerminal,
            )?;
            if terminal.range().start() != body.range().end() {
                return Err(CssParserContextContractError::TerminalBoundaryMismatch);
            }
        }
        CssParserContextTermination::ParserResourceLimit { terminal } => {
            require_source(
                expected,
                terminal,
                CssParserContextEvidenceRole::ParserResourceLimitTerminal,
            )?;
            must_be_empty(
                terminal,
                CssParserContextEvidenceRole::ParserResourceLimitTerminal,
            )?;
            if terminal.range().start() != body.range().end() {
                return Err(CssParserContextContractError::TerminalBoundaryMismatch);
            }
        }
    }
    Ok(())
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssParserContextEvidenceRole,
) -> Result<(), CssParserContextContractError> {
    let actual = anchor.source_id();
    if actual != expected {
        return Err(CssParserContextContractError::SourceIdentityMismatch {
            role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn non_empty(
    anchor: &SourceAnchor,
    role: CssParserContextEvidenceRole,
) -> Result<(), CssParserContextContractError> {
    if anchor.range().is_empty() {
        return Err(CssParserContextContractError::EmptyEvidence { role });
    }
    Ok(())
}

fn must_be_empty(
    anchor: &SourceAnchor,
    role: CssParserContextEvidenceRole,
) -> Result<(), CssParserContextContractError> {
    if !anchor.range().is_empty() {
        return Err(CssParserContextContractError::TerminalMustBeEmpty { role });
    }
    Ok(())
}

fn exact(
    anchor: &SourceAnchor,
    role: CssParserContextEvidenceRole,
    expected: &'static str,
) -> Result<(), CssParserContextContractError> {
    if anchor.fragment() != expected {
        return Err(CssParserContextContractError::FixedSpellingMismatch { role, expected });
    }
    Ok(())
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    fn qualified_rule_block(
        source_text: &SourceText,
        header: (usize, usize),
        block_opener: (usize, usize),
        body: (usize, usize),
        termination: CssParserContextTermination,
    ) -> Result<CssParserContextRecord, CssParserContextContractError> {
        CssParserContextRecord::new(
            source_text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            source_text.anchor(header.0, header.1).unwrap(),
            source_text.anchor(block_opener.0, block_opener.1).unwrap(),
            source_text.anchor(body.0, body.1).unwrap(),
            termination,
        )
    }

    #[test]
    fn valid_authored_right_curly_context_constructs() {
        let text = source(1, "a{color:red;}");
        let record = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 12),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(12, 13).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(record.extent_start(), 0);
        assert_eq!(record.extent_end(), 13);
        assert_eq!(record.id(), CssParserContextId::new(0));
        assert!(record.parent().is_none());
    }

    #[test]
    fn valid_true_eof_context_constructs() {
        let text = source(2, "a{color:red");
        let record = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 11),
            CssParserContextTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(record.extent_end(), 11);
    }

    #[test]
    fn valid_upstream_incomplete_context_constructs_without_requiring_source_end() {
        // The upstream-incomplete terminal sits well before the retained
        // source's true end, unlike `EndOfInput`.
        let text = source(3, "a{color:red;background:blue;}");
        let record = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 12),
            CssParserContextTermination::UpstreamTokenizerIncomplete {
                terminal: text.anchor(12, 12).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(record.extent_end(), 12);
    }

    #[test]
    fn valid_parser_resource_context_constructs() {
        let text = source(4, "a{color:red;background:blue;}");
        let record = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 12),
            CssParserContextTermination::ParserResourceLimit {
                terminal: text.anchor(12, 12).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(record.extent_end(), 12);
    }

    #[test]
    fn foreign_source_id_is_rejected() {
        let text = source(5, "a{color:red;}");
        let other = source(6, "a{color:red;}");
        let result = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            other.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 12).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(12, 13).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserContextContractError::SourceIdentityMismatch {
                role: CssParserContextEvidenceRole::Header,
                ..
            })
        ));
    }

    #[test]
    fn invalid_opener_spelling_is_rejected() {
        let text = source(7, "a[color:red;]");
        let result = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 12),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(12, 13).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::FixedSpellingMismatch {
                role: CssParserContextEvidenceRole::BlockOpener,
                expected: "{",
            })
        );
    }

    #[test]
    fn invalid_authored_closer_spelling_is_rejected() {
        let text = source(8, "a{color:red;]");
        let result = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 12),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(12, 13).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::FixedSpellingMismatch {
                role: CssParserContextEvidenceRole::AuthoredRightCurly,
                expected: "}",
            })
        );
    }

    #[test]
    fn body_opener_boundary_mismatch_is_rejected() {
        let text = source(9, "a{ color:red;}");
        let result = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (3, 13),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(13, 14).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::BodyOpenerBoundaryMismatch)
        );
    }

    #[test]
    fn body_termination_boundary_mismatch_is_rejected() {
        let text = source(10, "a{color:red;}");
        let result = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 11),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(12, 13).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::AuthoredRightCurlyBoundaryMismatch)
        );
    }

    #[test]
    fn non_empty_partial_terminal_is_rejected() {
        let text = source(11, "a{color:red");
        let result = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 10),
            CssParserContextTermination::EndOfInput {
                terminal: text.anchor(10, 11).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::TerminalMustBeEmpty {
                role: CssParserContextEvidenceRole::EndOfInputTerminal,
            })
        );
    }

    #[test]
    fn end_of_input_terminal_not_at_source_end_is_rejected() {
        let text = source(12, "a{color:red};");
        let result = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 11),
            CssParserContextTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::TerminalNotAtSourceEnd)
        );
    }

    #[test]
    fn header_empty_prelude_is_accepted() {
        let text = source(13, "{color:red;}");
        let record = qualified_rule_block(
            &text,
            (0, 0),
            (0, 1),
            (1, 11),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(11, 12).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(record.extent_start(), 0);
        assert!(record.header().range().is_empty());
    }

    #[test]
    fn parent_id_and_item_ordinal_are_retained_verbatim() {
        let text = source(14, "a{b{color:red;}}");
        let parent_id = CssParserContextId::new(0);
        let item_ordinal = CssParserDirectItemOrdinal::new(0);
        let record = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            Some(parent_id),
            item_ordinal,
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 14).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(record.parent(), Some(parent_id));
        assert_eq!(record.item_ordinal(), item_ordinal);
    }

    #[test]
    fn top_level_context_carries_its_own_root_scoped_item_ordinal() {
        let text = source(140, "a{x}");
        let record = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(3),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(3, 4).unwrap(),
            },
        )
        .unwrap();
        assert!(record.parent().is_none());
        assert_eq!(record.item_ordinal(), CssParserDirectItemOrdinal::new(3));
    }

    #[test]
    fn debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "secret-context-body-content";
        let text = source(15, &format!("a{{{SECRET}}}"));
        let record = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 2 + SECRET.len()),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(2 + SECRET.len(), 3 + SECRET.len()).unwrap(),
            },
        )
        .unwrap();
        let debug = format!("{record:?}");
        assert!(!debug.contains(SECRET));
        let termination_debug = format!("{:?}", record.termination());
        assert!(!termination_debug.contains(SECRET));
    }

    #[test]
    fn context_id_ordering_supports_parent_before_child_checks() {
        assert!(CssParserContextId::new(0) < CssParserContextId::new(1));
        assert_eq!(CssParserContextId::new(2).index(), 2);
    }
}
