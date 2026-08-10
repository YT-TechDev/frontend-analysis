//! Parser-owned recovery and unsupported-region evidence (#138).
//!
//! Normal whitespace/comments/semicolons already retained by the tokenizer
//! are never recovery records. A speculative declaration failure that later
//! commits as an actual nested qualified rule must leave no recovery or
//! diagnostic evidence at all; that rollback invariant is enforced by the
//! future #139 producer, not by these contracts, which only describe what a
//! durable recovery/unsupported record must look like when one exists.

use std::error::Error;
use std::fmt;

use super::context::{CssParserContextId, CssParserDirectItemOrdinal};
use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserRecoveryKind {
    MalformedBlockItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserEvidenceRole {
    RecoveryRegion,
    RecoverySemicolon,
    RecoveryRightCurly,
    RecoveryEndOfInputTerminal,
    UnsupportedAtRuleComplete,
    UnsupportedAtKeyword,
    UnsupportedNestedRemainder,
    UnsupportedNestedAtRuleComplete,
    UnsupportedNestedAtRuleAtKeyword,
    DiscardRegion,
    DiscardPropertyName,
    DiscardColon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssParserEvidenceContractError {
    SourceIdentityMismatch {
        role: CssParserEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    EmptyEvidence {
        role: CssParserEvidenceRole,
    },
    FixedSpellingMismatch {
        role: CssParserEvidenceRole,
        expected: &'static str,
    },
    EvidenceOutsideContainer {
        role: CssParserEvidenceRole,
    },
    EvidenceOutOfOrder {
        role: CssParserEvidenceRole,
    },
    DecodedPropertyNameNotCustomPropertyShaped {
        role: CssParserEvidenceRole,
    },
    /// The evidence anchor must be an exact empty point, not authored
    /// content; true end-of-input is a boundary, never a fabricated
    /// delimiter.
    EvidenceMustBeEmpty {
        role: CssParserEvidenceRole,
    },
    /// The evidence anchor must sit at the retained source's true end.
    TerminalNotAtSourceEnd {
        role: CssParserEvidenceRole,
    },
}

impl fmt::Display for CssParserEvidenceContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS parser evidence contract violation: {self:?}"
        )
    }
}

impl Error for CssParserEvidenceContractError {}

/// Explicit recovery termination for a malformed supported-context block
/// item.
///
/// `EndOfInput` represents recovery that reached genuine tokenizer end of
/// input without an authored semicolon or authored enclosing right curly.
/// `terminal` is a point boundary at retained source end, never authored
/// content.
#[derive(Clone)]
pub(crate) enum CssParserRecoveryTermination {
    AuthoredSemicolon { semicolon: SourceAnchor },
    EnclosingBlockEnd { right_curly: SourceAnchor },
    EndOfInput { terminal: SourceAnchor },
}

impl PartialEq for CssParserRecoveryTermination {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::AuthoredSemicolon { semicolon: left },
                Self::AuthoredSemicolon { semicolon: right },
            ) => same_anchor(left, right),
            (
                Self::EnclosingBlockEnd { right_curly: left },
                Self::EnclosingBlockEnd { right_curly: right },
            ) => same_anchor(left, right),
            (Self::EndOfInput { terminal: left }, Self::EndOfInput { terminal: right }) => {
                same_anchor(left, right)
            }
            _ => false,
        }
    }
}

impl Eq for CssParserRecoveryTermination {}

impl fmt::Debug for CssParserRecoveryTermination {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthoredSemicolon { semicolon } => formatter
                .debug_struct("AuthoredSemicolon")
                .field("source_id", &semicolon.source_id())
                .field("semicolon", &semicolon.range())
                .finish(),
            Self::EnclosingBlockEnd { right_curly } => formatter
                .debug_struct("EnclosingBlockEnd")
                .field("source_id", &right_curly.source_id())
                .field("right_curly", &right_curly.range())
                .finish(),
            Self::EndOfInput { terminal } => formatter
                .debug_struct("EndOfInput")
                .field("source_id", &terminal.source_id())
                .field("terminal", &terminal.range())
                .finish(),
        }
    }
}

/// Source-backed evidence that a supported-context block item failed to
/// become a committed declaration or actual nested rule.
#[derive(Clone)]
pub(crate) struct CssParserRecoveryEvidence {
    region: SourceAnchor,
    kind: CssParserRecoveryKind,
    termination: CssParserRecoveryTermination,
}

impl CssParserRecoveryEvidence {
    pub(crate) fn new(
        source_text: &SourceText,
        region: SourceAnchor,
        kind: CssParserRecoveryKind,
        termination: CssParserRecoveryTermination,
    ) -> Result<Self, CssParserEvidenceContractError> {
        let expected = source_text.id();
        require_source(expected, &region, CssParserEvidenceRole::RecoveryRegion)?;
        non_empty(&region, CssParserEvidenceRole::RecoveryRegion)?;

        match &termination {
            CssParserRecoveryTermination::AuthoredSemicolon { semicolon } => {
                require_source(
                    expected,
                    semicolon,
                    CssParserEvidenceRole::RecoverySemicolon,
                )?;
                non_empty(semicolon, CssParserEvidenceRole::RecoverySemicolon)?;
                exact(semicolon, CssParserEvidenceRole::RecoverySemicolon, ";")?;
                if semicolon.range().start() < region.range().start()
                    || semicolon.range().end() != region.range().end()
                {
                    return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                        role: CssParserEvidenceRole::RecoverySemicolon,
                    });
                }
            }
            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly } => {
                require_source(
                    expected,
                    right_curly,
                    CssParserEvidenceRole::RecoveryRightCurly,
                )?;
                non_empty(right_curly, CssParserEvidenceRole::RecoveryRightCurly)?;
                exact(right_curly, CssParserEvidenceRole::RecoveryRightCurly, "}")?;
                if right_curly.range().start() != region.range().end() {
                    return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                        role: CssParserEvidenceRole::RecoveryRightCurly,
                    });
                }
            }
            CssParserRecoveryTermination::EndOfInput { terminal } => {
                require_source(
                    expected,
                    terminal,
                    CssParserEvidenceRole::RecoveryEndOfInputTerminal,
                )?;
                if !terminal.range().is_empty() {
                    return Err(CssParserEvidenceContractError::EvidenceMustBeEmpty {
                        role: CssParserEvidenceRole::RecoveryEndOfInputTerminal,
                    });
                }
                if terminal.range().start() != source_text.as_str().len() {
                    return Err(CssParserEvidenceContractError::TerminalNotAtSourceEnd {
                        role: CssParserEvidenceRole::RecoveryEndOfInputTerminal,
                    });
                }
                if terminal.range().start() != region.range().end() {
                    return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                        role: CssParserEvidenceRole::RecoveryEndOfInputTerminal,
                    });
                }
            }
        }

        Ok(Self {
            region,
            kind,
            termination,
        })
    }

    pub(crate) const fn region(&self) -> &SourceAnchor {
        &self.region
    }

    pub(crate) const fn kind(&self) -> CssParserRecoveryKind {
        self.kind
    }

    pub(crate) const fn termination(&self) -> &CssParserRecoveryTermination {
        &self.termination
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        (self.region.range().start(), self.region.range().end())
    }
}

impl PartialEq for CssParserRecoveryEvidence {
    fn eq(&self, other: &Self) -> bool {
        same_anchor(&self.region, &other.region)
            && self.kind == other.kind
            && self.termination == other.termination
    }
}

impl Eq for CssParserRecoveryEvidence {}

impl fmt::Debug for CssParserRecoveryEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssParserRecoveryEvidence")
            .field("source_id", &self.region.source_id())
            .field("region", &self.region.range())
            .field("kind", &self.kind)
            .field("termination", &self.termination)
            .finish()
    }
}

/// The first-slice unsupported-region categories.
///
/// `TopLevelAtRule` preserves the complete structurally consumed at-rule
/// region and its exact `AtKeyword` token anchor without descending into its
/// block. `NestedContentRemainder` covers the raw remainder of a supported
/// block once actual nesting ends the #137 leading declaration zone; #168
/// retains it only as historical/contract vocabulary and no longer produces
/// it from ordinary nested at-keyword dispatch (see [`Self::NestedAtRule`]).
/// `NestedAtRule` (#168) represents exactly one structurally consumed
/// unsupported nested at-rule -- an unknown/future at-rule, a registry
/// member whose prelude falls outside its approved bounded subset, a
/// registry member without a supported block shape, or an `@layer`
/// statement -- retained as one materialized direct item in its owning
/// context.
#[derive(Clone)]
pub(crate) enum CssParserUnsupportedRegion {
    TopLevelAtRule {
        complete: SourceAnchor,
        at_keyword: SourceAnchor,
    },
    NestedContentRemainder {
        region: SourceAnchor,
    },
    NestedAtRule {
        complete: SourceAnchor,
        at_keyword: SourceAnchor,
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
    },
}

impl CssParserUnsupportedRegion {
    pub(crate) fn new_top_level_at_rule(
        source_text: &SourceText,
        complete: SourceAnchor,
        at_keyword: SourceAnchor,
    ) -> Result<Self, CssParserEvidenceContractError> {
        let expected = source_text.id();
        require_source(
            expected,
            &complete,
            CssParserEvidenceRole::UnsupportedAtRuleComplete,
        )?;
        require_source(
            expected,
            &at_keyword,
            CssParserEvidenceRole::UnsupportedAtKeyword,
        )?;
        non_empty(&complete, CssParserEvidenceRole::UnsupportedAtRuleComplete)?;
        non_empty(&at_keyword, CssParserEvidenceRole::UnsupportedAtKeyword)?;

        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserEvidenceContractError::FixedSpellingMismatch {
                role: CssParserEvidenceRole::UnsupportedAtKeyword,
                expected: "@",
            });
        }
        if complete.range().start() != at_keyword.range().start() {
            return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::UnsupportedAtKeyword,
            });
        }
        if at_keyword.range().end() > complete.range().end() {
            return Err(CssParserEvidenceContractError::EvidenceOutsideContainer {
                role: CssParserEvidenceRole::UnsupportedAtKeyword,
            });
        }

        Ok(Self::TopLevelAtRule {
            complete,
            at_keyword,
        })
    }

    pub(crate) fn new_nested_content_remainder(
        source_text: &SourceText,
        region: SourceAnchor,
    ) -> Result<Self, CssParserEvidenceContractError> {
        require_source(
            source_text.id(),
            &region,
            CssParserEvidenceRole::UnsupportedNestedRemainder,
        )?;
        non_empty(&region, CssParserEvidenceRole::UnsupportedNestedRemainder)?;
        Ok(Self::NestedContentRemainder { region })
    }

    /// Constructs one context-aware unsupported nested at-rule (#168):
    /// `complete` is the exact structurally consumed at-rule region (from
    /// the at-keyword through its statement terminator or block close, or up
    /// to the boundary where execution stopped), and `at_keyword` is the
    /// exact authored at-keyword anchor starting it. `context_id`/
    /// `item_ordinal` retain the owning context and its shared direct-item
    /// ordinal structurally; the caller is responsible for committing this
    /// evidence and mutating that ownership atomically together.
    pub(crate) fn new_nested_at_rule(
        source_text: &SourceText,
        complete: SourceAnchor,
        at_keyword: SourceAnchor,
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
    ) -> Result<Self, CssParserEvidenceContractError> {
        let expected = source_text.id();
        require_source(
            expected,
            &complete,
            CssParserEvidenceRole::UnsupportedNestedAtRuleComplete,
        )?;
        require_source(
            expected,
            &at_keyword,
            CssParserEvidenceRole::UnsupportedNestedAtRuleAtKeyword,
        )?;
        non_empty(
            &complete,
            CssParserEvidenceRole::UnsupportedNestedAtRuleComplete,
        )?;
        non_empty(
            &at_keyword,
            CssParserEvidenceRole::UnsupportedNestedAtRuleAtKeyword,
        )?;

        if !at_keyword.fragment().starts_with('@') {
            return Err(CssParserEvidenceContractError::FixedSpellingMismatch {
                role: CssParserEvidenceRole::UnsupportedNestedAtRuleAtKeyword,
                expected: "@",
            });
        }
        if complete.range().start() != at_keyword.range().start() {
            return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::UnsupportedNestedAtRuleAtKeyword,
            });
        }
        if at_keyword.range().end() > complete.range().end() {
            return Err(CssParserEvidenceContractError::EvidenceOutsideContainer {
                role: CssParserEvidenceRole::UnsupportedNestedAtRuleAtKeyword,
            });
        }

        Ok(Self::NestedAtRule {
            complete,
            at_keyword,
            context_id,
            item_ordinal,
        })
    }

    pub(crate) fn region(&self) -> &SourceAnchor {
        match self {
            Self::TopLevelAtRule { complete, .. } | Self::NestedAtRule { complete, .. } => complete,
            Self::NestedContentRemainder { region } => region,
        }
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        let range = self.region().range();
        (range.start(), range.end())
    }
}

impl PartialEq for CssParserUnsupportedRegion {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::TopLevelAtRule {
                    complete: left_complete,
                    at_keyword: left_at_keyword,
                },
                Self::TopLevelAtRule {
                    complete: right_complete,
                    at_keyword: right_at_keyword,
                },
            ) => {
                same_anchor(left_complete, right_complete)
                    && same_anchor(left_at_keyword, right_at_keyword)
            }
            (
                Self::NestedContentRemainder { region: left },
                Self::NestedContentRemainder { region: right },
            ) => same_anchor(left, right),
            (
                Self::NestedAtRule {
                    complete: left_complete,
                    at_keyword: left_at_keyword,
                    context_id: left_context_id,
                    item_ordinal: left_item_ordinal,
                },
                Self::NestedAtRule {
                    complete: right_complete,
                    at_keyword: right_at_keyword,
                    context_id: right_context_id,
                    item_ordinal: right_item_ordinal,
                },
            ) => {
                same_anchor(left_complete, right_complete)
                    && same_anchor(left_at_keyword, right_at_keyword)
                    && left_context_id == right_context_id
                    && left_item_ordinal == right_item_ordinal
            }
            _ => false,
        }
    }
}

impl Eq for CssParserUnsupportedRegion {}

impl fmt::Debug for CssParserUnsupportedRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLevelAtRule {
                complete,
                at_keyword,
            } => formatter
                .debug_struct("TopLevelAtRule")
                .field("source_id", &complete.source_id())
                .field("complete", &complete.range())
                .field("at_keyword", &at_keyword.range())
                .finish(),
            Self::NestedContentRemainder { region } => formatter
                .debug_struct("NestedContentRemainder")
                .field("source_id", &region.source_id())
                .field("region", &region.range())
                .finish(),
            Self::NestedAtRule {
                complete,
                at_keyword,
                context_id,
                item_ordinal,
            } => formatter
                .debug_struct("NestedAtRule")
                .field("source_id", &complete.source_id())
                .field("complete", &complete.range())
                .field("at_keyword", &at_keyword.range())
                .field("context_id", context_id)
                .field("item_ordinal", item_ordinal)
                .finish(),
        }
    }
}

/// The first-slice parser discard categories.
///
/// Distinct from [`CssParserRecoveryEvidence`] (malformed supported-context
/// block-item recovery) and [`CssParserUnsupportedRegion`] (capability
/// coverage evidence): a discard record is structurally understood parser
/// behavior explicitly required by the pinned CSS Syntax grammar, not a
/// malformed item or an unsupported context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserDiscardKind {
    /// `consume a qualified rule` with `nested = false`, whose prelude's
    /// first two non-whitespace values are a custom-property-shaped Ident
    /// followed by a Colon: the qualified rule is not returned and its block
    /// is structurally consumed.
    TopLevelCustomPropertyLikeQualifiedRule,
}

/// Source-backed evidence that a top-level qualified-rule candidate was
/// structurally discarded per CSS Syntax rather than becoming a supported
/// declaration context.
#[derive(Clone)]
pub(crate) struct CssParserDiscardEvidence {
    region: SourceAnchor,
    property_name: SourceAnchor,
    colon: SourceAnchor,
    kind: CssParserDiscardKind,
}

impl CssParserDiscardEvidence {
    /// `decoded_property_name` is the already-decoded tokenizer `Ident`
    /// value used only to validate `starts_with("--")`; it is not retained
    /// afterward.
    pub(crate) fn new(
        source_text: &SourceText,
        region: SourceAnchor,
        property_name: SourceAnchor,
        colon: SourceAnchor,
        decoded_property_name: &str,
        kind: CssParserDiscardKind,
    ) -> Result<Self, CssParserEvidenceContractError> {
        let expected = source_text.id();
        require_source(expected, &region, CssParserEvidenceRole::DiscardRegion)?;
        require_source(
            expected,
            &property_name,
            CssParserEvidenceRole::DiscardPropertyName,
        )?;
        require_source(expected, &colon, CssParserEvidenceRole::DiscardColon)?;

        non_empty(&region, CssParserEvidenceRole::DiscardRegion)?;
        non_empty(&property_name, CssParserEvidenceRole::DiscardPropertyName)?;
        non_empty(&colon, CssParserEvidenceRole::DiscardColon)?;

        if region.range().start() != property_name.range().start() {
            return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::DiscardPropertyName,
            });
        }
        if property_name.range().end() > region.range().end() {
            return Err(CssParserEvidenceContractError::EvidenceOutsideContainer {
                role: CssParserEvidenceRole::DiscardPropertyName,
            });
        }

        if !decoded_property_name.starts_with("--") {
            return Err(
                CssParserEvidenceContractError::DecodedPropertyNameNotCustomPropertyShaped {
                    role: CssParserEvidenceRole::DiscardPropertyName,
                },
            );
        }

        exact(&colon, CssParserEvidenceRole::DiscardColon, ":")?;

        if property_name.range().end() > colon.range().start() {
            return Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::DiscardColon,
            });
        }
        if colon.source_id() != region.source_id()
            || colon.range().start() < region.range().start()
            || colon.range().end() > region.range().end()
        {
            return Err(CssParserEvidenceContractError::EvidenceOutsideContainer {
                role: CssParserEvidenceRole::DiscardColon,
            });
        }

        Ok(Self {
            region,
            property_name,
            colon,
            kind,
        })
    }

    pub(crate) const fn region(&self) -> &SourceAnchor {
        &self.region
    }

    pub(crate) const fn property_name(&self) -> &SourceAnchor {
        &self.property_name
    }

    pub(crate) const fn colon(&self) -> &SourceAnchor {
        &self.colon
    }

    pub(crate) const fn kind(&self) -> CssParserDiscardKind {
        self.kind
    }
}

impl PartialEq for CssParserDiscardEvidence {
    fn eq(&self, other: &Self) -> bool {
        same_anchor(&self.region, &other.region)
            && same_anchor(&self.property_name, &other.property_name)
            && same_anchor(&self.colon, &other.colon)
            && self.kind == other.kind
    }
}

impl Eq for CssParserDiscardEvidence {}

impl fmt::Debug for CssParserDiscardEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssParserDiscardEvidence")
            .field("source_id", &self.region.source_id())
            .field("region", &self.region.range())
            .field("property_name", &self.property_name.range())
            .field("colon", &self.colon.range())
            .field("kind", &self.kind)
            .finish()
    }
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssParserEvidenceRole,
) -> Result<(), CssParserEvidenceContractError> {
    let actual = anchor.source_id();
    if actual != expected {
        return Err(CssParserEvidenceContractError::SourceIdentityMismatch {
            role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn non_empty(
    anchor: &SourceAnchor,
    role: CssParserEvidenceRole,
) -> Result<(), CssParserEvidenceContractError> {
    if anchor.range().is_empty() {
        return Err(CssParserEvidenceContractError::EmptyEvidence { role });
    }
    Ok(())
}

fn exact(
    anchor: &SourceAnchor,
    role: CssParserEvidenceRole,
    expected: &'static str,
) -> Result<(), CssParserEvidenceContractError> {
    if anchor.fragment() != expected {
        return Err(CssParserEvidenceContractError::FixedSpellingMismatch { role, expected });
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

    #[test]
    fn recovery_with_authored_semicolon_includes_semicolon_in_region() {
        let text = source(1, "a{bogus;color:red;}");
        let evidence = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 8).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::AuthoredSemicolon {
                semicolon: text.anchor(7, 8).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(evidence.region().range().start(), 2);
        assert_eq!(evidence.region().range().end(), 8);
    }

    #[test]
    fn recovery_stopping_at_enclosing_brace_excludes_the_brace() {
        let text = source(2, "a{bogus}");
        let evidence = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 7).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EnclosingBlockEnd {
                right_curly: text.anchor(7, 8).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(evidence.region().range().end(), 7);
    }

    #[test]
    fn recovery_right_curly_must_be_adjacent_to_region_end() {
        let text = source(3, "a{bogus }");
        let result = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 7).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EnclosingBlockEnd {
                right_curly: text.anchor(8, 9).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EvidenceOutOfOrder { .. })
        ));
    }

    #[test]
    fn recovery_end_of_input_accepts_true_eof_terminal_adjacent_to_region() {
        let text = source(20, "a{color red");
        let evidence = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
        )
        .unwrap();
        assert_eq!(evidence.region().range().end(), 11);
        assert_eq!(
            evidence.termination(),
            &CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            }
        );
    }

    #[test]
    fn recovery_end_of_input_terminal_must_be_empty() {
        let text = source(21, "a{color redx");
        let result = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(11, 12).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserEvidenceContractError::EvidenceMustBeEmpty {
                role: CssParserEvidenceRole::RecoveryEndOfInputTerminal,
            })
        );
    }

    #[test]
    fn recovery_end_of_input_terminal_must_be_at_source_end() {
        let text = source(22, "a{color red};");
        let result = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserEvidenceContractError::TerminalNotAtSourceEnd {
                role: CssParserEvidenceRole::RecoveryEndOfInputTerminal,
            })
        );
    }

    #[test]
    fn recovery_end_of_input_terminal_must_be_adjacent_to_region_end() {
        let text = source(23, "a{color red ");
        let result = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(12, 12).unwrap(),
            },
        );
        assert_eq!(
            result,
            Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::RecoveryEndOfInputTerminal,
            })
        );
    }

    #[test]
    fn recovery_end_of_input_cross_source_terminal_is_rejected() {
        let text = source(24, "a{color red");
        let other = source(25, "a{color red");
        let result = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: other.anchor(11, 11).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn recovery_end_of_input_debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "secret-eof-malformed-region";
        let text = source(26, &format!("a{{{SECRET}"));
        let evidence = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 2 + SECRET.len()).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(2 + SECRET.len(), 2 + SECRET.len()).unwrap(),
            },
        )
        .unwrap();
        assert!(!format!("{evidence:?}").contains(SECRET));
    }

    #[test]
    fn top_level_at_rule_requires_complete_start_at_at_keyword_start() {
        let text = source(4, "@font-face{}");
        let region = CssParserUnsupportedRegion::new_top_level_at_rule(
            &text,
            text.anchor(0, 12).unwrap(),
            text.anchor(0, 10).unwrap(),
        )
        .unwrap();
        assert_eq!(region.region().range(), text.anchor(0, 12).unwrap().range());
    }

    #[test]
    fn at_keyword_must_start_with_at_sign() {
        let text = source(5, "font-face{}");
        let result = CssParserUnsupportedRegion::new_top_level_at_rule(
            &text,
            text.anchor(0, 11).unwrap(),
            text.anchor(0, 9).unwrap(),
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::FixedSpellingMismatch { .. })
        ));
    }

    #[test]
    fn nested_content_remainder_requires_non_empty_region() {
        let text = source(6, "a{color:red;b{}}");
        let result = CssParserUnsupportedRegion::new_nested_content_remainder(
            &text,
            text.anchor(12, 12).unwrap(),
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EmptyEvidence { .. })
        ));
    }

    #[test]
    fn nested_at_rule_constructs_with_exact_boundary_relationships() {
        let text = source(27, "a{@unknown-rule foo;}");
        let region = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 20).unwrap(),
            text.anchor(2, 15).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        )
        .unwrap();
        assert_eq!(region.region().range(), text.anchor(2, 20).unwrap().range());
    }

    #[test]
    fn nested_at_rule_at_keyword_must_start_with_at_sign() {
        let text = source(28, "a{unknown-rule foo;}");
        let result = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 19).unwrap(),
            text.anchor(2, 14).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::FixedSpellingMismatch { .. })
        ));
    }

    #[test]
    fn nested_at_rule_complete_must_start_at_at_keyword_start() {
        let text = source(29, "a{ @unknown-rule foo;}");
        let result = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 21).unwrap(),
            text.anchor(3, 16).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EvidenceOutOfOrder { .. })
        ));
    }

    #[test]
    fn nested_at_rule_cross_source_at_keyword_is_rejected() {
        let text = source(31, "a{@unknown-rule foo;}");
        let other = source(32, "a{@unknown-rule foo;}");
        let result = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 20).unwrap(),
            other.anchor(2, 15).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn nested_at_rule_debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "secret-nested-at-rule-remainder";
        let text = source(30, &format!("a{{@x {SECRET};}}"));
        let region = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 6 + SECRET.len()).unwrap(),
            text.anchor(2, 4).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        )
        .unwrap();
        assert!(!format!("{region:?}").contains(SECRET));
    }

    #[test]
    fn cross_source_recovery_region_is_rejected() {
        let text = source(7, "a{bogus}");
        let other = source(8, "a{bogus}");
        let result = CssParserRecoveryEvidence::new(
            &text,
            other.anchor(2, 7).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EnclosingBlockEnd {
                right_curly: text.anchor(7, 8).unwrap(),
            },
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "secret-bogus-block-item-region";
        let text = source(9, &format!("a{{{SECRET}}}"));
        let evidence = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 2 + SECRET.len()).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EnclosingBlockEnd {
                right_curly: text.anchor(2 + SECRET.len(), 3 + SECRET.len()).unwrap(),
            },
        )
        .unwrap();
        assert!(!format!("{evidence:?}").contains(SECRET));
    }

    #[test]
    fn discard_evidence_constructs_with_exact_boundary_relationships() {
        let text = source(10, "--foo:bar{color:red;}");
        let evidence = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, 21).unwrap(),
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        )
        .unwrap();
        assert_eq!(evidence.region().range().start(), 0);
        assert_eq!(evidence.region().range().end(), 21);
        assert_eq!(
            evidence.property_name().range(),
            text.anchor(0, 5).unwrap().range()
        );
        assert_eq!(evidence.colon().range(), text.anchor(5, 6).unwrap().range());
        assert_eq!(
            evidence.kind(),
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule
        );
    }

    #[test]
    fn discard_decoded_name_not_custom_property_shaped_is_rejected() {
        let text = source(11, "foo:bar{color:red;}");
        let result = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, 19).unwrap(),
            text.anchor(0, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            "foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::DecodedPropertyNameNotCustomPropertyShaped { .. })
        ));
    }

    #[test]
    fn discard_colon_must_be_exact_fixed_spelling() {
        let text = source(12, "--foo;bar{color:red;}");
        let result = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, 21).unwrap(),
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert_eq!(
            result,
            Err(CssParserEvidenceContractError::FixedSpellingMismatch {
                role: CssParserEvidenceRole::DiscardColon,
                expected: ":",
            })
        );
    }

    #[test]
    fn discard_cross_source_evidence_is_rejected() {
        let text = source(13, "--foo:bar{color:red;}");
        let other = source(14, "--foo:bar{color:red;}");
        let result = CssParserDiscardEvidence::new(
            &text,
            other.anchor(0, 21).unwrap(),
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn discard_property_name_start_must_equal_region_start() {
        let text = source(15, " --foo:bar{color:red;}");
        let result = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, 22).unwrap(),
            text.anchor(1, 6).unwrap(),
            text.anchor(6, 7).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::DiscardPropertyName,
            })
        ));
    }

    #[test]
    fn discard_colon_must_follow_property_name() {
        // The supplied colon at [0,1) is a real ":" character, but it sits
        // before the property name at [1,6) rather than after it.
        let text = source(16, ":--foo:bar{color:red;}");
        let result = CssParserDiscardEvidence::new(
            &text,
            text.anchor(1, 22).unwrap(),
            text.anchor(1, 6).unwrap(),
            text.anchor(0, 1).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EvidenceOutOfOrder {
                role: CssParserEvidenceRole::DiscardColon,
            })
        ));
    }

    #[test]
    fn discard_colon_must_be_contained_in_region() {
        // The region only covers "--foo:bar{", but the supplied colon is the
        // later, unrelated ":" inside "color:red" at [15,16), which lies
        // outside that region.
        let text = source(17, "--foo:bar{color:red;}");
        let result = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, 10).unwrap(),
            text.anchor(0, 5).unwrap(),
            text.anchor(15, 16).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EvidenceOutsideContainer {
                role: CssParserEvidenceRole::DiscardColon,
            })
        ));
    }

    #[test]
    fn discard_empty_region_is_rejected() {
        let text = source(18, "--foo:bar{}");
        let result = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, 0).unwrap(),
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            "--foo",
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        );
        assert!(matches!(
            result,
            Err(CssParserEvidenceContractError::EmptyEvidence {
                role: CssParserEvidenceRole::DiscardRegion,
            })
        ));
    }

    #[test]
    fn discard_debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "--secret-bogus-property";
        let text = source(19, &format!("{SECRET}:bar{{}}"));
        let evidence = CssParserDiscardEvidence::new(
            &text,
            text.anchor(0, SECRET.len() + 4).unwrap(),
            text.anchor(0, SECRET.len()).unwrap(),
            text.anchor(SECRET.len(), SECRET.len() + 1).unwrap(),
            SECRET,
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        )
        .unwrap();
        assert!(!format!("{evidence:?}").contains(SECRET));
    }
}
