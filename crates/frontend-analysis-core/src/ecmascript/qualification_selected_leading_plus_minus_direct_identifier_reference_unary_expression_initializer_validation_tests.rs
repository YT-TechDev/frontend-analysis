//! Candidate-independent leading `+`/`-` direct `IdentifierReference`
//! `UnaryExpression` fact-preservation validation for Issue #746 (durable
//! research: Issue #688 comment `5714316722`; accepted predecessor
//! authority: Issue #742 / PR #743 leading `+`/`-` decimal `UnaryExpression`
//! Oracle, Issue #744 / PR #745 leading `+`/`-` decimal `UnaryExpression`
//! production, Issue #237 direct escape-free `IdentifierReference`
//! source/name boundary).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedLeadingPlusMinusDirectIdentifierReferenceUnaryExpression ::=
//!     SelectedUnaryPlusMinus
//!     SelectedUnaryOperandTrivia
//!     SelectedDirectIdentifierReference
//!
//! SelectedUnaryPlusMinus ::= "+" | "-"
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The load-bearing capability question is not merely whether source such as
//! `-foo` can be recognized. It is whether one selected expression wrapper
//! can own syntax around an already source-backed `IdentifierReference` fact
//! without destroying, widening, reconstructing, or normalizing that fact --
//! so that existing selected Binding/Scope meaning still composes from the
//! inner fact unchanged.
//!
//! `SelectedUnaryOperandTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by
//! Issue #742/#743 (the same code-point set `is_selected_trivia` in
//! `selected_lexical_slice.rs` recognizes, restated here rather than
//! imported): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`, `LINE SEPARATOR`,
//! `PARAGRAPH SEPARATOR`, and the frozen Unicode 17 `Space_Separator`
//! property (exposed through the stable, normative `is_space_separator`
//! primitive already used by predecessor accepted oracles). Comments remain
//! outside this contract and are never accepted as trivia.
//!
//! `SelectedDirectIdentifierReference` independently restates only the
//! already-accepted Issue #237 direct escape-free `IdentifierName` code-point
//! shape (`is_id_start`/`is_id_continue` plus `$`/`_`) and the unconditionally
//! reserved word exclusion, never a `BindingIdentifier` policy, an escaped
//! form, or a broader `PrimaryExpression`.
//!
//! This is a validation-only leaf: production supports leading `+`/`-` only
//! over the accepted decimal atom family at the #746 baseline, so every
//! positive fixture below remains `UnsupportedCoverage` under current
//! production and exists only as independent Oracle evidence for a future,
//! separately authorized production decision. No completion successor file
//! accompanies this leaf: this Issue adds zero production capability, so the
//! frozen `193 / 10 / 183 / {}` completion partition cannot move, matching
//! the precedent set by #742/#743 (which likewise added zero production
//! capability and shipped without one).

use std::cmp::Ordering;

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 746;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_UNARY_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_leading_plus_minus_decimal_unary_expression_initializer_validation_tests.rs"
);
const PREVIOUS_IDENTIFIER_REFERENCE_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_identifier_reference_initializer_validation_tests.rs");
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_leading_plus_minus_direct_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "leading plus/minus direct IdentifierReference UnaryExpression frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "escaped IdentifierReference operands, nested/update unary, other unary ",
    "operators, non-IdentifierReference operands, parenthesized operands, ",
    "richer expression tails, comments, or top-level/Block var placement"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

const UNCONDITIONALLY_RESERVED_WORDS: &[&str] = &[
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
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedLexicalBindingOrder {
    Before,
    Same,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedBindingScopeTarget {
    SameSourceSelectedLexicalBinding(ExpectedLexicalBindingOrder),
    NoSameSourceSelectedLexicalBinding,
}

fn slice(text: &str, Range(start, end): Range) -> &str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("invalid UTF-8 range ({start},{end}) in {text:?}"))
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

/// Independently restates only the already-accepted Issue #237 direct
/// `IdentifierStart` code-point shape.
fn is_direct_identifier_start(code_point: char) -> bool {
    matches!(code_point, '$' | '_') || is_id_start(code_point as u32)
}

/// Independently restates only the already-accepted Issue #237 direct
/// `IdentifierPart` code-point shape.
fn is_direct_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

/// Independently restates only the already-accepted Issue #237 direct
/// escape-free `IdentifierName` code-point shape: a pure code-point-family
/// check, never a reserved-word policy on its own (matching the #237
/// convention: `escape_free_identifier_name_shape_is_exact_and_not_a_reserved_word_policy`).
fn is_escape_free_identifier_name(spelling: &str) -> bool {
    let mut code_points = spelling.chars();
    let Some(first) = code_points.next() else {
        return false;
    };
    is_direct_identifier_start(first) && code_points.all(is_direct_identifier_part)
}

/// `SelectedDirectIdentifierReference`: the already-accepted direct
/// escape-free `IdentifierName` shape, minus the unconditionally reserved
/// words that Script/non-strict/`Yield=false`/`Await=false` never admits as
/// an `IdentifierReference`. This is the operand policy this Issue consumes
/// -- never a `BindingIdentifier` policy, which is a separate, unowned
/// concern this leaf never leaks into.
fn is_selected_direct_identifier_reference(candidate: &str) -> bool {
    is_escape_free_identifier_name(candidate)
        && !UNCONDITIONALLY_RESERVED_WORDS.contains(&candidate)
}

/// Independently restates the complete already-accepted
/// `SelectedUnaryOperandTrivia` theorem established by Issue #742/#743: the
/// frozen `is_selected_trivia` code-point set (`TAB`, `VT`, `FF`, `BOM`,
/// `LF`, `CR`, `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`) plus the frozen
/// Unicode 17 `Space_Separator` property. This is not a call into the
/// production selected-trivia recognizer: `is_space_separator` is a stable,
/// normative Unicode property primitive this oracle is independently
/// entitled to consult, not `is_selected_trivia` or `skip_selected_trivia`
/// themselves.
fn is_selected_unary_operand_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

/// Central Issue #746 theorem: exactly one leading authored `+`/`-`,
/// followed by zero or more already-accepted trivia code points, followed by
/// exactly one complete direct escape-free `IdentifierReference` occupying
/// the rest of the candidate. Whole-string: no richer tail, no nested
/// operator, no escaped or non-identifier operand survives.
///
/// The trivia scan advances a UTF-8 byte offset by each accepted code
/// point's own `len_utf8()` as it is examined -- a single forward pass that
/// is this oracle's owned recognition, never a later search, rescan, or
/// reparse over already-classified source.
fn is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(
    candidate: &str,
) -> bool {
    let mut chars = candidate.chars();
    match chars.next() {
        Some('+') | Some('-') => {}
        _ => return false,
    }
    let after_operator = &candidate[1..];
    let mut operand_start = 0usize;
    for code_point in after_operator.chars() {
        if !is_selected_unary_operand_trivia(code_point) {
            break;
        }
        operand_start += code_point.len_utf8();
    }
    let operand = &after_operator[operand_start..];
    is_selected_direct_identifier_reference(operand)
}

/// Independently restates only the already-accepted premise that a
/// selected `IdentifierReference` composes with existing lexical Binding/
/// Scope meaning by comparing declaration-order positions -- the same
/// conceptual relation the production Binding/Scope analysis in
/// `selected_binding_scope.rs` derives, restated here from a fixture-owned
/// authored binding inventory rather than by calling production. `Same`
/// arises exactly when the reference's own containing binding is also its
/// same-name target (a self-referencing initializer), matching that
/// module's equal-position branch.
fn expected_binding_scope_target(
    binding_inventory: &[&str],
    containing_binding_name: &str,
    reference_semantic_name: &str,
) -> ExpectedBindingScopeTarget {
    let containing_position = binding_inventory
        .iter()
        .position(|name| *name == containing_binding_name)
        .expect("fixture-owned containing binding must be present in the inventory");

    match binding_inventory
        .iter()
        .position(|name| *name == reference_semantic_name)
    {
        None => ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        Some(target_position) => {
            let order = match target_position.cmp(&containing_position) {
                Ordering::Less => ExpectedLexicalBindingOrder::Before,
                Ordering::Equal => ExpectedLexicalBindingOrder::Same,
                Ordering::Greater => ExpectedLexicalBindingOrder::After,
            };
            ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(order)
        }
    }
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 746);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_UNARY_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 742"));
    assert!(
        PREVIOUS_UNARY_ORACLE_SOURCE
            .contains("leading plus/minus decimal UnaryExpression frontier only")
    );
    assert!(PREVIOUS_IDENTIFIER_REFERENCE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 237"));
    assert!(
        FRONTIER_SCOPE_NOTE.contains(
            "leading plus/minus direct IdentifierReference UnaryExpression frontier only"
        )
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));

    for forbidden in [
        concat!("use super::", "selected_lexical_slice"),
        concat!("use super::", "selected_static_semantics"),
        concat!("use super::", "selected_qualification_integration"),
        concat!(
            "use super::",
            "selected_variable_statement_name_correspondence"
        ),
        concat!("use super::", "selected_binding_scope"),
        concat!("use super::", "selected_one_level_block_binding_scope"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!(
            "consume_selected_leading_plus_minus_decimal_",
            "unary_expression"
        ),
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        // Candidate independence for trivia recognition specifically: the
        // production selected-trivia *recognizer* is never imported or
        // called, only the stable normative Unicode `Space_Separator`
        // property primitive (`is_space_separator`) is reused.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult (also already used by predecessor accepted
    // oracles at the same `super::unicode` path); none of them is the
    // production selected-trivia recognizer or a production expression
    // parser.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );
}

/// Required positive matrix: both operators over direct ASCII and direct
/// multibyte `IdentifierReference` operands, with fixture-owned literal byte
/// ranges for operator, inner reference, and the complete unary occurrence
/// -- never derived by search, rescan, or reparse. The inner reference range
/// excludes the operator byte (W1/W2).
#[test]
fn exact_positive_matrix_pins_fixture_owned_operator_and_reference_ranges() {
    const POSITIVE_MATRIX: &[(&str, Range, Range, Range, &str, &str)] = &[
        (
            "const x = -a;",
            Range(10, 11),
            Range(11, 12),
            Range(10, 12),
            "-",
            "a",
        ),
        (
            "const x = +foo;",
            Range(10, 11),
            Range(11, 14),
            Range(10, 14),
            "+",
            "foo",
        ),
        (
            "const x = -\u{03C0};",
            Range(10, 11),
            Range(11, 13),
            Range(10, 13),
            "-",
            "\u{03C0}",
        ),
        (
            "const x = +\u{1D49C};",
            Range(10, 11),
            Range(11, 15),
            Range(10, 15),
            "+",
            "\u{1D49C}",
        ),
    ];

    for (index, (text, operator, reference, whole, expected_operator, expected_reference)) in
        POSITIVE_MATRIX.iter().enumerate()
    {
        assert_eq!(slice(text, *operator), *expected_operator);
        assert_eq!(slice(text, *reference), *expected_reference);
        // The inner reference range never includes the operator byte.
        assert_eq!(reference.0, operator.1);
        assert_eq!(whole.0, operator.0);
        assert_eq!(whole.1, reference.1);

        let whole_text = slice(text, *whole);
        assert!(
            is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(whole_text),
            "{whole_text:?}"
        );
        assert!(is_selected_direct_identifier_reference(slice(
            text, *reference
        )));
        assert_eq!(
            authored_anchor(746_000 + index as u64, text, *whole),
            whole_text
        );
        // The inner reference anchor is exactly the direct spelling, never
        // the operator-prefixed spelling (W1).
        assert_eq!(
            authored_anchor(746_050 + index as u64, text, *reference),
            *expected_reference
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

/// First validation carrier from Issue #746: `let a; const x = -a;`. This is
/// a validation-harness choice, not a claim that the theorem is
/// `LexicalDeclaration`-specific.
#[test]
fn first_validation_carrier_form_is_recognized() {
    let source = "let a;\nconst x = -a;";
    assert!(
        is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(slice(
            source,
            Range(17, 19)
        ))
    );
}

/// Freezes the complete already-accepted selected trivia policy between
/// operator and operand: ASCII TAB/LF, the frozen Unicode 17
/// `Space_Separator` property beyond ASCII space (NBSP, IDEOGRAPHIC SPACE),
/// BOM, LINE SEPARATOR, and PARAGRAPH SEPARATOR all compose; comments never
/// do; and a visually-space-like code point the frozen contract excludes
/// (ZERO WIDTH SPACE, immediately outside the accepted `Space_Separator`
/// range) is proven still rejected (W6).
#[test]
fn selected_trivia_matrix_between_operator_and_operand() {
    let fixtures: &[(&str, Range, Range, Range, &str)] = &[
        (
            "const x = + a;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "+ a",
        ),
        (
            "const x = -\ta;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "-\ta",
        ),
        (
            "const x = +\na;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "+\na",
        ),
        (
            "const x = -\u{00A0}a;",
            Range(10, 11),
            Range(13, 14),
            Range(10, 14),
            "-\u{00A0}a",
        ),
        (
            "const x = +\u{3000}a;",
            Range(10, 11),
            Range(14, 15),
            Range(10, 15),
            "+\u{3000}a",
        ),
        (
            "const x = -\u{FEFF}a;",
            Range(10, 11),
            Range(14, 15),
            Range(10, 15),
            "-\u{FEFF}a",
        ),
        (
            "const x = +\u{2028}a;",
            Range(10, 11),
            Range(14, 15),
            Range(10, 15),
            "+\u{2028}a",
        ),
        (
            "const x = -\u{2029}a;",
            Range(10, 11),
            Range(14, 15),
            Range(10, 15),
            "-\u{2029}a",
        ),
    ];

    for (index, (text, operator, reference, whole, expected_whole)) in fixtures.iter().enumerate() {
        let whole_text = slice(text, *whole);
        assert_eq!(whole_text, *expected_whole);
        assert!(matches!(slice(text, *operator), "+" | "-"), "{operator:?}");
        assert_eq!(slice(text, *reference), "a");
        assert!(
            is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(whole_text),
            "{whole_text:?}"
        );
        assert_eq!(
            authored_anchor(746_100 + index as u64, text, *whole),
            whole_text
        );
    }

    // Comments never compose as operand trivia.
    for comment_control in ["+/*c*/a", "-/*c*/a", "+//c\na"] {
        assert!(
            !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(
                comment_control
            ),
            "{comment_control:?}"
        );
    }

    // Adversarial boundary: ZERO WIDTH SPACE (U+200B) sits immediately
    // outside the frozen `Space_Separator` range and is never accepted
    // merely because it visually resembles spacing (W6).
    assert!(!is_selected_unary_operand_trivia('\u{200B}'));
    assert!(is_selected_unary_operand_trivia('\u{00A0}'));
    assert!(
        !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression("+\u{200B}a")
    );
}

/// Direct-only boundary (W7): an authored `UnicodeEscapeSequence` operand
/// remains outside this theorem even though it visually resembles an
/// accepted direct spelling once decoded. This oracle never infers that an
/// already-accepted escaped `IdentifierReference` initializer elsewhere in
/// the grammar validates an escaped unary operand.
#[test]
fn direct_only_boundary_excludes_escaped_operands() {
    let escaped_lower: &str = concat!("-", "\\", "u0061");
    let escaped_word: &str = concat!("+", "\\", "u0066oo");

    for control in [escaped_lower, escaped_word] {
        assert!(
            !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    // The corresponding direct spellings remain accepted.
    assert!(is_selected_leading_plus_minus_direct_identifier_reference_unary_expression("-a"));
    assert!(is_selected_leading_plus_minus_direct_identifier_reference_unary_expression("+foo"));
}

/// Identifier policy composition (representative categories only, not a
/// re-test of Issue #237): ordinary direct, multibyte direct, strict-only
/// restricted words valid in this fixed non-strict envelope, the
/// `Yield=false`/`Await=false` special forms, and the `eval`/`arguments`
/// binding-contrast names all compose as valid operands, while
/// unconditionally reserved tokens remain outside this theorem (W4/W10
/// adjacent: the operand policy stays `IdentifierReference`, never
/// `BindingIdentifier`).
#[test]
fn identifier_reference_policy_composition_matrix() {
    const ACCEPTED_OPERANDS: &[&str] = &[
        "a",
        "foo",
        "\u{03C0}",
        "\u{1D49C}",
        "let",
        "static",
        "yield",
        "await",
        "eval",
        "arguments",
    ];
    for operand in ACCEPTED_OPERANDS {
        assert!(
            is_selected_direct_identifier_reference(operand),
            "{operand:?}"
        );
        let signed = format!("-{operand}");
        assert!(
            is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(&signed),
            "{signed:?}"
        );
    }

    // Unconditionally reserved tokens remain outside `IdentifierReference`
    // (and therefore outside this unary theorem), whatever their code-point
    // shape.
    for reserved in ["if", "this", "true", "false", "null", "typeof", "delete"] {
        assert!(!is_selected_direct_identifier_reference(reserved));
        let signed = format!("-{reserved}");
        assert!(
            !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(&signed),
            "{signed:?}"
        );
    }

    assert_eq!(UNCONDITIONALLY_RESERVED_WORDS.len(), 36);
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"yield"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"await"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"let"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"eval"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"arguments"));
}

/// Unicode identity firewall (W5): authored spelling never becomes
/// normalized identity. A precomposed direct operand and its
/// canonically-equivalent combining-mark decomposition remain distinct
/// authored semantic names and distinct byte lengths.
#[test]
fn direct_unicode_operand_is_not_normalized() {
    let precomposed = "\u{00E9}"; // "e" WITH ACUTE, 2 UTF-8 bytes
    let decomposed = "e\u{0301}"; // "e" + COMBINING ACUTE ACCENT, 3 UTF-8 bytes

    assert!(is_selected_direct_identifier_reference(precomposed));
    assert!(is_selected_direct_identifier_reference(decomposed));
    assert_ne!(precomposed, decomposed);
    assert_ne!(precomposed.len(), decomposed.len());

    let precomposed_signed = format!("-{precomposed}");
    let decomposed_signed = format!("-{decomposed}");
    assert!(
        is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(
            &precomposed_signed
        )
    );
    assert!(
        is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(
            &decomposed_signed
        )
    );
    assert_ne!(precomposed_signed, decomposed_signed);
}

/// Operator / nested-unary firewall (W8/W9): keeps `++`, `--`, mixed
/// double-operator forms, spaced double-operator forms, and other unary
/// operator families outside this bounded theorem.
#[test]
fn operator_and_nested_unary_firewall_remain_unowned() {
    const NESTED_AND_OTHER_UNARY_CONTROLS: &[&str] = &[
        "++a", "--a", "+-a", "-+a", "+ +a", "- -a", "!a", "~a", "delete a", "void a", "typeof a",
    ];
    for control in NESTED_AND_OTHER_UNARY_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Operand firewall (W10): non-`IdentifierReference` operands remain
/// outside this theorem, including the accepted decimal unary predecessor
/// family, boolean/null/`this`/string literals, and a parenthesized operand.
#[test]
fn non_identifier_reference_operand_firewall_remains_unowned() {
    const NON_IDENTIFIER_OPERAND_CONTROLS: &[&str] =
        &["-1", "+1e2", "+true", "-null", "-this", "-\"x\"", "-(a)"];
    for control in NON_IDENTIFIER_OPERAND_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Richer-expression tail firewall (W11): a complete unary-reference prefix
/// never authorizes member, call, binary, assignment, conditional,
/// exponentiation, comment, or unexpected-tail continuation.
#[test]
fn richer_expression_tail_controls_preserve_whole_source_transactionality() {
    const RICHER_EXPRESSION_CONTROLS: &[&str] = &[
        "-a.b",
        "+a()",
        "-a + b",
        "+a = b",
        "-a ? b : c",
        "-a ** 2",
        "-a/*comment*/",
        "-a unexpected",
    ];
    for control in RICHER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    // The complete unary-reference prefix itself remains independently
    // valid; only the richer tail is unowned.
    assert!(is_selected_leading_plus_minus_direct_identifier_reference_unary_expression("-a"));

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Exact fact-preservation theorem: for representative `Before`, `Same`,
/// `After`, and no-target sources, the inner `IdentifierReference` fact
/// (semantic name and authored anchor) is frozen independently of the
/// operator/trivia wrapper, and the existing selected lexical Binding/Scope
/// relation composes from that inner fact unchanged (W3/W12/W13).
#[test]
fn lexical_binding_scope_before_same_after_and_no_target_relations_are_independently_frozen() {
    struct BindingScopeFixture {
        id: &'static str,
        source: &'static str,
        binding_inventory: &'static [&'static str],
        containing_binding_name: &'static str,
        containing_binding_range: Range,
        operator_range: Range,
        reference_range: Range,
        target_binding_range: Option<Range>,
        expected: ExpectedBindingScopeTarget,
    }

    let fixtures = [
        BindingScopeFixture {
            id: "BINDSCOPE-BEFORE-001",
            source: "let a;\nconst x = -a;",
            binding_inventory: &["a", "x"],
            containing_binding_name: "x",
            containing_binding_range: Range(13, 14),
            operator_range: Range(17, 18),
            reference_range: Range(18, 19),
            target_binding_range: Some(Range(4, 5)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
        },
        BindingScopeFixture {
            id: "BINDSCOPE-SAME-001",
            source: "let a = -a;",
            binding_inventory: &["a"],
            containing_binding_name: "a",
            containing_binding_range: Range(4, 5),
            operator_range: Range(8, 9),
            reference_range: Range(9, 10),
            target_binding_range: Some(Range(4, 5)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Same,
            ),
        },
        BindingScopeFixture {
            id: "BINDSCOPE-AFTER-001",
            source: "const x = +a;\nlet a;",
            binding_inventory: &["x", "a"],
            containing_binding_name: "x",
            containing_binding_range: Range(6, 7),
            operator_range: Range(10, 11),
            reference_range: Range(11, 12),
            target_binding_range: Some(Range(18, 19)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::After,
            ),
        },
        BindingScopeFixture {
            id: "BINDSCOPE-NOTARGET-001",
            source: "const x = -z;",
            binding_inventory: &["x"],
            containing_binding_name: "x",
            containing_binding_range: Range(6, 7),
            operator_range: Range(10, 11),
            reference_range: Range(11, 12),
            target_binding_range: None,
            expected: ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        let whole = Range(fixture.operator_range.0, fixture.reference_range.1);
        let whole_text = slice(fixture.source, whole);
        assert!(
            is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(whole_text),
            "{}: {whole_text:?}",
            fixture.id
        );

        assert_eq!(
            slice(fixture.source, fixture.containing_binding_range),
            fixture.containing_binding_name,
            "{}",
            fixture.id
        );
        let reference_semantic_name = slice(fixture.source, fixture.reference_range);
        assert!(
            is_selected_direct_identifier_reference(reference_semantic_name),
            "{}",
            fixture.id
        );
        // The inner reference anchor never includes the operator (W1/W2).
        assert_eq!(fixture.reference_range.0, fixture.operator_range.1);

        let computed = expected_binding_scope_target(
            fixture.binding_inventory,
            fixture.containing_binding_name,
            reference_semantic_name,
        );
        assert_eq!(computed, fixture.expected, "{}", fixture.id);

        match (fixture.target_binding_range, computed) {
            (
                Some(target_range),
                ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(_),
            ) => {
                assert_eq!(
                    slice(fixture.source, target_range),
                    reference_semantic_name,
                    "{}",
                    fixture.id
                );
                assert_eq!(
                    authored_anchor(746_200 + index as u64, fixture.source, target_range),
                    reference_semantic_name,
                    "{}",
                    fixture.id
                );
            }
            (None, ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding) => {}
            _ => panic!("{}: target range and expected target disagree", fixture.id),
        }

        assert_eq!(
            authored_anchor(
                746_250 + index as u64,
                fixture.source,
                fixture.containing_binding_range
            ),
            fixture.containing_binding_name,
            "{}",
            fixture.id
        );
        assert_eq!(
            authored_anchor(
                746_300 + index as u64,
                fixture.source,
                fixture.reference_range
            ),
            reference_semantic_name,
            "{}",
            fixture.id
        );
    }

    // `Same` is proven with a clean, already-valid selected source (a
    // self-referencing lexical initializer) rather than an invented
    // fixture: `let a = -a;` needs no grammar/static widening because a
    // direct `IdentifierReference` RHS is already an accepted source shape,
    // and `expected_binding_scope_target` derives `Same` from the identical
    // declaration-order position, exactly mirroring the
    // `target_position == containing_position` branch already accepted in
    // `selected_binding_scope.rs` (consulted here only as read-only
    // vocabulary, never called).
    assert_eq!(
        expected_binding_scope_target(&["a"], "a", "a"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Same
        )
    );

    // No-target composes as absence of a same-source selected lexical
    // binding, never as a runtime unresolved-identifier exception claim
    // (W14 adjacent): this oracle asserts no such runtime vocabulary
    // appears.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("Reference", "Error")));
}

/// Static-semantics composition (W13 adjacent): the unary operator and the
/// inner RHS semantic name never become declaration names or static
/// rejection keys. Existing duplicate/collision and missing-initializer
/// authority remains driven only by authored `BindingIdentifier` bindings.
#[test]
fn static_semantics_composition_remains_driven_by_authored_bindings() {
    struct StaticCompositionFixture {
        source: &'static str,
        subject: Range,
        subject_fragment: &'static str,
        control_gold_id: &'static str,
    }

    const FIXTURES: &[StaticCompositionFixture] = &[
        StaticCompositionFixture {
            source: "let let = -a;",
            subject: Range(4, 7),
            subject_fragment: "let",
            control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        },
        StaticCompositionFixture {
            source: "let x = -a, x = foo;",
            subject: Range(12, 13),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        },
        StaticCompositionFixture {
            source: "const x = -a, y;",
            subject: Range(14, 15),
            subject_fragment: "y",
            control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        },
        StaticCompositionFixture {
            source: "let x = -a; let x = foo;",
            subject: Range(16, 17),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        },
    ];

    for fixture in FIXTURES {
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment
        );
        assert!(
            gold_source(fixture.control_gold_id).is_some(),
            "{}",
            fixture.control_gold_id
        );
    }

    // Initializer presence: `const x = -a;` alone has an initializer
    // present, so it never triggers the missing-const-initializer rejection
    // that the bare `y` sibling above does.
    let with_initializer = "const x = -a;";
    assert!(
        is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(slice(
            with_initializer,
            Range(10, 12)
        ))
    );

    // The RHS reference name never becomes an additional `BoundName`: `a`
    // is declared exactly once (`let a;`) even though it also appears as
    // the unary operand's reference in `x`'s initializer.
    let rhs_not_a_new_bound_name = "let a;\nconst x = -a;";
    let binding_inventory = ["a", "x"];
    assert_eq!(binding_inventory.len(), 2);
    assert_eq!(
        expected_binding_scope_target(&binding_inventory, "x", "a"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Before
        )
    );
    assert!(
        is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(slice(
            rhs_not_a_new_bound_name,
            Range(17, 19)
        ))
    );

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

/// Whole-source transactionality (W3 adjacent): a valid earlier
/// unary-reference initializer never escapes as a committed selected
/// success when a later binding in the same source is incomplete or
/// triggers an already-owned malformed `BindingIdentifier` Grammar failure.
#[test]
fn whole_source_transactionality_prevents_partial_prefix_commitment() {
    struct FailedTransactionFixture {
        source: &'static str,
        earlier_unary_rhs: Range,
        outcome: FrontierOutcome,
        grammar_subject: Option<Range>,
    }

    let fixtures = [
        FailedTransactionFixture {
            source: "let a = -a, b = ;",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "const x=-a, y=",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a; const x=-a, y=;",
            earlier_unary_rhs: Range(15, 17),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: r"let a = -a, \u{}=2;",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::SyntaxRejected,
            grammar_subject: Some(Range(12, 16)),
        },
    ];

    for fixture in &fixtures {
        let earlier = slice(fixture.source, fixture.earlier_unary_rhs);
        assert!(
            is_selected_leading_plus_minus_direct_identifier_reference_unary_expression(earlier),
            "{earlier:?}"
        );
        assert_ne!(fixture.outcome, FrontierOutcome::SelectedAcceptedIncomplete);

        if let Some(subject) = fixture.grammar_subject {
            assert_eq!(fixture.outcome, FrontierOutcome::SyntaxRejected);
            assert_eq!(slice(fixture.source, subject), r"\u{}");
        }
    }

    // Composing with an already-owned malformed BindingIdentifier Grammar
    // case does not retroactively invalidate the earlier accepted unary
    // occurrence itself -- the failure is a whole-source outcome, not
    // evidence that the earlier occurrence was mis-owned.
    assert!(is_selected_leading_plus_minus_direct_identifier_reference_unary_expression("-a"));
}

/// Freezes the presence-only, unqualified, validation-only handoff: this
/// leaf composes to `SelectedAcceptedIncomplete`, `UnsupportedCoverage`,
/// `StaticSemanticsRejected`, and `SyntaxRejected` remain distinct, and no
/// retained production representation, runtime, or general-parser
/// vocabulary is introduced (W15/W16/W17 adjacent).
#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert!(THIS_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(THIS_ORACLE_SOURCE.contains("UnsupportedCoverage"));
    assert!(THIS_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
    assert!(THIS_ORACLE_SOURCE.contains("SyntaxRejected"));
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("enum Selected", "Literal"),
        concat!("enum SelectedPrimary", "Expression"),
        concat!("enum SelectedExpression", "Kind"),
        concat!("enum Selected", "UnaryExpressionKind"),
        concat!("struct Selected", "UnaryExpressionNode"),
        concat!("enum Operator", "Kind"),
        concat!("enum Plus", "Minus"),
        concat!("struct Signed", "Flag"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct Unary", "SourceAnchor"),
        concat!("struct Number", "Value"),
        concat!("struct Reference", "Record"),
        concat!("struct Environment", "Record"),
        concat!("enum Initializer", "Kind"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
        concat!("To", "Numeric("),
        concat!("To", "Number("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
