use crate::{SourceId, SourceText};

use super::first_lexical_slice::{
    recognize_first_lexical_slice, FirstLexicalSliceOutcome, SelectedLexicalScript,
};
use super::qualification_validation_tests::{gold_source, gold_subject_range};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(204), text.to_owned())
}

fn recognized(text: &str) -> SelectedLexicalScript {
    match recognize_first_lexical_slice(&source(text)) {
        FirstLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        FirstLexicalSliceOutcome::UnsupportedCoverage => {
            panic!("expected selected-slice recognition, got unsupported coverage for {text:?}")
        }
        FirstLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence => {
            panic!("first Leaf must not emit definitive grammar rejection for {text:?}")
        }
        FirstLexicalSliceOutcome::ResourceLimited => {
            panic!("unexpected resource limitation for {text:?}")
        }
        FirstLexicalSliceOutcome::InternalFailure => {
            panic!("unexpected internal failure for {text:?}")
        }
    }
}

fn assert_unsupported(text: &str) {
    assert!(matches!(
        recognize_first_lexical_slice(&source(text)),
        FirstLexicalSliceOutcome::UnsupportedCoverage
    ));
}

#[test]
fn recognizes_selected_decimal_and_exact_authored_provenance() {
    let script = recognized("let x=123;");
    assert_eq!(script.declarations().len(), 1);

    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "let x=123;");
    assert_eq!(declaration.declaration().range().start(), 0);
    assert_eq!(declaration.declaration().range().end(), 10);
    assert_eq!(declaration.binding().fragment(), "x");
    assert_eq!(declaration.semicolon().fragment(), ";");
    assert_eq!(declaration.semicolon().range().start(), 9);
    assert_eq!(declaration.semicolon().range().end(), 10);
}

#[test]
fn consumes_candidate_independent_valid_and_multibyte_gold() {
    let valid = gold_source("JS-GOLD-SCRIPT-VALID-001").expect("valid gold must exist");
    let _ = recognized(valid);

    let multibyte =
        gold_source("JS-GOLD-SCRIPT-MULTIBYTE-001").expect("multibyte gold must exist");
    let (expected_start, expected_end) = gold_subject_range("JS-GOLD-SCRIPT-MULTIBYTE-001")
        .expect("multibyte gold must retain its authoritative subject range");
    let script = recognized(multibyte);
    let binding = script.declarations()[0].binding();
    assert_eq!(binding.fragment(), "π");
    assert_eq!(binding.range().start(), expected_start);
    assert_eq!(binding.range().end(), expected_end);
}

#[test]
fn preserves_duplicate_binding_occurrences_in_source_order() {
    let duplexical =
        gold_source("JS-GOLD-SCRIPT-DUPLEXICAL-001").expect("duplexical gold must exist");
    let script = recognized(duplexical);
    assert_eq!(script.declarations().len(), 2);
    assert_eq!(script.declarations()[0].binding().fragment(), "x");
    assert_eq!(script.declarations()[0].binding().range().start(), 4);
    assert_eq!(script.declarations()[1].binding().fragment(), "x");
    assert_eq!(script.declarations()[1].binding().range().start(), 11);
}

#[test]
fn recognizes_es2026_identifier_composition_without_normalization() {
    for text in [
        "let $=1;",
        "let _=1;",
        "let a0$=1;",
        "let a\u{200C}=1;",
        "let a\u{0301}=1;",
    ] {
        let _ = recognized(text);
    }

    let script = recognized("let é; let e\u{0301};");
    assert_eq!(script.declarations()[0].binding().fragment(), "é");
    assert_eq!(script.declarations()[1].binding().fragment(), "e\u{0301}");
    assert_ne!(
        script.declarations()[0].binding().fragment(),
        script.declarations()[1].binding().fragment()
    );
}

#[test]
fn recognizes_non_strict_top_level_binding_identifier_cases() {
    for text in [
        "let await=1;",
        "let yield=1;",
        "let static=1;",
        "let implements=1;",
        "let arguments=1;",
        "let eval=1;",
        "let let;",
    ] {
        let _ = recognized(text);
    }
}

#[test]
fn rejects_unconditional_reserved_words_from_selected_binding_path() {
    for text in ["let if=1;", "let enum=1;", "let class=1;", "let true=1;"] {
        assert_unsupported(text);
    }
}

#[test]
fn recognizes_complete_core_whitespace_and_line_terminator_set() {
    let _ = recognized("\u{FEFF}let\u{00A0}x=1;\u{2028}");
    let _ = recognized("let\u{3000}x=1;");
    let _ = recognized("\t\u{000B}\u{000C}let\rx=1;\n\u{2029}");
    let _ = recognized("let\nx=1;");
    let _ = recognized("let x=1\n;");
}

#[test]
fn does_not_use_generic_unicode_whitespace() {
    assert_unsupported("let\u{0085}x=1;");
}

#[test]
fn identifier_continue_does_not_become_identifier_start() {
    assert_unsupported("let \u{200D}x=1;");
}

#[test]
fn does_not_split_let_keyword_prefix_from_identifier_name() {
    for text in ["letx=1;", "letπ=1;", "let$=1;", "let_foo=1;", "let\\u0061=1;"] {
        assert_unsupported(text);
    }
}

#[test]
fn escaped_identifiers_are_unsupported_without_prefix_fact_leakage() {
    for text in ["let \\u0078=1;", "let x\\u0061=1;", "l\\u0065t x=1;"] {
        assert_unsupported(text);
    }
}

#[test]
fn selected_decimal_subset_is_exact_and_not_a_numeric_prefix_parser() {
    for text in ["let x=0;", "let x=1;", "let x=123;"] {
        let _ = recognized(text);
    }

    for text in [
        "let x=1.0;",
        "let x=1.;",
        "let x=.1;",
        "let x=1e3;",
        "let x=1_000;",
        "let x=1n;",
        "let x=0x10;",
        "let x=0b10;",
        "let x=0o10;",
        "let x=01;",
        "let x=08;",
        "let x=-1;",
        "let x=+1;",
        "let x=1/2;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn asi_comments_hashbang_and_broader_grammar_remain_unsupported() {
    let asi_gold = gold_source("JS-GOLD-ASI-NO-FABRICATED-RANGE-001")
        .expect("ASI provenance gold must exist");
    assert_unsupported(asi_gold);

    for text in [
        "let x=1",
        "let x=1\nlet y=2;",
        "let x=1\nx++",
        "let/*comment*/x=1;",
        "let x=1;//comment",
        "#!node\nlet x=1;",
        "let x=/a/;",
        "let x,y;",
        "let [x]=y;",
        "let {x}=y;",
        "const x=1;",
        "var x=1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn whole_source_transaction_prevents_prefix_success() {
    for text in ["let x=1; foo();", "let x=1;;", ";let x=1;"] {
        assert_unsupported(text);
    }
}

#[test]
fn malformed_and_truncated_inputs_fail_closed_without_rejection_claims() {
    for text in [
        "",
        " ",
        "l",
        "le",
        "let",
        "let ",
        "let x",
        "let x=",
        "let x=;",
        "let x==1;",
        "let x=>1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn repeated_recognition_preserves_equivalent_order_and_ranges() {
    let first = recognized("let π=1; let x=0;");
    let second = recognized("let π=1; let x=0;");

    let first_ranges: Vec<_> = first
        .declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.declaration().range(),
                declaration.binding().range(),
                declaration.semicolon().range(),
            )
        })
        .collect();
    let second_ranges: Vec<_> = second
        .declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.declaration().range(),
                declaration.binding().range(),
                declaration.semicolon().range(),
            )
        })
        .collect();

    assert_eq!(first_ranges, second_ranges);
}
