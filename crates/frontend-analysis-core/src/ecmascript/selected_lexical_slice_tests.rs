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

fn escaped_first_ascii_code_point(name: &str) -> String {
    assert!(name.is_ascii());
    let mut chars = name.chars();
    let first = chars
        .next()
        .expect("reserved-word control must be non-empty");
    format!(r"\u{:04X}{}", first as u32, chars.as_str())
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
            assert_eq!(
                (semicolon.range().start(), semicolon.range().end()),
                (10, 11)
            );
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
        assert_eq!(
            declaration.declaration().fragment(),
            expected_declaration,
            "{text}"
        );
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
    let text =
        "let x = 1 \t\n\r\u{2028}\u{2029}\u{00A0}\u{1680}\u{2000}\u{202F}\u{205F}\u{3000}\u{FEFF}";
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
        "const x=/a/;",
        "const x=[1];",
        "const x={};",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn recognizes_escape_free_identifier_reference_initializers_as_presence_only() {
    for text in [
        gold_source("JS-GOLD-LEXDECL-CONST-IDENTIFIER-INIT-001").expect("identifier-init gold"),
        "const x = foo",
        "let x = foo;",
        "let x = foo",
        "let x = foo, y = bar;",
        "let x = 1, y = foo;",
        "let x = foo, y = 1;",
        "const x = foo, y = bar;",
        "let x = 1; const y = foo",
        "let x = foo; const y = bar",
        "const π = 𝒜;",
        r"const \u0078 = foo;",
    ] {
        let script = recognized(text);
        assert!(
            script
                .declarations()
                .iter()
                .flat_map(|declaration| declaration.bindings())
                .any(|binding| binding.initializer() == SelectedInitializerState::SelectedPresent)
        );
    }
}

#[test]
fn identifier_reference_direct_code_point_family_is_recognized_without_normalization() {
    for text in [
        "const x = $;",
        "const x = _;",
        "const x = a0;",
        "const x = a\u{0301};",
        "const x = a\u{200C}b;",
        "const x = a\u{200D}b;",
        "const x = 𝒜;",
        "const x = ifx;",
    ] {
        let _ = recognized(text);
    }

    for text in [
        "const x = 0a;",
        "const x = -foo;",
        "const x = foo-bar;",
        "const x = foo💥;",
    ] {
        assert_unsupported(text);
    }

    let composed = recognized("const x = é; const y = e\u{0301};");
    assert_eq!(composed.declarations().len(), 2);
}

#[test]
fn identifier_reference_name_policy_is_fixed_to_non_strict_script_yield_false_await_false() {
    for name in [
        "let",
        "static",
        "implements",
        "interface",
        "package",
        "private",
        "protected",
        "public",
        "yield",
        "await",
        "eval",
        "arguments",
    ] {
        let text = format!("const x = {name};");
        let _ = recognized(&text);
    }

    for name in [
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "new",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
    ] {
        let text = format!("const x = {name};");
        assert_unsupported(&text);
    }
}

#[test]
fn recognizes_escaped_identifier_reference_initializers_as_presence_only_and_maximal() {
    for text in [
        r"const x = \u0066oo;",
        r"const x = f\u006Fo;",
        r"let x = \u{66}oo, y = bar;",
        r"const x = \u{00000066}oo",
        r"const x = \u0066\u006F\u006F;",
        r"const x = \u{1D49C};",
        r"const x = \u0024;",
        r"const x = \u005F;",
        r"const x = a\u0030;",
        r"const x = a\u0301;",
        r"const x = a\u200Cb;",
        r"const x = a\u200Db;",
        r"const \u0078 = \u0066oo;",
        r"let x = \u0066oo, y = bar;",
        r"let x = foo, y = \u0062ar;",
        r"let x = 1, y = \u0062ar;",
        r"let x = \u0066oo, y = 1;",
        r"const x = \u0066oo, y = \u0062ar;",
        r"const x = \u00E9; const y = e\u0301;",
        r"const x = \u0069fx;",
    ] {
        let script = recognized(text);
        assert!(
            script
                .declarations()
                .iter()
                .flat_map(|declaration| declaration.bindings())
                .any(|binding| binding.initializer() == SelectedInitializerState::SelectedPresent),
            "{text}"
        );
    }
}

#[test]
fn escaped_identifier_reference_name_policy_matches_fixed_selected_context() {
    for rhs in [
        r"\u006Cet",
        r"\u0073tatic",
        r"\u0069mplements",
        r"\u0069nterface",
        r"\u0070ackage",
        r"\u0070rivate",
        r"\u0070rotected",
        r"\u0070ublic",
        r"\u0079ield",
        r"a\u0077ait",
        r"\u0065val",
        r"\u0061rguments",
    ] {
        let text = format!("const x = {rhs};");
        let _ = recognized(&text);
    }

    for name in [
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "new",
        "null",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
    ] {
        let rhs = escaped_first_ascii_code_point(name);
        let text = format!("const x = {rhs};");
        assert_unsupported(&text);
    }
}

#[test]
fn escaped_identifier_reference_invalid_and_unowned_rhs_remain_unsupported() {
    for text in [
        r"const x = \u{};",
        r"const x = \u0;",
        r"const x = \u{61",
        r"const x = \u{110000};",
        r"const x = \u{G};",
        r"const x = \u00G0;",
        r"const x = \u{61;",
        r"const x = \u0x;",
        r"const x = \u;",
        r"const x = \u{",
        r"const x = \u0030;",
        r"const x = a\u002D;",
        r"const x = \uD800;",
        r"const x = \u{D800};",
        r"const x = \u200C;",
        r"const x = \u200D;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn escaped_identifier_reference_valid_atom_does_not_widen_expression_or_asi_coverage() {
    for text in [
        r"const x = \u0066oo.bar;",
        r"const x = \u0066oo();",
        r"const x = \u0066oo + 1;",
        r"const x = \u0066oo = bar;",
        r"const x = \u0066oo ? bar : baz;",
        r"const x = \u0066oo/*comment*/;",
        r"const x = \u0066oo unexpected;",
        r"const x = \u0066oo;;",
        "const x = \\u0066oo\nconst y = bar;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn escaped_identifier_reference_eof_asi_preserves_complete_authored_significant_end() {
    let text = r"const x = \u0066oo ";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), r"const x = \u0066oo");
    assert_eq!(
        (
            declaration.declaration().range().start(),
            declaration.declaration().range().end()
        ),
        (0, 18)
    );
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn escaped_identifier_reference_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = \u0066oo; let \u{};", r"\u{}", (24, 28)),
        (r"const x = \u0066oo; let a\u{};", r"\u{}", (25, 29)),
        (r"const x = \u0066oo; let \u{61", r"\u{61", (24, 29)),
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
fn escaped_identifier_reference_pathological_inputs_remain_linear_and_deterministic() {
    let zeros = "0".repeat(4096);
    let long_escape = format!(r"const x = \u{{{zeros}66}}oo;");
    let first = recognized(&long_escape);
    let second = recognized(&long_escape);
    assert_eq!(
        first.declarations()[0].declaration().fragment(),
        second.declarations()[0].declaration().fragment()
    );

    let long_utf8 = format!(r"const x = \u0061{};", "α".repeat(4096));
    let first = recognized(&long_utf8);
    let second = recognized(&long_utf8);
    assert_eq!(
        first.declarations()[0].declaration().fragment(),
        second.declarations()[0].declaration().fragment()
    );
}

#[test]
fn identifier_reference_eof_asi_preserves_significant_end_before_selected_trivia() {
    let text = "const x = foo \t\n\r\u{2028}\u{2029}\u{00A0}\u{1680}\u{2000}\u{202F}\u{205F}\u{3000}\u{FEFF}";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = foo");
    assert_eq!(
        (
            declaration.declaration().range().start(),
            declaration.declaration().range().end()
        ),
        (0, 13)
    );
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn identifier_reference_prefixes_do_not_widen_richer_expression_or_escape_coverage() {
    for text in [
        "const x = foo.bar;",
        "const x = foo();",
        "const x = foo + 1;",
        "const x = (foo);",
        "const x = foo = bar;",
        "const x = foo ? bar : baz;",
        "const x = foo/*comment*/;",
        "const x = foo unexpected;",
        "const x = foo; foo()",
        "const x = foo;;",
        "; const x = foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn identifier_reference_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = foo; let \u{};", r"\u{}", (19, 23)),
        (r"const x = foo; let a\u{};", r"\u{}", (20, 24)),
        (r"const x = foo; let \u{61", r"\u{61", (19, 24)),
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
fn initializer_transaction_never_degrades_failed_rhs_to_absent() {
    for text in [
        gold_source("JS-GOLD-LEXDECL-CONST-MALFORMED-INIT-001").expect("malformed-init gold"),
        "const x=/a/;",
        "const x=;",
        "let x=;",
        "let x==1;",
        "const x=>1;",
        "const x=if;",
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
        "const x=foo; foo();",
        "const x=foo\nfoo();",
        "const x=foo;;",
        ";const x=foo;",
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
        "let x = foo\nconst y = bar;",
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
    assert_unsupported(r"let \u0030; foo();");

    let script = recognized(r"let \u0030 = foo;");
    let binding = &script.declarations()[0].bindings()[0];
    assert_eq!(
        binding.initializer(),
        SelectedInitializerState::SelectedPresent
    );
    match binding.name_state() {
        SelectedBindingNameState::InvalidEscapedPosition { position, escape } => {
            assert_eq!(*position, SelectedInvalidEscapePosition::Start);
            assert_eq!(escape.fragment(), r"\u0030");
            assert_eq!((escape.range().start(), escape.range().end()), (4, 10));
        }
        other => panic!("expected invalid escaped start, got {other:?}"),
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

    let long = format!("const x = a{};", "α".repeat(4096));
    let first = ranges(&long);
    let second = ranges(&long);
    assert_eq!(first, second);
}

#[test]
fn direct_boolean_literal_initializers_compose_as_presence_only() {
    for text in [
        "const x = true;",
        "const x = false;",
        "const x = true",
        "const x = false   ",
        "let x = true, y = foo;",
        "let x = foo, y = false;",
        "let x = 1, y = true;",
        "const x = true, y = false;",
        "const x = true \t\n;",
    ] {
        let script = recognized(text);
        assert!(
            script
                .declarations()
                .iter()
                .flat_map(|declaration| declaration.bindings())
                .any(|binding| binding.initializer() == SelectedInitializerState::SelectedPresent),
            "{text:?}"
        );
    }
}

#[test]
fn direct_boolean_literal_boundary_preserves_maximal_identifier_reference_routing() {
    for text in [
        "const x = truex;",
        "const x = falseValue;",
        "const x = trueπ;",
        "const x = true0;",
        "const x = true$;",
        "const x = true_;",
        r"const x = true\u0061;",
        r"const x = false\u0061;",
        r"const x = true\u{61};",
        r"const x = true\u{00000061};",
        r"const x = true\u0030;",
        r"const x = true\u200C;",
        r"const x = true\u200D;",
    ] {
        let _ = recognized(text);
    }

    for text in [
        r"const x = true\u002D;",
        r"const x = true\uD800;",
        r"const x = true\u{D800};",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_boolean_literal_does_not_claim_escaped_malformed_or_richer_neighbors() {
    for text in [
        r"const x = \u0074rue;",
        r"const x = \u0066alse;",
        r"const x = t\u0072ue;",
        r"const x = f\u0061lse;",
        r"const x = true\u{};",
        r"const x = false\u0;",
        r"const x = true\u{61",
        "const x = true.foo;",
        "const x = false();",
        "const x = true + x;",
        "const x = false = x;",
        "const x = true ? x : y;",
        "const x = true/*comment*/;",
        "const x = true unexpected;",
        "const x = true;;",
        "const x = true\nconst y = foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_boolean_literal_eof_asi_preserves_significant_end_before_selected_trivia() {
    let text = "const x = false   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = false");
    assert_eq!(
        (
            declaration.declaration().range().start(),
            declaration.declaration().range().end()
        ),
        (0, 15)
    );
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn direct_boolean_literal_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = true; let \u{};", r"\u{}", (20, 24)),
        (r"const x = true; let a\u{};", r"\u{}", (21, 25)),
        (r"const x = true; let \u{61", r"\u{61", (20, 25)),
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
fn direct_boolean_literal_aggregate_lifecycle_remains_incomplete_or_existing_rejection() {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = true;",
        "const x = false;",
        "let x = true, y = foo;",
        "const x = true, y = false;",
    ] {
        assert!(
            matches!(
                attempt_selected_qualification(&source(text)),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }

    for (text, expected_fragment, expected_range) in [
        (r"let \u0030 = true;", r"\u0030", (4, 10)),
        (r"let \u0069f = true;", r"\u0069f", (4, 11)),
        ("let let = true;", "let", (4, 7)),
        ("let x = true, x = foo;", "x", (14, 15)),
        ("const x = true, y;", "y", (16, 17)),
        ("let x = true; let x = foo;", "x", (18, 19)),
    ] {
        let SelectedQualificationAttempt::Outcome(outcome) =
            attempt_selected_qualification(&source(text))
        else {
            panic!("expected static rejection for {text}");
        };
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome
            .rejection_evidence()
            .expect("static rejection evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let subject = evidence
            .subject()
            .authored_anchor()
            .expect("static subject must remain authored");
        assert_eq!(subject.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (subject.range().start(), subject.range().end()),
            expected_range,
            "{text}"
        );
    }

    let SelectedQualificationAttempt::Outcome(outcome) =
        attempt_selected_qualification(&source(r"const x = true; let \u{};"))
    else {
        panic!("expected existing Grammar rejection to become reachable");
    };
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("Grammar rejection evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar);
    let subject = evidence
        .subject()
        .authored_anchor()
        .expect("Grammar subject must remain authored");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (20, 24));
}

#[test]
fn direct_null_literal_initializers_compose_as_presence_only() {
    for text in [
        "const x = null;",
        "const x = null",
        "const x = null   \t\n",
        "let x = null, y = foo;",
        "let x = foo, y = null;",
        "let x = 1, y = null;",
        "const x = null, y = false;",
        "const x = true, y = null;",
        r"const x = \u0066oo, y = null;",
        "const x = null \t\n;",
    ] {
        let script = recognized(text);
        assert!(
            script
                .declarations()
                .iter()
                .flat_map(|declaration| declaration.bindings())
                .any(|binding| binding.initializer() == SelectedInitializerState::SelectedPresent),
            "{text:?}"
        );
    }
}

#[test]
fn direct_null_literal_boundary_preserves_maximal_identifier_reference_routing() {
    for text in [
        "const x = nullx;",
        "const x = nullValue;",
        "const x = nullπ;",
        "const x = null0;",
        "const x = null$;",
        "const x = null_;",
        r"const x = null\u0061;",
        r"const x = null\u{61};",
        r"const x = null\u{00000061};",
        r"const x = null\u0030;",
        r"const x = null\u200C;",
        r"const x = null\u200D;",
    ] {
        let _ = recognized(text);
    }

    for text in [
        r"const x = null\u002D;",
        r"const x = null\uD800;",
        r"const x = null\u{D800};",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_null_literal_does_not_claim_escaped_malformed_or_richer_neighbors() {
    for text in [
        r"const x = \u006Eull;",
        r"const x = n\u0075ll;",
        r"const x = nu\u006Cl;",
        r"const x = nul\u006C;",
        r"const x = null\u{};",
        r"const x = null\u0;",
        r"const x = null\u{61",
        r"const x = null\u{110000};",
        "const x = null.foo;",
        "const x = null();",
        "const x = null + x;",
        "const x = null = x;",
        "const x = null ? x : y;",
        "const x = null/*comment*/;",
        "const x = null unexpected;",
        "const x = null;;",
        "const x = null\nconst y = foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_null_literal_eof_asi_preserves_significant_end_before_selected_trivia() {
    let text = "const x = null   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = null");
    assert_eq!(
        (
            declaration.declaration().range().start(),
            declaration.declaration().range().end()
        ),
        (0, 14)
    );
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn direct_null_literal_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = null; let \u{};", r"\u{}", (20, 24)),
        (r"const x = null; let a\u{};", r"\u{}", (21, 25)),
        (r"const x = null; let \u{61", r"\u{61", (20, 25)),
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
fn direct_null_literal_aggregate_lifecycle_remains_incomplete_or_existing_rejection() {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = null;",
        "const x = null",
        "let x = null, y = foo;",
        "const x = true, y = null;",
    ] {
        assert!(
            matches!(
                attempt_selected_qualification(&source(text)),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }

    for (text, expected_fragment, expected_range) in [
        (r"let \u0030 = null;", r"\u0030", (4, 10)),
        (r"let \u0069f = null;", r"\u0069f", (4, 11)),
        ("let let = null;", "let", (4, 7)),
        ("let x = null, x = foo;", "x", (14, 15)),
        ("const x = null, y;", "y", (16, 17)),
        ("let x = null; let x = foo;", "x", (18, 19)),
    ] {
        let SelectedQualificationAttempt::Outcome(outcome) =
            attempt_selected_qualification(&source(text))
        else {
            panic!("expected static rejection for {text}");
        };
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome
            .rejection_evidence()
            .expect("static rejection evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let subject = evidence
            .subject()
            .authored_anchor()
            .expect("static subject must remain authored");
        assert_eq!(subject.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (subject.range().start(), subject.range().end()),
            expected_range,
            "{text}"
        );
    }

    let SelectedQualificationAttempt::Outcome(outcome) =
        attempt_selected_qualification(&source(r"const x = null; let \u{};"))
    else {
        panic!("expected existing Grammar rejection to become reachable");
    };
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("Grammar rejection evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar);
    let subject = evidence
        .subject()
        .authored_anchor()
        .expect("Grammar subject must remain authored");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (20, 24));
}
