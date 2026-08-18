use crate::{SourceAnchor, SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::selected_lexical_slice::{
    SelectedBindingNameState, SelectedDeclarationTerminator, SelectedInitializerState,
    SelectedInvalidEscapePosition, SelectedLexicalDeclarationKind, SelectedLexicalScript,
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
        SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { .. } => {
            panic!("expected selected-slice recognition, got grammar rejection for {text:?}")
        }
        SelectedLexicalSliceOutcome::ResourceLimited => {
            panic!("unexpected resource limitation for {text:?}")
        }
        SelectedLexicalSliceOutcome::InternalFailure => {
            panic!("unexpected internal failure for {text:?}")
        }
    }
}

fn grammar_rejection(text: &str) -> SourceAnchor {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { subject } => subject,
        other => panic!("expected grammar rejection for {text:?}, got {other:?}"),
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
    match first.terminator() {
        SelectedDeclarationTerminator::AuthoredSemicolon(semicolon) => {
            assert_eq!(semicolon.fragment(), ";");
            assert_eq!((semicolon.range().start(), semicolon.range().end()), (10, 11));
        }
        SelectedDeclarationTerminator::AutomaticAtEof => {
            panic!("explicit semicolon must retain authored terminator provenance")
        }
    }

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
fn recognizes_eof_only_asi_with_truthful_terminator_provenance() {
    for (text, expected_declaration) in [
        ("let x = 1", "let x = 1"),
        ("const x = 1", "const x = 1"),
        ("let x", "let x"),
        ("let x, y", "let x, y"),
        ("const x = 1, y = 2", "const x = 1, y = 2"),
    ] {
        let script = recognized(text);
        let declaration = &script.declarations()[0];
        assert_eq!(declaration.declaration().fragment(), expected_declaration, "{text}");
        assert!(matches!(
            declaration.terminator(),
            SelectedDeclarationTerminator::AutomaticAtEof
        ));
    }

    let script = recognized("let x; const y = 1");
    assert_eq!(script.declarations().len(), 2);
    assert!(matches!(
        script.declarations()[0].terminator(),
        SelectedDeclarationTerminator::AuthoredSemicolon(_)
    ));
    assert!(matches!(
        script.declarations()[1].terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn eof_asi_excludes_selected_trailing_trivia_from_declaration_anchor() {
    let text = "let x = 1 \t\n\r\u{2028}\u{2029}\u{00A0}\u{1680}\u{2000}\u{202F}\u{205F}\u{3000}\u{FEFF}";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "let x = 1");
    assert_eq!(
        (
            declaration.declaration().range().start(),
            declaration.declaration().range().end()
        ),
        (0, 9)
    );
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));

    let again = recognized(text);
    assert_eq!(
        (
            again.declarations()[0].declaration().range().start(),
            again.declarations()[0].declaration().range().end()
        ),
        (0, 9)
    );
}

#[test]
fn eof_asi_composes_with_existing_escaped_binding_capability() {
    let script = recognized(r"let \u0061");
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), r"let \u0061");
    assert_eq!(declaration.bindings()[0].binding().fragment(), r"\u0061");
    assert_eq!(declaration.bindings()[0].semantic_name(), Some("a"));
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
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
        "l\\u0065t x=1;",
    ] {
        assert_unsupported(text);
    }

    for text in ["let \\u0078=1;", "let x\\u0061=1;", "const \\u0078=1;"] {
        let _ = recognized(text);
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
        "let x,",
        "const x=",
        "let x=1; foo();",
        "let x=1\nfoo();",
        "let x=1;;",
        ";let x=1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn eof_asi_keeps_non_eof_incomplete_and_deferred_neighbors_unsupported() {
    for text in [
        "let x\nconst y = 1",
        "let x =",
        "let x,",
        "let x y",
        "const x = foo",
        "let x/*comment*/",
        "#!node\nlet x = 1",
        r"let \u",
        r"let \u{",
        r"let \u0",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn escaped_binding_state_preserves_authored_and_decoded_identity_separately() {
    let script = recognized(r"let \u0061, a;");
    let first = &script.declarations()[0].bindings()[0];
    assert_eq!(first.binding().fragment(), r"\u0061");
    match first.name_state() {
        SelectedBindingNameState::EscapedValid { decoded } => assert_eq!(decoded, "a"),
        other => panic!("expected escaped valid state, got {other:?}"),
    }
    assert_eq!(first.semantic_name(), Some("a"));

    let second = &script.declarations()[0].bindings()[1];
    assert!(matches!(
        second.name_state(),
        SelectedBindingNameState::Unescaped
    ));
    assert_eq!(second.semantic_name(), Some("a"));

    let script = recognized(r"let \u0030;");
    let invalid = &script.declarations()[0].bindings()[0];
    match invalid.name_state() {
        SelectedBindingNameState::InvalidEscapedPosition { position, escape } => {
            assert_eq!(*position, SelectedInvalidEscapePosition::Start);
            assert_eq!(escape.fragment(), r"\u0030");
            assert_eq!((escape.range().start(), escape.range().end()), (4, 10));
        }
        other => panic!("expected invalid escaped start, got {other:?}"),
    }
    assert_eq!(invalid.semantic_name(), None);
}

#[test]
fn escaped_binding_recognition_separates_formed_invalid_from_bounded_grammar_rejection() {
    for text in [
        r"let \u0030;",
        r"let a\u002D;",
        r"let \uD800;",
        r"let \u{D800};",
    ] {
        let _ = recognized(text);
    }

    for (text, expected_fragment, expected_range) in [
        (r"let \u{};", r"\u{}", (4, 8)),
        (r"let \u0;", r"\u0", (4, 7)),
        (r"let \u{61", r"\u{61", (4, 9)),
        (r"let \u{110000};", r"\u{110000}", (4, 14)),
        (r"let a\u{};", r"\u{}", (5, 9)),
        (r"let a\u0;", r"\u0", (5, 8)),
        (r"let a\u{61", r"\u{61", (5, 10)),
        (r"let a\u{110000};", r"\u{110000}", (5, 15)),
    ] {
        let subject = grammar_rejection(text);
        assert_eq!(subject.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (subject.range().start(), subject.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn formed_unicode_escape_extends_literal_keyword_candidate_without_backtracking() {
    for text in [
        r"let\u0030;",
        r"let\u002D;",
        r"let\u00001;",
        r"const\u0030;",
        r"let\u{00000061};",
        r"let\u002D\u{};",
    ] {
        assert_unsupported(text);
    }

    for (text, expected_range) in [(r"let\u{};", (3, 7)), (r"let\u{110000};", (3, 13))] {
        let subject = grammar_rejection(text);
        assert_eq!(
            (subject.range().start(), subject.range().end()),
            expected_range,
            "{text}"
        );
    }

    for text in [r"let\u0;", r"let\u{61", r"const\u{};"] {
        assert_unsupported(text);
    }
}

#[test]
fn adjacent_malformed_classes_remain_unsupported() {
    for text in [
        r"let \u{G};",
        r"let \u00G0;",
        r"let \u{61;",
        r"let \u0x;",
        r"let \u;",
        r"let \u{",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn escaped_binding_whole_source_transaction_distinguishes_unsupported_tail_from_owned_grammar() {
    for text in [r"let \u0030; foo();", r"let \u0030 = foo;"] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"let a\u00001\u{};");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (12, 16));
}

#[test]
fn selected_binding_recognition_accepts_long_and_contextual_escaped_names() {
    for text in [
        r"let \u{00000061};",
        r"let \u0061wait, \u0079ield, \u0073tatic, \u0069mplements, \u0061rguments, \u0065val;",
        r"let \u0069f;",
        r"let \u006Cet;",
        "let \\u{1D49C}, 𝒜;",
    ] {
        let _ = recognized(text);
    }
}

#[test]
fn repeated_recognition_preserves_equivalent_declaration_binding_order_and_ranges() {
    type ByteRange = (usize, usize);
    type DeclarationRanges = (ByteRange, Vec<ByteRange>);

    fn ranges(text: &str) -> Vec<DeclarationRanges> {
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

    let first = ranges("let π=1, x \u{2028}\u{00A0}");
    let second = ranges("let π=1, x \u{2028}\u{00A0}");
    assert_eq!(first, second);
}
