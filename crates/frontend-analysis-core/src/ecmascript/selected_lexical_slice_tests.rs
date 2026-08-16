use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::selected_lexical_slice::{
    SelectedInitializerState, SelectedLexicalDeclarationKind, SelectedLexicalScript,
    SelectedLexicalSliceOutcome, recognize_selected_lexical_slice,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(218), text.to_owned())
}

fn recognized(text: &str) -> SelectedLexicalScript {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        SelectedLexicalSliceOutcome::UnsupportedCoverage => {
            panic!("expected selected-slice recognition, got unsupported coverage for {text:?}")
        }
        SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence => {
            panic!("selected slice must not emit definitive grammar rejection for {text:?}")
        }
        SelectedLexicalSliceOutcome::ResourceLimited => {
            panic!("unexpected resource limitation for {text:?}")
        }
        SelectedLexicalSliceOutcome::InternalFailure => {
            panic!("unexpected internal failure for {text:?}")
        }
    }
}

fn assert_unsupported(text: &str) {
    assert!(matches!(
        recognize_selected_lexical_slice(&source(text)),
        SelectedLexicalSliceOutcome::UnsupportedCoverage
    ));
}

#[test]
fn retains_selected_declaration_kind_binding_order_and_initializer_state() {
    let script = recognized("let x=1, y; const z = 2;");
    assert_eq!(script.declarations().len(), 2);

    let first = &script.declarations()[0];
    assert_eq!(first.kind(), SelectedLexicalDeclarationKind::Let);
    assert_eq!(first.declaration().fragment(), "let x=1, y;");
    assert_eq!(first.bindings().len(), 2);
    assert_eq!(first.bindings()[0].binding().fragment(), "x");
    assert_eq!(
        first.bindings()[0].initializer(),
        SelectedInitializerState::SelectedPresent
    );
    assert_eq!(first.bindings()[1].binding().fragment(), "y");
    assert_eq!(
        first.bindings()[1].initializer(),
        SelectedInitializerState::Absent
    );
    assert_eq!(first.semicolon().fragment(), ";");

    let second = &script.declarations()[1];
    assert_eq!(second.kind(), SelectedLexicalDeclarationKind::Const);
    assert_eq!(second.bindings().len(), 1);
    assert_eq!(second.bindings()[0].binding().fragment(), "z");
    assert_eq!(
        second.bindings()[0].initializer(),
        SelectedInitializerState::SelectedPresent
    );
}

#[test]
fn consumes_second_slice_candidate_independent_positive_gold() {
    for fixture_id in [
        "JS-GOLD-SCRIPT-VALID-001",
        "JS-GOLD-LEXDECL-CONST-VALID-001",
        "JS-GOLD-LEXDECL-MULTIBIND-VALID-001",
        "JS-GOLD-LEXDECL-CONST-MULTIBIND-VALID-001",
        "JS-GOLD-LEXDECL-MULTIBIND-CANONICAL-DISTINCT-001",
        "JS-GOLD-LEXDECL-EE04-AWAIT-YIELD-001",
        "JS-GOLD-LEXDECL-EE04-FUTURE-RESERVED-001",
        "JS-GOLD-LEXDECL-EE04-EVAL-ARGUMENTS-001",
    ] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} must exist"));
        let _ = recognized(text);
    }
}

#[test]
fn preserves_multibyte_binding_provenance_without_normalization() {
    let multibyte = gold_source("JS-GOLD-SCRIPT-MULTIBYTE-001").expect("multibyte gold");
    let expected = gold_subject_range("JS-GOLD-SCRIPT-MULTIBYTE-001").expect("subject range");
    let script = recognized(multibyte);
    let binding = script.declarations()[0].bindings()[0].binding();
    assert_eq!(binding.fragment(), "π");
    assert_eq!((binding.range().start(), binding.range().end()), expected);

    let script = recognized("let é, e\u{0301};");
    assert_eq!(
        script.declarations()[0].bindings()[0].binding().fragment(),
        "é"
    );
    assert_eq!(
        script.declarations()[0].bindings()[1].binding().fragment(),
        "e\u{0301}"
    );
    assert_ne!(
        script.declarations()[0].bindings()[0].binding().fragment(),
        script.declarations()[0].bindings()[1].binding().fragment()
    );
}

#[test]
fn preserves_es2026_selected_identifier_boundary_for_every_binding() {
    for text in [
        "let $=1, _=2;",
        "let a0$=1, a\u{200C}=2;",
        "let a\u{0301}=1;",
        "let await, yield;",
        "let static, implements;",
        "const arguments=1, eval=2;",
        "let let;",
    ] {
        let _ = recognized(text);
    }

    for text in [
        "let if=1;",
        "let x, enum=1;",
        "const class=1;",
        "let true=1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn recognizes_complete_selected_trivia_without_generic_unicode_whitespace() {
    let _ = recognized("\u{FEFF}let\u{00A0}x=1,\u{3000}y;\u{2028}const\rz=2;\n");
    let _ = recognized("\t\u{000B}\u{000C}let\nx=1;\u{2029}");
    assert_unsupported("let\u{0085}x=1;");
}

#[test]
fn declaration_keywords_and_identifiers_do_not_split_prefixes_or_escapes() {
    for text in [
        "letx=1;",
        "letπ=1;",
        "let$=1;",
        "let_foo=1;",
        "let\\u0061=1;",
        "constx=1;",
        "constπ=1;",
        "const_foo=1;",
        "const\\u0061=1;",
        "let \\u0078=1;",
        "let x\\u0061=1;",
        "const \\u0078=1;",
        "l\\u0065t x=1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn selected_decimal_subset_is_exact_for_each_initializer() {
    for text in [
        "let x=0;",
        "let x=1, y=123;",
        "const x=0;",
        "const x=1, y=123;",
    ] {
        let _ = recognized(text);
    }

    for text in [
        "let x=1.0;",
        "let x=.1;",
        "let x=1e3;",
        "let x=1_000;",
        "let x=1n;",
        "let x=0x10;",
        "let x=01;",
        "let x=-1;",
        "const x=1/2;",
        "const x=foo;",
        "const x=/a/;",
        "const x=[1];",
        "const x={};",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn initializer_transaction_never_degrades_failed_rhs_to_absent() {
    for text in [
        gold_source("JS-GOLD-LEXDECL-CONST-IDENTIFIER-INIT-001").expect("identifier-init gold"),
        gold_source("JS-GOLD-LEXDECL-CONST-MALFORMED-INIT-001").expect("malformed-init gold"),
        "const x=/a/;",
        "const x=;",
        "let x=;",
        "let x==1;",
        "const x=>1;",
    ] {
        assert_unsupported(text);
    }

    let script = recognized("const x;");
    assert_eq!(
        script.declarations()[0].bindings()[0].initializer(),
        SelectedInitializerState::Absent
    );
}

#[test]
fn broader_script_grammar_remains_unsupported() {
    let asi_gold = gold_source("JS-GOLD-ASI-NO-FABRICATED-RANGE-001").expect("ASI gold must exist");
    assert_unsupported(asi_gold);

    for text in [
        "let x=1",
        "let x=1\nlet y=2;",
        "let/*comment*/x=1;",
        "let x=1;//comment",
        "#!node\nlet x=1;",
        "let [x]=y;",
        "let {x}=y;",
        "var x=1;",
        "'use strict'; let x=1;",
        "super.x;",
        "obj.#x;",
        "label: break label;",
        "function f(){}",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn whole_source_transaction_prevents_prefix_success_and_truncated_facts() {
    for text in [
        "",
        " ",
        "l",
        "le",
        "let",
        "let ",
        "const",
        "const ",
        "let x",
        "let x,",
        "const x=",
        "let x=1; foo();",
        "let x=1;;",
        ";let x=1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn repeated_recognition_preserves_equivalent_declaration_binding_order_and_ranges() {
    fn ranges(text: &str) -> Vec<((usize, usize), Vec<(usize, usize)>)> {
        recognized(text)
            .declarations()
            .iter()
            .map(|declaration| {
                (
                    (
                        declaration.declaration().range().start(),
                        declaration.declaration().range().end(),
                    ),
                    declaration
                        .bindings()
                        .iter()
                        .map(|binding| {
                            (
                                binding.binding().range().start(),
                                binding.binding().range().end(),
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }

    let first = ranges("let π=1, x; const y=2;");
    let second = ranges("let π=1, x; const y=2;");
    assert_eq!(first, second);
}
