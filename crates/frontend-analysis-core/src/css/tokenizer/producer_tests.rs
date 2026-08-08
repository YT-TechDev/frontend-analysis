//! Focused producer regression tests not covered by #135 gold fixtures.

use crate::{SourceId, SourceText};

use super::producer::run;
use super::resource::CssTokenizerLimits;
use super::result::{CssTokenizerCompletion, CssTokenizerTermination};
use crate::css::token::{CssLexicalItem, CssTokenKind};

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn generous_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

fn ranges(items: &[CssLexicalItem]) -> Vec<(usize, usize)> {
    items
        .iter()
        .map(|item| (item.source().range().start(), item.source().range().end()))
        .collect()
}

#[test]
fn quoted_url_disambiguation_whitespace_attaches_to_following_whitespace_token() {
    let text = source(1, "url(  \"x\")");
    let result = run(&text, generous_limits()).unwrap();

    assert_eq!(result.completion(), CssTokenizerCompletion::Complete);
    assert_eq!(result.termination(), &CssTokenizerTermination::EndOfInput);
    assert!(result.diagnostics().is_empty());

    let items = result.lexical_items();
    assert_eq!(items.len(), 4);
    assert_eq!(ranges(items), vec![(0, 4), (4, 6), (6, 9), (9, 10)]);

    let CssLexicalItem::SemanticToken(function) = &items[0] else {
        panic!("expected function token");
    };
    assert_eq!(function.kind(), &CssTokenKind::Function("url".to_owned()));
    assert!(function.source().fragment().ends_with('('));

    let CssLexicalItem::SemanticToken(whitespace) = &items[1] else {
        panic!("expected whitespace token");
    };
    assert_eq!(whitespace.kind(), &CssTokenKind::Whitespace);
    assert_eq!(whitespace.source().fragment(), "  ");

    let CssLexicalItem::SemanticToken(string) = &items[2] else {
        panic!("expected string token");
    };
    assert_eq!(string.kind(), &CssTokenKind::String("x".to_owned()));

    assert!(matches!(
        &items[3],
        CssLexicalItem::SemanticToken(token) if token.kind() == &CssTokenKind::RightParenthesis
    ));
}

#[test]
fn quoted_url_disambiguation_whitespace_covers_full_tab_run() {
    let text = source(2, "url(\t\t'x')");
    let result = run(&text, generous_limits()).unwrap();

    assert_eq!(result.completion(), CssTokenizerCompletion::Complete);
    let items = result.lexical_items();
    assert_eq!(items.len(), 4);
    assert_eq!(ranges(items), vec![(0, 4), (4, 6), (6, 9), (9, 10)]);

    let CssLexicalItem::SemanticToken(whitespace) = &items[1] else {
        panic!("expected whitespace token");
    };
    assert_eq!(whitespace.kind(), &CssTokenKind::Whitespace);
    assert_eq!(whitespace.source().fragment(), "\t\t");

    let CssLexicalItem::SemanticToken(string) = &items[2] else {
        panic!("expected string token");
    };
    assert_eq!(string.kind(), &CssTokenKind::String("x".to_owned()));
}

#[test]
fn quoted_url_without_pending_whitespace_has_no_intervening_whitespace_token() {
    let text = source(3, "url(\"x\")");
    let result = run(&text, generous_limits()).unwrap();

    let items = result.lexical_items();
    assert_eq!(items.len(), 3);
    assert_eq!(ranges(items), vec![(0, 4), (4, 7), (7, 8)]);
    assert!(matches!(
        &items[0],
        CssLexicalItem::SemanticToken(token) if token.kind() == &CssTokenKind::Function("url".to_owned())
    ));
    assert!(matches!(
        &items[1],
        CssLexicalItem::SemanticToken(token) if token.kind() == &CssTokenKind::String("x".to_owned())
    ));
}

#[test]
fn bad_string_newline_reconsume_gives_the_newline_to_the_following_item() {
    let text = source(4, "\"a\nb");
    let result = run(&text, generous_limits()).unwrap();

    assert_eq!(result.completion(), CssTokenizerCompletion::Complete);
    let items = result.lexical_items();
    assert_eq!(items.len(), 3);
    assert_eq!(ranges(items), vec![(0, 2), (2, 3), (3, 4)]);

    assert!(matches!(
        &items[0],
        CssLexicalItem::SemanticToken(token) if token.kind() == &CssTokenKind::BadString
    ));
    assert!(matches!(
        &items[1],
        CssLexicalItem::SemanticToken(token) if token.kind() == &CssTokenKind::Whitespace
    ));
    assert!(matches!(
        &items[2],
        CssLexicalItem::SemanticToken(token) if token.kind() == &CssTokenKind::Ident("b".to_owned())
    ));
}
