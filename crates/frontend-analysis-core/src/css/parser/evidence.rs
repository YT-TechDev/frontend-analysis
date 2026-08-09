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
    UnsupportedAtRuleComplete,
    UnsupportedAtKeyword,
    UnsupportedNestedRemainder,
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
#[derive(Clone)]
pub(crate) enum CssParserRecoveryTermination {
    AuthoredSemicolon { semicolon: SourceAnchor },
    EnclosingBlockEnd { right_curly: SourceAnchor },
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
/// block once actual nesting ends the #137 leading declaration zone.
#[derive(Clone)]
pub(crate) enum CssParserUnsupportedRegion {
    TopLevelAtRule {
        complete: SourceAnchor,
        at_keyword: SourceAnchor,
    },
    NestedContentRemainder {
        region: SourceAnchor,
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

    pub(crate) fn region(&self) -> &SourceAnchor {
        match self {
            Self::TopLevelAtRule { complete, .. } => complete,
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
        }
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
}
