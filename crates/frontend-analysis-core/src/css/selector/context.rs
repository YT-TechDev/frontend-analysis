//! Selector grammar-context derivation from retained parser ancestry (#182).
//!
//! This module never inspects raw CSS text. It derives only which selector
//! grammar entry point a future qualifier must use for an already-retained
//! `QualifiedRuleBlock`.

use std::error::Error;
use std::fmt;

use super::super::parser::context::{
    CssParserContextId, CssParserContextKind, CssParserContextRecord, CssParserGroupRuleKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CssSelectorGrammarContext {
    NormalSelectorList,
    NestedRelativeSelectorList {
        nearest_qualified_ancestor: CssParserContextId,
    },
    ScopedRelativeSelectorList {
        scope_context: CssParserContextId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorContextContractError {
    ContextNotFound {
        context: CssParserContextId,
    },
    ContextIsNotQualifiedRule {
        context: CssParserContextId,
    },
    ParentNotFound {
        context: CssParserContextId,
        parent: CssParserContextId,
    },
}

impl fmt::Display for CssSelectorContextContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS selector context contract violation: {self:?}"
        )
    }
}

impl Error for CssSelectorContextContractError {}

/// Derives the #181-approved grammar mode from the retained context table.
///
/// Precedence is exact:
///
/// 1. innermost retained `@scope` ancestor;
/// 2. retained nearest qualified ancestor;
/// 3. normal selector-list grammar.
///
/// Parent relationships are followed exclusively through `ContextId`; no
/// range-based parent discovery or source rescanning is permitted.
pub(crate) fn derive_selector_grammar_context(
    records: &[CssParserContextRecord],
    context_id: CssParserContextId,
) -> Result<CssSelectorGrammarContext, CssSelectorContextContractError> {
    let record = get_record(records, context_id)?;
    if !matches!(record.kind(), CssParserContextKind::QualifiedRuleBlock) {
        return Err(CssSelectorContextContractError::ContextIsNotQualifiedRule {
            context: context_id,
        });
    }

    let mut parent = record.parent();
    while let Some(parent_id) = parent {
        let parent_record = get_record(records, parent_id).map_err(|_| {
            CssSelectorContextContractError::ParentNotFound {
                context: context_id,
                parent: parent_id,
            }
        })?;
        if matches!(
            parent_record.kind(),
            CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Scope)
        ) {
            return Ok(CssSelectorGrammarContext::ScopedRelativeSelectorList {
                scope_context: parent_id,
            });
        }
        parent = parent_record.parent();
    }

    if let Some(nearest_qualified_ancestor) = record.nearest_qualified_ancestor() {
        return Ok(CssSelectorGrammarContext::NestedRelativeSelectorList {
            nearest_qualified_ancestor,
        });
    }

    Ok(CssSelectorGrammarContext::NormalSelectorList)
}

fn get_record(
    records: &[CssParserContextRecord],
    context_id: CssParserContextId,
) -> Result<&CssParserContextRecord, CssSelectorContextContractError> {
    records
        .get(context_id.index())
        .filter(|record| record.id() == context_id)
        .ok_or(CssSelectorContextContractError::ContextNotFound {
            context: context_id,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::context::{CssParserContextTermination, CssParserDirectItemOrdinal};
    use crate::{SourceId, SourceText};

    fn right_curly(source: &SourceText, start: usize) -> CssParserContextTermination {
        CssParserContextTermination::AuthoredRightCurly {
            right_curly: source.anchor(start, start + 1).unwrap(),
        }
    }

    #[test]
    fn root_qualified_rule_uses_normal_selector_list() {
        let source = SourceText::new(SourceId::new(1), "a{}".to_owned());
        let record = CssParserContextRecord::new_qualified_rule_block(
            &source,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            source.anchor(0, 1).unwrap(),
            source.anchor(1, 2).unwrap(),
            source.anchor(2, 2).unwrap(),
            right_curly(&source, 2),
        )
        .unwrap();

        assert_eq!(
            derive_selector_grammar_context(&[record], CssParserContextId::new(0)).unwrap(),
            CssSelectorGrammarContext::NormalSelectorList
        );
    }

    #[test]
    fn nested_qualified_rule_uses_retained_nearest_qualified_ancestor() {
        let source = SourceText::new(SourceId::new(2), "a{b{}}".to_owned());
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &source,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            source.anchor(0, 1).unwrap(),
            source.anchor(1, 2).unwrap(),
            source.anchor(2, 5).unwrap(),
            right_curly(&source, 5),
        )
        .unwrap();
        let inner = CssParserContextRecord::new_qualified_rule_block(
            &source,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(0)),
            source.anchor(2, 3).unwrap(),
            source.anchor(3, 4).unwrap(),
            source.anchor(4, 4).unwrap(),
            right_curly(&source, 4),
        )
        .unwrap();

        assert_eq!(
            derive_selector_grammar_context(&[outer, inner], CssParserContextId::new(1)).unwrap(),
            CssSelectorGrammarContext::NestedRelativeSelectorList {
                nearest_qualified_ancestor: CssParserContextId::new(0),
            }
        );
    }

    #[test]
    fn innermost_scope_takes_precedence_over_outer_qualified_ancestor() {
        let source = SourceText::new(SourceId::new(3), "a{@scope{b{}}}".to_owned());
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &source,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            source.anchor(0, 1).unwrap(),
            source.anchor(1, 2).unwrap(),
            source.anchor(2, 13).unwrap(),
            right_curly(&source, 13),
        )
        .unwrap();
        let scope = CssParserContextRecord::new_group_rule_block(
            &source,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(0)),
            CssParserGroupRuleKind::Scope,
            source.anchor(2, 8).unwrap(),
            "scope",
            source.anchor(2, 8).unwrap(),
            source.anchor(8, 9).unwrap(),
            source.anchor(9, 12).unwrap(),
            right_curly(&source, 12),
        )
        .unwrap();
        let inner = CssParserContextRecord::new_qualified_rule_block(
            &source,
            CssParserContextId::new(2),
            Some(CssParserContextId::new(1)),
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(0)),
            source.anchor(9, 10).unwrap(),
            source.anchor(10, 11).unwrap(),
            source.anchor(11, 11).unwrap(),
            right_curly(&source, 11),
        )
        .unwrap();

        assert_eq!(
            derive_selector_grammar_context(&[outer, scope, inner], CssParserContextId::new(2))
                .unwrap(),
            CssSelectorGrammarContext::ScopedRelativeSelectorList {
                scope_context: CssParserContextId::new(1),
            }
        );
    }
}
