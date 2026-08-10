//! Source-backed CSS parser context contracts (#166).
//!
//! Establishes the parser-domain vocabulary required before any nested
//! qualified-rule producer execution (#167): context identity, parent
//! relationships, direct-item ordering, and honest termination. #166
//! established these contracts while the producer still allocated zero
//! [`CssParserContextId`]s and constructed zero [`CssParserContextRecord`]s;
//! #167 now produces real `QualifiedRuleBlock` context records for structurally
//! recognized nested qualified rules, so
//! [`super::result::CssParserRunResult::context_records`] is non-empty
//! whenever a run retains at least one such context.
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
//! [`CssParserContextKind`] now also carries `GroupRuleBlock`, the finite
//! nested `@media`/`@supports`/`@container`/`@layer`/`@scope` registry
//! approved for #168, and `DescriptorRuleBlock`, the finite stylesheet-root-
//! only `@font-face`/`@property` registry approved for #169; page (#170) and
//! keyframe (#171) context meanings remain deliberately absent rather than
//! pre-populated speculatively. Context nesting depth
//! ([`super::resource::CssParserResourceKind::PeakContextDepth`]) is tracked
//! independently of component-value nesting
//! ([`super::resource::CssParserResourceKind::PeakComponentDepth`]), and
//! [`super::resource::MAX_ACTIVE_SPECULATIVE_CHECKPOINT_DEPTH`] remains
//! exactly one: no #166/#168 change widens the checkpoint model.
//!
//! # Nearest qualified ancestry (#141/#168)
//!
//! Each record also retains [`Self::nearest_qualified_ancestor`]: the
//! nearest ancestor `QualifiedRuleBlock` context id, or `None` at the
//! implicit stylesheet root's direct children. A `GroupRuleBlock` context
//! interposes structurally without itself counting as a qualified ancestor;
//! [`super::result`] proves this value from the retained parent table rather
//! than trusting producer-provided ancestry.

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

/// The finite, deliberately minimal production context-kind vocabulary.
///
/// #166/#167 needed only `QualifiedRuleBlock`. #168 adds the finite nested
/// group-rule registry approved by #141: exactly `@media`, `@supports`,
/// `@container`, `@layer`, and `@scope`, never a generic at-rule/plugin
/// vocabulary. #169 adds the finite stylesheet-root-only descriptor-rule
/// registry: exactly `@font-face` and `@property`, never a generic
/// descriptor-block flag/plugin vocabulary. Later approved Leaves extend this
/// enum explicitly (#170 page/page-margin contexts, #171 keyframe contexts);
/// this Leaf does not pre-populate them.
///
/// The shared `RuleBlock` suffix is deliberate, load-bearing naming (every
/// variant is a kind of retained rule-block context); stripping it per
/// `clippy::enum_variant_names` would fight the architecture's own
/// vocabulary rather than clarify it, so the lint is suppressed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CssParserContextKind {
    QualifiedRuleBlock,
    GroupRuleBlock(CssParserGroupRuleKind),
    DescriptorRuleBlock(CssParserDescriptorRuleKind),
    /// The root-owned `@page` context (#170). CSSWG #11271 means Page must
    /// not introduce a new stable declaration-run semantic contract; see
    /// [`super::declaration::CssPageDeclarationPlacement`].
    PageRuleBlock,
    /// A page-margin-at-rule context nested only inside a retained
    /// `PageRuleBlock` (#170).
    PageMarginRuleBlock(CssParserPageMarginRuleKind),
    /// A qualified `@keyframes` rule context (#171).
    KeyframesRuleBlock,
    /// A bounded keyframe-selector block nested only inside a retained
    /// `KeyframesRuleBlock` (#171).
    KeyframeBlock,
}

/// The finite #168 nested group-rule registry approved by #141.
///
/// Registry membership alone is necessary but never sufficient to establish
/// a [`CssParserContextKind::GroupRuleBlock`]: the producer additionally
/// requires the bounded per-kind prelude subset recorded in the approved
/// architecture before a context of this kind is ever entered. No
/// `AnyAtRule`/string-keyed/plugin registry is introduced; a name outside
/// this finite set can never become a group-rule kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserGroupRuleKind {
    Media,
    Supports,
    Container,
    Layer,
    Scope,
}

impl CssParserGroupRuleKind {
    /// Matches a tokenizer-decoded `AtKeyword` value (without the leading
    /// `@`) against the finite registry, ASCII-case-insensitively. Returns
    /// `None` for any name outside the five approved kinds -- including any
    /// future/unknown at-rule name, which remains explicit unsupported
    /// evidence rather than a speculative registry member.
    pub(crate) fn from_decoded_at_keyword(decoded: &str) -> Option<Self> {
        if decoded.eq_ignore_ascii_case("media") {
            Some(Self::Media)
        } else if decoded.eq_ignore_ascii_case("supports") {
            Some(Self::Supports)
        } else if decoded.eq_ignore_ascii_case("container") {
            Some(Self::Container)
        } else if decoded.eq_ignore_ascii_case("layer") {
            Some(Self::Layer)
        } else if decoded.eq_ignore_ascii_case("scope") {
            Some(Self::Scope)
        } else {
            None
        }
    }
}

/// The finite #169 stylesheet-root-only descriptor-rule registry approved by
/// #141: exactly `@font-face` and `@property`, never a generic
/// descriptor-block flag/plugin/framework. Registry membership alone is
/// necessary but never sufficient to establish a
/// [`CssParserContextKind::DescriptorRuleBlock`]: the producer additionally
/// requires the bounded per-kind parent-qualification subset (an empty
/// `@font-face` prelude, or a single `<custom-property-name>` `@property`
/// prelude) before a context of this kind is ever entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserDescriptorRuleKind {
    FontFace,
    Property,
}

impl CssParserDescriptorRuleKind {
    /// Matches a tokenizer-decoded `AtKeyword` value (without the leading
    /// `@`) against the finite registry, ASCII-case-insensitively. Returns
    /// `None` for any name outside the two approved kinds, which remains
    /// explicit unsupported evidence rather than a speculative registry
    /// member.
    pub(crate) fn from_decoded_at_keyword(decoded: &str) -> Option<Self> {
        if decoded.eq_ignore_ascii_case("font-face") {
            Some(Self::FontFace)
        } else if decoded.eq_ignore_ascii_case("property") {
            Some(Self::Property)
        } else {
            None
        }
    }
}

/// The finite #170 page-margin-rule registry: exactly the sixteen CSS Paged
/// Media margin-box names, never a generic at-rule/plugin vocabulary.
/// Registry membership alone is necessary but never sufficient to establish
/// a [`CssParserContextKind::PageMarginRuleBlock`]: the producer additionally
/// requires a semantically empty prelude and a real `PageRuleBlock` parent
/// before a context of this kind is ever entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserPageMarginRuleKind {
    TopLeftCorner,
    TopLeft,
    TopCenter,
    TopRight,
    TopRightCorner,
    BottomLeftCorner,
    BottomLeft,
    BottomCenter,
    BottomRight,
    BottomRightCorner,
    LeftTop,
    LeftMiddle,
    LeftBottom,
    RightTop,
    RightMiddle,
    RightBottom,
}

impl CssParserPageMarginRuleKind {
    /// Matches a tokenizer-decoded `AtKeyword` value (without the leading
    /// `@`) against the finite sixteen-member registry, ASCII-case-
    /// insensitively. Returns `None` for any name outside that set, which
    /// remains explicit unsupported evidence rather than a speculative
    /// registry member.
    pub(crate) fn from_decoded_at_keyword(decoded: &str) -> Option<Self> {
        if decoded.eq_ignore_ascii_case("top-left-corner") {
            Some(Self::TopLeftCorner)
        } else if decoded.eq_ignore_ascii_case("top-left") {
            Some(Self::TopLeft)
        } else if decoded.eq_ignore_ascii_case("top-center") {
            Some(Self::TopCenter)
        } else if decoded.eq_ignore_ascii_case("top-right") {
            Some(Self::TopRight)
        } else if decoded.eq_ignore_ascii_case("top-right-corner") {
            Some(Self::TopRightCorner)
        } else if decoded.eq_ignore_ascii_case("bottom-left-corner") {
            Some(Self::BottomLeftCorner)
        } else if decoded.eq_ignore_ascii_case("bottom-left") {
            Some(Self::BottomLeft)
        } else if decoded.eq_ignore_ascii_case("bottom-center") {
            Some(Self::BottomCenter)
        } else if decoded.eq_ignore_ascii_case("bottom-right") {
            Some(Self::BottomRight)
        } else if decoded.eq_ignore_ascii_case("bottom-right-corner") {
            Some(Self::BottomRightCorner)
        } else if decoded.eq_ignore_ascii_case("left-top") {
            Some(Self::LeftTop)
        } else if decoded.eq_ignore_ascii_case("left-middle") {
            Some(Self::LeftMiddle)
        } else if decoded.eq_ignore_ascii_case("left-bottom") {
            Some(Self::LeftBottom)
        } else if decoded.eq_ignore_ascii_case("right-top") {
            Some(Self::RightTop)
        } else if decoded.eq_ignore_ascii_case("right-middle") {
            Some(Self::RightMiddle)
        } else if decoded.eq_ignore_ascii_case("right-bottom") {
            Some(Self::RightBottom)
        } else {
            None
        }
    }
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
    /// A `GroupRuleBlock`/`DescriptorRuleBlock` `at_keyword` must start with
    /// `@` (#168/#169: shared at-rule identity evidence, no longer
    /// group-specific).
    AtKeywordMissingSigil,
    /// An `at_keyword.start()` must equal `header.start()`.
    AtKeywordHeaderStartMismatch,
    /// An `at_keyword` must be contained within `header`.
    AtKeywordOutsideHeader,
    /// The decoded tokenizer `AtKeyword` value supplied at construction does
    /// not correspond to the declared [`CssParserGroupRuleKind`].
    GroupKindDecodedMismatch,
    /// The decoded tokenizer `AtKeyword` value supplied at construction does
    /// not correspond to the declared [`CssParserDescriptorRuleKind`] (#169).
    DescriptorKindDecodedMismatch,
    /// A `FontFace` descriptor context was constructed with custom-property-
    /// name evidence; `@font-face` has an empty semantic prelude and carries
    /// no such evidence (#169).
    FontFaceCarriesPropertyNameEvidence,
    /// A `Property` descriptor context was constructed without the required
    /// custom-property-name evidence (#169).
    PropertyMissingCustomPropertyNameEvidence,
    /// A `Property` descriptor context's decoded custom-property-name
    /// evidence does not satisfy the bounded `<custom-property-name>`
    /// subset: `starts_with("--")` and not exactly `"--"` (#169).
    PropertyNameNotCustomPropertyShaped,
    /// A `Property` descriptor context's custom-property-name anchor does
    /// not start after the at-keyword ends (#169).
    PropertyNameOutOfOrder,
    /// A `Property` descriptor context's custom-property-name anchor is not
    /// contained within `header` (#169).
    PropertyNameOutsideHeader,
    /// The decoded tokenizer `AtKeyword` value supplied at construction does
    /// not decode to `page` (#170).
    PageAtKeywordDecodedMismatch,
    /// The decoded tokenizer `AtKeyword` value supplied at construction does
    /// not correspond to the declared [`CssParserPageMarginRuleKind`] (#170).
    PageMarginKindDecodedMismatch,
    /// A `PageRuleBlock`'s `page_selector_list` anchor does not start after
    /// the at-keyword ends (#170).
    PageSelectorListOutOfOrder,
    /// A `PageRuleBlock`'s `page_selector_list` anchor is not contained
    /// within `header` (#170).
    PageSelectorListOutsideHeader,
    /// The decoded at-keyword for a `KeyframesRuleBlock` was not `keyframes`.
    KeyframesAtKeywordDecodedMismatch,
    /// The retained keyframes-name must follow the at-keyword.
    KeyframesNameOutOfOrder,
    /// The retained keyframes-name is not contained in the header.
    KeyframesNameOutsideHeader,
    /// A `KeyframeBlock` selector-list is not contained in its header.
    KeyframeSelectorListOutsideHeader,
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
    /// Shared source-backed at-rule identity evidence (#169 generalization
    /// of the #168 group-specific field): the exact authored `AtKeyword`
    /// anchor for a `GroupRuleBlock` or `DescriptorRuleBlock`; always `None`
    /// for a `QualifiedRuleBlock`. Not a generic at-rule semantic object --
    /// only source identity evidence. Structurally unrepresentable as a
    /// mismatch: the three dedicated constructors are the only way to build
    /// a record, and only
    /// [`Self::new_group_rule_block`]/[`Self::new_descriptor_rule_block`]
    /// can attach this evidence.
    at_keyword: Option<SourceAnchor>,
    /// The exact authored custom-property-name anchor for a
    /// `DescriptorRuleBlock(Property)`; always `None` otherwise, including
    /// for `DescriptorRuleBlock(FontFace)` (whose semantic prelude is empty)
    /// (#169).
    descriptor_property_name: Option<SourceAnchor>,
    /// The exact authored `<page-selector-list>` envelope for a
    /// `PageRuleBlock` with a non-empty prelude; `None` for a selector-less
    /// `PageRuleBlock` and always `None` for every other kind, including
    /// `PageMarginRuleBlock` (#170). Owns no selector-component vector or
    /// selector AST -- only the exact authored source range.
    page_selector_list: Option<SourceAnchor>,
    /// Exact authored `<keyframes-name>` token evidence for a
    /// `KeyframesRuleBlock`; `None` for every other context kind.
    keyframes_name: Option<SourceAnchor>,
    /// Exact authored `<keyframe-selector>#` envelope for a `KeyframeBlock`;
    /// no selector AST or normalized selector list is retained.
    keyframe_selector_list: Option<SourceAnchor>,
    /// The nearest ancestor `QualifiedRuleBlock` context, or `None` at the
    /// implicit stylesheet root's direct children (#141/#168). Structural
    /// qualified-rule ancestry only: never a selector-validity or
    /// style-rule-semantic claim.
    nearest_qualified_ancestor: Option<CssParserContextId>,
    header: SourceAnchor,
    block_opener: SourceAnchor,
    body: SourceAnchor,
    termination: CssParserContextTermination,
}

impl CssParserContextRecord {
    /// Constructs a `QualifiedRuleBlock` record. `header` is the exact
    /// retained raw interval from the authored construct start up to, but
    /// excluding, `block_opener`; it may be empty where CSS Syntax permits
    /// an empty qualified-rule prelude. `body` is the exact retained raw
    /// interval after `block_opener` and before the termination boundary; it
    /// never includes a closing `}`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_qualified_rule_block(
        source_text: &SourceText,
        id: CssParserContextId,
        parent: Option<CssParserContextId>,
        item_ordinal: CssParserDirectItemOrdinal,
        nearest_qualified_ancestor: Option<CssParserContextId>,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;

        Ok(Self {
            id,
            parent,
            item_ordinal,
            kind: CssParserContextKind::QualifiedRuleBlock,
            at_keyword: None,
            descriptor_property_name: None,
            page_selector_list: None,
            keyframes_name: None,
            keyframe_selector_list: None,
            nearest_qualified_ancestor,
            header,
            block_opener,
            body,
            termination,
        })
    }

    /// Constructs a `GroupRuleBlock` record (#168): a supported nested
    /// `@media`/`@supports`/`@container`/`@layer`/`@scope` context. `at_keyword`
    /// must be the exact authored at-keyword anchor starting `header`.
    /// `decoded_at_keyword` is the already-decoded tokenizer value (without
    /// the leading `@`) used only to prove `at_keyword` genuinely
    /// corresponds to `kind`; it is not retained.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_group_rule_block(
        source_text: &SourceText,
        id: CssParserContextId,
        parent: Option<CssParserContextId>,
        item_ordinal: CssParserDirectItemOrdinal,
        nearest_qualified_ancestor: Option<CssParserContextId>,
        kind: CssParserGroupRuleKind,
        at_keyword: SourceAnchor,
        decoded_at_keyword: &str,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;

        let expected = source_text.id();
        require_source(expected, &at_keyword, CssParserContextEvidenceRole::Header)?;
        non_empty(&at_keyword, CssParserContextEvidenceRole::Header)?;
        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserContextContractError::AtKeywordMissingSigil);
        }
        if at_keyword.range().start() != header.range().start() {
            return Err(CssParserContextContractError::AtKeywordHeaderStartMismatch);
        }
        if at_keyword.range().end() > header.range().end() {
            return Err(CssParserContextContractError::AtKeywordOutsideHeader);
        }
        if CssParserGroupRuleKind::from_decoded_at_keyword(decoded_at_keyword) != Some(kind) {
            return Err(CssParserContextContractError::GroupKindDecodedMismatch);
        }

        Ok(Self {
            id,
            parent,
            item_ordinal,
            kind: CssParserContextKind::GroupRuleBlock(kind),
            at_keyword: Some(at_keyword),
            descriptor_property_name: None,
            page_selector_list: None,
            keyframes_name: None,
            keyframe_selector_list: None,
            nearest_qualified_ancestor,
            header,
            block_opener,
            body,
            termination,
        })
    }

    /// Constructs a `DescriptorRuleBlock` record (#169): a supported
    /// stylesheet-root-only `@font-face`/`@property` descriptor context.
    /// `at_keyword` must be the exact authored at-keyword anchor starting
    /// `header`. `decoded_at_keyword` is the already-decoded tokenizer value
    /// (without the leading `@`) used only to prove `at_keyword` genuinely
    /// corresponds to `kind`; it is not retained.
    ///
    /// `property_name`/`decoded_property_name` must both be `Some` for
    /// `Property` (the exact authored custom-property-name anchor and its
    /// tokenizer-decoded value, used only to prove the bounded
    /// `<custom-property-name>` subset) and both `None` for `FontFace`,
    /// whose semantic prelude is empty; any other combination is rejected.
    ///
    /// Descriptor contexts are stylesheet-root-only in #169: `parent` and
    /// `nearest_qualified_ancestor` are always `None` and are not accepted as
    /// parameters, so this constructor cannot itself construct a nested
    /// descriptor context.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_descriptor_rule_block(
        source_text: &SourceText,
        id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
        kind: CssParserDescriptorRuleKind,
        at_keyword: SourceAnchor,
        decoded_at_keyword: &str,
        property_name: Option<SourceAnchor>,
        decoded_property_name: Option<&str>,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;

        let expected = source_text.id();
        require_source(expected, &at_keyword, CssParserContextEvidenceRole::Header)?;
        non_empty(&at_keyword, CssParserContextEvidenceRole::Header)?;
        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserContextContractError::AtKeywordMissingSigil);
        }
        if at_keyword.range().start() != header.range().start() {
            return Err(CssParserContextContractError::AtKeywordHeaderStartMismatch);
        }
        if at_keyword.range().end() > header.range().end() {
            return Err(CssParserContextContractError::AtKeywordOutsideHeader);
        }
        if CssParserDescriptorRuleKind::from_decoded_at_keyword(decoded_at_keyword) != Some(kind) {
            return Err(CssParserContextContractError::DescriptorKindDecodedMismatch);
        }

        match kind {
            CssParserDescriptorRuleKind::FontFace => {
                if property_name.is_some() || decoded_property_name.is_some() {
                    return Err(CssParserContextContractError::FontFaceCarriesPropertyNameEvidence);
                }
            }
            CssParserDescriptorRuleKind::Property => {
                let name_anchor = property_name.as_ref().ok_or(
                    CssParserContextContractError::PropertyMissingCustomPropertyNameEvidence,
                )?;
                let decoded_name = decoded_property_name.ok_or(
                    CssParserContextContractError::PropertyMissingCustomPropertyNameEvidence,
                )?;
                require_source(expected, name_anchor, CssParserContextEvidenceRole::Header)?;
                non_empty(name_anchor, CssParserContextEvidenceRole::Header)?;
                if name_anchor.range().start() < at_keyword.range().end() {
                    return Err(CssParserContextContractError::PropertyNameOutOfOrder);
                }
                if name_anchor.range().end() > header.range().end() {
                    return Err(CssParserContextContractError::PropertyNameOutsideHeader);
                }
                if !decoded_name.starts_with("--") || decoded_name == "--" {
                    return Err(CssParserContextContractError::PropertyNameNotCustomPropertyShaped);
                }
            }
        }

        Ok(Self {
            id,
            parent: None,
            item_ordinal,
            kind: CssParserContextKind::DescriptorRuleBlock(kind),
            at_keyword: Some(at_keyword),
            descriptor_property_name: property_name,
            page_selector_list: None,
            keyframes_name: None,
            keyframe_selector_list: None,
            nearest_qualified_ancestor: None,
            header,
            block_opener,
            body,
            termination,
        })
    }

    /// Constructs a `PageRuleBlock` record (#170): the root-owned `@page`
    /// context. `at_keyword` must be the exact authored at-keyword anchor
    /// starting `header`. `decoded_at_keyword` is the already-decoded
    /// tokenizer value (without the leading `@`) used only to prove
    /// `at_keyword` decodes to `page`; it is not retained.
    ///
    /// `page_selector_list` is `None` for a selector-less `@page` and
    /// `Some` with the exact authored `<page-selector-list>` envelope
    /// otherwise; it owns no selector-component vector or selector AST.
    ///
    /// `PageRuleBlock` is root-owned in #170: `parent` and
    /// `nearest_qualified_ancestor` are always `None` and are not accepted
    /// as parameters, mirroring [`Self::new_descriptor_rule_block`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_page_rule_block(
        source_text: &SourceText,
        id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
        at_keyword: SourceAnchor,
        decoded_at_keyword: &str,
        page_selector_list: Option<SourceAnchor>,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;

        let expected = source_text.id();
        require_source(expected, &at_keyword, CssParserContextEvidenceRole::Header)?;
        non_empty(&at_keyword, CssParserContextEvidenceRole::Header)?;
        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserContextContractError::AtKeywordMissingSigil);
        }
        if at_keyword.range().start() != header.range().start() {
            return Err(CssParserContextContractError::AtKeywordHeaderStartMismatch);
        }
        if at_keyword.range().end() > header.range().end() {
            return Err(CssParserContextContractError::AtKeywordOutsideHeader);
        }
        if !decoded_at_keyword.eq_ignore_ascii_case("page") {
            return Err(CssParserContextContractError::PageAtKeywordDecodedMismatch);
        }

        if let Some(selector_list) = &page_selector_list {
            require_source(
                expected,
                selector_list,
                CssParserContextEvidenceRole::Header,
            )?;
            non_empty(selector_list, CssParserContextEvidenceRole::Header)?;
            if selector_list.range().start() < at_keyword.range().end() {
                return Err(CssParserContextContractError::PageSelectorListOutOfOrder);
            }
            if selector_list.range().end() > header.range().end() {
                return Err(CssParserContextContractError::PageSelectorListOutsideHeader);
            }
        }

        Ok(Self {
            id,
            parent: None,
            item_ordinal,
            kind: CssParserContextKind::PageRuleBlock,
            at_keyword: Some(at_keyword),
            descriptor_property_name: None,
            page_selector_list,
            keyframes_name: None,
            keyframe_selector_list: None,
            nearest_qualified_ancestor: None,
            header,
            block_opener,
            body,
            termination,
        })
    }

    /// Constructs a `PageMarginRuleBlock` record (#170): a supported
    /// page-margin-rule context nested only inside a retained `PageRuleBlock`
    /// `parent`. `at_keyword` must be the exact authored at-keyword anchor
    /// starting `header`. `decoded_at_keyword` is the already-decoded
    /// tokenizer value (without the leading `@`) used only to prove
    /// `at_keyword` genuinely corresponds to `kind`; it is not retained.
    ///
    /// `nearest_qualified_ancestor` is always `None`, never inferred from
    /// `parent`'s own ancestry: a `PageRuleBlock` parent is never itself a
    /// `QualifiedRuleBlock`, and #170 does not infer Page semantic
    /// qualification from group-rule ancestry.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_page_margin_rule_block(
        source_text: &SourceText,
        id: CssParserContextId,
        parent: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
        kind: CssParserPageMarginRuleKind,
        at_keyword: SourceAnchor,
        decoded_at_keyword: &str,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;

        let expected = source_text.id();
        require_source(expected, &at_keyword, CssParserContextEvidenceRole::Header)?;
        non_empty(&at_keyword, CssParserContextEvidenceRole::Header)?;
        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserContextContractError::AtKeywordMissingSigil);
        }
        if at_keyword.range().start() != header.range().start() {
            return Err(CssParserContextContractError::AtKeywordHeaderStartMismatch);
        }
        if at_keyword.range().end() > header.range().end() {
            return Err(CssParserContextContractError::AtKeywordOutsideHeader);
        }
        if CssParserPageMarginRuleKind::from_decoded_at_keyword(decoded_at_keyword) != Some(kind) {
            return Err(CssParserContextContractError::PageMarginKindDecodedMismatch);
        }

        Ok(Self {
            id,
            parent: Some(parent),
            item_ordinal,
            kind: CssParserContextKind::PageMarginRuleBlock(kind),
            at_keyword: Some(at_keyword),
            descriptor_property_name: None,
            page_selector_list: None,
            keyframes_name: None,
            keyframe_selector_list: None,
            nearest_qualified_ancestor: None,
            header,
            block_opener,
            body,
            termination,
        })
    }

    /// Constructs a qualified `@keyframes` context (#171). The caller has
    /// already established the bounded `<keyframes-name>` grammar; this
    /// constructor independently protects exact source ownership and the
    /// at-keyword/name/header relationship. Parent qualification is validated
    /// at result level from the retained context graph.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_keyframes_rule_block(
        source_text: &SourceText,
        id: CssParserContextId,
        parent: Option<CssParserContextId>,
        item_ordinal: CssParserDirectItemOrdinal,
        at_keyword: SourceAnchor,
        decoded_at_keyword: &str,
        keyframes_name: SourceAnchor,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;
        let expected = source_text.id();
        require_source(expected, &at_keyword, CssParserContextEvidenceRole::Header)?;
        require_source(
            expected,
            &keyframes_name,
            CssParserContextEvidenceRole::Header,
        )?;
        non_empty(&at_keyword, CssParserContextEvidenceRole::Header)?;
        non_empty(&keyframes_name, CssParserContextEvidenceRole::Header)?;
        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserContextContractError::AtKeywordMissingSigil);
        }
        if at_keyword.range().start() != header.range().start() {
            return Err(CssParserContextContractError::AtKeywordHeaderStartMismatch);
        }
        if at_keyword.range().end() > header.range().end() {
            return Err(CssParserContextContractError::AtKeywordOutsideHeader);
        }
        if !decoded_at_keyword.eq_ignore_ascii_case("keyframes") {
            return Err(CssParserContextContractError::KeyframesAtKeywordDecodedMismatch);
        }
        if keyframes_name.range().start() < at_keyword.range().end() {
            return Err(CssParserContextContractError::KeyframesNameOutOfOrder);
        }
        if keyframes_name.range().end() > header.range().end() {
            return Err(CssParserContextContractError::KeyframesNameOutsideHeader);
        }
        Ok(Self {
            id,
            parent,
            item_ordinal,
            kind: CssParserContextKind::KeyframesRuleBlock,
            at_keyword: Some(at_keyword),
            descriptor_property_name: None,
            page_selector_list: None,
            keyframes_name: Some(keyframes_name),
            keyframe_selector_list: None,
            nearest_qualified_ancestor: None,
            header,
            block_opener,
            body,
            termination,
        })
    }

    /// Constructs a bounded keyframe-selector child block (#171). The
    /// selector-list anchor is the exact authored qualifying envelope; no
    /// selector components are retained or normalized.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_keyframe_block(
        source_text: &SourceText,
        id: CssParserContextId,
        parent: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
        keyframe_selector_list: SourceAnchor,
        header: SourceAnchor,
        block_opener: SourceAnchor,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<Self, CssParserContextContractError> {
        validate_shared_shape(source_text, &header, &block_opener, &body, &termination)?;
        let expected = source_text.id();
        require_source(
            expected,
            &keyframe_selector_list,
            CssParserContextEvidenceRole::Header,
        )?;
        non_empty(
            &keyframe_selector_list,
            CssParserContextEvidenceRole::Header,
        )?;
        if keyframe_selector_list.range().start() < header.range().start()
            || keyframe_selector_list.range().end() > header.range().end()
        {
            return Err(CssParserContextContractError::KeyframeSelectorListOutsideHeader);
        }
        Ok(Self {
            id,
            parent: Some(parent),
            item_ordinal,
            kind: CssParserContextKind::KeyframeBlock,
            at_keyword: None,
            descriptor_property_name: None,
            page_selector_list: None,
            keyframes_name: None,
            keyframe_selector_list: Some(keyframe_selector_list),
            nearest_qualified_ancestor: None,
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

    pub(crate) const fn at_keyword(&self) -> Option<&SourceAnchor> {
        self.at_keyword.as_ref()
    }

    /// The exact authored custom-property-name anchor for a
    /// `DescriptorRuleBlock(Property)` context; `None` for every other kind,
    /// including `DescriptorRuleBlock(FontFace)` (#169).
    pub(crate) const fn descriptor_property_name(&self) -> Option<&SourceAnchor> {
        self.descriptor_property_name.as_ref()
    }

    /// The exact authored `<page-selector-list>` envelope for a
    /// `PageRuleBlock` with a non-empty prelude; `None` for a selector-less
    /// `PageRuleBlock` and every other context kind (#170).
    pub(crate) const fn page_selector_list(&self) -> Option<&SourceAnchor> {
        self.page_selector_list.as_ref()
    }

    /// Exact authored `<keyframes-name>` evidence for a qualified outer
    /// keyframes rule, never a normalized animation name.
    pub(crate) const fn keyframes_name(&self) -> Option<&SourceAnchor> {
        self.keyframes_name.as_ref()
    }

    /// Exact authored selector-list envelope for a qualified keyframe block.
    pub(crate) const fn keyframe_selector_list(&self) -> Option<&SourceAnchor> {
        self.keyframe_selector_list.as_ref()
    }

    pub(crate) const fn nearest_qualified_ancestor(&self) -> Option<CssParserContextId> {
        self.nearest_qualified_ancestor
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

/// Test-only corruption-constructor support (#170 audit finding). No
/// production constructor can ever produce a `PageMarginRuleBlock` record
/// carrying `page_selector_list` evidence: only [`Self::new_page_rule_block`]
/// accepts a `page_selector_list` parameter, and it always sets
/// `kind = PageRuleBlock`. `result.rs`'s independent
/// `PageMarginContextCarriesSelectorList` result-level check exists as
/// defense-in-depth for exactly this otherwise-unreachable combination, so
/// proving it fires requires this narrow, `#[cfg(test)]`-gated escape hatch
/// rather than any production-visible constructor change.
#[cfg(test)]
impl CssParserContextRecord {
    pub(crate) fn new_test_only_page_margin_rule_block_with_selector_list(
        valid_page_margin_rule_block: Self,
        page_selector_list: SourceAnchor,
    ) -> Self {
        Self {
            page_selector_list: Some(page_selector_list),
            ..valid_page_margin_rule_block
        }
    }
}

impl PartialEq for CssParserContextRecord {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.item_ordinal == other.item_ordinal
            && self.kind == other.kind
            && match (&self.at_keyword, &other.at_keyword) {
                (Some(left), Some(right)) => same_anchor(left, right),
                (None, None) => true,
                _ => false,
            }
            && match (
                &self.descriptor_property_name,
                &other.descriptor_property_name,
            ) {
                (Some(left), Some(right)) => same_anchor(left, right),
                (None, None) => true,
                _ => false,
            }
            && match (&self.page_selector_list, &other.page_selector_list) {
                (Some(left), Some(right)) => same_anchor(left, right),
                (None, None) => true,
                _ => false,
            }
            && match (&self.keyframes_name, &other.keyframes_name) {
                (Some(left), Some(right)) => same_anchor(left, right),
                (None, None) => true,
                _ => false,
            }
            && match (&self.keyframe_selector_list, &other.keyframe_selector_list) {
                (Some(left), Some(right)) => same_anchor(left, right),
                (None, None) => true,
                _ => false,
            }
            && self.nearest_qualified_ancestor == other.nearest_qualified_ancestor
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
            .field(
                "at_keyword",
                &self.at_keyword.as_ref().map(SourceAnchor::range),
            )
            .field(
                "descriptor_property_name",
                &self
                    .descriptor_property_name
                    .as_ref()
                    .map(SourceAnchor::range),
            )
            .field(
                "page_selector_list",
                &self.page_selector_list.as_ref().map(SourceAnchor::range),
            )
            .field(
                "keyframes_name",
                &self.keyframes_name.as_ref().map(SourceAnchor::range),
            )
            .field(
                "keyframe_selector_list",
                &self
                    .keyframe_selector_list
                    .as_ref()
                    .map(SourceAnchor::range),
            )
            .field(
                "nearest_qualified_ancestor",
                &self.nearest_qualified_ancestor,
            )
            .field("header", &self.header.range())
            .field("block_opener", &self.block_opener.range())
            .field("body", &self.body.range())
            .field("termination", &self.termination)
            .finish()
    }
}

/// The shape/boundary validation shared by all three constructors,
/// independent of `at_keyword`/kind correspondence.
fn validate_shared_shape(
    source_text: &SourceText,
    header: &SourceAnchor,
    block_opener: &SourceAnchor,
    body: &SourceAnchor,
    termination: &CssParserContextTermination,
) -> Result<(), CssParserContextContractError> {
    let expected = source_text.id();
    require_source(expected, header, CssParserContextEvidenceRole::Header)?;
    require_source(
        expected,
        block_opener,
        CssParserContextEvidenceRole::BlockOpener,
    )?;
    require_source(expected, body, CssParserContextEvidenceRole::Body)?;

    non_empty(block_opener, CssParserContextEvidenceRole::BlockOpener)?;
    exact(block_opener, CssParserContextEvidenceRole::BlockOpener, "{")?;

    if header.range().end() != block_opener.range().start() {
        return Err(CssParserContextContractError::HeaderOpenerBoundaryMismatch);
    }
    if body.range().start() != block_opener.range().end() {
        return Err(CssParserContextContractError::BodyOpenerBoundaryMismatch);
    }

    validate_termination(source_text, body, termination)
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
        CssParserContextRecord::new_qualified_rule_block(
            source_text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            source_text.anchor(header.0, header.1).unwrap(),
            source_text.anchor(block_opener.0, block_opener.1).unwrap(),
            source_text.anchor(body.0, body.1).unwrap(),
            termination,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn descriptor_rule_block(
        source_text: &SourceText,
        kind: CssParserDescriptorRuleKind,
        at_keyword: (usize, usize),
        decoded_at_keyword: &str,
        property_name: Option<(usize, usize)>,
        decoded_property_name: Option<&str>,
        header: (usize, usize),
        block_opener: (usize, usize),
        body: (usize, usize),
        termination: CssParserContextTermination,
    ) -> Result<CssParserContextRecord, CssParserContextContractError> {
        CssParserContextRecord::new_descriptor_rule_block(
            source_text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            kind,
            source_text.anchor(at_keyword.0, at_keyword.1).unwrap(),
            decoded_at_keyword,
            property_name.map(|(start, end)| source_text.anchor(start, end).unwrap()),
            decoded_property_name,
            source_text.anchor(header.0, header.1).unwrap(),
            source_text.anchor(block_opener.0, block_opener.1).unwrap(),
            source_text.anchor(body.0, body.1).unwrap(),
            termination,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn group_rule_block(
        source_text: &SourceText,
        kind: CssParserGroupRuleKind,
        decoded_at_keyword: &str,
        at_keyword: (usize, usize),
        header: (usize, usize),
        block_opener: (usize, usize),
        body: (usize, usize),
        termination: CssParserContextTermination,
    ) -> Result<CssParserContextRecord, CssParserContextContractError> {
        CssParserContextRecord::new_group_rule_block(
            source_text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            kind,
            source_text.anchor(at_keyword.0, at_keyword.1).unwrap(),
            decoded_at_keyword,
            source_text.anchor(header.0, header.1).unwrap(),
            source_text.anchor(block_opener.0, block_opener.1).unwrap(),
            source_text.anchor(body.0, body.1).unwrap(),
            termination,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn page_rule_block(
        source_text: &SourceText,
        at_keyword: (usize, usize),
        decoded_at_keyword: &str,
        page_selector_list: Option<(usize, usize)>,
        header: (usize, usize),
        block_opener: (usize, usize),
        body: (usize, usize),
        termination: CssParserContextTermination,
    ) -> Result<CssParserContextRecord, CssParserContextContractError> {
        CssParserContextRecord::new_page_rule_block(
            source_text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            source_text.anchor(at_keyword.0, at_keyword.1).unwrap(),
            decoded_at_keyword,
            page_selector_list.map(|(start, end)| source_text.anchor(start, end).unwrap()),
            source_text.anchor(header.0, header.1).unwrap(),
            source_text.anchor(block_opener.0, block_opener.1).unwrap(),
            source_text.anchor(body.0, body.1).unwrap(),
            termination,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn page_margin_rule_block(
        source_text: &SourceText,
        parent: CssParserContextId,
        kind: CssParserPageMarginRuleKind,
        at_keyword: (usize, usize),
        decoded_at_keyword: &str,
        header: (usize, usize),
        block_opener: (usize, usize),
        body: (usize, usize),
        termination: CssParserContextTermination,
    ) -> Result<CssParserContextRecord, CssParserContextContractError> {
        CssParserContextRecord::new_page_margin_rule_block(
            source_text,
            CssParserContextId::new(1),
            parent,
            CssParserDirectItemOrdinal::new(0),
            kind,
            source_text.anchor(at_keyword.0, at_keyword.1).unwrap(),
            decoded_at_keyword,
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
        let result = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let record = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            Some(parent_id),
            item_ordinal,
            Some(parent_id),
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
        let record = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(3),
            None,
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
    fn valid_group_rule_block_constructs_with_at_keyword_evidence() {
        let text = source(200, "a{@media screen{p:v;}}");
        let record = group_rule_block(
            &text,
            CssParserGroupRuleKind::Media,
            "media",
            (2, 8),
            (2, 15),
            (15, 16),
            (16, 21),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(21, 22).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(
            record.kind(),
            CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Media)
        );
        assert_eq!(
            record.at_keyword().map(SourceAnchor::range),
            Some(text.anchor(2, 8).unwrap().range())
        );
        assert!(record.nearest_qualified_ancestor().is_none());
    }

    #[test]
    fn qualified_rule_block_never_carries_at_keyword() {
        let text = source(201, "a{p:v;}");
        let record = qualified_rule_block(
            &text,
            (0, 1),
            (1, 2),
            (2, 6),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(6, 7).unwrap(),
            },
        )
        .unwrap();
        assert!(record.at_keyword().is_none());
    }

    #[test]
    fn at_keyword_missing_sigil_is_rejected() {
        let text = source(202, "a{media screen{p:v;}}");
        let result = group_rule_block(
            &text,
            CssParserGroupRuleKind::Media,
            "media",
            (2, 7),
            (2, 14),
            (14, 15),
            (15, 20),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(20, 21).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::AtKeywordMissingSigil)
        );
    }

    #[test]
    fn at_keyword_header_start_mismatch_is_rejected() {
        let text = source(203, "a{ @media screen{p:v;}}");
        let result = group_rule_block(
            &text,
            CssParserGroupRuleKind::Media,
            "media",
            (3, 9),
            (2, 16),
            (16, 17),
            (17, 22),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(22, 23).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::AtKeywordHeaderStartMismatch)
        );
    }

    #[test]
    fn at_keyword_outside_header_is_rejected() {
        let text = source(204, "a{@media screen{p:v;}}");
        let result = group_rule_block(
            &text,
            CssParserGroupRuleKind::Media,
            "media",
            (2, 20),
            (2, 15),
            (15, 16),
            (16, 21),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(21, 22).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::AtKeywordOutsideHeader)
        );
    }

    #[test]
    fn at_keyword_foreign_source_id_is_rejected() {
        let text = source(206, "a{@media screen{p:v;}}");
        let other = source(207, "a{@media screen{p:v;}}");
        let result = CssParserContextRecord::new_group_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            CssParserGroupRuleKind::Media,
            other.anchor(2, 8).unwrap(),
            "media",
            text.anchor(2, 15).unwrap(),
            text.anchor(15, 16).unwrap(),
            text.anchor(16, 21).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(21, 22).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserContextContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn group_kind_decoded_mismatch_is_rejected() {
        let text = source(205, "a{@media screen{p:v;}}");
        let result = group_rule_block(
            &text,
            CssParserGroupRuleKind::Layer,
            "media",
            (2, 8),
            (2, 15),
            (15, 16),
            (16, 21),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(21, 22).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::GroupKindDecodedMismatch)
        );
    }

    // -- #169 descriptor-context construction corruption tests --------------

    #[test]
    fn descriptor_kind_decoded_mismatch_is_rejected() {
        let text = source(300, "@font-face{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::FontFace,
            (0, 10),
            "property",
            None,
            None,
            (0, 10),
            (10, 11),
            (11, 15),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(15, 16).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::DescriptorKindDecodedMismatch)
        );
    }

    #[test]
    fn font_face_carries_property_name_evidence_is_rejected() {
        let text = source(301, "@font-face{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::FontFace,
            (0, 10),
            "font-face",
            Some((11, 12)),
            Some("--x"),
            (0, 10),
            (10, 11),
            (11, 15),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(15, 16).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::FontFaceCarriesPropertyNameEvidence)
        );
    }

    #[test]
    fn property_missing_custom_property_name_evidence_is_rejected() {
        let text = source(302, "@property{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::Property,
            (0, 9),
            "property",
            None,
            None,
            (0, 9),
            (9, 10),
            (10, 14),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PropertyMissingCustomPropertyNameEvidence)
        );
    }

    #[test]
    fn property_name_not_custom_property_shaped_is_rejected() {
        let text = source(303, "@property color{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::Property,
            (0, 9),
            "property",
            Some((10, 15)),
            Some("color"),
            (0, 15),
            (15, 16),
            (16, 20),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(20, 21).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PropertyNameNotCustomPropertyShaped)
        );
    }

    #[test]
    fn property_name_reserved_double_hyphen_is_rejected() {
        let text = source(304, "@property --{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::Property,
            (0, 9),
            "property",
            Some((10, 12)),
            Some("--"),
            (0, 12),
            (12, 13),
            (13, 17),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(17, 18).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PropertyNameNotCustomPropertyShaped)
        );
    }

    #[test]
    fn property_name_out_of_order_is_rejected() {
        let text = source(305, "@property --x{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::Property,
            (0, 9),
            "property",
            // Starts before at_keyword.end() (9): overlaps the at-keyword.
            Some((5, 8)),
            Some("--x"),
            (0, 13),
            (13, 14),
            (14, 18),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(18, 19).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PropertyNameOutOfOrder)
        );
    }

    #[test]
    fn property_name_outside_header_is_rejected() {
        let text = source(306, "@property --x{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::Property,
            (0, 9),
            "property",
            // Extends past header.end() (13) into the block opener/body.
            Some((10, 15)),
            Some("--x"),
            (0, 13),
            (13, 14),
            (14, 18),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(18, 19).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PropertyNameOutsideHeader)
        );
    }

    #[test]
    fn descriptor_at_keyword_missing_sigil_is_rejected() {
        let text = source(307, "@font-face{p:v;}");
        let result = descriptor_rule_block(
            &text,
            CssParserDescriptorRuleKind::FontFace,
            // "font-face" without the leading '@'.
            (1, 10),
            "font-face",
            None,
            None,
            (1, 10),
            (10, 11),
            (11, 15),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(15, 16).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::AtKeywordMissingSigil)
        );
    }

    #[test]
    fn descriptor_at_keyword_foreign_source_id_is_rejected() {
        let text = source(308, "@font-face{p:v;}");
        let other = source(309, "@font-face{p:v;}");
        let result = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::FontFace,
            other.anchor(0, 10).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(0, 10).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 15).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(15, 16).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserContextContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn property_name_foreign_source_id_is_rejected() {
        let text = source(310, "@property --x{p:v;}");
        let other = source(311, "@property --x{p:v;}");
        let result = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::Property,
            text.anchor(0, 9).unwrap(),
            "property",
            Some(other.anchor(10, 13).unwrap()),
            Some("--x"),
            text.anchor(0, 13).unwrap(),
            text.anchor(13, 14).unwrap(),
            text.anchor(14, 18).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(18, 19).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserContextContractError::SourceIdentityMismatch { .. })
        ));
    }

    // -- #170 page-context construction corruption tests --------------------

    #[test]
    fn page_at_keyword_decoded_mismatch_is_rejected() {
        let text = source(400, "@page{p:v;}");
        let result = page_rule_block(
            &text,
            (0, 5),
            "notpage",
            None,
            (0, 5),
            (5, 6),
            (6, 10),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PageAtKeywordDecodedMismatch)
        );
    }

    #[test]
    fn page_margin_kind_decoded_mismatch_is_rejected() {
        let text = source(401, "@top-center{p:v;}");
        let result = page_margin_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserPageMarginRuleKind::TopCenter,
            (0, 11),
            "top-left",
            (0, 11),
            (11, 12),
            (12, 16),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(16, 17).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PageMarginKindDecodedMismatch)
        );
    }

    #[test]
    fn page_selector_list_out_of_order_is_rejected() {
        let text = source(402, "@page foo{p:v;}");
        let result = page_rule_block(
            &text,
            (0, 5),
            "page",
            // Starts before at_keyword.end() (5): overlaps the at-keyword.
            Some((3, 9)),
            (0, 9),
            (9, 10),
            (10, 14),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PageSelectorListOutOfOrder)
        );
    }

    #[test]
    fn page_selector_list_outside_header_is_rejected() {
        let text = source(403, "@page foo{p:v;}");
        let result = page_rule_block(
            &text,
            (0, 5),
            "page",
            // Starts after at_keyword.end() (5) but extends past header.end()
            // (9) into the block opener.
            Some((6, 10)),
            (0, 9),
            (9, 10),
            (10, 14),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserContextContractError::PageSelectorListOutsideHeader)
        );
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
