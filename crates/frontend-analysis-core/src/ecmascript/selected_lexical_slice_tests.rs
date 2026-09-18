use crate::{SourceAnchor, SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::selected_lexical_slice::{
    SelectedBindingNameState, SelectedBlockReferenceUseEnabledScript,
    SelectedBlockReferenceUseEnabledTopLevelItem, SelectedDeclarationTerminator,
    SelectedIdentifierReferenceExpressionStatementScript, SelectedInitializerState,
    SelectedInvalidEscapePosition, SelectedLexicalDeclarationKind, SelectedLexicalScript,
    SelectedLexicalSliceOutcome, SelectedReferenceUseEnabledTopLevelItem,
    SelectedUseSiteEnabledBlockItem, SelectedVariableStatement, SelectedVariableStatementScript,
    SelectedVariableStatementTerminator, SelectedVariableTopLevelItem,
    recognize_selected_lexical_slice,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(218), text.to_owned())
}

fn recognized(text: &str) -> SelectedLexicalScript {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_) => {
            panic!("expected flat selected-slice recognition, got Block-enabled slice for {text:?}")
        }
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(_) => {
            panic!("expected flat selected-slice recognition, got var-enabled slice for {text:?}")
        }
        SelectedLexicalSliceOutcome::RecognizedIdentifierReferenceExpressionStatementSlice(_) => {
            panic!(
                "expected flat selected-slice recognition, got reference-use-enabled slice for {text:?}"
            )
        }
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(_) => {
            panic!(
                "expected flat selected-slice recognition, got Block-reference-use-enabled slice for {text:?}"
            )
        }
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

fn recognized_block(text: &str) -> super::selected_lexical_slice::SelectedOneLevelBlockScript {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(script) => script,
        other => panic!("expected one-level Block recognition for {text:?}, got {other:?}"),
    }
}

fn recognized_variable(text: &str) -> SelectedVariableStatementScript {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(script) => script,
        other => panic!("expected var-enabled recognition for {text:?}, got {other:?}"),
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

fn only_variable_statement(script: &SelectedVariableStatementScript) -> &SelectedVariableStatement {
    let [SelectedVariableTopLevelItem::VariableStatement(statement)] = script.items() else {
        panic!("expected exactly one selected VariableStatement item");
    };
    statement
}

fn binding_ranges(statement: &SelectedVariableStatement) -> Vec<(usize, usize)> {
    statement
        .bindings()
        .iter()
        .map(|binding| {
            (
                binding.binding().range().start(),
                binding.binding().range().end(),
            )
        })
        .collect()
}

fn binding_fragments(statement: &SelectedVariableStatement) -> Vec<&str> {
    statement
        .bindings()
        .iter()
        .map(|binding| binding.binding().fragment())
        .collect()
}

fn binding_semantic_names(statement: &SelectedVariableStatement) -> Vec<Option<&str>> {
    statement
        .bindings()
        .iter()
        .map(|binding| binding.semantic_name())
        .collect()
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
        // Issue #730 makes the selected `LexicalDeclaration` initializer
        // position additionally admit a plain fractional `DecimalLiteral`;
        // see the "Issue #730" test section below for full coverage.
        "let x=1.0;",
        "let x=.1;",
        // Issue #738 makes the selected `LexicalDeclaration` initializer
        // position additionally admit a plain exponent `DecimalLiteral`;
        // see the "Issue #738" test section below for full coverage.
        "let x=1e3;",
    ] {
        let _ = recognized(text);
    }

    // "let x=-1;" is deliberately not listed below: Issue #744 makes a
    // direct-authored leading `+`/`-` decimal `UnaryExpression` initializer
    // a selected accepted form (see the "Issue #744" test section below for
    // full coverage).
    for text in [
        "let x=1_000;",
        "let x=1n;",
        "let x=0x10;",
        "let x=01;",
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

    for text in ["const x = 0a;", "const x = !foo;", "const x = foo💥;"] {
        assert_unsupported(text);
    }

    // Issue #754 stale fixture migration: `foo-bar` was `UnsupportedCoverage`
    // (a valid `foo` `IdentifierReference` initializer followed by trailing
    // `-bar` the declaration terminator check then rejected) before the
    // selected two-`IdentifierReference` additive initializer landed. It is
    // now the selected `SelectedTwoIdentifierReferenceAdditiveInitializer`
    // theorem fixed by #752/#753: exactly two ordered facts, `foo` then
    // `bar`.
    let migrated = recognized("const x = foo-bar;");
    let facts: Vec<_> = migrated.declarations()[0].bindings()[0]
        .identifier_reference_initializer_facts()
        .collect();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].semantic_name(), "foo");
    assert_eq!(facts[1].semantic_name(), "bar");

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
        let script = recognized(&text);
        let binding = &script.declarations()[0].bindings()[0];
        assert_eq!(
            binding.initializer(),
            SelectedInitializerState::SelectedPresent
        );
        assert!(binding.escaped_reserved_initializer_identifier().is_none());
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
        let script = recognized(&text);
        let binding = &script.declarations()[0].bindings()[0];
        assert_eq!(
            binding.initializer(),
            SelectedInitializerState::SelectedPresent
        );
        let identifier = binding
            .escaped_reserved_initializer_identifier()
            .expect("escaped reserved initializer evidence");
        assert_eq!(identifier.fragment(), rhs);
        assert_eq!(
            (identifier.range().start(), identifier.range().end()),
            (10, 10 + rhs.len())
        );
    }
}

#[test]
fn escaped_reserved_initializer_evidence_is_exact_and_direct_owners_remain_separate() {
    for rhs in [
        r"\u0074his",
        r"\u006Eull",
        r"\u0074rue",
        r"\u0066alse",
        r"\u0069f",
        r"\u0063lass",
        r"\u0069mport",
        r"\u0065xport",
        r"n\u0075ll",
        r"tr\u0075e",
        r"thi\u0073",
    ] {
        let text = format!("const x = {rhs};");
        let script = recognized(&text);
        let binding = &script.declarations()[0].bindings()[0];
        assert_eq!(
            binding.initializer(),
            SelectedInitializerState::SelectedPresent
        );
        let identifier = binding
            .escaped_reserved_initializer_identifier()
            .expect("escaped reserved initializer must retain authored evidence");
        assert_eq!(identifier.fragment(), rhs);
        assert_eq!(
            (identifier.range().start(), identifier.range().end()),
            (10, 10 + rhs.len())
        );
    }

    for text in [
        "const x = true;",
        "const x = false;",
        "const x = null;",
        "const x = this;",
    ] {
        let script = recognized(text);
        let binding = &script.declarations()[0].bindings()[0];
        assert!(binding.escaped_reserved_initializer_identifier().is_none());
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
    // Issue #758: `let\u0030;`, `const\u0030;`, and
    // `let\u{00000061};` decode to the accepted non-ReservedWord names
    // `let0`/`const0`/`leta` and are now selected top-level
    // `IdentifierReference` `ExpressionStatement` use-sites (proved by
    // `top_level_identifier_reference_expression_statement_use_site_tests`
    // in this file); `let\u002D;` (decodes to `-`, an invalid
    // `IdentifierPart`) and `let\u00001;`/`let\u002D\u{};`
    // (invalid start / malformed continuation) remain outside every leaf
    // and stay `UnsupportedCoverage` here, unaffected by the new use-site
    // probe.
    for text in [r"let\u002D;", r"let\u00001;", r"let\u002D\u{};"] {
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

#[test]
fn direct_this_initializers_compose_as_presence_only() {
    for text in [
        "const x = this;",
        "const x = this",
        "const x = this   \t\n",
        "let x = this, y = foo;",
        "let x = foo, y = this;",
        "let x = 1, y = this;",
        "const x = this, y = null;",
        "const x = null, y = this;",
        "const x = true, y = this;",
        r"const x = \u0066oo, y = this;",
        "const x = this \t\n;",
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
fn direct_this_boundary_preserves_maximal_identifier_reference_routing() {
    for text in [
        "const x = thisx;",
        "const x = thisValue;",
        "const x = thisπ;",
        "const x = this0;",
        "const x = this$;",
        "const x = this_;",
        r"const x = this\u0061;",
        r"const x = this\u{61};",
        r"const x = this\u{00000061};",
        r"const x = this\u0030;",
        r"const x = this\u200C;",
        r"const x = this\u200D;",
    ] {
        let _ = recognized(text);
    }

    for text in [
        r"const x = this\u002D;",
        r"const x = this\uD800;",
        r"const x = this\u{D800};",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_this_does_not_claim_other_frontiers() {
    for text in [
        r"const x = this\u{};",
        r"const x = this\u0;",
        r"const x = this\u{61",
        r"const x = this\u{110000};",
        "const x = this.foo;",
        "const x = this();",
        "const x = this + x;",
        "const x = this = x;",
        "const x = this ? x : y;",
        "const x = this/*comment*/;",
        "const x = this unexpected;",
        "const x = this;;",
        "const x = this\nconst y = foo;",
        "let this = 1;",
        "let this;",
        "this;",
        "const x = this, this;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_this_eof_asi_preserves_significant_end_before_selected_trivia() {
    let text = "const x = this   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = this");
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
fn direct_this_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = this; let \u{};", r"\u{}", (20, 24)),
        (r"const x = this; let a\u{};", r"\u{}", (21, 25)),
        (r"const x = this; let \u{61", r"\u{61", (20, 25)),
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
fn direct_this_aggregate_lifecycle_remains_incomplete_or_existing_rejection() {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = this;",
        "const x = this",
        "let x = this, y = foo;",
        "const x = null, y = this;",
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
        (r"let \u0030 = this;", r"\u0030", (4, 10)),
        (r"let \u0069f = this;", r"\u0069f", (4, 11)),
        ("let let = this;", "let", (4, 7)),
        ("let x = this, x = foo;", "x", (14, 15)),
        ("const x = this, y;", "y", (16, 17)),
        ("let x = this; let x = foo;", "x", (18, 19)),
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
        attempt_selected_qualification(&source(r"const x = this; let \u{};"))
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
fn escape_free_string_literal_initializers_compose_as_presence_only() {
    for text in [
        "const x = \"\";",
        "const x = '';",
        "const x = \"abc\";",
        "const x = 'abc';",
        "const x = \"𝒜\";",
        "const x = \"a'b\";",
        "const x = 'a\"b';",
        "const x = \"a b\";",
        "const x = \"a\tb\";",
        "const x = \"a\u{FEFF}b\";",
        "const x = \"a\u{2028}b\";",
        "const x = 'a\u{2029}b';",
        "const x = \"abc\"",
        "const x = \"abc\"   \t\n",
        "let x = \"abc\", y = foo;",
        "let x = foo, y = \"abc\";",
        "let x = 1, y = \"abc\";",
        "const x = \"abc\", y = null;",
        "const x = this, y = \"abc\";",
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
fn escape_free_string_literal_raw_line_terminator_boundary_is_exact() {
    for text in [
        "const x = \"a\nb\";",
        "const x = \"a\rb\";",
        "const x = \"a\r\nb\";",
        "const x = 'a\nb';",
        "const x = 'a\rb';",
    ] {
        assert_unsupported(text);
    }

    for text in [
        "const x = \"a\u{2028}b\";",
        "const x = \"a\u{2029}b\";",
        "const x = 'a\u{2028}b';",
        "const x = 'a\u{2029}b';",
    ] {
        let _ = recognized(text);
    }
}

#[test]
fn escape_free_string_literal_does_not_claim_escape_malformed_or_richer_frontiers() {
    for text in [
        r#"const x = "\n";"#,
        r#"const x = "\u0061";"#,
        r#"const x = "\x61";"#,
        r#"const x = "\\";"#,
        r#"const x = "\"";"#,
        "const x = '\\'';",
        "const x = \"\\\n\";",
        "const x = '\\\r\n';",
        "const x = \"abc;",
        "const x = 'abc;",
        "const x = \"abc';",
        "const x = 'abc\";",
        "const x = \"abc\".length;",
        "const x = \"abc\"();",
        "const x = \"abc\" + x;",
        "const x = \"abc\" = x;",
        "const x = \"abc\" ? x : y;",
        "const x = \"abc\"/*comment*/;",
        "const x = \"abc\" unexpected;",
        "const x = \"abc\";;",
        "const x = \"abc\"\nconst y = foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn escape_free_string_literal_eof_asi_preserves_closing_quote_significant_end() {
    let text = "const x = \"abc\"   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = \"abc\"");
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
fn escape_free_string_literal_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r#"const x = "abc"; let \u{};"#, r"\u{}", (21, 25)),
        (r#"const x = "abc"; let a\u{};"#, r"\u{}", (22, 26)),
        (r#"const x = "abc"; let \u{61"#, r"\u{61", (21, 26)),
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
fn escape_free_string_literal_aggregate_lifecycle_remains_incomplete_or_existing_rejection() {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = \"abc\";",
        "const x = 'abc';",
        "const x = \"a\u{2028}b\";",
        "let x = \"abc\", y = foo;",
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
        (r#"let \u0030 = "abc";"#, r"\u0030", (4, 10)),
        (r#"let \u0069f = "abc";"#, r"\u0069f", (4, 11)),
        ("let let = \"abc\";", "let", (4, 7)),
        ("let x = \"abc\", x = foo;", "x", (15, 16)),
        ("const x = \"abc\", y;", "y", (17, 18)),
        ("let x = \"abc\"; let x = foo;", "x", (19, 20)),
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
        attempt_selected_qualification(&source(r#"const x = "abc"; let \u{};"#))
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
    assert_eq!((subject.range().start(), subject.range().end()), (21, 25));
}

#[test]
fn one_level_block_retains_ordered_items_and_exact_source_provenance() {
    use super::selected_lexical_slice::SelectedTopLevelItem;

    let script = recognized_block("let a=1; { let a=2; let x=a; } let y=a;");
    assert_eq!(script.items().len(), 3);

    let SelectedTopLevelItem::LexicalDeclaration(first) = &script.items()[0] else {
        panic!("first selected item must remain the top-level declaration");
    };
    assert_eq!(
        (
            first.declaration().range().start(),
            first.declaration().range().end()
        ),
        (0, 8)
    );

    let SelectedTopLevelItem::Block(block) = &script.items()[1] else {
        panic!("second selected item must be the one-level Block");
    };
    assert_eq!(block.block().fragment(), "{ let a=2; let x=a; }");
    assert_eq!(
        (block.block().range().start(), block.block().range().end()),
        (9, 30)
    );
    let block_declarations: Vec<_> = block.declarations().collect();
    assert_eq!(block_declarations.len(), 2);
    assert_eq!(
        (
            block_declarations[0].declaration().range().start(),
            block_declarations[0].declaration().range().end()
        ),
        (11, 19)
    );
    assert_eq!(
        (
            block_declarations[1].declaration().range().start(),
            block_declarations[1].declaration().range().end()
        ),
        (20, 28)
    );
    let rhs = block_declarations[1].bindings()[0]
        .identifier_reference_initializer()
        .expect("selected RHS IdentifierReference fact");
    assert_eq!(
        (
            rhs.reference().range().start(),
            rhs.reference().range().end()
        ),
        (26, 27)
    );
    assert_eq!(rhs.semantic_name(), "a");

    let SelectedTopLevelItem::LexicalDeclaration(last) = &script.items()[2] else {
        panic!("last selected item must remain the top-level declaration");
    };
    assert_eq!(
        (
            last.declaration().range().start(),
            last.declaration().range().end()
        ),
        (31, 39)
    );
}

#[test]
fn one_level_block_preserves_sibling_and_identifier_reference_provenance() {
    use super::selected_lexical_slice::SelectedTopLevelItem;

    let siblings = recognized_block("{ let a=1; } { let x=a; }");
    let SelectedTopLevelItem::Block(first) = &siblings.items()[0] else {
        panic!("first item must be Block");
    };
    let SelectedTopLevelItem::Block(second) = &siblings.items()[1] else {
        panic!("second item must be Block");
    };
    assert_eq!(
        (first.block().range().start(), first.block().range().end()),
        (0, 12)
    );
    assert_eq!(
        (second.block().range().start(), second.block().range().end()),
        (13, 25)
    );

    let escaped = recognized_block(r"let a=1; { let x=\u0061; }");
    let SelectedTopLevelItem::Block(block) = &escaped.items()[1] else {
        panic!("escaped fixture must retain Block item");
    };
    let block_declarations: Vec<_> = block.declarations().collect();
    let reference = block_declarations[0].bindings()[0]
        .identifier_reference_initializer()
        .expect("escaped RHS reference fact");
    assert_eq!(reference.reference().fragment(), r"\u0061");
    assert_eq!(
        (
            reference.reference().range().start(),
            reference.reference().range().end()
        ),
        (17, 23)
    );
    assert_eq!(reference.semantic_name(), "a");

    let canonical = recognized_block(r"let é=1; { let x=e\u0301; }");
    let SelectedTopLevelItem::LexicalDeclaration(outer) = &canonical.items()[0] else {
        panic!("canonical fixture must retain outer declaration");
    };
    assert_eq!(outer.bindings()[0].binding().fragment(), "é");
    let SelectedTopLevelItem::Block(block) = &canonical.items()[1] else {
        panic!("canonical fixture must retain Block");
    };
    let block_declarations: Vec<_> = block.declarations().collect();
    let reference = block_declarations[0].bindings()[0]
        .identifier_reference_initializer()
        .expect("canonical-distinct RHS reference");
    assert_eq!(reference.semantic_name(), "e\u{301}");
    assert_ne!(
        reference.semantic_name(),
        outer.bindings()[0].binding().fragment()
    );
    assert_eq!(
        (
            reference.reference().range().start(),
            reference.reference().range().end()
        ),
        (18, 25)
    );
}

#[test]
fn one_level_block_frontier_does_not_widen_nested_empty_comment_statement_or_asi_coverage() {
    for text in [
        "{}",
        "{ let a=1 }",
        "{ { let a=1; } }",
        "{ let a=1; /*c*/ let x=a; }",
        "{ function f(){} }",
        "{ 1; }",
    ] {
        assert_unsupported(text);
    }

    assert!(matches!(
        recognize_selected_lexical_slice(&source("let a=1; let x=a;")),
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(_)
    ));
}

#[test]
fn one_level_block_var_statement_retains_ordered_1_to_n_declarators() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_ranges, expected_names) in [
        ("{ var x; }", &[(6, 7)][..], &["x"][..]),
        ("{ var x, y; }", &[(6, 7), (9, 10)][..], &["x", "y"][..]),
        (
            "{ var x, y, z; }",
            &[(6, 7), (9, 10), (12, 13)][..],
            &["x", "y", "z"][..],
        ),
        ("{ var x, x; }", &[(6, 7), (9, 10)][..], &["x", "x"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let ranges: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| {
                (
                    binding.binding().range().start(),
                    binding.binding().range().end(),
                )
            })
            .collect();
        assert_eq!(ranges, expected_ranges, "{text:?}");
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
    }
}

#[test]
fn one_level_block_var_statement_commits_no_partial_declarator_prefix_on_later_failure() {
    // A valid declarator prefix (e.g. "x") must never escape as a committed
    // Block var contributor when a later declarator or the terminator fails:
    // the whole statement is transactional.
    for text in [
        "{ var x, ; }",
        "{ var x, y, ; }",
        "{ var x, y = ; }",
        "{ var x, /* c */ y; }",
        "{ var x, y /* c */; }",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"{ var x, \u{}; }");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (9, 13));
}

#[test]
fn one_level_block_var_statement_admits_optional_selected_decimal_initializer_per_declarator() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_ranges, expected_names) in [
        ("{ var a=0; }", &[(6, 7)][..], &["a"][..]),
        ("{ var a=12345; }", &[(6, 7)][..], &["a"][..]),
        ("{ var a=1,b; }", &[(6, 7), (10, 11)][..], &["a", "b"][..]),
        ("{ var a,b=2; }", &[(6, 7), (8, 9)][..], &["a", "b"][..]),
        ("{ var a=1,b=2; }", &[(6, 7), (10, 11)][..], &["a", "b"][..]),
        (
            "{ var a=1,b,c=2; }",
            &[(6, 7), (10, 11), (12, 13)][..],
            &["a", "b", "c"][..],
        ),
        (
            "{ var a,b=2,c; }",
            &[(6, 7), (8, 9), (12, 13)][..],
            &["a", "b", "c"][..],
        ),
        (
            "{ var a=1,b=2,c=3; }",
            &[(6, 7), (10, 11), (14, 15)][..],
            &["a", "b", "c"][..],
        ),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let ranges: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| {
                (
                    binding.binding().range().start(),
                    binding.binding().range().end(),
                )
            })
            .collect();
        assert_eq!(ranges, expected_ranges, "{text:?}");
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
    }
}

#[test]
fn one_level_block_var_statement_duplicate_and_escaped_contributors_stay_distinct() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for text in ["{ var a=1,a; }", r"{ var a=1,\u0061=2; }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(statement.bindings().len(), 2, "{text:?}");
        assert_eq!(
            statement.bindings()[0].semantic_name(),
            Some("a"),
            "{text:?}"
        );
        assert_eq!(
            statement.bindings()[1].semantic_name(),
            Some("a"),
            "{text:?}"
        );
    }
}

// --- Issue #710: one-level Block `var` widened to a direct-authored, ------
// escape-free `IdentifierReference` initializer
//
// These focused production tests seal the candidate against the accepted
// #688/#710 theorem with independently authored expected values rather than
// values derived from production output.

#[test]
fn one_level_block_var_direct_identifier_reference_initializer_retains_exact_source_and_semantic_identity()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_binding, expected_reference, expected_fragment, expected_semantic) in [
        ("{ var x=a; }", (6, 7), (8, 9), "a", "a"),
        ("{ var x = y; }", (6, 7), (10, 11), "y", "y"),
        ("{ var x=x; }", (6, 7), (8, 9), "x", "x"),
        ("{ var π=𝒜; }", (6, 8), (9, 13), "𝒜", "𝒜"),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let [binding] = statement.bindings() else {
            panic!("expected exactly one Block var declarator for {text:?}");
        };
        assert_eq!(
            (
                binding.binding().range().start(),
                binding.binding().range().end()
            ),
            expected_binding,
            "{text:?}"
        );
        let reference = binding
            .identifier_reference_initializer()
            .expect("selected Block var RHS reference fact");
        assert_eq!(
            (
                reference.reference().range().start(),
                reference.reference().range().end()
            ),
            expected_reference,
            "{text:?}"
        );
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn one_level_block_var_multiple_identifier_reference_initializers_preserve_declarator_order() {
    // V2 (kills "first-reference-only" and "last-reference-only" wrong
    // models): two declarators of one Block `var` statement each retain
    // their own independent RHS reference fact, in exact authored order.
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let text = "{ var x=a,y=b; }";
    let script = recognized_block(text);
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    assert_eq!(statement.bindings().len(), 2);

    let first_reference = statement.bindings()[0]
        .identifier_reference_initializer()
        .expect("first declarator RHS reference fact");
    assert_eq!(
        (
            first_reference.reference().range().start(),
            first_reference.reference().range().end()
        ),
        (8, 9)
    );
    assert_eq!(first_reference.semantic_name(), "a");

    let second_reference = statement.bindings()[1]
        .identifier_reference_initializer()
        .expect("second declarator RHS reference fact");
    assert_eq!(
        (
            second_reference.reference().range().start(),
            second_reference.reference().range().end()
        ),
        (12, 13)
    );
    assert_eq!(second_reference.semantic_name(), "b");
}

#[test]
fn one_level_block_var_decimal_and_absent_initializers_retain_no_identifier_reference_fact() {
    // W10: a decimal or absent initializer must never fabricate an
    // `identifier_reference_initializer` fact.
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for text in ["{ var x; }", "{ var x=1; }", "{ var x,y=2; }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn one_level_block_var_identifier_reference_initializer_transactional_failure_commits_no_prefix() {
    // V5 / W6 / W13 (kills "a valid earlier direct or escaped RHS fact
    // leaks when a later declarator fails"): a valid declarator prefix
    // carrying a committed RHS fact (e.g. "x=a" in "{ var x=a,y=; }", or
    // "x=\u0066oo" in "{ var x=\u0066oo,y=; }") must never escape as part
    // of a committed statement when a later list element prevents statement
    // completion. "x=\u0069f" (Issue #715 classification-only C6 evidence)
    // must be held to the same rollback discipline: it must not escape as a
    // committed tentative fact when a later declarator (e.g. "y=" in
    // "{ var x=\u0069f,y=; }") prevents statement completion.
    for text in [
        "{ var x=a,y=; }",
        r"{ var x=\u0066oo,y=; }",
        r"{ var x=\u0069f,y=; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_escaped_reserved_identifier_name_initializer_no_prefix_commit() {
    // Richer-expression / no-prefix-commit firewall (Issue #715): a
    // member-expression suffix after the escaped ReservedWord
    // `IdentifierName` keeps the whole statement outside this leaf's
    // accepted profile; the recognizer must not commit a C6 candidate from
    // "\u0069f" and silently ignore the ".foo" suffix.
    for text in [r"{ var x=\u0069f.foo; }", r"{ var x=\u{69}f.foo; }"] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_escaped_reserved_initializer_retains_classification_only_exact_anchor() {
    // Issue #715: an escaped spelling that the shared recognizer classifies
    // as a ReservedWord now reaches complete selected recognition through
    // the Block-var RHS position, retaining only a classification-only
    // exact authored initializer anchor (not a `SelectedIdentifierReferenceFact`
    // and not the decoded semantic name) for the later Tier-1 `EE-04-R08`
    // static-semantics consumer, mirroring the accepted top-level var C6
    // theorem (#340/#341/#342/#343).
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_fragment, expected_range) in [
        (r"{ var x=\u0069f; }", r"\u0069f", (8, 15)),
        (r"{ var x=n\u0075ll; }", r"n\u0075ll", (8, 17)),
        (r"{ var x=\u{69}f; }", r"\u{69}f", (8, 15)),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let [binding] = statement.bindings() else {
            panic!("expected exactly one Block var declarator for {text:?}");
        };
        assert!(
            binding.identifier_reference_initializer().is_none(),
            "{text}"
        );
        let identifier = binding
            .escaped_reserved_initializer_identifier()
            .expect("C6 initializer must retain classification-only authored evidence");
        assert_eq!(identifier.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (identifier.range().start(), identifier.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn one_level_block_var_escaped_reserved_initializer_preserves_per_binding_cardinality_and_order() {
    // Multi-declarator authored Tier-1 ordering (Issue #715): distinct
    // declarators retain independent C6/C1/absent states in exact authored
    // order, matching the accepted top-level var cardinality theorem.
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let text = r"{ var a=1,b=\u0069f,c; }";
    let script = recognized_block(text);
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    assert_eq!(statement.bindings().len(), 3);
    assert!(
        statement.bindings()[0]
            .escaped_reserved_initializer_identifier()
            .is_none()
    );
    let second = statement.bindings()[1]
        .escaped_reserved_initializer_identifier()
        .expect("second declarator C6 evidence");
    assert_eq!((second.range().start(), second.range().end()), (12, 19));
    assert!(
        statement.bindings()[2]
            .escaped_reserved_initializer_identifier()
            .is_none()
    );

    let text = r"{ var a=\u0066oo,b=\u0069f; }";
    let script = recognized_block(text);
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    let c1 = statement.bindings()[0]
        .identifier_reference_initializer()
        .expect("first declarator must remain C1");
    assert_eq!(
        (c1.reference().range().start(), c1.reference().range().end()),
        (8, 16)
    );
    assert!(
        statement.bindings()[0]
            .escaped_reserved_initializer_identifier()
            .is_none()
    );
    let c6 = statement.bindings()[1]
        .escaped_reserved_initializer_identifier()
        .expect("second declarator must retain C6 evidence");
    assert_eq!((c6.range().start(), c6.range().end()), (19, 26));
    assert!(
        statement.bindings()[1]
            .identifier_reference_initializer()
            .is_none()
    );
}

#[test]
fn one_level_block_var_decimal_initializer_numeric_neighbors_remain_unsupported() {
    // "{ var a=1.0; }" is deliberately not listed here: Issue #732 makes a
    // direct-authored plain fractional `DecimalLiteral` initializer a
    // selected accepted form (see the "Issue #732" test section below).
    // "{ var a=1e2; }" is also deliberately not listed here: Issue #740
    // makes a direct-authored, separator-free exponent `DecimalLiteral`
    // initializer a selected accepted form (see the "Issue #740" test
    // section below). "{ var a=+1; }" and "{ var a=-1; }" are also
    // deliberately not listed here: Issue #744 makes a direct-authored
    // leading `+`/`-` decimal `UnaryExpression` initializer a selected
    // accepted form (see the "Issue #744" test section below).
    for text in ["{ var a=01; }", "{ var a=1_0; }", "{ var a=1n; }"] {
        assert_unsupported(text);
    }
}

// The former `one_level_block_var_other_initializer_families_remain_unsupported`
// sentinel is removed here: its only remaining case, `{ var a="x"; }`, is no
// longer `UnsupportedCoverage`. Issue #725 makes a direct-authored,
// escape-free `StringLiteral` initializer a selected accepted form (see the
// "Issue #725" test section below). Every other family this sentinel used
// to guard (IdentifierReference, escaped IdentifierReference,
// escaped-ReservedWord C6, BooleanLiteral, NullLiteral, and `this`) was
// already migrated to its own accepted test section by the corresponding
// earlier Issue.

// --- Issue #713: one-level Block `var` widened to a selected escaped ------
// non-ReservedWord `IdentifierReference` initializer.
//
// These focused production tests seal the candidate against the accepted
// #712/#713 theorem with independently authored expected values rather
// than values derived from production output. They reuse the existing
// #336/#338 escaped-IdentifierReference name-policy authority and the
// #710 Block-var composition authority rather than cloning the full
// top-level oracle.

#[test]
fn one_level_block_var_escaped_identifier_reference_initializer_retains_exact_source_and_semantic_identity()
 {
    // V1/W1/W2/W3 (kills "authored spelling used as semantic identity",
    // "first-position escape only", and "fixed-form \uXXXX only, rejecting
    // braced/supplementary-plane forms"): a formed, position-valid escaped
    // `IdentifierReference` RHS retains its exact authored anchor and its
    // independently decoded semantic name, in every escape position and
    // form the shared recognizer accepts.
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_binding, expected_reference, expected_fragment, expected_semantic) in [
        (r"{ var x=\u0061; }", (6, 7), (8, 14), r"\u0061", "a"),
        (r"{ var x=\u0066oo; }", (6, 7), (8, 16), r"\u0066oo", "foo"),
        (r"{ var x=f\u006Fo; }", (6, 7), (8, 16), r"f\u006Fo", "foo"),
        (r"{ var x=\u{1D49C}; }", (6, 7), (8, 17), r"\u{1D49C}", "𝒜"),
        (r"{ var x=a\u0030; }", (6, 7), (8, 15), r"a\u0030", "a0"),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let [binding] = statement.bindings() else {
            panic!("expected exactly one Block var declarator for {text:?}");
        };
        assert_eq!(
            (
                binding.binding().range().start(),
                binding.binding().range().end()
            ),
            expected_binding,
            "{text:?}"
        );
        let reference = binding
            .identifier_reference_initializer()
            .expect("selected Block var RHS reference fact");
        assert_eq!(
            (
                reference.reference().range().start(),
                reference.reference().range().end()
            ),
            expected_reference,
            "{text:?}"
        );
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn one_level_block_var_multiple_escaped_and_direct_identifier_reference_initializers_preserve_declarator_order()
 {
    // W11 (kills "only first or last escaped RHS survives in a
    // multi-declarator statement"): an escaped and a direct RHS reference in
    // the same Block `var` statement each retain their own independent
    // fact, in exact authored order.
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let text = r"{ var x=\u0066oo,y=bar; }";
    let script = recognized_block(text);
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    assert_eq!(statement.bindings().len(), 2);

    let first_reference = statement.bindings()[0]
        .identifier_reference_initializer()
        .expect("first declarator RHS reference fact");
    assert_eq!(
        (
            first_reference.reference().range().start(),
            first_reference.reference().range().end()
        ),
        (8, 16)
    );
    assert_eq!(first_reference.semantic_name(), "foo");

    let second_reference = statement.bindings()[1]
        .identifier_reference_initializer()
        .expect("second declarator RHS reference fact");
    assert_eq!(
        (
            second_reference.reference().range().start(),
            second_reference.reference().range().end()
        ),
        (19, 22)
    );
    assert_eq!(second_reference.semantic_name(), "bar");
}

#[test]
fn one_level_block_var_escaped_identifier_reference_name_policy_preserves_c1_c6_firewall() {
    // W4/W5 (kills "a broad keyword filter incorrectly rejects yield/
    // await/let/strict-only names/eval/arguments" and "escaped ReservedWord
    // accidentally accepted"): reuses the existing #336/#338 name-policy
    // authority for the fixed non-strict `Yield=false, Await=false`
    // envelope, composed with the #710 Block-var placement.
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (rhs, expected_semantic) in [
        (r"\u006Cet", "let"),
        (r"\u0073tatic", "static"),
        (r"\u0069mplements", "implements"),
        (r"\u0069nterface", "interface"),
        (r"\u0070ackage", "package"),
        (r"\u0070rivate", "private"),
        (r"\u0070rotected", "protected"),
        (r"\u0070ublic", "public"),
        (r"\u0079ield", "yield"),
        (r"a\u0077ait", "await"),
        (r"\u0065val", "eval"),
        (r"\u0061rguments", "arguments"),
    ] {
        let text = format!("{{ var x={rhs}; }}");
        let script = recognized_block(&text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .expect("C1 escaped name-policy positive");
        assert_eq!(reference.reference().fragment(), rhs, "{text}");
        assert_eq!(reference.semantic_name(), expected_semantic, "{text}");
    }
}

#[test]
fn one_level_block_var_escaped_identifier_reference_initializer_ee01_and_malformed_firewall() {
    // W6/W7 (kills "RHS position-invalid escapes routed to EE-01" and
    // "invalid escaped IdentifierPart truncated to a positive direct
    // prefix"): position-invalid, surrogate, and malformed escapes in the
    // Block-var RHS position remain `UnsupportedCoverage`, and a partially
    // valid prefix (e.g. "a" in "a\u002Db") never commits as a positive
    // direct reference.
    for text in [
        r"{ var x=\u0030; }",
        r"{ var x=a\u002Db; }",
        r"{ var x=\u{}; }",
        r"{ var x=\u{110000}; }",
        r"{ var x=\uD800; }",
        r"{ var x=\u{D800}; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_decimal_initializer_composes_with_close_brace_asi() {
    // Issue #717: a decimal initializer composes unchanged with the new
    // bounded close-brace ASI terminator route.
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    for (text, expected_names) in [
        ("{ var a=1 }", &["a"][..]),
        ("{ var a=1,b=2 }", &["a", "b"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(
            statement.terminator(),
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
            "{text:?}"
        );
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
    }
}

#[test]
fn one_level_block_var_decimal_initializer_rejects_comment_trivia() {
    for text in [
        "{ var a=1, /* c */ b; }",
        "{ var a=/* c */1; }",
        "{ var a=1 /* c */; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_decimal_initializer_transactional_failure_commits_no_prefix() {
    // "{ var a=1,b=1.0; }" is deliberately not listed here: Issue #732 makes
    // a direct-authored plain fractional `DecimalLiteral` initializer a
    // selected accepted form, so this source is no longer a failure case.
    for text in ["{ var a=1, ; }", "{ var a=1,b= ; }"] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"{ var a=1,b,\u{}=2; }");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (12, 16));
}

// --- Issue #717: one-level Block `var` widened to admit bounded --------
// close-brace ASI as an additional statement terminator
//
// These focused production tests exercise `recognize_selected_lexical_slice`
// directly, sealing the new terminator route (statement-owned provenance,
// close-brace ownership, and composition with every already-accepted
// declarator shape) against the accepted #688-comment-5685046994 theorem
// and the existing #318/#320 EOF-only ASI terminator-provenance precedent.

#[test]
fn one_level_block_var_close_brace_asi_retains_statement_owned_terminator_provenance() {
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    for (text, expected_terminator) in [
        (
            "{ var x; }",
            SelectedBlockVarStatementTerminator::AuthoredSemicolon,
        ),
        (
            "{ var x }",
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
        ),
        (
            "{ var x   }",
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
        ),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(statement.terminator(), expected_terminator, "{text:?}");
        let [binding] = statement.bindings() else {
            panic!("expected exactly one declarator for {text:?}");
        };
        assert_eq!(binding.binding().fragment(), "x", "{text:?}");
        assert_eq!(
            (
                binding.binding().range().start(),
                binding.binding().range().end()
            ),
            (6, 7),
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_var_close_brace_asi_terminator_is_statement_owned_not_block_global() {
    // One Block item terminated by authored `;` and a later one terminated
    // by close-brace ASI must retain independent per-statement provenance,
    // proving the terminator is not a Block-global flag.
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    let script = recognized_block("{ var x; let y; var z }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [
        SelectedBlockItem::Var(first),
        SelectedBlockItem::LexicalDeclaration(_),
        SelectedBlockItem::Var(second),
    ] = block.items()
    else {
        panic!("expected var, lexical, var Block items");
    };
    assert_eq!(
        first.terminator(),
        SelectedBlockVarStatementTerminator::AuthoredSemicolon
    );
    assert_eq!(
        second.terminator(),
        SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose
    );
}

#[test]
fn one_level_block_var_close_brace_asi_composes_with_existing_declarator_shapes() {
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    for (text, expected_names) in [
        ("{ var x }", &["x"][..]),
        ("{ var x, y }", &["x", "y"][..]),
        ("{ var x=a }", &["x"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(
            statement.terminator(),
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
            "{text:?}"
        );
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
    }
}

#[test]
fn one_level_block_var_close_brace_asi_composes_with_escaped_initializers() {
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    let script = recognized_block(r"{ var x=\u0061 }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    assert_eq!(
        statement.terminator(),
        SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose
    );
    let [binding] = statement.bindings() else {
        panic!("expected exactly one declarator");
    };
    let reference = binding
        .identifier_reference_initializer()
        .expect("expected escaped non-ReservedWord IdentifierReference initializer");
    assert_eq!(reference.semantic_name(), "a");

    let script = recognized_block(r"{ var x=\u0069f }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    assert_eq!(
        statement.terminator(),
        SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose
    );
    let [binding] = statement.bindings() else {
        panic!("expected exactly one declarator");
    };
    let anchor = binding
        .escaped_reserved_initializer_identifier()
        .expect("expected classification-only escaped ReservedWord anchor");
    assert_eq!(anchor.fragment(), r"\u0069f");
}

#[test]
fn one_level_block_var_close_brace_asi_eof_is_not_equivalent_to_close_brace() {
    // EOF must never be silently treated as this leaf's bounded close-brace
    // ASI route: EOF-ASI is a distinct, materially different capability
    // outside this leaf's scope for Block-contained `var`.
    assert_unsupported("{ var x");
}

#[test]
fn one_level_block_var_close_brace_asi_does_not_repair_incomplete_declarator_or_initializer() {
    for text in ["{ var x, }", "{ var x= }"] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_close_brace_asi_does_not_commit_tentative_c6_evidence_on_later_failure() {
    assert_unsupported(r"{ var x=\u0069f, y= }");
}

#[test]
fn one_level_block_var_close_brace_asi_does_not_widen_to_line_terminator_asi() {
    // General LineTerminator-triggered ASI before another statement remains
    // outside this leaf; only the bounded "next significant token == }"
    // route is recognized.
    for text in ["{\n  var x\n  let y;\n}", "{\n  var x\n  var y;\n}"] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_close_brace_asi_does_not_widen_to_comments_or_initializer_family() {
    // "{ var a=true }" is deliberately not listed here: Issue #719 makes a
    // direct-authored `BooleanLiteral` initializer compose with close-brace
    // ASI as a selected accepted form (see the "Issue #719" test section
    // below). "{ var a=null }" is also deliberately not listed here: Issue
    // #721 makes a direct-authored `NullLiteral` initializer compose with
    // close-brace ASI as a selected accepted form (see the "Issue #721" test
    // section below). "{ var a=this }" is also deliberately not listed
    // here: Issue #723 makes a direct-authored `PrimaryExpression : this`
    // initializer compose with close-brace ASI as a selected accepted form
    // (see the "Issue #723" test section below). `{ var a="x" }` is also
    // deliberately not listed here: Issue #725 makes a direct-authored,
    // escape-free `StringLiteral` initializer compose with close-brace ASI
    // as a selected accepted form (see the "Issue #725" test section below).
    for text in ["{ var x /* c */ }", "{ var x=1 /* c */ }"] {
        assert_unsupported(text);
    }
}

// --- Issue #719: one-level Block `var` widened to a direct-authored ------
// `BooleanLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5690396598 / #719 theorem, reusing the existing accepted
// #245/#247/#248 direct-Boolean recognizer (`consume_selected_boolean_literal`)
// and its maximal-IdentifierName boundary rather than cloning that
// independent oracle.

#[test]
fn one_level_block_var_direct_boolean_literal_initializer_composes_as_presence_only() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_names) in [
        ("{ var x=true; }", &["x"][..]),
        ("{ var x=false; }", &["x"][..]),
        ("{ var x=true }", &["x"][..]),
        ("{ var x=false }", &["x"][..]),
        ("{ var a=true,b=false; }", &["a", "b"][..]),
        ("{ var a=true,b=false }", &["a", "b"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn one_level_block_var_mixed_decimal_boolean_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let script = recognized_block("{ var a=false,b=1,c=x }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    assert!(
        statement.bindings()[0]
            .identifier_reference_initializer()
            .is_none()
    );
    assert!(
        statement.bindings()[1]
            .identifier_reference_initializer()
            .is_none()
    );
    let reference = statement.bindings()[2]
        .identifier_reference_initializer()
        .expect("third declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn one_level_block_var_direct_boolean_literal_boundary_preserves_maximal_identifier_reference_routing()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_fragment, expected_semantic) in [
        ("{ var x=truex; }", "truex", "truex"),
        ("{ var x=falseValue; }", "falseValue", "falseValue"),
        (r"{ var x=true\u0061; }", r"true\u0061", "truea"),
        (r"{ var x=false\u0061; }", r"false\u0061", "falsea"),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .unwrap_or_else(|| panic!("expected IdentifierReference routing for {text:?}"));
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn one_level_block_var_direct_boolean_literal_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        "{ var x=true.foo; }",
        "{ var x=false(); }",
        "{ var x=true + y; }",
        "{ var x=false = y; }",
        "{ var x=true ? a : b; }",
        "{ var x=true.foo }",
        "{ var x=false() }",
        "{ var x=true + y }",
        "{ var x=true/*c*/; }",
        "{ var x=true /*c*/ }",
        r"{ var x=true\u{}; }",
        r"{ var x=false\u0; }",
        r"{ var x=true\u{61; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_direct_boolean_literal_escaped_reserved_spelling_is_not_boolean_literal() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for text in [r"{ var x=\u0074rue; }", r"{ var x=\u0066alse; }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert!(
            statement.bindings()[0]
                .escaped_reserved_initializer_identifier()
                .is_some(),
            "{text:?}"
        );
        assert!(
            statement.bindings()[0]
                .identifier_reference_initializer()
                .is_none(),
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_var_direct_boolean_literal_transactional_failure_commits_no_prefix() {
    for text in ["{ var a=true,b=; }", "{ var a=false,b= }"] {
        assert_unsupported(text);
    }
}

// --- Issue #721: one-level Block `var` widened to a direct-authored ------
// `NullLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5690791939 / #721 theorem, reusing the existing accepted
// #249/#250/#251/#252 direct-NullLiteral recognizer
// (`consume_selected_null_literal`) and its maximal-IdentifierName boundary
// rather than cloning that independent oracle. The escaped-semantic-`null`
// C6 theorem is already sealed by
// `one_level_block_var_escaped_reserved_initializer_retains_classification_only_exact_anchor`
// above and is intentionally not duplicated here.

#[test]
fn one_level_block_var_direct_null_literal_initializer_composes_as_presence_only() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_names) in [
        ("{ var x=null; }", &["x"][..]),
        ("{ var x=null }", &["x"][..]),
        ("{ var a=null,b=null; }", &["a", "b"][..]),
        ("{ var a=null,b=null }", &["a", "b"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn one_level_block_var_mixed_decimal_boolean_null_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let script = recognized_block("{ var a=null,b=1,c=true,d=x }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    for index in 0..3 {
        assert!(
            statement.bindings()[index]
                .identifier_reference_initializer()
                .is_none()
        );
    }
    let reference = statement.bindings()[3]
        .identifier_reference_initializer()
        .expect("fourth declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn one_level_block_var_direct_null_literal_boundary_preserves_maximal_identifier_reference_routing()
{
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_fragment, expected_semantic) in [
        ("{ var x=nullx; }", "nullx", "nullx"),
        ("{ var x=nullValue; }", "nullValue", "nullValue"),
        (r"{ var x=null\u0061; }", r"null\u0061", "nulla"),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .unwrap_or_else(|| panic!("expected IdentifierReference routing for {text:?}"));
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn one_level_block_var_direct_null_literal_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        "{ var x=null.foo; }",
        "{ var x=null(); }",
        "{ var x=null + y; }",
        "{ var x=null = y; }",
        "{ var x=null ? a : b; }",
        "{ var x=null.foo }",
        "{ var x=null() }",
        "{ var x=null + y }",
        "{ var x=null/*c*/; }",
        "{ var x=null /*c*/ }",
        r"{ var x=null\u{}; }",
        r"{ var x=null\u0; }",
        r"{ var x=null\u{61; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_direct_null_literal_transactional_failure_commits_no_prefix() {
    for text in ["{ var a=null,b=; }", "{ var a=null,b= }"] {
        assert_unsupported(text);
    }
}

// --- Issue #723: one-level Block `var` widened to a direct-authored ------
// `PrimaryExpression : this` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5691238988 / #723 theorem, reusing the existing accepted
// #253/#254/#255/#256 direct-`this` recognizer
// (`consume_selected_this_expression`) and its maximal-IdentifierName
// boundary rather than cloning that independent oracle. The escaped-
// semantic-`this` C6 theorem is already sealed by
// `one_level_block_var_escaped_reserved_initializer_retains_classification_only_exact_anchor`
// above and further sealed below with dedicated `this`-specific spellings;
// it is intentionally not fully duplicated here.

#[test]
fn one_level_block_var_direct_this_initializer_composes_as_presence_only() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_names) in [
        ("{ var x=this; }", &["x"][..]),
        ("{ var x=this }", &["x"][..]),
        ("{ var a=this,b=this; }", &["a", "b"][..]),
        ("{ var a=this,b=this }", &["a", "b"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn one_level_block_var_mixed_decimal_boolean_null_this_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let script = recognized_block("{ var a=false,b=1,c=null,d=this,e=x }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    for index in 0..4 {
        assert!(
            statement.bindings()[index]
                .identifier_reference_initializer()
                .is_none()
        );
    }
    let reference = statement.bindings()[4]
        .identifier_reference_initializer()
        .expect("fifth declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn one_level_block_var_direct_this_boundary_preserves_maximal_identifier_reference_routing() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_fragment, expected_semantic) in [
        ("{ var x=thisx; }", "thisx", "thisx"),
        ("{ var x=thisValue; }", "thisValue", "thisValue"),
        ("{ var x=this0; }", "this0", "this0"),
        ("{ var x=this$; }", "this$", "this$"),
        ("{ var x=this_; }", "this_", "this_"),
        (r"{ var x=this\u0061; }", r"this\u0061", "thisa"),
        (r"{ var x=this\u{61}; }", r"this\u{61}", "thisa"),
        (r"{ var x=this\u0030; }", r"this\u0030", "this0"),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .unwrap_or_else(|| panic!("expected IdentifierReference routing for {text:?}"));
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn one_level_block_var_direct_this_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        "{ var x=this.foo; }",
        "{ var x=this(); }",
        "{ var x=this + y; }",
        "{ var x=this = y; }",
        "{ var x=this ? a : b; }",
        "{ var x=this.foo }",
        "{ var x=this() }",
        "{ var x=this + y }",
        "{ var x=this/*c*/; }",
        "{ var x=this /*c*/ }",
        r"{ var x=this\u{}; }",
        r"{ var x=this\u0; }",
        r"{ var x=this\u{61; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_direct_this_escaped_reserved_spelling_is_not_this_expression() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for text in [r"{ var x=\u0074his; }", r"{ var x=t\u0068is; }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert!(
            statement.bindings()[0]
                .escaped_reserved_initializer_identifier()
                .is_some(),
            "{text:?}"
        );
        assert!(
            statement.bindings()[0]
                .identifier_reference_initializer()
                .is_none(),
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_var_direct_this_transactional_failure_commits_no_prefix() {
    for text in ["{ var a=this,b=; }", "{ var a=this,b= }"] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_variable_statement_retains_minimal_source_backed_binding_fact() {
    for (text, fragment, semantic_name, range) in [
        ("var x;", "x", "x", (4, 5)),
        ("var let;", "let", "let", (4, 7)),
        (r"var \u006Cet;", r"\u006Cet", "let", (4, 12)),
        (r"var \u0069f;", r"\u0069f", "if", (4, 11)),
        ("var x=1;", "x", "x", (4, 5)),
        ("var x = 123;", "x", "x", (4, 5)),
        (r"var \u0069f=1;", r"\u0069f", "if", (4, 11)),
    ] {
        let script = recognized_variable(text);
        assert_eq!(script.items().len(), 1, "{text}");
        let SelectedVariableTopLevelItem::VariableStatement(statement) = &script.items()[0] else {
            panic!("expected bounded VariableStatement for {text:?}");
        };
        let [binding] = statement.bindings() else {
            panic!("expected exactly one selected declarator for {text:?}");
        };
        assert_eq!(binding.binding().fragment(), fragment, "{text}");
        assert_eq!(binding.semantic_name(), Some(semantic_name), "{text}");
        assert_eq!(
            (
                binding.binding().range().start(),
                binding.binding().range().end()
            ),
            range,
            "{text}"
        );
        assert!(binding.identifier_reference_initializer().is_none());
    }
}

#[test]
fn variable_statement_eof_asi_retains_statement_owned_terminator_provenance() {
    for (text, expected_terminator, fragment, semantic_name, expected_range) in [
        (
            "var a;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            "a",
            "a",
            (4, 5),
        ),
        (
            "var a   ;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            "a",
            "a",
            (4, 5),
        ),
        (
            "var a",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "a",
            "a",
            (4, 5),
        ),
        (
            "var a   \t\n",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "a",
            "a",
            (4, 5),
        ),
        (
            r"var \u0061",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            r"\u0061",
            "a",
            (4, 10),
        ),
        (
            "var é",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "é",
            "é",
            (4, 6),
        ),
        (
            r"var e\u0301",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            r"e\u0301",
            "e\u{301}",
            (4, 11),
        ),
        (
            "var a=1;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            "a",
            "a",
            (4, 5),
        ),
        (
            "var a=1",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "a",
            "a",
            (4, 5),
        ),
    ] {
        let script = recognized_variable(text);
        let [SelectedVariableTopLevelItem::VariableStatement(statement)] = script.items() else {
            panic!("expected exactly one selected VariableStatement for {text:?}");
        };
        assert_eq!(statement.terminator(), expected_terminator, "{text:?}");
        let [binding] = statement.bindings() else {
            panic!("expected exactly one selected declarator for {text:?}");
        };
        assert_eq!(binding.binding().fragment(), fragment, "{text:?}");
        assert_eq!(binding.semantic_name(), Some(semantic_name), "{text:?}");
        assert_eq!(
            (
                binding.binding().range().start(),
                binding.binding().range().end()
            ),
            expected_range,
            "{text:?}"
        );
    }

    let script = recognized_variable("var a; var a");
    let [
        SelectedVariableTopLevelItem::VariableStatement(first),
        SelectedVariableTopLevelItem::VariableStatement(second),
    ] = script.items()
    else {
        panic!("repeated var fixture must retain two VariableStatement owners");
    };
    assert_eq!(
        first.terminator(),
        SelectedVariableStatementTerminator::AuthoredSemicolon
    );
    assert_eq!(
        second.terminator(),
        SelectedVariableStatementTerminator::AutomaticAtEof
    );
    assert_eq!(binding_ranges(first), [(4, 5)]);
    assert_eq!(binding_ranges(second), [(11, 12)]);
}

#[test]
fn variable_statement_promotes_only_var_bearing_sources_to_distinct_representation() {
    let script = recognized_variable("let a; { let b; } var c; let d");
    assert_eq!(script.items().len(), 4);
    assert!(matches!(
        script.items()[0],
        SelectedVariableTopLevelItem::LexicalDeclaration(_)
    ));
    assert!(matches!(
        script.items()[1],
        SelectedVariableTopLevelItem::Block(_)
    ));
    assert!(matches!(
        script.items()[2],
        SelectedVariableTopLevelItem::VariableStatement(_)
    ));
    let SelectedVariableTopLevelItem::LexicalDeclaration(last) = &script.items()[3] else {
        panic!("trailing lexical declaration must stay lexical");
    };
    assert!(matches!(
        last.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));

    assert!(matches!(
        recognize_selected_lexical_slice(&source("let a; let b;")),
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(_)
    ));
    assert!(matches!(
        recognize_selected_lexical_slice(&source("let a; { let b; }")),
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_)
    ));
}

#[test]
fn variable_statement_frontier_keeps_non_eof_and_broader_var_grammar_unsupported() {
    // "{ var x; }" and "{ var x, y; }" are deliberately not listed here:
    // Issue #691/#695 make one-level Block `var` statements with 1..N
    // declarators a selected accepted form (see
    // `selected_lexical_slice.rs`'s `SelectedBlockVarStatement` recognition
    // and the Block-item coverage tests in
    // `selected_qualification_integration_tests.rs`).
    for text in [
        "var x\nvar y;",
        "var {x} = y;",
        "for (var x;;) {}",
        "var x/*comment*/",
        r"var\u{};",
        "var x=foo.bar;",
        "var x=foo();",
        "var x=foo+1;",
        "var x=(foo);",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"var \u{};");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (4, 8));
}

#[test]
fn variable_statement_authored_semicolon_composes_with_existing_trailing_lexical_eof_asi() {
    let script = recognized_variable("var x; let y");
    assert_eq!(script.items().len(), 2);
    let SelectedVariableTopLevelItem::VariableStatement(statement) = &script.items()[0] else {
        panic!("first item must be VariableStatement");
    };
    assert_eq!(binding_fragments(statement), ["x"]);
    assert_eq!(
        statement.terminator(),
        SelectedVariableStatementTerminator::AuthoredSemicolon
    );
    let SelectedVariableTopLevelItem::LexicalDeclaration(declaration) = &script.items()[1] else {
        panic!("second item must be lexical declaration");
    };
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn variable_declaration_list_retains_every_authored_declarator_in_list_order() {
    let two = recognized_variable("var a,b;");
    let two = only_variable_statement(&two);
    assert_eq!(binding_ranges(two), [(4, 5), (6, 7)]);
    assert_eq!(binding_fragments(two), ["a", "b"]);
    assert_eq!(binding_semantic_names(two), [Some("a"), Some("b")]);
    assert_eq!(
        two.terminator(),
        SelectedVariableStatementTerminator::AuthoredSemicolon
    );

    let migrated = recognized_variable("var x, y;");
    let migrated = only_variable_statement(&migrated);
    assert_eq!(binding_ranges(migrated), [(4, 5), (7, 8)]);
    assert_eq!(binding_fragments(migrated), ["x", "y"]);

    let three = recognized_variable("var a,b,c;");
    let three = only_variable_statement(&three);
    assert_eq!(binding_ranges(three), [(4, 5), (6, 7), (8, 9)]);
    assert_eq!(binding_fragments(three), ["a", "b", "c"]);

    let repeated = recognized_variable("var a,a;");
    let repeated = only_variable_statement(&repeated);
    assert_eq!(binding_ranges(repeated), [(4, 5), (6, 7)]);
    assert_eq!(binding_semantic_names(repeated), [Some("a"), Some("a")]);

    let mixed_spelling = recognized_variable(r"var a,\u0061;");
    let mixed_spelling = only_variable_statement(&mixed_spelling);
    assert_eq!(binding_ranges(mixed_spelling), [(4, 5), (6, 12)]);
    assert_eq!(binding_fragments(mixed_spelling), ["a", r"\u0061"]);
    assert_eq!(
        binding_semantic_names(mixed_spelling),
        [Some("a"), Some("a")]
    );

    let unnormalized = recognized_variable(r"var é,e\u0301;");
    let unnormalized = only_variable_statement(&unnormalized);
    assert_eq!(binding_ranges(unnormalized), [(4, 6), (7, 14)]);
    assert_eq!(
        binding_semantic_names(unnormalized),
        [Some("é"), Some("e\u{301}")]
    );

    for (text, expected_ranges) in [
        ("var a=1,b;", &[(4, 5), (8, 9)][..]),
        ("var a,b=2;", &[(4, 5), (6, 7)][..]),
        ("var a=1,b=2;", &[(4, 5), (8, 9)][..]),
        ("var a=1,b,c=2;", &[(4, 5), (8, 9), (10, 11)][..]),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(binding_ranges(statement), expected_ranges, "{text:?}");
    }

    let escaped_initialized = recognized_variable(r"var a=1,\u0061=2;");
    let escaped_initialized = only_variable_statement(&escaped_initialized);
    assert_eq!(binding_ranges(escaped_initialized), [(4, 5), (8, 14)]);
    assert_eq!(
        binding_semantic_names(escaped_initialized),
        [Some("a"), Some("a")]
    );
}

#[test]
fn one_statement_owned_terminator_is_independent_of_declarator_cardinality() {
    for (text, expected_terminator, expected_ranges) in [
        (
            "var a,b;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            &[(4, 5), (6, 7)][..],
        ),
        (
            "var a,b",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            &[(4, 5), (6, 7)][..],
        ),
        (
            "var a,b,c   \t\n",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            &[(4, 5), (6, 7), (8, 9)][..],
        ),
        (
            "var a,\n b",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            &[(4, 5), (8, 9)][..],
        ),
        (
            "var a\n, b;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            &[(4, 5), (8, 9)][..],
        ),
        (
            "var a , b ;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            &[(4, 5), (8, 9)][..],
        ),
        (
            "var a=1,b=2;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            &[(4, 5), (8, 9)][..],
        ),
        (
            "var a=1,b=2",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            &[(4, 5), (8, 9)][..],
        ),
        (
            "var a=1\n, b=2;",
            SelectedVariableStatementTerminator::AuthoredSemicolon,
            &[(4, 5), (10, 11)][..],
        ),
        (
            "var a=1,\n b=2",
            SelectedVariableStatementTerminator::AutomaticAtEof,
            &[(4, 5), (10, 11)][..],
        ),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(statement.terminator(), expected_terminator, "{text:?}");
        assert_eq!(binding_ranges(statement), expected_ranges, "{text:?}");
    }
}

#[test]
fn later_declarators_reuse_the_general_binding_identifier_grammar_route() {
    let subject = grammar_rejection(r"var a,\u{};");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (6, 10));

    let subject = grammar_rejection(r"var a,b,\u{};");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (8, 12));

    let subject = grammar_rejection(r"var a,\u0;");
    assert_eq!(subject.fragment(), r"\u0");
    assert_eq!((subject.range().start(), subject.range().end()), (6, 9));

    let subject = grammar_rejection(r"var a,\u{}=1;");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (6, 10));

    let script = recognized_variable(r"var a, \u0069f;");
    let statement = only_variable_statement(&script);
    assert_eq!(binding_ranges(statement), [(4, 5), (7, 14)]);
    assert_eq!(binding_semantic_names(statement), [Some("a"), Some("if")]);

    let script = recognized_variable(r"var a,\u0061=1;");
    let statement = only_variable_statement(&script);
    assert_eq!(binding_ranges(statement), [(4, 5), (6, 12)]);
    assert_eq!(binding_semantic_names(statement), [Some("a"), Some("a")]);

    assert_unsupported("var a, if;");
    let script = recognized_variable("var a, let;");
    let statement = only_variable_statement(&script);
    assert_eq!(binding_ranges(statement), [(4, 5), (7, 10)]);
    assert_eq!(binding_semantic_names(statement), [Some("a"), Some("let")]);
}

#[test]
fn selected_var_decimal_initializer_boundary_is_exact() {
    // "var a=this;" is deliberately not listed here: Issue #723 makes a
    // direct-authored `PrimaryExpression : this` initializer a selected
    // accepted form (see the "Issue #723" test section below). "var
    // a=\"x\";" is also deliberately not listed here: Issue #725 makes a
    // direct-authored, escape-free `StringLiteral` initializer a selected
    // accepted form (see the "Issue #725" test section below). "var
    // a=1.0;" is also deliberately not listed here: Issue #732 makes a
    // direct-authored plain fractional `DecimalLiteral` initializer a
    // selected accepted form (see the "Issue #732" test section below).
    // "var a=1e2;" is also deliberately not listed here: Issue #740 makes a
    // direct-authored, separator-free exponent `DecimalLiteral` initializer
    // a selected accepted form (see the "Issue #740" test section below).
    // "var a=+1;" and "var a=-1;" are also deliberately not listed here:
    // Issue #744 makes a direct-authored leading `+`/`-` decimal
    // `UnaryExpression` initializer a selected accepted form (see the
    // "Issue #744" test section below).
    for text in ["var a=01;", "var a=1_0;", "var a=1n;"] {
        assert_unsupported(text);
    }
}

#[test]
fn var_identifier_reference_initializer_retains_exact_source_and_semantic_identity() {
    for (text, expected_binding, expected_reference, expected_fragment, expected_semantic) in [
        ("var a=foo;", (4, 5), (6, 9), "foo", "foo"),
        ("var x = y;", (4, 5), (8, 9), "y", "y"),
        ("var π=𝒜;", (4, 6), (7, 11), "𝒜", "𝒜"),
        ("var x=$;", (4, 5), (6, 7), "$", "$"),
        ("var x=_;", (4, 5), (6, 7), "_", "_"),
        ("var x=a0;", (4, 5), (6, 8), "a0", "a0"),
        ("var x=let;", (4, 5), (6, 9), "let", "let"),
        ("var x=yield;", (4, 5), (6, 11), "yield", "yield"),
        ("var x=await;", (4, 5), (6, 11), "await", "await"),
        ("var x=eval;", (4, 5), (6, 10), "eval", "eval"),
        (
            "var x=arguments;",
            (4, 5),
            (6, 15),
            "arguments",
            "arguments",
        ),
        (r"var x=\u0066oo;", (4, 5), (6, 14), r"\u0066oo", "foo"),
        (r"var x=f\u006Fo;", (4, 5), (6, 14), r"f\u006Fo", "foo"),
        (r"var x=\u{1D49C};", (4, 5), (6, 15), r"\u{1D49C}", "𝒜"),
        (r"var x=a\u0030;", (4, 5), (6, 13), r"a\u0030", "a0"),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        let [binding] = statement.bindings() else {
            panic!("expected one selected variable binding for {text:?}");
        };
        assert_eq!(
            (
                binding.binding().range().start(),
                binding.binding().range().end()
            ),
            expected_binding,
            "{text:?}"
        );
        let reference = binding
            .identifier_reference_initializer()
            .expect("selected var RHS reference fact");
        assert_eq!(
            (
                reference.reference().range().start(),
                reference.reference().range().end()
            ),
            expected_reference,
            "{text:?}"
        );
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn escaped_var_identifier_reference_name_policy_preserves_c1_c6_firewall() {
    for (rhs, expected_semantic) in [
        (r"\u006Cet", "let"),
        (r"\u0073tatic", "static"),
        (r"\u0069mplements", "implements"),
        (r"\u0069nterface", "interface"),
        (r"\u0070ackage", "package"),
        (r"\u0070rivate", "private"),
        (r"\u0070rotected", "protected"),
        (r"\u0070ublic", "public"),
        (r"\u0079ield", "yield"),
        (r"a\u0077ait", "await"),
        (r"\u0065val", "eval"),
        (r"\u0061rguments", "arguments"),
    ] {
        let text = format!("var x={rhs};");
        let script = recognized_variable(&text);
        let statement = only_variable_statement(&script);
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .expect("C1 escaped name-policy positive");
        assert_eq!(reference.reference().fragment(), rhs, "{text}");
        assert_eq!(reference.semantic_name(), expected_semantic, "{text}");
    }

    for text in [
        r"var x=\u0030;",
        r"var x=a\u002Db;",
        r"var x=\u{};",
        r"var x=\u{110000};",
        r"var x=\uD800;",
        r"var x=\u{D800};",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn var_identifier_reference_boundary_keeps_richer_expression_and_literal_neighbors_unsupported() {
    // "var x=this;" is deliberately not listed here: Issue #723 makes a
    // direct-authored `PrimaryExpression : this` initializer a selected
    // accepted form (see the "Issue #723" test section below). "var
    // x="foo";" is also deliberately not listed here: Issue #725 makes a
    // direct-authored, escape-free `StringLiteral` initializer a selected
    // accepted form (see the "Issue #725" test section below).
    for text in [
        "var x=foo.bar;",
        "var x=foo();",
        "var x=foo+1;",
        "var x=(foo);",
        "var x=foo=bar;",
        "var x=foo?bar:baz;",
        "var x=foo/*comment*/;",
        r"var x=\u0066oo.bar;",
        r"var x=\u0066oo();",
        r"var x=\u0066oo+1;",
        r"var x=\u0066oo/*comment*/;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn var_multi_reference_initializers_preserve_per_binding_association_order_and_semantics() {
    for (text, expected_refs, expected_terminator) in [
        (
            "var x=foo,y=bar;",
            &[Some((6, 9, "foo", "foo")), Some((12, 15, "bar", "bar"))][..],
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=\u0066oo,y=\u0062ar;",
            &[
                Some((6, 14, r"\u0066oo", "foo")),
                Some((17, 25, r"\u0062ar", "bar")),
            ][..],
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=\u0066oo,y=bar;",
            &[
                Some((6, 14, r"\u0066oo", "foo")),
                Some((17, 20, "bar", "bar")),
            ][..],
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=foo,y=\u0062ar;",
            &[
                Some((6, 9, "foo", "foo")),
                Some((12, 20, r"\u0062ar", "bar")),
            ][..],
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=\u0066oo,y=2,z=bar;",
            &[
                Some((6, 14, r"\u0066oo", "foo")),
                None,
                Some((21, 24, "bar", "bar")),
            ][..],
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=\u0066oo,y;",
            &[Some((6, 14, r"\u0066oo", "foo")), None][..],
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=\u0066oo,y=\u0062ar,z=baz",
            &[
                Some((6, 14, r"\u0066oo", "foo")),
                Some((17, 25, r"\u0062ar", "bar")),
                Some((28, 31, "baz", "baz")),
            ][..],
            SelectedVariableStatementTerminator::AutomaticAtEof,
        ),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(statement.bindings().len(), expected_refs.len(), "{text:?}");
        assert_eq!(statement.terminator(), expected_terminator, "{text:?}");
        for (binding, expected) in statement.bindings().iter().zip(expected_refs) {
            match (binding.identifier_reference_initializer(), expected) {
                (Some(reference), Some((start, end, fragment, semantic_name))) => {
                    assert_eq!(
                        (
                            reference.reference().range().start(),
                            reference.reference().range().end()
                        ),
                        (*start, *end),
                        "{text:?}"
                    );
                    assert_eq!(reference.reference().fragment(), *fragment, "{text:?}");
                    assert_eq!(reference.semantic_name(), *semantic_name, "{text:?}");
                }
                (None, None) => {}
                (actual, expected) => {
                    panic!(
                        "wrong var-reference association for {text:?}: {actual:?} vs {expected:?}"
                    )
                }
            }
        }
    }
}

#[test]
fn incomplete_or_widened_declarator_lists_commit_no_selected_statement() {
    // "{ var a,b; }" is deliberately not listed here: Issue #695 makes
    // multi-declarator Block `var` statements a selected accepted form.
    // "var a=1,b=1.0;" is also deliberately not listed here: Issue #732
    // makes a direct-authored plain fractional `DecimalLiteral` initializer
    // a selected accepted form.
    for text in [
        "var a,",
        "var a,b,",
        "var ,a;",
        "var a,,b;",
        "var a=1,",
        "var a=1,b=",
        "var a,{b}=c;",
        "var a, /*comment*/ b;",
        "var a=1, /*comment*/ b;",
        "var a,b\nvar c;",
        "var a=1\nvar c;",
        "var x=foo,y=bar,z=",
        r"var x=\u0066oo,y=\u0062ar,z=",
        "for (var a,b;;) {}",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn multi_declarator_statements_compose_with_existing_selected_top_level_items() {
    let script = recognized_variable("let a; { let b; } var c,d; var e; let f");
    assert_eq!(script.items().len(), 5);
    let SelectedVariableTopLevelItem::VariableStatement(first) = &script.items()[2] else {
        panic!("third item must be the multi-declarator VariableStatement");
    };
    assert_eq!(binding_fragments(first), ["c", "d"]);
    assert_eq!(binding_ranges(first), [(22, 23), (24, 25)]);
    let SelectedVariableTopLevelItem::VariableStatement(second) = &script.items()[3] else {
        panic!("fourth item must be the single-declarator VariableStatement");
    };
    assert_eq!(binding_fragments(second), ["e"]);
    let SelectedVariableTopLevelItem::LexicalDeclaration(last) = &script.items()[4] else {
        panic!("trailing lexical declaration must stay lexical");
    };
    assert!(matches!(
        last.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));
}

#[test]
fn failed_declaration_lists_commit_no_selected_binding_or_statement_state() {
    // "var a=1,b=1.0;" is deliberately not listed here: Issue #732 makes a
    // direct-authored plain fractional `DecimalLiteral` initializer a
    // selected accepted form.
    for text in [
        r"var a,\u{};",
        r"var a,b,\u{};",
        "var a,b,",
        "var a=1,",
        "var a=1,b=",
        r"var a=1,b,\u{}=2;",
        "var x=foo,y=bar,z=",
        r"var x=foo,y=bar,\u{}=baz;",
        r"var x=\u0066oo,y=\u0062ar,z=",
        r"var x=\u0066oo,y=bar,\u{}=baz;",
    ] {
        match recognize_selected_lexical_slice(&source(text)) {
            SelectedLexicalSliceOutcome::UnsupportedCoverage
            | SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { .. } => {}
            other => {
                panic!("incomplete list must not commit selected state for {text:?}: {other:?}")
            }
        }
    }

    let subject = grammar_rejection(r"var a=1,b,\u{}=2;");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (10, 14));

    let subject = grammar_rejection(r"var x=foo,y=bar,\u{}=baz;");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (16, 20));

    let subject = grammar_rejection(r"var x=\u0066oo,y=bar,\u{}=baz;");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (21, 25));
}

#[test]
fn var_escaped_reserved_initializer_retains_classification_only_exact_anchor() {
    for (text, expected_fragment, expected_range, expected_terminator) in [
        (
            r"var x=\u0069f;",
            r"\u0069f",
            (6, 13),
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=n\u0075ll;",
            r"n\u0075ll",
            (6, 15),
            SelectedVariableStatementTerminator::AuthoredSemicolon,
        ),
        (
            r"var x=\u0069f",
            r"\u0069f",
            (6, 13),
            SelectedVariableStatementTerminator::AutomaticAtEof,
        ),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        let [binding] = statement.bindings() else {
            panic!("expected exactly one binding for {text:?}");
        };
        assert!(
            binding.identifier_reference_initializer().is_none(),
            "{text}"
        );
        let identifier = binding
            .escaped_reserved_initializer_identifier()
            .expect("C6 initializer must retain classification-only authored evidence");
        assert_eq!(identifier.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (identifier.range().start(), identifier.range().end()),
            expected_range,
            "{text}"
        );
        assert_eq!(statement.terminator(), expected_terminator, "{text}");
    }
}

#[test]
fn var_escaped_reserved_initializer_preserves_per_binding_cardinality_and_order() {
    let script = recognized_variable(r"var a=1,b=\u0069f,c;");
    let statement = only_variable_statement(&script);
    assert_eq!(binding_fragments(statement), ["a", "b", "c"]);
    assert!(
        statement.bindings()[0]
            .escaped_reserved_initializer_identifier()
            .is_none()
    );
    let second = statement.bindings()[1]
        .escaped_reserved_initializer_identifier()
        .expect("second declarator C6 evidence");
    assert_eq!((second.range().start(), second.range().end()), (10, 17));
    assert!(
        statement.bindings()[2]
            .escaped_reserved_initializer_identifier()
            .is_none()
    );

    let script = recognized_variable(r"var a=\u0069f,b=\u0074his;");
    let statement = only_variable_statement(&script);
    let first = statement.bindings()[0]
        .escaped_reserved_initializer_identifier()
        .expect("first C6 evidence");
    let second = statement.bindings()[1]
        .escaped_reserved_initializer_identifier()
        .expect("second C6 evidence");
    assert_eq!((first.range().start(), first.range().end()), (6, 13));
    assert_eq!((second.range().start(), second.range().end()), (16, 25));

    let script = recognized_variable(r"var a=\u0066oo,b=\u0069f;");
    let statement = only_variable_statement(&script);
    let c1 = statement.bindings()[0]
        .identifier_reference_initializer()
        .expect("first declarator must remain C1");
    assert_eq!(
        (c1.reference().range().start(), c1.reference().range().end()),
        (6, 14)
    );
    assert!(
        statement.bindings()[0]
            .escaped_reserved_initializer_identifier()
            .is_none()
    );
    let c6 = statement.bindings()[1]
        .escaped_reserved_initializer_identifier()
        .expect("second declarator must retain C6 evidence");
    assert_eq!((c6.range().start(), c6.range().end()), (17, 24));
    assert!(
        statement.bindings()[1]
            .identifier_reference_initializer()
            .is_none()
    );
}

#[test]
fn var_escaped_reserved_initializer_preserves_transaction_and_neighbor_firewalls() {
    for text in [
        r"var x=\u0069f,y=",
        r"var x=\u0069f.foo;",
        "var x=if;",
        r"var x=\u0030;",
        r"var x=a\u002Db;",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"var x=\u0069f,\u{};");
    assert_eq!(subject.fragment(), r"\u{}");
    assert_eq!((subject.range().start(), subject.range().end()), (14, 18));
}

// --- Issue #719: top-level `VariableStatement` widened to a direct- ------
// authored `BooleanLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5690396598 / #719 theorem, reusing the existing accepted
// #245/#247/#248 direct-Boolean recognizer (`consume_selected_boolean_literal`)
// and its maximal-IdentifierName boundary rather than cloning that
// independent oracle.

#[test]
fn top_level_var_direct_boolean_literal_initializer_composes_as_presence_only() {
    for (text, expected_names) in [
        ("var x=true;", &["x"][..]),
        ("var x=false;", &["x"][..]),
        ("var x=true", &["x"][..]),
        ("var x=false", &["x"][..]),
        ("var a=true,b=false;", &["a", "b"][..]),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(binding_fragments(statement), expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn top_level_var_mixed_decimal_boolean_identifier_reference_preserves_correspondence_ownership() {
    let script = recognized_variable("var a=1,b=true,c=x;");
    let statement = only_variable_statement(&script);
    assert!(
        statement.bindings()[0]
            .identifier_reference_initializer()
            .is_none()
    );
    assert!(
        statement.bindings()[1]
            .identifier_reference_initializer()
            .is_none()
    );
    let reference = statement.bindings()[2]
        .identifier_reference_initializer()
        .expect("third declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn top_level_var_direct_boolean_literal_boundary_preserves_maximal_identifier_reference_routing() {
    for (text, expected_fragment, expected_semantic) in [
        ("var x=truex;", "truex", "truex"),
        ("var x=falseValue;", "falseValue", "falseValue"),
        (r"var x=true\u0061;", r"true\u0061", "truea"),
        (r"var x=false\u0061;", r"false\u0061", "falsea"),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .unwrap_or_else(|| panic!("expected IdentifierReference routing for {text:?}"));
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn top_level_var_direct_boolean_literal_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        "var x=true.foo;",
        "var x=false();",
        "var x=true + y;",
        "var x=false = y;",
        "var x=true ? a : b;",
        "var x=true/*c*/;",
        r"var x=true\u{};",
        r"var x=false\u0;",
        r"var x=true\u{61",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_direct_boolean_literal_escaped_reserved_spelling_is_not_boolean_literal() {
    for text in [r"var x=\u0074rue;", r"var x=\u0066alse;"] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert!(
            statement.bindings()[0]
                .escaped_reserved_initializer_identifier()
                .is_some(),
            "{text:?}"
        );
        assert!(
            statement.bindings()[0]
                .identifier_reference_initializer()
                .is_none(),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_var_direct_boolean_literal_transactional_failure_commits_no_prefix() {
    for text in ["var a=true,b=;", "var a=false,b="] {
        assert_unsupported(text);
    }
}

// --- Issue #721: top-level `VariableStatement` widened to a direct- ------
// authored `NullLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5690791939 / #721 theorem, reusing the existing accepted
// #249/#250/#251/#252 direct-NullLiteral recognizer
// (`consume_selected_null_literal`) and its maximal-IdentifierName boundary
// rather than cloning that independent oracle. The escaped-semantic-`null`
// C6 theorem is already sealed by
// `var_escaped_reserved_initializer_retains_classification_only_exact_anchor`
// above and is intentionally not duplicated here.

#[test]
fn top_level_var_direct_null_literal_initializer_composes_as_presence_only() {
    for (text, expected_names) in [
        ("var x=null;", &["x"][..]),
        ("var x=null", &["x"][..]),
        ("var a=null,b=null;", &["a", "b"][..]),
        ("var a=null,b=null", &["a", "b"][..]),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(binding_fragments(statement), expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn top_level_var_mixed_decimal_boolean_null_identifier_reference_preserves_correspondence_ownership()
 {
    let script = recognized_variable("var a=null,b=1,c=true,d=x;");
    let statement = only_variable_statement(&script);
    for index in 0..3 {
        assert!(
            statement.bindings()[index]
                .identifier_reference_initializer()
                .is_none()
        );
    }
    let reference = statement.bindings()[3]
        .identifier_reference_initializer()
        .expect("fourth declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn top_level_var_direct_null_literal_boundary_preserves_maximal_identifier_reference_routing() {
    for (text, expected_fragment, expected_semantic) in [
        ("var x=nullx;", "nullx", "nullx"),
        ("var x=nullValue;", "nullValue", "nullValue"),
        (r"var x=null\u0061;", r"null\u0061", "nulla"),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .unwrap_or_else(|| panic!("expected IdentifierReference routing for {text:?}"));
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn top_level_var_direct_null_literal_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        "var x=null.foo;",
        "var x=null();",
        "var x=null + y;",
        "var x=null = y;",
        "var x=null ? a : b;",
        "var x=null/*c*/;",
        r"var x=null\u{};",
        r"var x=null\u0;",
        r"var x=null\u{61",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_direct_null_literal_transactional_failure_commits_no_prefix() {
    for text in ["var a=null,b=;", "var a=null,b="] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_null_literal_formed_invalid_continuation_does_not_steal_identifier_reference_boundary() {
    // Issue #721 adversarial audit: a formed authored UES continuation that
    // cannot extend the maximal IdentifierName (`\u002D` decodes to `-`, not
    // an IdentifierPart) must not let `consume_selected_null_literal` commit
    // a `null` prefix in either selected `var` placement. The existing
    // IdentifierReference/UES owner remains authoritative for the whole-
    // source outcome, distinct from the malformed-UES controls above
    // (`null\u{}`, `null\u0`, `null\u{61`) and from the non-CodePoint
    // control below (`null\u{110000}`).
    for text in [r"var x=null\u002D;", r"{ var x=null\u002D; }"] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_null_literal_non_codepoint_continuation_remains_unsupported() {
    // Issue #721 adversarial audit: a UES continuation encoding a value
    // outside the Unicode code point range must not let
    // `consume_selected_null_literal` commit a partial `null` prefix, or
    // fabricate any RHS Grammar/static evidence, in either selected `var`
    // placement. The existing owning layer remains authoritative for the
    // whole-source classification.
    for text in [r"var x=null\u{110000};", r"{ var x=null\u{110000}; }"] {
        assert_unsupported(text);
    }
}

// --- Issue #723: top-level `VariableStatement` widened to a direct- ------
// authored `PrimaryExpression : this` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5691238988 / #723 theorem, reusing the existing accepted
// #253/#254/#255/#256 direct-`this` recognizer
// (`consume_selected_this_expression`) and its maximal-IdentifierName
// boundary rather than cloning that independent oracle. The escaped-
// semantic-`this` C6 theorem is already sealed by
// `var_escaped_reserved_initializer_retains_classification_only_exact_anchor`
// above and further sealed below with dedicated `this`-specific spellings;
// it is intentionally not fully duplicated here.

#[test]
fn top_level_var_direct_this_initializer_composes_as_presence_only() {
    for (text, expected_names) in [
        ("var x=this;", &["x"][..]),
        ("var x=this", &["x"][..]),
        ("var a=this,b=this;", &["a", "b"][..]),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(binding_fragments(statement), expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn top_level_var_mixed_decimal_boolean_null_this_identifier_reference_preserves_correspondence_ownership()
 {
    let script = recognized_variable("var a=1,b=true,c=null,d=this,e=x;");
    let statement = only_variable_statement(&script);
    for index in 0..4 {
        assert!(
            statement.bindings()[index]
                .identifier_reference_initializer()
                .is_none()
        );
    }
    let reference = statement.bindings()[4]
        .identifier_reference_initializer()
        .expect("fifth declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn top_level_var_direct_this_boundary_preserves_maximal_identifier_reference_routing() {
    for (text, expected_fragment, expected_semantic) in [
        ("var x=thisx;", "thisx", "thisx"),
        ("var x=thisValue;", "thisValue", "thisValue"),
        ("var x=this0;", "this0", "this0"),
        ("var x=this$;", "this$", "this$"),
        ("var x=this_;", "this_", "this_"),
        (r"var x=this\u0061;", r"this\u0061", "thisa"),
        (r"var x=this\u{61};", r"this\u{61}", "thisa"),
        (r"var x=this\u0030;", r"this\u0030", "this0"),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        let reference = statement.bindings()[0]
            .identifier_reference_initializer()
            .unwrap_or_else(|| panic!("expected IdentifierReference routing for {text:?}"));
        assert_eq!(
            reference.reference().fragment(),
            expected_fragment,
            "{text:?}"
        );
        assert_eq!(reference.semantic_name(), expected_semantic, "{text:?}");
    }
}

#[test]
fn top_level_var_direct_this_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        "var x=this.foo;",
        "var x=this();",
        "var x=this + y;",
        "var x=this = y;",
        "var x=this ? a : b;",
        "var x=this/*c*/;",
        r"var x=this\u{};",
        r"var x=this\u0;",
        r"var x=this\u{61",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_direct_this_escaped_reserved_spelling_is_not_this_expression() {
    for text in [r"var x=\u0074his;", r"var x=t\u0068is;"] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert!(
            statement.bindings()[0]
                .escaped_reserved_initializer_identifier()
                .is_some(),
            "{text:?}"
        );
        assert!(
            statement.bindings()[0]
                .identifier_reference_initializer()
                .is_none(),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_var_direct_this_transactional_failure_commits_no_prefix() {
    for text in ["var a=this,b=;", "var a=this,b="] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_this_formed_invalid_continuation_does_not_steal_identifier_reference_boundary() {
    // Issue #723 adversarial audit: a formed authored UES continuation that
    // cannot extend the maximal IdentifierName (`-` decodes to `-`, not
    // an IdentifierPart) must not let `consume_selected_this_expression`
    // commit a `this` prefix in either selected `var` placement. The
    // existing IdentifierReference/UES owner remains authoritative for the
    // whole-source outcome, distinct from the malformed-UES controls above
    // (`this\u{}`, `this\u0`, `this\u{61`) and from the surrogate/non-
    // CodePoint controls below (`this\uD800`, `this\u{D800}`,
    // `this\u{110000}`).
    for text in [r"var x=this\u002D;", r"{ var x=this\u002D; }"] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_this_surrogate_continuation_remains_unsupported() {
    // Issue #723 adversarial audit: a formed authored UES continuation that
    // decodes to a lone surrogate code point must not let
    // `consume_selected_this_expression` commit a partial `this` prefix, or
    // fabricate any RHS Grammar/static evidence, in either selected `var`
    // placement.
    for text in [
        r"var x=this\uD800;",
        r"var x=this\u{D800};",
        r"{ var x=this\uD800; }",
        r"{ var x=this\u{D800}; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn direct_this_non_codepoint_continuation_remains_unsupported() {
    // Issue #723 adversarial audit: a UES continuation encoding a value
    // outside the Unicode code point range must not let
    // `consume_selected_this_expression` commit a partial `this` prefix, or
    // fabricate any RHS Grammar/static evidence, in either selected `var`
    // placement. The existing owning layer remains authoritative for the
    // whole-source classification.
    for text in [r"var x=this\u{110000};", r"{ var x=this\u{110000}; }"] {
        assert_unsupported(text);
    }
}

// --- Issue #725: direct-authored, escape-free `StringLiteral` ------------
// initializers in both selected `var` placements
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5691565519 / #725 theorem: existing top-level and Block
// `var` terminator, transactionality, static-tier, and lifecycle authority
// must compose unchanged with the newly admitted direct, escape-free
// `StringLiteral` initializer reused unchanged from #257/#258/#259/#260.
// A quoted `StringLiteral` has no maximal-IdentifierName continuation
// boundary, so, unlike the preceding BooleanLiteral/NullLiteral/`this`
// keyword leaves, no dedicated boundary or escaped-reserved-spelling test
// is required here.

#[test]
fn top_level_var_direct_string_literal_initializer_composes_as_presence_only() {
    for (text, expected_names) in [
        ("var x=\"\";", &["x"][..]),
        ("var x='';", &["x"][..]),
        ("var x=\"𝒜\";", &["x"][..]),
        ("var x=\"a'b\";", &["x"][..]),
        ("var x='a\"b';", &["x"][..]),
        ("var x=\"a b\";", &["x"][..]),
        ("var x=\"a\tb\";", &["x"][..]),
        ("var x=\"abc\"", &["x"][..]),
        ("var a=\"x\",b='y';", &["a", "b"][..]),
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(binding_fragments(statement), expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn one_level_block_var_direct_string_literal_initializer_composes_as_presence_only() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for (text, expected_names) in [
        ("{ var x=\"\"; }", &["x"][..]),
        ("{ var x=''; }", &["x"][..]),
        ("{ var x=\"abc\"; }", &["x"][..]),
        ("{ var x='abc' }", &["x"][..]),
        ("{ var a=\"x\",b='y'; }", &["a", "b"][..]),
        ("{ var a=\"x\",b='y' }", &["a", "b"][..]),
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, expected_names, "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn var_direct_string_literal_line_separator_paragraph_separator_are_direct_content() {
    for text in [
        "var x=\"a\u{2028}b\";",
        "var x=\"a\u{2029}b\";",
        "var x='a\u{2028}b';",
        "var x='a\u{2029}b';",
    ] {
        let _ = recognized_variable(text);
    }

    for text in ["{ var x=\"a\u{2028}b\"; }", "{ var x='a\u{2029}b'; }"] {
        let _ = recognized_block(text);
    }
}

#[test]
fn var_direct_string_literal_raw_line_terminator_boundary_is_exact() {
    for text in [
        "var x=\"a\nb\";",
        "var x=\"a\rb\";",
        "var x=\"a\r\nb\";",
        "var x='a\nb';",
        "var x='a\rb';",
        "{ var x=\"a\nb\"; }",
        "{ var x='a\nb'; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_direct_string_literal_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        r#"var x="\n";"#,
        r#"var x="\u0061";"#,
        r#"var x="\x61";"#,
        r#"var x="\\";"#,
        r#"var x="\"";"#,
        "var x='\\'';",
        "var x=\"\\\n\";",
        "var x='\\\r\n';",
        "var x=\"abc;",
        "var x='abc;",
        "var x=\"abc';",
        "var x='abc\";",
        "var x=\"abc\".length;",
        "var x=\"abc\"();",
        "var x=\"abc\" + x;",
        "var x=\"abc\" = x;",
        "var x=\"abc\" ? x : y;",
        "var x=\"abc\"/*comment*/;",
        "var x=\"abc\" unexpected;",
        "var x=\"abc\";;",
        "var x=\"abc\"\nvar y=foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_direct_string_literal_does_not_claim_richer_or_malformed_neighbors() {
    for text in [
        r#"{ var x="\n"; }"#,
        r#"{ var x="\u0061"; }"#,
        r#"{ var x="\\"; }"#,
        "{ var x=\"abc; }",
        "{ var x=\"abc'; }",
        "{ var x=\"abc\".length; }",
        "{ var x=\"abc\"/*comment*/; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_mixed_decimal_boolean_null_this_string_identifier_reference_preserves_correspondence_ownership()
 {
    let script = recognized_variable("var a=1,b=true,c=null,d=this,e=\"s\",f=x;");
    let statement = only_variable_statement(&script);
    for index in 0..5 {
        assert!(
            statement.bindings()[index]
                .identifier_reference_initializer()
                .is_none()
        );
    }
    let reference = statement.bindings()[5]
        .identifier_reference_initializer()
        .expect("sixth declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn one_level_block_var_mixed_decimal_boolean_null_this_string_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let script = recognized_block("{ var a=false,b=1,c=null,d=this,e=\"s\",f=x }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    for index in 0..5 {
        assert!(
            statement.bindings()[index]
                .identifier_reference_initializer()
                .is_none()
        );
    }
    let reference = statement.bindings()[5]
        .identifier_reference_initializer()
        .expect("sixth declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn top_level_var_direct_string_literal_transactional_failure_commits_no_prefix() {
    for text in ["var a=\"x\",b=;", "var a=\"x\",b="] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_direct_string_literal_transactional_failure_commits_no_prefix() {
    for text in ["{ var a=\"x\",b=; }", "{ var a=\"x\",b= }"] {
        assert_unsupported(text);
    }
}

// --- Issue #730: selected `LexicalDeclaration` widened to a direct- --------
// authored plain fractional `DecimalLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #727/#728 candidate-independent atom theorem and the
// #729-comment-5697432768 / #688-comment-5697440908 production-placement
// authority: a new bounded `SelectedPlainFractionalDecimalLiteral`
// recognizer is tried before the unchanged `consume_selected_decimal_integer`
// predecessor in `parse_declaration` only. Top-level `var` and one-level
// Block `var` initializer dispatch remain untouched hard-zero surfaces for
// this leaf (see the dedicated asymmetry test below). No numeric value,
// digit, or source-anchor fact is retained beyond the existing
// `SelectedInitializerState::SelectedPresent`.

#[test]
fn plain_fractional_decimal_literal_initializers_compose_as_presence_only() {
    for text in [
        "const x = 0.;",
        "const x = 1.;",
        "const x = 1.0;",
        "const x = 12.34;",
        "const x = .0;",
        "const x = .5;",
        "const x = 123456789.987654321;",
        "const x = 0.0;",
        "const x = 999.;",
        "const x = .0001;",
        "const x = 1.0",
        "const x = .5   \t\n",
        "let x = 1.0, y;",
        "let x, y = .5;",
        "let x = 1, y = 2.5;",
        "let x = true, y = 3.0;",
        "let x = null, y = .25;",
        "let x = this, y = 4.0;",
        "let x = \"a\", y = .25;",
        "let x = foo, y = 2.0;",
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

    // Authored binding order and per-binding initializer presence are
    // preserved exactly, in both fractional-first and fractional-second
    // shape.
    let script = recognized("let x = 1.0, y;");
    let bindings = script.declarations()[0].bindings();
    assert_eq!(bindings[0].binding().fragment(), "x");
    assert_eq!(
        bindings[0].initializer(),
        SelectedInitializerState::SelectedPresent
    );
    assert_eq!(bindings[1].binding().fragment(), "y");
    assert_eq!(bindings[1].initializer(), SelectedInitializerState::Absent);

    let script = recognized("let x, y = .5;");
    let bindings = script.declarations()[0].bindings();
    assert_eq!(bindings[0].binding().fragment(), "x");
    assert_eq!(bindings[0].initializer(), SelectedInitializerState::Absent);
    assert_eq!(bindings[1].binding().fragment(), "y");
    assert_eq!(
        bindings[1].initializer(),
        SelectedInitializerState::SelectedPresent
    );
}

#[test]
fn plain_fractional_decimal_literal_preserves_integer_predecessor_and_overlapping_prefix() {
    // Existing selected decimal-integer initializers remain unchanged: the
    // new fractional recognizer must decline without commit so the
    // unmodified `consume_selected_decimal_integer` predecessor still owns
    // these atoms (falsifies W1/W2).
    for text in [
        "let x = 0;",
        "let x = 1;",
        "let x = 9;",
        "let x = 10;",
        "let x = 42;",
        "let x = 123;",
    ] {
        let _ = recognized(text);
    }

    // Overlapping-prefix theorem: `1` still routes through the unchanged
    // integer predecessor, `1.` / `1.0` / `.5` route through the new
    // fractional recognizer, and a bare `.` remains unsupported (falsifies
    // W3/W4/W5).
    let _ = recognized("let x = 1;");
    let _ = recognized("let x = 1.;");
    let _ = recognized("let x = 1.0;");
    let _ = recognized("let x = .5;");
    assert_unsupported("let x = .;");
}

#[test]
fn plain_fractional_decimal_literal_does_not_claim_numeric_or_richer_neighbors() {
    for text in [
        // numeric-neighbor firewall: remains whole-source `UnsupportedCoverage`
        //
        // `1e2` / `1.0e2` / `.5e2` are no longer negative controls here:
        // Issue #738 makes the selected `LexicalDeclaration` initializer
        // position additionally admit these as the new exponent atom; see
        // the "Issue #738" test section below for full coverage.
        "let x = 1_0;",
        "let x = 1.0_0;",
        "let x = .5_0;",
        "let x = 1n;",
        "let x = 0x10;",
        "let x = 0X10;",
        "let x = 0b10;",
        "let x = 0B10;",
        "let x = 0o10;",
        "let x = 0O10;",
        "let x = 01;",
        "let x = 01.0;",
        // "let x = +1.0;" / "let x = -1.0;" are no longer negative controls
        // here: Issue #744 makes the selected `LexicalDeclaration`
        // initializer position additionally admit these as the new leading
        // `+`/`-` decimal `UnaryExpression`; see the "Issue #744" test
        // section below for full coverage.
        // dot / richer-expression firewall: a locally complete fractional
        // prefix never authorizes the enclosing whole-declaration source
        "let x = .;",
        "let x = ..;",
        "let x = 1..foo;",
        "let x = 1.0.foo;",
        "let x = .5.foo;",
        "let x = 1.0();",
        "let x = .5();",
        "let x = 1.0 + x;",
        "let x = .5 + x;",
        "let x = 1.0 = x;",
        "let x = 1.0 ? x : y;",
        "let x = 1.0/*comment*/;",
        "let x = 1.0 unexpected;",
        "let x = 1.0;;",
        "let x = 1.0\nlet y = foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn plain_fractional_decimal_literal_eof_asi_preserves_significant_end_before_selected_trivia() {
    let text = "const x = 1.0   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = 1.0");
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
fn plain_fractional_decimal_literal_transactionality_commits_no_earlier_prefix() {
    // No valid earlier fractional binding escapes as committed selected
    // success when a later binding/initializer/terminator fails (falsifies
    // W14).
    for text in [
        "let a = 1.0, b = ;",
        "let a = .5, b = 1e;",
        "let a = 1.0, b =",
    ] {
        assert_unsupported(text);
    }

    // A later already-owned malformed `BindingIdentifier` Grammar-evidence
    // case remains authoritative when preceded by a fractional initializer.
    let subject = grammar_rejection(r"let a = 1.0, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn plain_fractional_decimal_literal_coverage_makes_later_existing_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = 1.0; let \u{};", r"\u{}", (19, 23)),
        (r"const x = 1.0; let a\u{};", r"\u{}", (20, 24)),
        (r"const x = 1.0; let \u{61", r"\u{61", (19, 24)),
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
fn plain_fractional_decimal_literal_aggregate_lifecycle_remains_incomplete_or_existing_rejection() {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = 1.0;",
        "const x = .5;",
        "const x = 12.34;",
        "let x = 1.0, y = foo;",
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
        (r"let \u0030 = 1.0;", r"\u0030", (4, 10)),
        (r"let \u0069f = 1.0;", r"\u0069f", (4, 11)),
        ("let let = 1.0;", "let", (4, 7)),
        ("let x = 1.0, x = foo;", "x", (13, 14)),
        ("const x = 1.0, y;", "y", (15, 16)),
        ("let x = 1.0; let x = foo;", "x", (17, 18)),
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
        attempt_selected_qualification(&source(r"const x = 1.0; let \u{};"))
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
    assert_eq!((subject.range().start(), subject.range().end()), (19, 23));
}

#[test]
fn top_level_var_and_block_var_plain_fractional_decimal_literal_composes_via_shared_recognizer() {
    // Issue #732 widens both selected `var` owners to additionally accept
    // the unchanged fractional recognizer already accepted by #730/#731 for
    // `parse_declaration`, superseding the #730-era hard-zero boundary that
    // kept top-level and Block `var` initializer dispatch on
    // `UnsupportedCoverage` for these sources. See the dedicated Issue #732
    // section below for the full positive/boundary/transactionality matrix.
    let _ = recognized_variable("var x = 1.0;");
    let _ = recognized_variable("var x = .5;");
    let _ = recognized_block("{ var x = 1.0; }");
    let _ = recognized_block("{ var x = .5; }");

    // Existing selected integer `var` recognition remains unaffected.
    let _ = recognized_variable("var x = 1;");
    let _ = recognized_block("{ var x = 1; }");
}

#[test]
fn plain_fractional_decimal_literal_recognition_is_deterministic_across_repeats() {
    let text = "let x = 1.0, y = .25; const z = 12.34;";
    let first = recognized(text);
    let second = recognized(text);
    assert_eq!(first.declarations().len(), second.declarations().len());
    for (a, b) in first
        .declarations()
        .iter()
        .zip(second.declarations().iter())
    {
        assert_eq!(
            (
                a.declaration().range().start(),
                a.declaration().range().end()
            ),
            (
                b.declaration().range().start(),
                b.declaration().range().end()
            )
        );
        assert_eq!(a.bindings().len(), b.bindings().len());
        for (ba, bb) in a.bindings().iter().zip(b.bindings().iter()) {
            assert_eq!(
                (ba.binding().range().start(), ba.binding().range().end()),
                (bb.binding().range().start(), bb.binding().range().end())
            );
            assert_eq!(ba.initializer(), bb.initializer());
        }
    }
}

// --- Issue #732: both selected `var` placements widened to a direct- ------
// authored plain fractional `DecimalLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5698108598 joint-composition theorem: the existing
// `SelectedPlainFractionalDecimalLiteral` recognizer (#727/#728, reused
// unchanged from #730/#731) is tried before the unchanged
// `consume_selected_decimal_integer` predecessor in both
// `parse_variable_statement` and `parse_selected_block_var_statement`. Both
// existing terminator owners (top-level EOF-only ASI, Block close-brace
// ASI), transactionality, and existing sibling-initializer/correspondence
// ownership remain unchanged and compose exactly as for the preceding
// Boolean/Null/`this`/StringLiteral leaves.

#[test]
fn top_level_var_plain_fractional_decimal_literal_initializers_compose_as_presence_only() {
    for text in [
        "var x = 0.;",
        "var x = 1.;",
        "var x = 1.0;",
        "var x = 12.34;",
        "var x = .0;",
        "var x = .5;",
        "var x = 123456789.987654321;",
        "var x = 0.0;",
        "var x = 999.;",
        "var x = .0001;",
        "var x = 1.0",
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(binding_fragments(statement), ["x"], "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn one_level_block_var_plain_fractional_decimal_literal_initializers_compose_as_presence_only() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    for text in [
        "{ var x = 0.; }",
        "{ var x = 1.; }",
        "{ var x = 1.0; }",
        "{ var x = 12.34; }",
        "{ var x = .0; }",
        "{ var x = .5; }",
        "{ var x = 1.0 }",
    ] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        let names: Vec<_> = statement
            .bindings()
            .iter()
            .map(|binding| binding.binding().fragment())
            .collect();
        assert_eq!(names, ["x"], "{text:?}");
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }
}

#[test]
fn top_level_var_plain_fractional_decimal_literal_preserves_integer_predecessor_and_overlapping_prefix()
 {
    // Existing selected integer `var` initializers remain unchanged: the
    // fractional recognizer must decline without commit so the unmodified
    // `consume_selected_decimal_integer` predecessor still owns these atoms
    // (falsifies W1/W2).
    for text in [
        "var x = 0;",
        "var x = 1;",
        "var x = 9;",
        "var x = 10;",
        "var x = 42;",
        "var x = 123;",
    ] {
        let _ = recognized_variable(text);
    }

    // Overlapping-prefix theorem: `1` still routes through the unchanged
    // integer predecessor, `1.` / `1.0` / `.5` route through the fractional
    // recognizer, and a bare `.` remains unsupported (falsifies W3/W4/W5).
    let _ = recognized_variable("var x = 1;");
    let _ = recognized_variable("var x = 1.;");
    let _ = recognized_variable("var x = 1.0;");
    let _ = recognized_variable("var x = .5;");
    assert_unsupported("var x = .;");
}

#[test]
fn one_level_block_var_plain_fractional_decimal_literal_preserves_integer_predecessor_and_overlapping_prefix()
 {
    for text in [
        "{ var x = 0; }",
        "{ var x = 1; }",
        "{ var x = 9; }",
        "{ var x = 10; }",
        "{ var x = 42; }",
        "{ var x = 123; }",
    ] {
        let _ = recognized_block(text);
    }

    let _ = recognized_block("{ var x = 1; }");
    let _ = recognized_block("{ var x = 1.; }");
    let _ = recognized_block("{ var x = 1.0; }");
    let _ = recognized_block("{ var x = .5; }");
    assert_unsupported("{ var x = .; }");
}

#[test]
fn top_level_var_plain_fractional_decimal_literal_does_not_claim_numeric_or_richer_neighbors() {
    for text in [
        // numeric-neighbor firewall: remains whole-source `UnsupportedCoverage`
        //
        // `1e2` / `1.0e2` / `.5e2` are no longer negative controls here:
        // Issue #740 makes both selected `var` initializer positions
        // additionally admit these as the new exponent atom; see the
        // "Issue #740" test section below for full coverage.
        "var x = 1_0;",
        "var x = 1.0_0;",
        "var x = .5_0;",
        "var x = 1n;",
        "var x = 0x10;",
        "var x = 0X10;",
        "var x = 0b10;",
        "var x = 0B10;",
        "var x = 0o10;",
        "var x = 0O10;",
        "var x = 01;",
        "var x = 01.0;",
        // "var x = +1.0;" / "var x = -1.0;" are no longer negative controls
        // here: Issue #744 makes both selected `var` initializer positions
        // additionally admit these as the new leading `+`/`-` decimal
        // `UnaryExpression`; see the "Issue #744" test section below for
        // full coverage.
        // dot / richer-expression firewall: a locally complete fractional
        // prefix never authorizes the enclosing whole-statement source
        "var x = .;",
        "var x = ..;",
        "var x = 1..foo;",
        "var x = 1.0.foo;",
        "var x = .5.foo;",
        "var x = 1.0();",
        "var x = .5();",
        "var x = 1.0 + x;",
        "var x = .5 + x;",
        "var x = 1.0 = x;",
        "var x = 1.0 ? x : y;",
        "var x = 1.0/*comment*/;",
        "var x = 1.0 unexpected;",
        "var x = 1.0;;",
        "var x = 1.0\nvar y = foo;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_plain_fractional_decimal_literal_does_not_claim_numeric_or_richer_neighbors()
{
    for text in [
        // `1e2` / `1.0e2` / `.5e2` are no longer negative controls here:
        // Issue #740 makes both selected `var` initializer positions
        // additionally admit these as the new exponent atom; see the
        // "Issue #740" test section below for full coverage.
        "{ var x = 1_0; }",
        "{ var x = 1.0_0; }",
        "{ var x = .5_0; }",
        "{ var x = 1n; }",
        "{ var x = 0x10; }",
        "{ var x = 0X10; }",
        "{ var x = 0b10; }",
        "{ var x = 0B10; }",
        "{ var x = 0o10; }",
        "{ var x = 0O10; }",
        "{ var x = 01; }",
        "{ var x = 01.0; }",
        // "{ var x = +1.0; }" / "{ var x = -1.0; }" are no longer negative
        // controls here: Issue #744 makes both selected `var` initializer
        // positions additionally admit these as the new leading `+`/`-`
        // decimal `UnaryExpression`; see the "Issue #744" test section
        // below for full coverage.
        "{ var x = .; }",
        "{ var x = ..; }",
        "{ var x = 1..foo; }",
        "{ var x = 1.0.foo; }",
        "{ var x = .5.foo; }",
        "{ var x = 1.0(); }",
        "{ var x = .5(); }",
        "{ var x = 1.0 + x; }",
        "{ var x = .5 + x; }",
        "{ var x = 1.0 = x; }",
        "{ var x = 1.0 ? x : y; }",
        "{ var x = 1.0/*comment*/; }",
        "{ var x = 1.0 unexpected; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_plain_fractional_decimal_literal_eof_asi_preserves_ownership() {
    for text in ["var x=1.0", "var x=.5", "var a=1.0,b=.5"] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(
            statement.terminator(),
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_var_plain_fractional_decimal_literal_close_brace_asi_preserves_ownership() {
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    for text in ["{ var x=1.0 }", "{ var x=.5 }", "{ var a=1.0,b=.5 }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(
            statement.terminator(),
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
            "{text:?}"
        );
    }
}

#[test]
fn top_level_var_plain_fractional_decimal_literal_transactionality_commits_no_earlier_prefix() {
    // No valid earlier fractional declarator escapes as committed selected
    // success when a later declarator/initializer/terminator fails
    // (falsifies W15/earlier-prefix-escapes).
    //
    // "var a = .5, b = 1e2;" no longer witnesses this: Issue #740 makes
    // `1e2` a complete accepted exponent atom, so that source is now
    // whole-statement selected success (see the "Issue #740" test section
    // below). `1e` is a genuinely incomplete exponent tail and preserves
    // the same later-initializer-failure theorem.
    for text in [
        "var a = 1.0, b = ;",
        "var a = .5, b = 1e;",
        "var a = 1.0, b =",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_plain_fractional_decimal_literal_transactionality_commits_no_earlier_prefix()
{
    // "{ var a = .5, b = 1e2; }" no longer witnesses this: Issue #740 makes
    // `1e2` a complete accepted exponent atom (see the "Issue #740" test
    // section below). `1e` is a genuinely incomplete exponent tail and
    // preserves the same later-initializer-failure theorem.
    for text in [
        "{ var a = 1.0, b = ; }",
        "{ var a = .5, b = 1e; }",
        "{ var a = 1.0, b = }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_fractional_mixed_with_sibling_atoms_and_identifier_reference_preserves_correspondence_ownership()
 {
    for text in [
        "var a=1.0,b=2.0;",
        "var a=1,b=2.5;",
        "var a=true,b=3.0;",
        "var a=null,b=.25;",
        "var a=this,b=4.0;",
        "var a=\"s\",b=.25;",
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }

    let script = recognized_variable("var a=1.0,b=foo;");
    let statement = only_variable_statement(&script);
    assert!(
        statement.bindings()[0]
            .identifier_reference_initializer()
            .is_none()
    );
    let reference = statement.bindings()[1]
        .identifier_reference_initializer()
        .expect("second declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "foo");
    assert_eq!(reference.semantic_name(), "foo");
}

#[test]
fn one_level_block_var_fractional_mixed_with_sibling_atoms_and_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    fn only_block_var_statement(
        script: &super::selected_lexical_slice::SelectedOneLevelBlockScript,
    ) -> &super::selected_lexical_slice::SelectedBlockVarStatement {
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement");
        };
        statement
    }

    for text in [
        "{ var a=1.0,b=2.0; }",
        "{ var a=1,b=2.5; }",
        "{ var a=true,b=3.0; }",
        "{ var a=null,b=.25; }",
        "{ var a=this,b=4.0; }",
        "{ var a=\"s\",b=.25; }",
    ] {
        let script = recognized_block(text);
        let statement = only_block_var_statement(&script);
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }

    let script = recognized_block("{ var a=1.0,b=foo; }");
    let statement = only_block_var_statement(&script);
    assert!(
        statement.bindings()[0]
            .identifier_reference_initializer()
            .is_none()
    );
    let reference = statement.bindings()[1]
        .identifier_reference_initializer()
        .expect("second declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "foo");
    assert_eq!(reference.semantic_name(), "foo");
}

// --- Issue #738: selected `LexicalDeclaration` widened to a direct- --------
// authored plain exponent `DecimalLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #735/#736 candidate-independent atom theorem and the
// #737-comment-5706111443 / #688-comment-5706113577 production-placement
// authority: a new bounded `SelectedPlainExponentDecimalLiteral` recognizer
// is tried before the unmodified `consume_selected_plain_fractional_decimal_literal`
// and `consume_selected_decimal_integer` predecessors in `parse_declaration`
// only. Top-level `var` and one-level Block `var` initializer dispatch were
// untouched hard-zero surfaces for this leaf; Issue #740 below composes the
// same accepted atom into both of those owners. No numeric value, digit, or
// source-anchor fact is retained beyond the existing
// `SelectedInitializerState::SelectedPresent`.

#[test]
fn plain_exponent_decimal_literal_initializers_compose_as_presence_only() {
    for text in [
        "const x = 1e2;",
        "const x = 1E2;",
        "const x = 1e+2;",
        "const x = 1e-2;",
        "const x = 1.e2;",
        "const x = 1.E+2;",
        "const x = 1.0e2;",
        "const x = 1.0E-2;",
        "const x = .5e2;",
        "const x = .5E+2;",
        "const x = 12.34E-56;",
        "let x = 1e2, y;",
        "let x, y = .5e2;",
        "let x = 1, y = 2e3;",
        "let x = 1.0, y = 2.5e-3;",
        "let x = true, y = 3E2;",
        "let x = \"a\", y = .25e+2;",
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

    // Authored binding order and per-binding initializer presence are
    // preserved exactly, in both exponent-first and exponent-second shape.
    let script = recognized("let x = 1e2, y;");
    let bindings = script.declarations()[0].bindings();
    assert_eq!(bindings[0].binding().fragment(), "x");
    assert_eq!(
        bindings[0].initializer(),
        SelectedInitializerState::SelectedPresent
    );
    assert_eq!(bindings[1].binding().fragment(), "y");
    assert_eq!(bindings[1].initializer(), SelectedInitializerState::Absent);

    let script = recognized("let x, y = .5e2;");
    let bindings = script.declarations()[0].bindings();
    assert_eq!(bindings[0].binding().fragment(), "x");
    assert_eq!(bindings[0].initializer(), SelectedInitializerState::Absent);
    assert_eq!(bindings[1].binding().fragment(), "y");
    assert_eq!(
        bindings[1].initializer(),
        SelectedInitializerState::SelectedPresent
    );
}

#[test]
fn plain_exponent_decimal_literal_preserves_integer_and_fractional_predecessors() {
    // Existing selected integer and fractional initializers remain
    // unchanged: the new exponent recognizer must decline without commit so
    // the unmodified `consume_selected_plain_fractional_decimal_literal` and
    // `consume_selected_decimal_integer` predecessors still own these atoms.
    for text in [
        "let x = 0;",
        "let x = 1;",
        "let x = 123;",
        "let x = 1.;",
        "let x = 1.0;",
        "let x = .5;",
        "let x = 12.34;",
    ] {
        let _ = recognized(text);
    }

    // Complete-atom ownership: production must select the complete
    // authored exponent atom, not commit only the predecessor-owned
    // mantissa prefix (`1`, `1.0`, `.5` respectively).
    for text in ["let x = 1e2;", "let x = 1.0e2;", "let x = .5e2;"] {
        let script = recognized(text);
        assert_eq!(
            script.declarations()[0].declaration().fragment(),
            text,
            "{text:?}"
        );
    }
}

#[test]
fn plain_exponent_decimal_literal_incomplete_tails_do_not_claim_whole_source() {
    for text in [
        "let x = 1e;",
        "let x = 1E;",
        "let x = 1e+;",
        "let x = 1e-;",
        "let x = 1.e;",
        "let x = 1.E+;",
        "let x = 1.0e-;",
        "let x = .5E+;",
        "let x = 1e",
        "let x = .5e",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn plain_exponent_decimal_literal_does_not_claim_numeric_or_richer_neighbors() {
    for text in [
        // numeric-separator / BigInt / non-decimal / legacy firewall
        "let x = 1e1_0;",
        "let x = 1_0e2;",
        "let x = 1.0_0e2;",
        "let x = .5e1_0;",
        "let x = 1n;",
        "let x = 0x10;",
        "let x = 0b10;",
        "let x = 0o10;",
        "let x = 01;",
        // "let x = +1e2;" / "let x = -1e2;" are no longer negative controls
        // here: Issue #744 makes the selected `LexicalDeclaration`
        // initializer position additionally admit these as the new leading
        // `+`/`-` decimal `UnaryExpression`; see the "Issue #744" test
        // section below for full coverage.
        // richer-expression / comment firewall: a valid exponent literal
        // prefix never authorizes a broader expression
        "let x = 1e2.foo;",
        "let x = 1e2();",
        "let x = 1e2 + x;",
        "let x = 1e2 = x;",
        "let x = 1e2 ? x : y;",
        "let x = 1e2/*comment*/;",
        "let x = 1e2 unexpected;",
        "let x = 1.0e2.foo;",
        "let x = .5e2 + x;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn plain_exponent_decimal_literal_binding_list_composes_in_authored_order() {
    for text in [
        "let x = 1e2, y;",
        "let x, y = .5e2;",
        "let x = 1, y = 2e3;",
        "let x = 1.0, y = 2.5e-3;",
        "let x = true, y = 3E2;",
        "let x = \"a\", y = .25e+2;",
    ] {
        let script = recognized(text);
        assert_eq!(script.declarations()[0].bindings().len(), 2, "{text:?}");
    }
}

#[test]
fn plain_exponent_decimal_literal_transactionality_commits_no_earlier_prefix() {
    // No valid earlier exponent binding escapes as committed selected
    // success when a later binding/initializer/terminator fails.
    for text in [
        "let a = 1e2, b = ;",
        "let a = 1.0e-2, b = 1e;",
        "let a = .5e2, b =",
    ] {
        assert_unsupported(text);
    }

    // A later already-owned malformed `BindingIdentifier` Grammar-evidence
    // case remains authoritative when preceded by a valid exponent
    // initializer: the earlier exponent atom is locally valid but the whole
    // declaration/source must not escape as accepted when later source
    // fails.
    let subject = grammar_rejection(r"let a = 1e2, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn plain_exponent_decimal_literal_eof_asi_and_authored_semicolon_termination_preserves_ownership() {
    let text = "const x = 1e2   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = 1e2");
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));

    let text = "const x = 1e2;";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AuthoredSemicolon(_)
    ));
}

#[test]
fn plain_exponent_decimal_literal_aggregate_lifecycle_remains_incomplete_or_existing_rejection() {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = 1e2;",
        "const x = .5e2;",
        "const x = 1.0e2;",
        "const x = 12.34E-56;",
        "let x = 1e2, y = foo;",
    ] {
        assert!(
            matches!(
                attempt_selected_qualification(&source(text)),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }

    let SelectedQualificationAttempt::Outcome(outcome) =
        attempt_selected_qualification(&source("let x = 1e2, x = foo;"))
    else {
        panic!("expected static rejection for duplicate-name declaration");
    };
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("static rejection evidence");
    assert_eq!(evidence.family(), RejectionFamily::StaticSemantics);
    let subject = evidence
        .subject()
        .authored_anchor()
        .expect("static subject must remain authored");
    assert_eq!(subject.fragment(), "x");
    assert_eq!((subject.range().start(), subject.range().end()), (13, 14));

    let SelectedQualificationAttempt::Outcome(outcome) =
        attempt_selected_qualification(&source(r"const x = 1e2; let \u{};"))
    else {
        panic!("expected existing Grammar rejection to remain reachable");
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
    assert_eq!((subject.range().start(), subject.range().end()), (19, 23));
}

// --- Issue #740: both selected `var` placements widened to a direct- ------
// authored, separator-free exponent `DecimalLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #735/#736 candidate-independent atom theorem, the #738/#739 accepted
// selected `LexicalDeclaration` exponent production, and the joint
// top-level `var` + Block `var` composition theorem: the unmodified
// `consume_selected_plain_exponent_decimal_literal` recognizer is reused,
// tried before the unmodified `consume_selected_plain_fractional_decimal_literal`
// and `consume_selected_decimal_integer` predecessors, at both
// `parse_variable_statement` and `parse_selected_block_var_statement` call
// sites. The former #738 asymmetry seal (top-level/Block `var` exponent
// `UnsupportedCoverage`) is superseded by the positive composition below.

#[test]
fn top_level_var_and_block_var_plain_exponent_decimal_literal_initializers_compose() {
    for text in [
        "var x = 1e2;",
        "var x = 1E2;",
        "var x = 1e+2;",
        "var x = 1e-2;",
        "var x = 1.e2;",
        "var x = 1.0e2;",
        "var x = .5E+2;",
        "var x = 12.34E-56;",
    ] {
        let _ = recognized_variable(text);
    }

    for text in [
        "{ var x = 1e2; }",
        "{ var x = 1E2; }",
        "{ var x = 1e+2; }",
        "{ var x = 1e-2; }",
        "{ var x = 1.e2; }",
        "{ var x = 1.0e2; }",
        "{ var x = .5E+2; }",
        "{ var x = 12.34E-56; }",
    ] {
        let _ = recognized_block(text);
    }
}

#[test]
fn top_level_var_and_block_var_plain_exponent_decimal_literal_preserves_integer_and_fractional_predecessors()
 {
    // Existing selected integer and fractional initializers remain
    // unchanged in both `var` owners: the exponent recognizer must decline
    // without commit so the unmodified predecessors still own these atoms
    // (falsifies W2/W9/W10).
    let _ = recognized_variable("var x = 1;");
    let _ = recognized_variable("var x = 1.;");
    let _ = recognized_variable("var x = 1.0;");
    let _ = recognized_variable("var x = .5;");
    let _ = recognized_block("{ var x = 1; }");
    let _ = recognized_block("{ var x = 1.; }");
    let _ = recognized_block("{ var x = 1.0; }");
    let _ = recognized_block("{ var x = .5; }");

    // Complete-atom ownership: production must select the complete
    // authored exponent atom, not commit only the predecessor-owned
    // mantissa prefix (`1`, `1.0`, `.5` respectively). The whole-source
    // transactional architecture makes recognition success itself the
    // proof: a partial mantissa-only commit would leave the remaining
    // exponent-part source unowned and the whole statement/source
    // `UnsupportedCoverage` (falsifies W3).
    for text in ["var x = 1e2;", "var x = 1.0e2;", "var x = .5e2;"] {
        let _ = recognized_variable(text);
    }
    for text in [
        "{ var x = 1e2; }",
        "{ var x = 1.0e2; }",
        "{ var x = .5e2; }",
    ] {
        let _ = recognized_block(text);
    }
}

#[test]
fn top_level_var_and_block_var_plain_exponent_decimal_literal_incomplete_tails_do_not_claim_whole_source()
 {
    for text in [
        "var x = 1e;",
        "var x = 1E;",
        "var x = 1e+;",
        "var x = 1e-;",
        "var x = 1.e;",
        "var x = 1.E+;",
        "var x = 1.0e-;",
        "var x = .5E+;",
    ] {
        assert_unsupported(text);
    }

    for text in [
        "{ var x = 1e; }",
        "{ var x = 1E; }",
        "{ var x = 1e+; }",
        "{ var x = 1e-; }",
        "{ var x = 1.e; }",
        "{ var x = 1.E+; }",
        "{ var x = 1.0e-; }",
        "{ var x = .5E+; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_and_block_var_plain_exponent_decimal_literal_does_not_claim_numeric_or_richer_neighbors()
 {
    for text in [
        // numeric-separator / BigInt / non-decimal / legacy firewall
        "var x = 1e1_0;",
        "var x = 1_0e2;",
        "var x = 1.0_0e2;",
        "var x = .5e1_0;",
        "var x = 1n;",
        "var x = 0x10;",
        "var x = 0b10;",
        "var x = 0o10;",
        "var x = 01;",
        // "var x = +1e2;" / "var x = -1e2;" are no longer negative controls
        // here: Issue #744 makes both selected `var` initializer positions
        // additionally admit these as the new leading `+`/`-` decimal
        // `UnaryExpression`; see the "Issue #744" test section below for
        // full coverage.
        // richer-expression / comment firewall: a valid exponent literal
        // prefix never authorizes a broader expression
        "var x = 1e2.foo;",
        "var x = 1e2();",
        "var x = 1e2 + x;",
        "var x = 1e2 = x;",
        "var x = 1e2 ? x : y;",
        "var x = 1e2/*comment*/;",
        "var x = 1e2 unexpected;",
        "var x = 1.0e2.foo;",
        "var x = .5e2 + x;",
    ] {
        assert_unsupported(text);
    }

    for text in [
        "{ var x = 1e1_0; }",
        "{ var x = 1_0e2; }",
        "{ var x = 1.0_0e2; }",
        "{ var x = .5e1_0; }",
        "{ var x = 1n; }",
        "{ var x = 0x10; }",
        "{ var x = 0b10; }",
        "{ var x = 0o10; }",
        "{ var x = 01; }",
        // "{ var x = +1e2; }" / "{ var x = -1e2; }" are no longer negative
        // controls here: Issue #744 makes both selected `var` initializer
        // positions additionally admit these as the new leading `+`/`-`
        // decimal `UnaryExpression`; see the "Issue #744" test section
        // below for full coverage.
        "{ var x = 1e2.foo; }",
        "{ var x = 1e2(); }",
        "{ var x = 1e2 + x; }",
        "{ var x = 1e2 = x; }",
        "{ var x = 1e2 ? x : y; }",
        "{ var x = 1e2/*comment*/; }",
        "{ var x = 1e2 unexpected; }",
        "{ var x = 1.0e2.foo; }",
        "{ var x = .5e2 + x; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_exponent_mixed_with_sibling_atoms_and_identifier_reference_preserves_correspondence_ownership()
 {
    for text in [
        "var a=1e2,b;",
        "var a,b=.5e2;",
        "var a=1,b=2e3;",
        "var a=1.0,b=2.5e-3;",
        "var a=true,b=3E2;",
        "var a=null,b=.25e+2;",
        "var a=this,b=4e1;",
        "var a=\"s\",b=.25e+2;",
    ] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }

    let script = recognized_variable("var a=foo,b=2e3;");
    let statement = only_variable_statement(&script);
    let reference = statement.bindings()[0]
        .identifier_reference_initializer()
        .expect("first declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "foo");
    assert_eq!(reference.semantic_name(), "foo");
    assert!(
        statement.bindings()[1]
            .identifier_reference_initializer()
            .is_none()
    );
}

#[test]
fn one_level_block_var_exponent_mixed_with_sibling_atoms_and_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    fn only_block_var_statement(
        script: &super::selected_lexical_slice::SelectedOneLevelBlockScript,
    ) -> &super::selected_lexical_slice::SelectedBlockVarStatement {
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement");
        };
        statement
    }

    for text in [
        "{ var a=1e2,b; }",
        "{ var a,b=.5e2; }",
        "{ var a=1,b=2e3; }",
        "{ var a=1.0,b=2.5e-3; }",
        "{ var a=true,b=3E2; }",
        "{ var a=null,b=.25e+2; }",
        "{ var a=this,b=4e1; }",
        "{ var a=\"s\",b=.25e+2; }",
    ] {
        let script = recognized_block(text);
        let statement = only_block_var_statement(&script);
        for binding in statement.bindings() {
            assert!(
                binding.identifier_reference_initializer().is_none(),
                "{text:?}"
            );
            assert!(
                binding.escaped_reserved_initializer_identifier().is_none(),
                "{text:?}"
            );
        }
    }

    let script = recognized_block("{ var a=foo,b=2e3; }");
    let statement = only_block_var_statement(&script);
    let reference = statement.bindings()[0]
        .identifier_reference_initializer()
        .expect("first declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "foo");
    assert_eq!(reference.semantic_name(), "foo");
    assert!(
        statement.bindings()[1]
            .identifier_reference_initializer()
            .is_none()
    );
}

#[test]
fn top_level_var_plain_exponent_decimal_literal_transactionality_commits_no_earlier_prefix() {
    // No valid earlier exponent declarator escapes as committed selected
    // success when a later declarator/initializer/terminator fails
    // (falsifies W13).
    for text in [
        "var a = 1e2, b = ;",
        "var a = 1.0e-2, b = 1e;",
        "var a = .5e2, b =",
    ] {
        assert_unsupported(text);
    }

    // A later already-owned malformed `BindingIdentifier` Grammar-evidence
    // case remains authoritative when preceded by a valid exponent
    // initializer (falsifies W14).
    let subject = grammar_rejection(r"var a = 1e2, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn one_level_block_var_plain_exponent_decimal_literal_transactionality_commits_no_earlier_prefix() {
    for text in [
        "{ var a = 1e2, b = ; }",
        "{ var a = 1.0e-2, b = 1e; }",
        "{ var a = .5e2, b = }",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"{ var a = 1e2, \u{} = 1; }");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn top_level_var_plain_exponent_decimal_literal_eof_asi_preserves_ownership() {
    for text in ["var x=1e2", "var a=1e2,b=.5e2"] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(
            statement.terminator(),
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_var_plain_exponent_decimal_literal_close_brace_asi_preserves_ownership() {
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    for text in ["{ var x=1e2 }", "{ var a=1e2,b=.5e2 }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(
            statement.terminator(),
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
            "{text:?}"
        );
    }
}

// --- Issue #744: all three selected initializer owners (`LexicalDeclaration`,
// top-level `var`, one-level Block `var`) widened to a bounded,
// placement-neutral leading `+`/`-` decimal `UnaryExpression`, reusing the
// unmodified exponent/fractional/decimal-integer helpers and the unmodified
// `skip_selected_trivia`, per the candidate-independent theorem accepted by
// #742/#743 -------------------------------------------------------------

#[test]
fn leading_plus_minus_decimal_unary_expression_positive_matrix_is_recognized() {
    for text in [
        "let x = +1;",
        "let x = -1;",
        "let x = +1.;",
        "let x = -1.0;",
        "let x = +.5;",
        "let x = -.5;",
        "let x = +1e2;",
        "let x = -1e2;",
        "let x = +1e+2;",
        "let x = -1e-2;",
        "let x = +1.0e2;",
        "let x = -.5E+2;",
    ] {
        let _ = recognized(text);
    }
}

#[test]
fn leading_plus_minus_decimal_unary_expression_preserves_two_layer_sign_ownership() {
    // Outer unary `-` composed with the complete exponent atom `1e-2`,
    // whose own internal `-` remains owned entirely by the exponent helper
    // (Issue #742/#743 critical witness).
    let script = recognized("const x = -1e-2;");
    assert_eq!(
        script.declarations()[0].declaration().fragment(),
        "const x = -1e-2;"
    );

    // Plus analogue.
    let script = recognized("const x = +1e+2;");
    assert_eq!(
        script.declarations()[0].declaration().fragment(),
        "const x = +1e+2;"
    );

    // Unsigned predecessor `1e-2` on its own remains an atom, not a unary
    // occurrence; it was already accepted by #740 and remains unaffected.
    let _ = recognized("const x = 1e-2;");

    // W4 falsification: the exponent-internal sign is never mistaken for
    // (or merged with) the outer unary sign.
    assert_unsupported("const x = --2;");
}

#[test]
fn leading_plus_minus_decimal_unary_expression_trivia_matrix_is_recognized() {
    for text in [
        "let x = + 1;",
        "let x = -\t1.0;",
        "let x = +\n1e2;",
        "let x = -\u{2028}.5;",
        "let x = +\u{00A0}1;",
        "let x = -\u{3000}1.0;",
        "let x = +\u{FEFF}1e2;",
        "let x = +\u{2029}1e-2;",
    ] {
        let _ = recognized(text);
    }

    // A nearby non-selected space-like code point remains unsupported: this
    // is exactly the accepted trivia theorem, not "any Unicode
    // whitespace-like code point".
    assert_unsupported("let x = +\u{200B}1;");
}

#[test]
fn leading_plus_minus_decimal_unary_expression_binding_list_composes_in_authored_order() {
    for text in ["let a=-1,b;", "let a,b=+1e2;", "const a=-1,b=+.5;"] {
        let script = recognized(text);
        assert_eq!(script.declarations()[0].bindings().len(), 2, "{text:?}");
    }
}

#[test]
fn leading_plus_minus_decimal_unary_expression_does_not_claim_nested_unary_or_other_operators() {
    for text in [
        "let x = ++1;",
        "let x = --1;",
        "let x = +-1;",
        "let x = -+1;",
        "let x = + +1;",
        "let x = - -1;",
        "let x = + -1;",
        "let x = - +1;",
        "let x = !1;",
        "let x = ~1;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn leading_plus_minus_decimal_unary_expression_does_not_claim_non_numeric_or_future_numeric_operands()
 {
    for text in [
        // operand firewall: only the accepted separator-free decimal atom
        // family is admitted (a direct or escaped non-Reserved
        // IdentifierReference operand is outside the decimal-unary theorem
        // and is instead owned by the separate leading +/- IdentifierReference
        // UnaryExpression leaf; see
        // leading_plus_minus_identifier_reference_unary_expression_accepts_escaped_non_reserved_operand_across_all_three_owners
        // for its own escaped-operand positive matrix composed by #750)
        "let x = -(1);",
        "let x = +(1);",
        "let x = -\"x\";",
        "let x = +true;",
        "let x = -false;",
        "let x = +null;",
        "let x = -null;",
        "let x = +this;",
        "let x = -this;",
        // numeric-frontier firewall: future numeric neighbors remain
        // outside this leaf
        "let x = -1_0;",
        "let x = +1.0_0;",
        "let x = -1e1_0;",
        "let x = -0x10;",
        "let x = +0b10;",
        "let x = -0o10;",
        "let x = +1n;",
        "let x = -01;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn leading_plus_minus_decimal_unary_expression_does_not_claim_richer_expression_or_comment_neighbors()
 {
    for text in [
        // incomplete exponent tail: a locally recognized shorter atom never
        // authorizes unowned trailing source
        "let x = -1e;",
        "let x = -1e+;",
        "let x = -1e-;",
        // richer-expression / comment firewall
        "let x = -1 + x;",
        "let x = +1 * x;",
        "let x = -1 = x;",
        "let x = -1 ? x : y;",
        "let x = -1();",
        "let x = -1.foo;",
        "let x = -1 ** 2;",
        "let x = -1/*comment*/;",
        "let x = -1 unexpected;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn leading_plus_minus_decimal_unary_expression_transactionality_commits_no_earlier_prefix() {
    for text in ["let a=-1,b=;", "let a=+1.0,b=1e;", "let a=-1e-2,b="] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"let a = -1, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn leading_plus_minus_decimal_unary_expression_eof_asi_and_authored_semicolon_termination_preserves_ownership()
 {
    let text = "const x = -1   \t\n";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert_eq!(declaration.declaration().fragment(), "const x = -1");
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AutomaticAtEof
    ));

    let text = "const x = -1;";
    let script = recognized(text);
    let declaration = &script.declarations()[0];
    assert!(matches!(
        declaration.terminator(),
        SelectedDeclarationTerminator::AuthoredSemicolon(_)
    ));
}

#[test]
fn leading_plus_minus_decimal_unary_expression_aggregate_lifecycle_remains_incomplete_or_existing_rejection()
 {
    use super::qualification::{QualificationVerdictKind, RejectionFamily};
    use super::selected_qualification_integration::{
        SelectedQualificationAttempt, attempt_selected_qualification,
    };

    for text in [
        "const x = -1;",
        "const x = +1e2;",
        "let x = -1e-2;",
        "const x = + .5;",
    ] {
        assert!(
            matches!(
                attempt_selected_qualification(&source(text)),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }

    let SelectedQualificationAttempt::Outcome(outcome) =
        attempt_selected_qualification(&source("let x = -1, x = foo;"))
    else {
        panic!("expected static rejection for duplicate-name declaration");
    };
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("static rejection evidence");
    assert_eq!(evidence.family(), RejectionFamily::StaticSemantics);

    let SelectedQualificationAttempt::Outcome(outcome) =
        attempt_selected_qualification(&source(r"const x = -1; let \u{};"))
    else {
        panic!("expected existing Grammar rejection to remain reachable");
    };
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("Grammar rejection evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar);
}

#[test]
fn top_level_var_leading_plus_minus_decimal_unary_expression_positive_matrix_is_recognized() {
    for text in [
        "var x = -1;",
        "var x = +1e2;",
        "var x = -1e-2;",
        "var x = + .5;",
        "var x = -1",
    ] {
        let _ = recognized_variable(text);
    }
}

#[test]
fn top_level_var_leading_plus_minus_decimal_unary_expression_does_not_claim_numeric_or_richer_neighbors()
 {
    for text in [
        "var x = -this;",
        "var x = -1_0;",
        "var x = +1n;",
        "var x = ++1;",
        "var x = !1;",
        "var x = -1e;",
        "var x = -1 + x;",
        "var x = -1 ** 2;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn top_level_var_leading_plus_minus_decimal_unary_expression_transactionality_commits_no_earlier_prefix()
 {
    for text in ["var a=-1,b=;", "var a=+1.0,b=1e;", "var a=-1e-2,b="] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"var a = -1, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn top_level_var_leading_plus_minus_decimal_unary_expression_eof_asi_preserves_ownership() {
    for text in ["var x=-1", "var a=-1,b=+1e2"] {
        let script = recognized_variable(text);
        let statement = only_variable_statement(&script);
        assert_eq!(
            statement.terminator(),
            SelectedVariableStatementTerminator::AutomaticAtEof,
            "{text:?}"
        );
    }
}

#[test]
fn top_level_var_leading_plus_minus_decimal_unary_expression_mixed_with_sibling_atoms_and_identifier_reference_preserves_correspondence_ownership()
 {
    let text = "var a=-1,b=1,c=1.0,d=1e2,e=true,f=null,g=this,h=\"s\",i=x;";
    let script = recognized_variable(text);
    let statement = only_variable_statement(&script);
    for binding in &statement.bindings()[..8] {
        assert!(
            binding.identifier_reference_initializer().is_none(),
            "{text:?}"
        );
    }
    let reference = statement.bindings()[8]
        .identifier_reference_initializer()
        .expect("final declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

#[test]
fn one_level_block_var_leading_plus_minus_decimal_unary_expression_positive_matrix_is_recognized() {
    for text in [
        "{ var x = -1; }",
        "{ var x = +1e2; }",
        "{ var x = -1e-2; }",
        "{ var x = + .5; }",
        "{ var x = -1 }",
    ] {
        let _ = recognized_block(text);
    }
}

#[test]
fn one_level_block_var_leading_plus_minus_decimal_unary_expression_does_not_claim_numeric_or_richer_neighbors()
 {
    for text in [
        "{ var x = -this; }",
        "{ var x = -1_0; }",
        "{ var x = +1n; }",
        "{ var x = ++1; }",
        "{ var x = !1; }",
        "{ var x = -1e; }",
        "{ var x = -1 + x; }",
        "{ var x = -1 ** 2; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn one_level_block_var_leading_plus_minus_decimal_unary_expression_transactionality_commits_no_earlier_prefix()
 {
    for text in [
        "{ var a=-1,b=; }",
        "{ var a=+1.0,b=1e; }",
        "{ var a=-1e-2,b= }",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"{ var a = -1, \u{} = 1; }");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn one_level_block_var_leading_plus_minus_decimal_unary_expression_close_brace_asi_preserves_ownership()
 {
    use super::selected_lexical_slice::{
        SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
    };

    for text in ["{ var x=-1 }", "{ var a=-1,b=+1e2 }"] {
        let script = recognized_block(text);
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item for {text:?}");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement for {text:?}");
        };
        assert_eq!(
            statement.terminator(),
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose,
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_var_leading_plus_minus_decimal_unary_expression_mixed_with_sibling_atoms_and_identifier_reference_preserves_correspondence_ownership()
 {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    fn only_block_var_statement(
        script: &super::selected_lexical_slice::SelectedOneLevelBlockScript,
    ) -> &super::selected_lexical_slice::SelectedBlockVarStatement {
        let [SelectedTopLevelItem::Block(block)] = script.items() else {
            panic!("expected exactly one Block item");
        };
        let [SelectedBlockItem::Var(statement)] = block.items() else {
            panic!("expected exactly one Block var statement");
        };
        statement
    }

    let text = "{ var a=-1,b=1,c=1.0,d=1e2,e=true,f=null,g=this,h=\"s\",i=x; }";
    let script = recognized_block(text);
    let statement = only_block_var_statement(&script);
    for binding in &statement.bindings()[..8] {
        assert!(
            binding.identifier_reference_initializer().is_none(),
            "{text:?}"
        );
    }
    let reference = statement.bindings()[8]
        .identifier_reference_initializer()
        .expect("final declarator must retain existing IdentifierReference fact");
    assert_eq!(reference.reference().fragment(), "x");
    assert_eq!(reference.semantic_name(), "x");
}

/// Issue #748: the new bounded leading `+`/`-` direct `IdentifierReference`
/// `UnaryExpression` helper composes into all three mature initializer
/// owners and, on each owner, retains the exact existing
/// `SelectedIdentifierReferenceFact` produced by the unmodified shared
/// `consume_selected_identifier_reference()` recognizer: the retained
/// anchor and semantic name are the inner operand only (`a`, not `-a` or
/// `- a`), never the operator or intervening trivia.
#[test]
fn leading_plus_minus_direct_identifier_reference_unary_expression_retains_exact_inner_provenance_across_all_three_owners()
 {
    let script = recognized("const x = - a;");
    let declaration = &script.declarations()[0];
    let [binding] = declaration.bindings() else {
        panic!("expected one selected lexical binding");
    };
    assert_eq!(
        binding.initializer(),
        SelectedInitializerState::SelectedPresent
    );
    let reference = binding
        .identifier_reference_initializer()
        .expect("unary-wrapped LexicalDeclaration RHS reference fact");
    assert_eq!(reference.reference().fragment(), "a");
    assert_eq!(reference.semantic_name(), "a");

    let script = recognized_variable("var x = +foo;");
    let statement = only_variable_statement(&script);
    let [binding] = statement.bindings() else {
        panic!("expected one selected variable binding");
    };
    let reference = binding
        .identifier_reference_initializer()
        .expect("unary-wrapped top-level var RHS reference fact");
    assert_eq!(reference.reference().fragment(), "foo");
    assert_eq!(reference.semantic_name(), "foo");

    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};
    let script = recognized_block("{ var x = -\tfoo; }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    let [binding] = statement.bindings() else {
        panic!("expected one selected Block var binding");
    };
    let reference = binding
        .identifier_reference_initializer()
        .expect("unary-wrapped Block var RHS reference fact");
    assert_eq!(reference.reference().fragment(), "foo");
    assert_eq!(reference.semantic_name(), "foo");
}

/// Issue #748 critical regression: the accepted decimal-unary predecessor
/// (#742/#743/#744) keeps sole ownership of `-1e-2`-shaped sources across
/// all three owners. The new direct-IdentifierReference unary helper is
/// tried only after the decimal-unary predecessor declines, so it never
/// observes, and never retains a fact for, this source.
#[test]
fn leading_plus_minus_decimal_unary_expression_precedence_remains_reference_silent_across_all_three_owners()
 {
    let script = recognized("const x = -1e-2;");
    let declaration = &script.declarations()[0];
    let [binding] = declaration.bindings() else {
        panic!("expected one selected lexical binding");
    };
    assert!(binding.identifier_reference_initializer().is_none());

    let script = recognized_variable("var x = -1e-2;");
    let statement = only_variable_statement(&script);
    let [binding] = statement.bindings() else {
        panic!("expected one selected variable binding");
    };
    assert!(binding.identifier_reference_initializer().is_none());

    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};
    let script = recognized_block("{ var x = -1e-2; }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    let [binding] = statement.bindings() else {
        panic!("expected one selected Block var binding");
    };
    assert!(binding.identifier_reference_initializer().is_none());
}

/// Issue #750: the generalized bounded leading `+`/`-` `IdentifierReference`
/// `UnaryExpression` helper composes a selected escaped non-ReservedWord
/// operand into all three mature initializer owners exactly as it already
/// composes a direct operand, retaining the exact existing
/// `SelectedIdentifierReferenceFact` produced by the unmodified shared
/// `consume_selected_identifier_reference()` recognizer: the authored
/// escaped `SourceAnchor` and the decoded semantic name remain distinct,
/// and distinct authored spellings sharing one decoded semantic name
/// retain distinct anchors (no Unicode normalization).
#[test]
fn leading_plus_minus_identifier_reference_unary_expression_accepts_escaped_non_reserved_operand_across_all_three_owners()
 {
    let script = recognized(r"const x = -\u{66}oo;");
    let declaration = &script.declarations()[0];
    let [binding] = declaration.bindings() else {
        panic!("expected one selected lexical binding");
    };
    assert_eq!(
        binding.initializer(),
        SelectedInitializerState::SelectedPresent
    );
    let reference = binding
        .identifier_reference_initializer()
        .expect("unary-wrapped LexicalDeclaration RHS escaped reference fact");
    assert_eq!(reference.reference().fragment(), r"\u{66}oo");
    assert_eq!(reference.semantic_name(), "foo");

    let script = recognized_variable(r"var x = +f\u{6F}o;");
    let statement = only_variable_statement(&script);
    let [binding] = statement.bindings() else {
        panic!("expected one selected variable binding");
    };
    let reference = binding
        .identifier_reference_initializer()
        .expect("unary-wrapped top-level var RHS escaped reference fact");
    // Distinct authored spelling from the LexicalDeclaration case above,
    // same decoded semantic name: no Unicode normalization collapses them.
    assert_eq!(reference.reference().fragment(), r"f\u{6F}o");
    assert_eq!(reference.semantic_name(), "foo");

    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};
    let script = recognized_block(r"{ var x = -\u{1D49C}; }");
    let [SelectedTopLevelItem::Block(block)] = script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    let [binding] = statement.bindings() else {
        panic!("expected one selected Block var binding");
    };
    let reference = binding
        .identifier_reference_initializer()
        .expect("unary-wrapped Block var RHS escaped reference fact");
    assert_eq!(reference.reference().fragment(), r"\u{1D49C}");
    assert_eq!(reference.semantic_name(), "\u{1D49C}");
}

/// Issue #750 escaped-ReservedWord firewall: a decoded ReservedWord operand
/// (`\u{69}f` decodes to `if`) remains outside this wrapper across all
/// three owners and does not leak into the existing plain
/// escaped-ReservedWord C6 / EE-04-R08 initializer route, which owns only
/// its own unwrapped source position, not this wrapper's leading `+`/`-`
/// source position.
#[test]
fn leading_plus_minus_identifier_reference_unary_expression_escaped_reserved_word_operand_remains_unsupported_across_all_three_owners()
 {
    for text in [
        r"const x = -\u{69}f;",
        r"var x = -\u{69}f;",
        r"{ var x = -\u{69}f; }",
    ] {
        assert_unsupported(text);
    }
}

/// Issue #750 malformed/position-invalid escape firewall: representative
/// controls equivalent to the shared `consume_selected_identifier_reference`
/// recognizer's own malformed/non-CodePoint/surrogate/position-invalid
/// boundaries remain unowned through this wrapper, exactly as through the
/// unwrapped plain `IdentifierReference` position; in particular a mixed
/// spelling with an invalid continuation escape does not truncate to a
/// successful direct prefix.
#[test]
fn leading_plus_minus_identifier_reference_unary_expression_malformed_or_position_invalid_escape_remains_unowned()
 {
    for text in [
        r"const x = -\u{30};",
        r"const x = -a\u{2D}b;",
        r"const x = -\u{D800};",
        r"const x = -\u{110000};",
        r"const x = -\u{};",
    ] {
        assert_unsupported(text);
    }
}

/// Issue #750: the same accepted selected trivia (`skip_selected_trivia`,
/// unchanged) recognized between the operator and a direct operand is
/// recognized identically before an escaped non-Reserved operand; a nearby
/// non-selected space-like code point (U+200B) remains outside the accepted
/// theorem exactly as for the direct-operand and decimal-unary
/// predecessors.
#[test]
fn leading_plus_minus_identifier_reference_unary_expression_trivia_matrix_is_recognized_for_escaped_operand()
 {
    for text in [
        "let x = - \\u0066oo;",
        "let x = +\t\\u0066oo;",
        "let x = -\u{00A0}\\u0066oo;",
        "let x = +\u{FEFF}\\u0066oo;",
        "let x = -\u{2028}\\u0066oo;",
        "let x = +\u{2029}\\u0066oo;",
    ] {
        let script = recognized(text);
        let declaration = &script.declarations()[0];
        let [binding] = declaration.bindings() else {
            panic!("expected one selected lexical binding, {text:?}");
        };
        let reference = binding
            .identifier_reference_initializer()
            .expect("unary-wrapped escaped RHS reference fact");
        assert_eq!(reference.semantic_name(), "foo", "{text:?}");
    }

    assert_unsupported("let x = +\u{200B}\\u0066oo;");
}

/// Issue #750 transactionality: a locally complete escaped-unary
/// `IdentifierReference` fact must not escape as committed selected state
/// when a later declarator prevents the enclosing owner (`LexicalDeclaration`,
/// top-level `var`, or Block `var`) from completing, exactly as for the
/// existing plain-escaped, decimal-unary, and direct-unary predecessors.
/// A preceding valid escaped-unary RHS also must not suppress or downgrade
/// later already-owned malformed `BindingIdentifier` `Grammar` evidence.
#[test]
fn leading_plus_minus_identifier_reference_unary_expression_escaped_operand_transactionality_commits_no_earlier_fact()
 {
    for text in [
        r"let x=-\u{61},y=;",
        r"var x=-\u{61},y=;",
        r"{ var x=-\u{61},y= }",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"let x=-\u{61}, \u{}=1;");
    assert_eq!(subject.fragment(), r"\u{}");

    let subject = grammar_rejection(r"var x=-\u{61}, \u{}=1;");
    assert_eq!(subject.fragment(), r"\u{}");

    let subject = grammar_rejection(r"{ var x=-\u{61}, \u{}=1; }");
    assert_eq!(subject.fragment(), r"\u{}");
}

// Issue #754: bounded ordered two-`IdentifierReference` additive
// initializer. #752/PR #753 already independently prove the exact bounded
// theorem (exactly two accepted `IdentifierReference` operands, exactly one
// binary `+`/`-`, Direct/Escaped non-Reserved cross-product, exact authored
// provenance, left-to-right order, `a + a` non-deduplication, and every
// firewall below) as a candidate-independent Oracle; these production tests
// do not replace that independent evidence. They seal only that this
// production slice composes the same bounded theorem, jointly owned by all
// three initializer owners, without widening beyond it.

#[test]
fn two_identifier_reference_additive_initializer_retains_two_ordered_facts() {
    for (text, expected_first, expected_second) in [
        ("let x = a + b;", "a", "b"),
        ("let x = a - b;", "a", "b"),
        ("const x = foo + bar;", "foo", "bar"),
        ("const x = foo - bar;", "foo", "bar"),
        ("let x = b + a;", "b", "a"),
        ("let x = a + a;", "a", "a"),
    ] {
        let script = recognized(text);
        let [binding] = script.declarations()[0].bindings() else {
            panic!("expected one selected lexical binding for {text:?}");
        };
        let facts: Vec<_> = binding.identifier_reference_initializer_facts().collect();
        assert_eq!(facts.len(), 2, "{text:?}");
        assert_eq!(facts[0].semantic_name(), expected_first, "{text:?}");
        assert_eq!(facts[1].semantic_name(), expected_second, "{text:?}");
        // A plain single-reference initializer's compatibility accessor
        // keeps returning exactly the first (here, only) fact; for a
        // two-fact initializer it returns the authored left operand.
        assert_eq!(
            binding
                .identifier_reference_initializer()
                .unwrap()
                .semantic_name(),
            expected_first,
            "{text:?}"
        );
    }
}

#[test]
fn two_identifier_reference_additive_initializer_direct_escaped_cross_product_preserves_authored_provenance()
 {
    // Classic four-hex-digit `\uXXXX` escape fragments are built with
    // `concat!` over individually escaped pieces (matching PR #753's own
    // technique) so the literal backslash-u-hex bytes reliably survive
    // intact, rather than risking collapse into a decoded Unicode scalar by
    // an intermediate write/review layer.
    let esc_61 = concat!("\\", "u0061"); // decodes to "a"
    let esc_62 = concat!("\\", "u0062"); // decodes to "b"
    let esc_61_upper_within = concat!("f", "\\", "u006F", "o"); // mixed-part left operand, decodes to "foo"
    let esc_61r = concat!("b", "\\", "u0061", "r"); // mixed-part right operand, decodes to "bar"

    for (
        text,
        expected_first_fragment,
        expected_first,
        expected_second_fragment,
        expected_second,
    ) in [
        (
            "let x = a + b;".to_owned(),
            "a".to_owned(),
            "a",
            "b".to_owned(),
            "b",
        ),
        (
            format!("let x = {esc_61} + b;"),
            esc_61.to_owned(),
            "a",
            "b".to_owned(),
            "b",
        ),
        (
            format!("let x = a + {esc_62};"),
            "a".to_owned(),
            "a",
            esc_62.to_owned(),
            "b",
        ),
        (
            format!("let x = {esc_61} - {esc_62};"),
            esc_61.to_owned(),
            "a",
            esc_62.to_owned(),
            "b",
        ),
        (
            "let x = \\u{61} + \\u{62};".to_owned(),
            "\\u{61}".to_owned(),
            "a",
            "\\u{62}".to_owned(),
            "b",
        ),
        (
            format!("let x = {esc_61_upper_within} + bar;"),
            esc_61_upper_within.to_owned(),
            "foo",
            "bar".to_owned(),
            "bar",
        ),
        (
            format!("let x = foo - {esc_61r};"),
            "foo".to_owned(),
            "foo",
            esc_61r.to_owned(),
            "bar",
        ),
    ] {
        let script = recognized(&text);
        let [binding] = script.declarations()[0].bindings() else {
            panic!("expected one selected lexical binding for {text:?}");
        };
        let facts: Vec<_> = binding.identifier_reference_initializer_facts().collect();
        assert_eq!(facts.len(), 2, "{text:?}");
        assert_eq!(
            facts[0].reference().fragment(),
            expected_first_fragment,
            "{text:?}"
        );
        assert_eq!(facts[0].semantic_name(), expected_first, "{text:?}");
        assert_eq!(
            facts[1].reference().fragment(),
            expected_second_fragment,
            "{text:?}"
        );
        assert_eq!(facts[1].semantic_name(), expected_second, "{text:?}");
    }
}

#[test]
fn two_identifier_reference_additive_initializer_firewalls_remain_unsupported() {
    for text in [
        // Cardinality: 3+ operands are never truncated to the first two.
        "let x = a + b + c;",
        "let x = a - b - c;",
        "let x = a + b - c;",
        // Unary operands on either side.
        "let x = +a + b;",
        "let x = -a + b;",
        "let x = a + +b;",
        "let x = a + -b;",
        // Non-IdentifierReference operands.
        "let x = 1 + b;",
        "let x = a + 1;",
        "let x = true + b;",
        "let x = a + null;",
        "let x = this + b;",
        "let x = \"a\" + b;",
        // UpdateExpression / assignment operator boundaries.
        "let x = a++b;",
        "let x = a--b;",
        "let x = a += b;",
        "let x = a -= b;",
        // Precedence / richer expressions.
        "let x = a + b * c;",
        "let x = a * b + c;",
        // Grouping / member / call neighbors.
        "let x = (a) + b;",
        "let x = a + (b);",
        "let x = a.b + c;",
        "let x = a + b.c;",
        "let x = a() + b;",
        "let x = a + b();",
        // Malformed / position-invalid second operand must not leak the
        // first operand as a committed fact.
        r"let x = a + \u{};",
    ] {
        assert_unsupported(text);
    }

    // Escaped ReservedWord operand: never an accepted operand, so it must
    // not become selected two-fact evidence. Built with `concat!` over an
    // individually escaped fragment (matching PR #753's own technique) so
    // the literal backslash-u-hex bytes survive intact.
    let escaped_if = concat!("\\", "u0069", "f"); // decodes to the reserved word "if"
    assert_unsupported(&format!("let x = {escaped_if} + a;"));
    assert_unsupported(&format!("let x = a + {escaped_if};"));

    // Malformed second operand mid-spelling: `-` decodes to `-`, which
    // is not a valid identifier-part, so this must not leak `a` (or `b`) as
    // a committed fact.
    let escaped_dash = concat!("\\", "u002D");
    assert_unsupported(&format!("let x = a + b{escaped_dash}c;"));
}

#[test]
fn two_identifier_reference_additive_initializer_transactionality_commits_no_earlier_fact() {
    for text in [
        "let x = a + b, y =;",
        "var x = a + b, y =;",
        "{ var x = a + b, y = }",
    ] {
        assert_unsupported(text);
    }

    let subject = grammar_rejection(r"let x = a + b, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");

    let subject = grammar_rejection(r"var x = a + b, \u{} = 1;");
    assert_eq!(subject.fragment(), r"\u{}");

    let subject = grammar_rejection(r"{ var x = a + b, \u{} = 1; }");
    assert_eq!(subject.fragment(), r"\u{}");
}

#[test]
fn two_identifier_reference_additive_initializer_jointly_composes_across_all_three_owners() {
    use super::selected_lexical_slice::{SelectedBlockItem, SelectedTopLevelItem};

    let script = recognized_variable("var x = a + b;");
    let statement = only_variable_statement(&script);
    let [binding] = statement.bindings() else {
        panic!("expected one selected top-level var binding");
    };
    let facts: Vec<_> = binding.identifier_reference_initializer_facts().collect();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].semantic_name(), "a");
    assert_eq!(facts[1].semantic_name(), "b");

    let block_script = recognized_block("{ var x = a - b; }");
    let [SelectedTopLevelItem::Block(block)] = block_script.items() else {
        panic!("expected exactly one Block item");
    };
    let [SelectedBlockItem::Var(statement)] = block.items() else {
        panic!("expected exactly one Block var statement");
    };
    let [binding] = statement.bindings() else {
        panic!("expected one selected Block var binding");
    };
    let facts: Vec<_> = binding.identifier_reference_initializer_facts().collect();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].semantic_name(), "a");
    assert_eq!(facts[1].semantic_name(), "b");

    // Joint placement-neutral firewall spot-check: the 3+ operand firewall
    // holds for every owner, not only `LexicalDeclaration`.
    assert_unsupported("var x = a + b + c;");
    assert_unsupported("{ var x = a + b + c; }");
}

// --- Issue #758: top-level free-standing `IdentifierReference`
// `ExpressionStatement` use-site leaf. ---

fn recognized_reference_use(text: &str) -> SelectedIdentifierReferenceExpressionStatementScript {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::RecognizedIdentifierReferenceExpressionStatementSlice(
            script,
        ) => script,
        other => panic!("expected reference-use-enabled recognition for {text:?}, got {other:?}"),
    }
}

fn only_use_site_fact(
    script: &SelectedIdentifierReferenceExpressionStatementScript,
) -> &super::selected_lexical_slice::SelectedIdentifierReferenceFact {
    let [SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(fact)] =
        script.items()
    else {
        panic!("expected exactly one selected use-site item");
    };
    fact
}

#[test]
fn direct_and_escaped_use_site_forms_are_selected() {
    for (text, expected_fragment, expected_name) in [
        ("a;", "a", "a"),
        (r"\u0061;", r"\u0061", "a"),
        (r"\u{61};", r"\u{61}", "a"),
        (r"f\u006Fo;", r"f\u006Fo", "foo"),
        // Escaped contextual `let`: composes escaped spelling, decoded
        // contextual semantic name, non-strict IdentifierReference
        // policy, and top-level use-site placement.
        (r"\u006Cet;", r"\u006Cet", "let"),
    ] {
        let script = recognized_reference_use(text);
        let fact = only_use_site_fact(&script);
        assert_eq!(fact.reference().fragment(), expected_fragment, "{text}");
        assert_eq!(fact.semantic_name(), expected_name, "{text}");
    }
}

#[test]
fn dispatch_selects_use_site_before_raw_top_level_dispatch() {
    // `let;` and `varfoo;` must become the new use-site, never a failed
    // `LexicalDeclaration` / `VariableStatement` recognition (acceptance
    // criteria 6-9). Inspecting the recognized carrier/item variant proves
    // this directly rather than inferring it from qualification alone.
    let script = recognized_reference_use("let;");
    assert_eq!(only_use_site_fact(&script).semantic_name(), "let");

    let script = recognized_reference_use("varfoo;");
    assert_eq!(only_use_site_fact(&script).semantic_name(), "varfoo");

    match recognize_selected_lexical_slice(&source("let a;")) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(_) => {}
        other => panic!("expected `let a;` to remain a LexicalDeclaration, got {other:?}"),
    }

    match recognize_selected_lexical_slice(&source("var a;")) {
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(_) => {}
        other => panic!("expected `var a;` to remain a VariableStatement, got {other:?}"),
    }

    // Escaped continuations that decode to an accepted non-ReservedWord name
    // (`let0`, `const0`, `leta`) are likewise selected use-sites, not failed
    // `LexicalDeclaration`s -- moved here from
    // `formed_unicode_escape_extends_literal_keyword_candidate_without_backtracking`
    // now that this leaf exists.
    // `var\u0061;` composes the direct textual prefix "var" with a
    // formed escaped IdentifierPart continuation into one maximal
    // IdentifierReference (`vara`), proving the use-site probe -- not
    // the raw `starts_with("var")` dispatch -- owns this source.
    let script = recognized_reference_use(r"var\u0061;");
    assert_eq!(only_use_site_fact(&script).semantic_name(), "vara");

    for (text, expected_name) in [
        (r"let\u0030;", "let0"),
        (r"const\u0030;", "const0"),
        (r"let\u{00000061};", "leta"),
    ] {
        let script = recognized_reference_use(text);
        assert_eq!(
            only_use_site_fact(&script).semantic_name(),
            expected_name,
            "{text}"
        );
    }
}

#[test]
fn top_level_block_composes_with_free_standing_use_site() {
    let script = recognized_reference_use("{ var a; }\na;");
    let [
        SelectedReferenceUseEnabledTopLevelItem::Block(_),
        SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(fact),
    ] = script.items()
    else {
        panic!("expected [Block, use-site] items");
    };
    assert_eq!(fact.semantic_name(), "a");

    let script = recognized_reference_use("{ let a; }\na;");
    let [
        SelectedReferenceUseEnabledTopLevelItem::Block(_),
        SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(fact),
    ] = script.items()
    else {
        panic!("expected [Block, use-site] items");
    };
    assert_eq!(fact.semantic_name(), "a");
}

#[test]
fn deeper_nested_block_use_site_remains_unsupported() {
    // One-level `{ a; }` becomes a selected Block-contained use-site as of
    // Issue #762 (see the "Issue #762" section below); only recursion
    // *beyond* one level remains outside selected coverage (W26 / issue
    // section "Empty Block / recursive Block boundaries").
    assert_unsupported("{ { a; } }");
    assert_unsupported("{ { let a; } }");
}

#[test]
fn general_expression_neighbors_remain_unsupported_without_valid_prefix_leakage() {
    // The bounded probe must not accept a valid `IdentifierReference`
    // prefix from a richer expression neighbor (acceptance criterion 31 /
    // wrong model W15): each of these must remain `UnsupportedCoverage`
    // through the existing whole-source transaction, never a committed
    // use-site for the leading `a`.
    for text in [
        "a+b;",
        "+a;",
        "-a;",
        "(a);",
        "a.b;",
        "a[b];",
        "a();",
        "a=b;",
        "a ? b : c;",
        "a && b;",
        "a, b;",
        "new a;",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn asi_forms_remain_unsupported_coverage_not_selected() {
    assert_unsupported("a");
    assert_unsupported("a\nlet b;");
}

#[test]
fn escaped_reserved_and_malformed_forms_are_not_selected() {
    for text in [r"\u0069f;", r"\u{};", r"\u0030;", r"a\u002Db;"] {
        assert_unsupported(text);
    }
}

#[test]
fn duplicate_use_sites_are_preserved_distinctly_in_authored_order() {
    let script = recognized_reference_use("let a;\na;\na;");
    let [
        SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(_),
        SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(first),
        SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(second),
    ] = script.items()
    else {
        panic!("expected [LexicalDeclaration, use-site, use-site] items");
    };
    assert_eq!(first.semantic_name(), "a");
    assert_eq!(second.semantic_name(), "a");
    assert_ne!(
        (
            first.reference().range().start(),
            first.reference().range().end()
        ),
        (
            second.reference().range().start(),
            second.reference().range().end()
        ),
        "duplicate use-sites must remain two distinct authored occurrences"
    );
    assert!(first.reference().range().start() < second.reference().range().start());
}

#[test]
fn authored_use_site_order_is_preserved_not_target_declaration_order() {
    let script = recognized_reference_use("let b;\nlet a;\na;\nb;");
    let use_site_names: Vec<&str> = script
        .items()
        .iter()
        .filter_map(|item| match item {
            SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                fact,
            ) => Some(fact.semantic_name()),
            _ => None,
        })
        .collect();
    assert_eq!(use_site_names, ["a", "b"]);
}

#[test]
fn whole_source_transactionality_use_site() {
    // A locally valid earlier use-site must not leak selected success if
    // later source fails.
    assert_unsupported("let a;\na;\n???");
    assert_unsupported("let a;\na;\nlet x = ;");
}

// --- Issue #762: one-level Block-contained free-standing
// `IdentifierReference` `ExpressionStatement` use-sites, composed with the
// new fifth / broadest Script carrier. ---

fn recognized_block_reference_use(text: &str) -> SelectedBlockReferenceUseEnabledScript {
    match recognize_selected_lexical_slice(&source(text)) {
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(script) => script,
        other => {
            panic!("expected Block-reference-use-enabled recognition for {text:?}, got {other:?}")
        }
    }
}

fn only_block_use_site_fact(
    script: &SelectedBlockReferenceUseEnabledScript,
) -> &super::selected_lexical_slice::SelectedIdentifierReferenceFact {
    let [SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block)] = script.items()
    else {
        panic!("expected exactly one use-site-enabled Block item");
    };
    let [SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(fact)] =
        block.items()
    else {
        panic!("expected exactly one Block-contained use-site item");
    };
    fact
}

#[test]
fn block_direct_and_escaped_use_site_forms_are_selected() {
    for (text, expected_fragment, expected_name) in [
        ("{ a; }", "a", "a"),
        (r"{ \u0061; }", r"\u0061", "a"),
        (r"{ \u{61}; }", r"\u{61}", "a"),
        (r"{ f\u006Fo; }", r"f\u006Fo", "foo"),
    ] {
        let script = recognized_block_reference_use(text);
        let fact = only_block_use_site_fact(&script);
        assert_eq!(fact.reference().fragment(), expected_fragment, "{text}");
        assert_eq!(fact.semantic_name(), expected_name, "{text}");
    }
}

#[test]
fn block_dispatch_selects_use_site_before_raw_block_dispatch() {
    // `{ let; }` and `{ varfoo; }` must become the new Block use-site,
    // never a failed lexical-declaration/Block-var recognition; `{ let a; }`
    // / `{ var a; }` must remain owned by the existing declaration
    // dispatch, producing the exact historical `SelectedBlock`
    // representation, not the new use-site-enabled one.
    let script = recognized_block_reference_use("{ let; }");
    assert_eq!(only_block_use_site_fact(&script).semantic_name(), "let");

    let script = recognized_block_reference_use("{ varfoo; }");
    assert_eq!(only_block_use_site_fact(&script).semantic_name(), "varfoo");

    match recognize_selected_lexical_slice(&source("{ let a; }")) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_) => {}
        other => panic!("expected `{{ let a; }}` to remain a historical Block, got {other:?}"),
    }

    match recognize_selected_lexical_slice(&source("{ var a; }")) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_) => {}
        other => panic!("expected `{{ var a; }}` to remain a historical Block, got {other:?}"),
    }
}

#[test]
fn block_use_site_composes_with_lexical_and_var_items_in_authored_order() {
    let script = recognized_block_reference_use("{ let a; a; }");
    let [SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block)] = script.items()
    else {
        panic!("expected exactly one use-site-enabled Block item");
    };
    let [
        SelectedUseSiteEnabledBlockItem::LexicalDeclaration(_),
        SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(fact),
    ] = block.items()
    else {
        panic!("expected [LexicalDeclaration, use-site] items");
    };
    assert_eq!(fact.semantic_name(), "a");

    let script = recognized_block_reference_use("{ var a; a; }");
    let [SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block)] = script.items()
    else {
        panic!("expected exactly one use-site-enabled Block item");
    };
    let [
        SelectedUseSiteEnabledBlockItem::Var(_),
        SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(fact),
    ] = block.items()
    else {
        panic!("expected [Var, use-site] items");
    };
    assert_eq!(fact.semantic_name(), "a");
}

#[test]
fn top_level_declaration_composes_with_block_use_site_in_authored_order() {
    let script = recognized_block_reference_use("let a;\n{ a; }");
    let [
        SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(_),
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block),
    ] = script.items()
    else {
        panic!("expected [LexicalDeclaration, UseSiteEnabledBlock] items");
    };
    assert_eq!(only_use_site_fact_of(block).semantic_name(), "a");

    let script = recognized_block_reference_use("let a;\n{ a; let a; }");
    let [
        SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(_),
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block),
    ] = script.items()
    else {
        panic!("expected [LexicalDeclaration, UseSiteEnabledBlock] items");
    };
    assert_eq!(only_use_site_fact_of(block).semantic_name(), "a");
}

fn only_use_site_fact_of(
    block: &super::selected_lexical_slice::SelectedUseSiteEnabledBlock,
) -> &super::selected_lexical_slice::SelectedIdentifierReferenceFact {
    block
        .items()
        .iter()
        .find_map(|item| match item {
            SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(fact) => {
                Some(fact)
            }
            _ => None,
        })
        .expect("expected exactly one Block-contained use-site item")
}

#[test]
fn historical_block_composes_with_later_block_use_site_without_reconstruction() {
    // `{ let a; }` first produces the exact historical `SelectedBlock`;
    // `{ a; }` then promotes the builder into the fifth carrier, moving the
    // already-owned historical Block across unchanged (never reparsed).
    let script = recognized_block_reference_use("{ let a; }\n{ a; }");
    let [
        SelectedBlockReferenceUseEnabledTopLevelItem::Block(_),
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block),
    ] = script.items()
    else {
        panic!("expected [Block, UseSiteEnabledBlock] items");
    };
    assert_eq!(only_use_site_fact_of(block).semantic_name(), "a");
}

#[test]
fn variable_statement_composes_with_later_block_use_site_without_reconstruction() {
    // Exercises promotion into the fifth carrier from the
    // `VariableEnabled` builder state specifically (Promotion invariants:
    // Flat / BlockEnabled / VariableEnabled / ReferenceUseEnabled must all
    // promote monotonically into `BlockReferenceUseEnabled`).
    let script = recognized_block_reference_use("var x;\n{ a; }");
    let [
        SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(_),
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block),
    ] = script.items()
    else {
        panic!("expected [VariableStatement, UseSiteEnabledBlock] items");
    };
    assert_eq!(only_use_site_fact_of(block).semantic_name(), "a");
}

#[test]
fn top_level_use_site_composes_with_block_use_site_as_separate_surfaces() {
    let script = recognized_block_reference_use("a;\n{ a; }\na;");
    let [
        SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(first),
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block),
        SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(third),
    ] = script.items()
    else {
        panic!("expected [use-site, UseSiteEnabledBlock, use-site] items");
    };
    assert_eq!(first.semantic_name(), "a");
    assert_eq!(only_use_site_fact_of(block).semantic_name(), "a");
    assert_eq!(third.semantic_name(), "a");
}

#[test]
fn empty_block_remains_unsupported_coverage() {
    assert_unsupported("{}");
}

#[test]
fn block_use_site_close_brace_asi_remains_unsupported_coverage() {
    // Only `AuthoredSemicolon` is selected for the new Block use-site (issue
    // section "ASI boundary" / W24); `{ a }` remains valid-but-unselected,
    // never a definitive grammar rejection.
    assert_unsupported("{ a }");
}

#[test]
fn block_use_site_general_expression_neighbors_remain_unsupported_without_valid_prefix_leakage() {
    for text in [
        "{ a+b; }",
        "{ +a; }",
        "{ -a; }",
        "{ (a); }",
        "{ a.b; }",
        "{ a[b]; }",
        "{ a(); }",
        "{ a=b; }",
        "{ a ? b : c; }",
        "{ a && b; }",
        "{ a, b; }",
        "{ new a; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn block_use_site_escaped_reserved_and_malformed_forms_are_not_selected() {
    for text in [
        r"{ \u0069f; }",
        r"{ \u{}; }",
        r"{ \u0030; }",
        r"{ a\u002Db; }",
    ] {
        assert_unsupported(text);
    }
}

#[test]
fn block_use_site_existing_block_var_close_brace_asi_remains_unaffected() {
    // The Block parser refactor must not regress existing Block-var
    // `AutomaticBeforeBlockClose` support: this must remain a historical
    // (not use-site-enabled) Block.
    match recognize_selected_lexical_slice(&source("{ var a }")) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(script) => {
            use super::selected_lexical_slice::{
                SelectedBlockItem, SelectedBlockVarStatementTerminator, SelectedTopLevelItem,
            };
            let [SelectedTopLevelItem::Block(block)] = script.items() else {
                panic!("expected exactly one Block item");
            };
            let [SelectedBlockItem::Var(statement)] = block.items() else {
                panic!("expected exactly one Var item");
            };
            assert_eq!(
                statement.terminator(),
                SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose
            );
        }
        other => panic!("expected historical Block recognition, got {other:?}"),
    }
}

#[test]
fn duplicate_block_use_sites_are_preserved_distinctly_in_authored_order() {
    let script = recognized_block_reference_use("{ a; a; }");
    let [SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block)] = script.items()
    else {
        panic!("expected exactly one use-site-enabled Block item");
    };
    let [
        SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(first),
        SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(second),
    ] = block.items()
    else {
        panic!("expected two use-site items");
    };
    assert_eq!(first.semantic_name(), "a");
    assert_eq!(second.semantic_name(), "a");
    assert!(first.reference().range().start() < second.reference().range().start());
}

#[test]
fn distinct_blocks_own_distinct_use_sites() {
    let script = recognized_block_reference_use("{ a; }\n{ a; }");
    let [
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(first_block),
        SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(second_block),
    ] = script.items()
    else {
        panic!("expected two use-site-enabled Block items");
    };
    assert_ne!(
        (
            first_block.block().range().start(),
            first_block.block().range().end()
        ),
        (
            second_block.block().range().start(),
            second_block.block().range().end()
        ),
        "distinct Blocks must retain distinct authored Block anchors"
    );
}

#[test]
fn block_use_site_whole_source_transactionality() {
    // A locally valid earlier Block use-site must not leak selected success
    // if later source fails, whether the failure is inside the same Block,
    // a later declaration, or a later top-level item.
    assert_unsupported("{ a; ??? }");
    assert_unsupported("{ a; let x = ; }");
    assert_unsupported("{ a; }\n???");
}
