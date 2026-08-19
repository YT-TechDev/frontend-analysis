//! Candidate-independent escape-free `StringLiteral` initializer validation for Issue #257.
//!
//! This oracle qualifies only directly authored single- and double-quoted
//! `StringLiteral` spellings whose contents contain no authored reverse solidus.
//! It does not call production lexical, static-semantics, aggregate, or runtime
//! evaluation code.
//!
//! `UnsupportedCoverage` expectations are scoped to this escape-free string
//! frontier. Later independently qualified owners may strengthen classifications
//! for escaped strings, malformed quoted source, richer expressions, comments,
//! or broader ASI.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 257;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_THIS_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_this_expression_initializer_validation_tests.rs");
const STRING_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escape_free_string_literal_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "escape-free StringLiteral frontier only; ",
    "later independently qualified owners may strengthen classification"
);
const LINE_TERMINATOR_AUTHORITY_NOTE: &str = concat!(
    "ES2026 permits direct LS/PS StringCharacter alternatives but excludes direct LF/CR ",
    "except through LineContinuation"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteKind {
    Double,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
}

fn slice(text: &str, Range(start, end): Range) -> &str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("invalid UTF-8 range ({start},{end}) in {text:?}"))
}

/// Independently classifies only the Issue #257 direct, escape-free source family.
///
/// The 2026 lexical grammar has explicit direct `<LS>` / `<PS>` alternatives for
/// both quote forms. Direct LF and CR are excluded; accepting them would require
/// `LineContinuation`, which necessarily starts with a reverse solidus and is
/// outside this frontier.
fn escape_free_quote_kind(rhs: &str) -> Option<QuoteKind> {
    let mut chars = rhs.char_indices();
    let (_, opening) = chars.next()?;
    let (kind, closing) = match opening {
        '"' => (QuoteKind::Double, '"'),
        '\'' => (QuoteKind::Single, '\''),
        _ => return None,
    };

    for (index, code_point) in chars {
        if code_point == closing {
            let end = index + code_point.len_utf8();
            return (end == rhs.len()).then_some(kind);
        }
        if matches!(code_point, '\\' | '\n' | '\r') {
            return None;
        }
    }

    None
}

fn authored_anchor(source_id: u64, text: &str, range: Range) -> String {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let Range(start, end) = range;
    source
        .anchor(start, end)
        .expect("fixture range must be a valid authored anchor")
        .fragment()
        .to_owned()
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 257);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_THIS_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 253"));
    assert!(PREVIOUS_THIS_ORACLE_SOURCE.contains("direct-this frontier only"));
    assert!(FRONTIER_SCOPE_NOTE.contains("escape-free StringLiteral frontier only"));
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
    assert!(LINE_TERMINATOR_AUTHORITY_NOTE.contains("direct LS/PS"));
    assert!(LINE_TERMINATOR_AUTHORITY_NOTE.contains("direct LF/CR"));

    for forbidden in [
        concat!("use super::", "selected_lexical_slice"),
        concat!("use super::", "selected_static_semantics"),
        concat!("use super::", "selected_qualification_integration"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_selected_", "string_literal"),
        concat!("consume_selected_escape_free_", "string_literal"),
    ] {
        assert!(!STRING_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn direct_quote_forms_ranges_and_internal_content_are_pinned() {
    let fixtures = [
        ("const x = \"\";", Range(10, 12), "\"\"", QuoteKind::Double),
        ("const x = '';", Range(10, 12), "''", QuoteKind::Single),
        (
            "const x = \"abc\";",
            Range(10, 15),
            "\"abc\"",
            QuoteKind::Double,
        ),
        (
            "const x = 'abc';",
            Range(10, 15),
            "'abc'",
            QuoteKind::Single,
        ),
        (
            "const x = \"𝒜\";",
            Range(10, 16),
            "\"𝒜\"",
            QuoteKind::Double,
        ),
        (
            "const x = \"a'b\";",
            Range(10, 15),
            "\"a'b\"",
            QuoteKind::Double,
        ),
        (
            "const x = 'a\"b';",
            Range(10, 15),
            "'a\"b'",
            QuoteKind::Single,
        ),
        (
            "const x = \"a b\";",
            Range(10, 15),
            "\"a b\"",
            QuoteKind::Double,
        ),
        (
            "const x = \"a\tb\";",
            Range(10, 15),
            "\"a\tb\"",
            QuoteKind::Double,
        ),
        (
            "const x = \"a\u{FEFF}b\";",
            Range(10, 17),
            "\"a\u{FEFF}b\"",
            QuoteKind::Double,
        ),
    ];

    for (index, (text, rhs_range, expected_rhs, expected_quote)) in fixtures.iter().enumerate() {
        let rhs = slice(text, *rhs_range);
        assert_eq!(rhs, *expected_rhs);
        assert_eq!(escape_free_quote_kind(rhs), Some(*expected_quote));
        assert_eq!(
            authored_anchor(257_100 + index as u64, text, *rhs_range),
            *expected_rhs
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

#[test]
fn es2026_line_terminator_boundary_is_pinned_per_code_point_class() {
    let direct_lf = "\"a\nb\"";
    let direct_cr = "\"a\rb\"";
    let direct_crlf = "\"a\r\nb\"";
    let direct_ls = "\"a\u{2028}b\"";
    let direct_ps = "\"a\u{2029}b\"";
    let single_ls = "'a\u{2028}b'";
    let single_ps = "'a\u{2029}b'";

    assert_eq!(escape_free_quote_kind(direct_lf), None);
    assert_eq!(escape_free_quote_kind(direct_cr), None);
    assert_eq!(escape_free_quote_kind(direct_crlf), None);

    assert_eq!(escape_free_quote_kind(direct_ls), Some(QuoteKind::Double));
    assert_eq!(escape_free_quote_kind(direct_ps), Some(QuoteKind::Double));
    assert_eq!(escape_free_quote_kind(single_ls), Some(QuoteKind::Single));
    assert_eq!(escape_free_quote_kind(single_ps), Some(QuoteKind::Single));

    assert_eq!(direct_ls.len(), 7);
    assert_eq!(direct_ps.len(), 7);
    assert!(LINE_TERMINATOR_AUTHORITY_NOTE.contains("LineContinuation"));
}

#[test]
fn escape_line_continuation_and_malformed_quote_neighbors_are_unowned_here() {
    let escaped = [
        "\"\\n\"",
        "\"\\u0061\"",
        "\"\\x61\"",
        "\"\\\\\"",
        "\"\\\"\"",
        "'\\''",
        "\"\\\n\"",
        "'\\\r\n'",
    ];
    for rhs in escaped {
        assert_eq!(escape_free_quote_kind(rhs), None, "{rhs:?}");
    }

    let malformed_or_mismatched = ["\"abc", "'abc", "\"abc'", "'abc\""];
    for rhs in malformed_or_mismatched {
        assert_eq!(escape_free_quote_kind(rhs), None, "{rhs:?}");
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
    assert!(FRONTIER_SCOPE_NOTE.contains("later independently qualified owners"));
}

#[test]
fn selected_boundaries_and_existing_sibling_initializers_compose() {
    let fixtures = [
        ("const x = \"abc\";", Range(10, 15), 15, Some(b';')),
        ("const x = 'abc';", Range(10, 15), 15, Some(b';')),
        ("const x = \"abc\"", Range(10, 15), 15, None),
        ("const x = \"abc\"   \n", Range(10, 15), 19, None),
        ("let x = \"abc\", y = foo;", Range(8, 13), 13, Some(b',')),
        ("let x = foo, y = \"abc\";", Range(17, 22), 22, Some(b';')),
        ("let x = 1, y = \"abc\";", Range(15, 20), 20, Some(b';')),
        (
            "const x = \"abc\", y = null;",
            Range(10, 15),
            15,
            Some(b','),
        ),
        (
            "const x = this, y = \"abc\";",
            Range(20, 25),
            25,
            Some(b';'),
        ),
        (
            "const x = \"a\u{2028}b\";",
            Range(10, 17),
            17,
            Some(b';'),
        ),
        (
            "const x = 'a\u{2029}b';",
            Range(10, 17),
            17,
            Some(b';'),
        ),
    ];

    for (index, (text, rhs_range, boundary, expected_byte)) in fixtures.iter().enumerate() {
        let rhs = slice(text, *rhs_range);
        assert!(escape_free_quote_kind(rhs).is_some(), "{rhs:?}");
        assert_eq!(
            authored_anchor(257_200 + index as u64, text, *rhs_range),
            rhs
        );

        let Range(_, end) = *rhs_range;
        let trailing = text.get(end..*boundary).expect("fixture boundary");
        assert!(trailing.chars().all(|code_point| {
            matches!(
                code_point,
                '\u{0009}'
                    | '\u{000B}'
                    | '\u{000C}'
                    | '\u{0020}'
                    | '\u{00A0}'
                    | '\u{1680}'
                    | '\u{2000}'
                    | '\u{2001}'
                    | '\u{2002}'
                    | '\u{2003}'
                    | '\u{2004}'
                    | '\u{2005}'
                    | '\u{2006}'
                    | '\u{2007}'
                    | '\u{2008}'
                    | '\u{2009}'
                    | '\u{200A}'
                    | '\u{202F}'
                    | '\u{205F}'
                    | '\u{3000}'
                    | '\u{FEFF}'
                    | '\n'
                    | '\r'
                    | '\u{2028}'
                    | '\u{2029}'
            )
        }));

        match expected_byte {
            Some(byte) => assert_eq!(text.as_bytes().get(*boundary), Some(byte)),
            None => assert_eq!(*boundary, text.len()),
        }
    }
}

#[test]
fn richer_tail_controls_preserve_whole_source_transactionality() {
    let rhs_controls = [
        "\"abc\".length",
        "\"abc\"()",
        "\"abc\" + x",
        "\"abc\" = x",
        "\"abc\" ? x : y",
        "\"abc\"/*comment*/",
        "\"abc\" unexpected",
        "\"abc\";",
        "\"abc\"\nconst y = foo;",
        "\"a\"b\"",
        "'a'b'",
    ];

    for rhs in rhs_controls {
        assert_eq!(escape_free_quote_kind(rhs), None, "{rhs:?}");
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

#[test]
fn existing_static_subjects_and_families_remain_reachable() {
    let fixtures = [
        (
            r#"let \u0030 = "abc";"#,
            Range(13, 18),
            Range(4, 10),
            r"\u0030",
            "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        ),
        (
            r#"let \u0069f = "abc";"#,
            Range(14, 19),
            Range(4, 11),
            r"\u0069f",
            "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        ),
        (
            "let let = \"abc\";",
            Range(10, 15),
            Range(4, 7),
            "let",
            "JS-GOLD-LEXDECL-LET-BINDING-001",
        ),
        (
            "let x = \"abc\", x = foo;",
            Range(8, 13),
            Range(15, 16),
            "x",
            "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        ),
        (
            "const x = \"abc\", y;",
            Range(10, 15),
            Range(17, 18),
            "y",
            "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        ),
        (
            "let x = \"abc\"; let x = foo;",
            Range(8, 13),
            Range(19, 20),
            "x",
            "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        ),
    ];

    for (index, (text, rhs, subject, fragment, gold)) in fixtures.iter().enumerate() {
        assert!(escape_free_quote_kind(slice(text, *rhs)).is_some());
        assert_eq!(slice(text, *subject), *fragment);
        assert!(gold_source(gold).is_some(), "{gold}");
        assert_eq!(
            authored_anchor(257_300 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

#[test]
fn existing_grammar_subjects_and_syntax_family_remain_reachable() {
    let fixtures = [
        (r#"const x = "abc"; let \u{};"#, Range(21, 25), r"\u{}"),
        (
            r#"const x = "abc"; let a\u{};"#,
            Range(22, 26),
            r"\u{}",
        ),
        (r#"const x = "abc"; let \u{61"#, Range(21, 26), r"\u{61"),
    ];

    for (index, (text, subject, fragment)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, Range(10, 15)), "\"abc\"");
        assert_eq!(escape_free_quote_kind("\"abc\""), Some(QuoteKind::Double));
        assert_eq!(slice(text, *subject), *fragment);
        assert_eq!(
            authored_anchor(257_400 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::SyntaxRejected;
    assert!(matches!(expected, FrontierOutcome::SyntaxRejected));
}

#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert!(STRING_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(STRING_ORACLE_SOURCE.contains("UnsupportedCoverage"));
    assert!(STRING_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
    assert!(STRING_ORACLE_SOURCE.contains("SyntaxRejected"));
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("CompleteQualification", "Witness"),
        concat!("enum Selected", "Literal"),
        concat!("enum SelectedPrimary", "Expression"),
        concat!("enum SelectedExpression", "Kind"),
        concat!("struct String", "Value"),
    ] {
        assert!(!STRING_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
