use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssExponentSign, CssNumberSign, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransformMatrixFunction, CssTransformQualificationOutcome, CssTransformUnsupportedReason,
    CssTransformValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
}

fn parser_limits() -> CssParserLimits {
    parser_limits_with_occurrences(8192)
}

fn parser_limits_with_occurrences(max_declaration_occurrences: usize) -> CssParserLimits {
    CssParserLimits::new(
        100_000,
        256,
        256,
        max_declaration_occurrences,
        1024,
        1024,
        1024,
        1024,
        8192,
    )
    .unwrap()
}

fn qualify(source_id: u64, css: &str) -> CssValueQualificationRunResult {
    qualify_with_limits(source_id, css, parser_limits())
}

fn qualify_with_limits(
    source_id: u64,
    css: &str,
    parser_limits: CssParserLimits,
) -> CssValueQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), css.to_owned());
    let parser_result = analyze_css_source(&source, tokenizer_limits(), parser_limits).unwrap();
    run(parser_result).unwrap()
}

fn outcome_at(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &CssTransformQualificationOutcome {
    result
        .transform_observations()
        .get(index)
        .unwrap_or_else(|| panic!("missing transform observation at {index}"))
        .outcome()
}

fn assert_whole_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformQualificationOutcome::Qualified(CssTransformValue::None),
        "expected whole-value none at {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected selected-profile invalidity at {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssTransformUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected unsupported selected-profile coverage at {index}"
    );
}

fn qualified_functions(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssTransformMatrixFunction] {
    match outcome_at(result, index) {
        CssTransformQualificationOutcome::Qualified(CssTransformValue::Functions(functions)) => {
            functions
        }
        other => panic!("expected qualified matrix components at {index}, got {other:?}"),
    }
}

/// Reconstructs one retained `Number` token's authored numeric structure
/// from tokenizer-owned evidence alone -- sign spelling, integer digits,
/// fraction digits, and exponent sign/digits -- so `1`, `1.0`, `1e0`, `+1`,
/// and `-0` stay distinguishable in assertions. The exponent's authored
/// `e`/`E` letter case is not part of retained numeric structure and is
/// therefore not reconstructed here. No machine float is ever produced.
fn authored_number_spelling(token: &CssTokenKind) -> String {
    let CssTokenKind::Number { value, .. } = token else {
        panic!("matrix argument evidence did not resolve to a Number token");
    };

    let mut spelling = String::new();
    match value.sign() {
        Some(CssNumberSign::Plus) => spelling.push('+'),
        Some(CssNumberSign::Minus) => spelling.push('-'),
        None => {}
    }
    spelling.push_str(value.decimal().integer_digits());
    let fraction_digits = value.decimal().fraction_digits();
    if !fraction_digits.is_empty() {
        spelling.push('.');
        spelling.push_str(fraction_digits);
    }
    if let Some(exponent) = value.decimal().exponent() {
        spelling.push('e');
        match exponent.sign() {
            Some(CssExponentSign::Plus) => spelling.push('+'),
            Some(CssExponentSign::Minus) => spelling.push('-'),
            None => {}
        }
        spelling.push_str(exponent.digits());
    }
    spelling
}

fn argument_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> Vec<String> {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing matrix component {function_index} at {index}"));
    function
        .arguments()
        .iter()
        .map(|evidence| {
            let token = result
                .transform_matrix_argument_token(*evidence)
                .expect("matrix argument evidence did not resolve");
            authored_number_spelling(token)
        })
        .collect()
}

fn assert_all_invalid(source_id: u64, values: &[&str]) {
    for (offset, value) in values.iter().enumerate() {
        let css = format!("a{{transform:{value};}}");
        let result = qualify(source_id + offset as u64, &css);
        assert_eq!(
            result.transform_observations().len(),
            1,
            "expected one observation for {value:?}"
        );
        assert_eq!(
            outcome_at(&result, 0),
            &CssTransformQualificationOutcome::InvalidForSelectedValueGrammar,
            "expected selected-profile invalidity for {value:?}"
        );
    }
}

fn assert_all_unsupported(source_id: u64, reason: CssTransformUnsupportedReason, values: &[&str]) {
    for (offset, value) in values.iter().enumerate() {
        let css = format!("a{{transform:{value};}}");
        let result = qualify(source_id + offset as u64, &css);
        assert_eq!(
            result.transform_observations().len(),
            1,
            "expected one observation for {value:?}"
        );
        assert_eq!(
            outcome_at(&result, 0),
            &CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
            "expected unsupported selected-profile coverage for {value:?}"
        );
    }
}

// 1. Alternatives and exclusivity: `transform = none | <transform-list>`
// is an exclusive alternation, so `none` is a whole-value branch and never
// a `<transform-list>` component in either authored order.

#[test]
fn whole_none_qualifies_and_remains_exclusive() {
    let result = qualify(
        418100,
        concat!(
            "a{transform:none;}",
            "b{transform:NONE;}",
            "c{transform:n\\6f ne;}",
            "d{transform: /**/ none /**/ ;}",
            "e{transform:none matrix(1,0,0,1,0,0);}",
            "f{transform:matrix(1,0,0,1,0,0) none;}",
            "g{transform:none none;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 7);
    assert_whole_none(&result, 0);
    assert_whole_none(&result, 1);
    assert_whole_none(&result, 2);
    assert_whole_none(&result, 3);
    assert_invalid(&result, 4);
    assert_invalid(&result, 5);
    assert_invalid(&result, 6);
}

// 2. `matrix() = matrix(<number>#{6})`: exactly six ordered direct
// `<number>` arguments qualify, and each argument's authored numeric
// identity stays source-backed tokenizer evidence -- never normalized
// through machine floating point, so `1`, `1.0`, and `1e0` stay distinct.

#[test]
fn canonical_matrix_qualifies_with_six_ordered_arguments() {
    let result = qualify(418110, "a{transform:matrix(1,0,0,1,0,0);}");

    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(qualified_functions(&result, 0).len(), 1);
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
}

#[test]
fn authored_numeric_identity_is_preserved_without_normalization() {
    let result = qualify(
        418111,
        concat!(
            "a{transform:matrix(+1,-0,0.0,1e0,10,-10);}",
            "b{transform:matrix(1,1.0,1e0,1e+2,1e-2,.5);}",
            "c{transform:matrix(0,-0,+0,0.0,0e0,00);}",
        ),
    );

    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["+1", "-0", "0.0", "1e0", "10", "-10"]
    );
    // The final argument is authored `.5`: the absent leading integer digit
    // is canonicalized to `0` by the tokenizer's own retained numeric
    // contract, upstream of this leaf. This slice preserves exactly what
    // the tokenizer retained and normalizes nothing further itself.
    assert_eq!(
        argument_spellings(&result, 1, 0),
        ["1", "1.0", "1e0", "1e+2", "1e-2", "0.5"]
    );
    assert_eq!(
        argument_spellings(&result, 2, 0),
        ["0", "-0", "+0", "0.0", "0e0", "00"]
    );

    // The five authored spellings the profile must keep distinguishable
    // are pairwise distinct as retained evidence, never collapsed to one
    // interpreted magnitude.
    let distinct = argument_spellings(&result, 1, 0);
    assert_ne!(distinct[0], distinct[1]);
    assert_ne!(distinct[1], distinct[2]);
    assert_ne!(distinct[0], distinct[2]);
}

// 3. `<transform-list> = <transform-function>+`: repeated components are
// whitespace-separated, preserved in exact authored order, and -- because
// CSS tokenization needs no separating whitespace after `)` -- also
// partition correctly when authored adjacently.

#[test]
fn repeated_matrix_components_preserve_authored_order() {
    let result = qualify(
        418120,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) matrix(2,0,0,2,10,-10);}",
            "b{transform:matrix(2,0,0,2,10,-10) matrix(1,0,0,1,0,0);}",
            "c{transform:matrix(1,0,0,1,0,0) matrix(1,0,0,1,0,0) matrix(3,0,0,3,0,0);}",
        ),
    );

    assert_eq!(qualified_functions(&result, 0).len(), 2);
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_eq!(
        argument_spellings(&result, 0, 1),
        ["2", "0", "0", "2", "10", "-10"]
    );

    // The reversed authoring order yields the reversed component order:
    // repetition order is authored evidence, never sorted or deduplicated.
    assert_eq!(
        argument_spellings(&result, 1, 0),
        ["2", "0", "0", "2", "10", "-10"]
    );
    assert_eq!(
        argument_spellings(&result, 1, 1),
        ["1", "0", "0", "1", "0", "0"]
    );

    assert_eq!(qualified_functions(&result, 2).len(), 3);
    assert_eq!(
        argument_spellings(&result, 2, 2),
        ["3", "0", "0", "3", "0", "0"]
    );
}

#[test]
fn adjacent_matrix_components_without_whitespace_partition_correctly() {
    let result = qualify(
        418121,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0)matrix(2,0,0,2,10,-10);}",
            "b{transform:matrix(1,0,0,1,0,0)/**/matrix(2,0,0,2,10,-10);}",
        ),
    );

    for index in 0..2 {
        assert_eq!(qualified_functions(&result, index).len(), 2);
        assert_eq!(
            argument_spellings(&result, index, 0),
            ["1", "0", "0", "1", "0", "0"]
        );
        assert_eq!(
            argument_spellings(&result, index, 1),
            ["2", "0", "0", "2", "10", "-10"]
        );
    }
}

// 4. `#{6}` is an exact arity: every other directly visible matrix-level
// argument count is decisive selected-profile invalidity.

#[test]
fn matrix_argument_cardinality_other_than_six_is_invalid() {
    assert_all_invalid(
        418130,
        &[
            "matrix()",
            "matrix(1)",
            "matrix(1,2)",
            "matrix(1,2,3,4,5)",
            "matrix(1,2,3,4,5,6,7)",
            "matrix(1,2,3,4,5,6,7,8)",
        ],
    );

    // The adjacent accepted arity is the only qualifying one, sealing the
    // off-by-one boundary from both sides.
    let result = qualify(418139, "a{transform:matrix(1,2,3,4,5,6);}");
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "2", "3", "4", "5", "6"]
    );
}

// 5. `#` is a comma-separated repetition: whitespace is not a separator,
// and an authored-empty argument position is preserved as its own ordered
// slot and then rejected -- never collapsed away into a shorter list.

#[test]
fn matrix_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        418140,
        &[
            "matrix(1 0 0 1 0 0)",
            "matrix(1,0 0,1,0,0)",
            "matrix(,1,0,0,1,0,0)",
            "matrix(1,,0,1,0,0)",
            "matrix(1,0,0,1,0,0,)",
            "matrix(1,0,,,1,0,0)",
            "matrix(,,,,,)",
        ],
    );
}

// 6. `<number>` admits only a direct retained `Number` token: every other
// direct token category at an argument position is a decisive direct
// token-category failure.

#[test]
fn direct_non_number_argument_categories_are_invalid() {
    assert_all_invalid(
        418150,
        &[
            "matrix(1px,0,0,1,0,0)",
            "matrix(50%,0,0,1,0,0)",
            "matrix(foo,0,0,1,0,0)",
            "matrix(\"1\",0,0,1,0,0)",
            "matrix(1,0,0,1,0,1px)",
            "matrix(1,0,0,1,0,50%)",
            "matrix(1,0,0,1,0,none)",
            "matrix(1,0,0,1,0,#1)",
            "matrix(1deg,0,0,1,0,0)",
        ],
    );
}

// 7. An opaque non-deferred Function at an argument position is
// structurally feasible but its validity depends on calculated-value
// semantics this slice does not own, so it stays unsupported. Critically,
// a comma retained inside that nested Function is at a deeper relative
// depth and must never become a matrix argument separator.

#[test]
fn opaque_matrix_argument_function_is_unsupported() {
    assert_all_unsupported(
        418160,
        CssTransformUnsupportedReason::FunctionValuedMatrixArgument,
        &[
            "matrix(calc(1),0,0,1,0,0)",
            "matrix(1,0,0,1,0,calc(1))",
            "matrix(calc(1),calc(2),0,1,0,0)",
            "matrix(min(1,2),0,0,1,0,0)",
            "matrix(1,0,0,1,0,0) matrix(calc(1),0,0,1,0,0)",
        ],
    );
}

#[test]
fn nested_commas_never_change_matrix_arity() {
    // `calc(1,2)` contributes exactly ONE matrix-level slot: its inner
    // comma is at matrix-body relative depth one. Were it leaking, this
    // would be a seven-slot shell and therefore decisively Invalid, so the
    // Unsupported outcome is itself the depth-isolation proof.
    assert_all_unsupported(
        418170,
        CssTransformUnsupportedReason::FunctionValuedMatrixArgument,
        &[
            "matrix(calc(1,2),0,0,1,0,0)",
            "matrix(1,0,0,1,0,calc(1,2))",
            "matrix(calc(1,2,3,4,5,6,7),0,0,1,0,0)",
            "matrix(calc(calc(1,2),3),0,0,1,0,0)",
        ],
    );

    // Nested non-parenthesis blocks isolate their commas identically.
    assert_all_unsupported(
        418180,
        CssTransformUnsupportedReason::FunctionValuedMatrixArgument,
        &[
            "matrix(calc([1,2]),0,0,1,0,0)",
            "matrix(calc({1,2}),0,0,1,0,0)",
        ],
    );

    // A nested comma that genuinely does add a matrix-level slot once the
    // nesting closes is still counted: this rules out "ignore every comma
    // after a Function" as an accidental passing implementation.
    assert_all_invalid(
        418190,
        &["matrix(calc(1,2),0,0,1,0,0,7)", "matrix(calc(1,2),0)"],
    );
}

// 8. Mixed shapes: directly visible structural shell (arity, empty slot)
// and direct token-category failure both outrank opaque unsupported
// semantics found in a sibling slot.

#[test]
fn directly_visible_structure_outranks_opaque_argument_semantics() {
    assert_all_invalid(
        418200,
        &[
            "matrix(calc(1),0)",
            "matrix(calc(1),0,0,1,0,0,2)",
            "matrix(1,,calc(1),1,0,0)",
            "matrix(calc(1),,0,1,0,0)",
            "matrix(1px,calc(1),0,1,0,0)",
            "matrix(calc(1),1px,0,1,0,0)",
            "matrix(calc(1),0,0,1,0,foo)",
            "matrix(calc(1) 1,0,0,1,0,0)",
            "matrix(1 calc(1),0,0,1,0,0)",
            "matrix(calc(1)calc(2),0,0,1,0,0)",
            // Near-miss arity beside an opaque argument: the directly
            // visible five- and seven-slot shells stay decisive, so a
            // structurally feasible but unevaluated `calc()` can never
            // relax the exact `#{6}` boundary by one position in either
            // direction.
            "matrix(calc(1),0,0,1,0)",
            "matrix(1,0,0,1,0,calc(1),0)",
            "matrix(calc(1),0,0,1,0,0,calc(2))",
        ],
    );
}

// 9. Deferred substitution can still change the surrounding token
// sequence, separators, and cardinality, so it is resolved before any
// surrounding shape conclusion -- including an arity that looks decisive.

#[test]
fn deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        418210,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "var(--x)",
            "matrix(var(--x),0)",
            "matrix(var(--x),0,0,1,0,0)",
            "matrix(1,0,0,1,0,0,var(--x))",
            "matrix(1,0,0,1,0,0) var(--x)",
            "var(--x) matrix(1,0,0,1,0,0)",
            "none var(--x)",
            "matrix(calc(var(--x)),0,0,1,0,0)",
            "rotate(var(--x))",
            "env(--x)",
        ],
    );
}

// 10. Outer coverage boundary: every non-`matrix` `<transform-function>`
// stays outside selected-profile coverage rather than being decided here,
// in either authored order and regardless of a sibling qualified matrix.

#[test]
fn non_matrix_transform_functions_remain_outside_selected_profile() {
    assert_all_unsupported(
        418220,
        CssTransformUnsupportedReason::NonMatrixTransformFunction,
        &[
            "rotate(1deg)",
            "matrix(1,0,0,1,0,0) rotate(1deg)",
            "rotate(1deg) matrix(1,0,0,1,0,0)",
            "translate(10px,20px)",
            "scale(2)",
            "skew(1deg)",
            "perspective(1px)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "unknownfunction(1,0,0,1,0,0)",
            "rotate(1deg) rotate(2deg)",
        ],
    );
}

// 11. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, so it is decisive invalidity --
// including in the `none, matrix(...)` shape.

#[test]
fn top_level_comma_is_never_a_transform_list_separator() {
    assert_all_invalid(
        418230,
        &[
            "matrix(1,0,0,1,0,0), matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0),matrix(1,0,0,1,0,0)",
            "none, matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0), none",
            "none,none",
            ",matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0),",
        ],
    );
}

// 12. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `matrix()` extent is qualified from retained
// interior evidence alone. No closing `)` is synthesized, searched for, or
// reconstructed, and this never generalizes into a blanket missing-close
// acceptance: retained material that lands inside a slot still decides it.

#[test]
fn true_stylesheet_eof_ended_matrix_extent_follows_parser_authority() {
    let result = qualify(418240, "a{transform:matrix(1,0,0,1,0,0");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );

    // Negative side: without a closer, later authored material is absorbed
    // into the final slot by real retained structure, so it no longer
    // satisfies exactly one direct Number.
    let absorbed = qualify(418241, "a{transform:matrix(1,0,0,1,0,0 7");
    assert_eq!(
        absorbed.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_invalid(&absorbed, 0);

    // A short EOF-ended extent stays short: EOF never fills missing slots.
    let short = qualify(418242, "a{transform:matrix(1,0,0");
    assert_invalid(&short, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a matrix".
    let trailing = qualify(418243, "a{transform:matrix(1,0,0,1,0,0) 7;}");
    assert_invalid(&trailing, 0);
}

#[test]
fn trivia_never_changes_matrix_slot_interpretation() {
    let result = qualify(
        418250,
        concat!(
            "a{transform:matrix(1,/**/0,0,1,0,0);}",
            "b{transform:matrix(1/**/,0,0,1,0,0);}",
            "c{transform:matrix(1,0,0,1,0,0/**/);}",
            "d{transform:matrix(/**/1,0,0,1,0,0);}",
            "e{transform:matrix( 1 , 0 , 0 , 1 , 0 , 0 );}",
            "f{transform:matrix(1,/*,*/0,0,1,0,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        assert_eq!(
            argument_spellings(&result, index, 0),
            ["1", "0", "0", "1", "0", "0"],
            "trivia changed slot interpretation at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(418260, &["matrix(1,/**/,0,1,0,0)", "matrix(1,0,0,1,0/**/)"]);
}

// 13. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser and is never upgraded, reconstructed, or re-scanned here.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(
        418265,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) !important;}",
            "b{transform:matrix(1,0,0,1,0,0)!important;}",
            "c{transform:none !important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_eq!(
        argument_spellings(&result, 1, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_whole_none(&result, 2);

    for index in 0..3 {
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
}

#[test]
fn nonordinary_contexts_do_not_enter_transform_dispatch() {
    for (source_id, css) in [
        (418270, "@font-face{transform:matrix(1,0,0,1,0,0);}"),
        (418271, "@page{transform:matrix(1,0,0,1,0,0);}"),
        (418272, "@keyframes k{from{transform:matrix(1,0,0,1,0,0);}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transform_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_termination_preserves_committed_prefix_only() {
    let incomplete = qualify_with_limits(
        418280,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0);}",
            "b{transform:matrix(2,0,0,2,0,0);}",
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.transform_observations().len(), 1);
    assert_eq!(
        argument_spellings(&incomplete, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
}

#[test]
fn unsupported_region_material_never_produces_transform_observations() {
    let result = qualify(
        418290,
        concat!(
            "@media screen{a{transform:matrix(9,0,0,9,0,0);}}",
            "b{transform:matrix(1,0,0,1,0,0);}",
        ),
    );

    assert!(
        !result
            .upstream_parser_result()
            .unsupported_regions()
            .is_empty(),
        "expected retained unsupported-region evidence"
    );

    // Only the ordinary declaration outside the unsupported region is
    // qualified: unsupported-region material is propagated unchanged, never
    // reconstructed into a qualified occurrence.
    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
}

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:none;}",
        "b{transform:matrix(1,0,0,1,0,0) matrix(2,0,0,2,0,0);}",
        "c{transform:matrix(calc(1,2),0,0,1,0,0);}",
        "d{transform:rotate(1deg);}",
        "e{transform:matrix(var(--x),0);}",
        "f{transform:matrix(1,2);}",
        "g{transform:matrix(1,0,0,1,0,0) none;}",
    );

    let first = qualify(418300, css);
    let repeated = qualify(418300, css);
    let another_source = qualify(418301, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_whole_none(&first, 0);
    assert_eq!(qualified_functions(&first, 1).len(), 2);
    assert_unsupported(
        &first,
        2,
        CssTransformUnsupportedReason::FunctionValuedMatrixArgument,
    );
    assert_unsupported(
        &first,
        3,
        CssTransformUnsupportedReason::NonMatrixTransformFunction,
    );
    assert_unsupported(
        &first,
        4,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
    );
    assert_invalid(&first, 5);
    assert_invalid(&first, 6);
}

// Adversarial sealing: outcome precedence must follow evidence authority,
// never which component or token the walk happened to reach first.

#[test]
fn invalid_unsupported_precedence_is_scan_order_independent() {
    // Decisive invalidity outranks unsupported coverage in both orders.
    assert_all_invalid(
        418310,
        &[
            "rotate(1deg) matrix(1,2)",
            "matrix(1,2) rotate(1deg)",
            "calc(1) matrix(1,2)",
            "matrix(1,2) calc(1)",
            "matrix(calc(1),0,0,1,0,0) matrix(1,2)",
            "matrix(1,2) matrix(calc(1),0,0,1,0,0)",
        ],
    );

    // Between the two unsupported reasons the coarser outer grammar level
    // is reported first, identically in both authored orders.
    assert_all_unsupported(
        418320,
        CssTransformUnsupportedReason::NonMatrixTransformFunction,
        &[
            "matrix(calc(1),0,0,1,0,0) rotate(1deg)",
            "rotate(1deg) matrix(calc(1),0,0,1,0,0)",
        ],
    );

    // Deferred substitution outranks both, in every authored position.
    assert_all_unsupported(
        418330,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "var(--x) rotate(1deg) matrix(1,2)",
            "matrix(1,2) rotate(1deg) var(--x)",
            "rotate(1deg) matrix(var(--x),0) matrix(1,2)",
        ],
    );
}

#[test]
fn whole_value_boundaries_are_preserved() {
    assert_all_unsupported(
        418340,
        CssTransformUnsupportedReason::CssWideKeyword,
        &["inherit", "initial", "unset", "revert", "REVERT-LAYER"],
    );

    assert_all_unsupported(
        418350,
        CssTransformUnsupportedReason::WholeValueFunction,
        &["first-valid(matrix(1,0,0,1,0,0))", "cycle(none)"],
    );

    // A whole-value-only Function has no independent meaning embedded in a
    // `<transform-list>` or at a matrix argument position.
    assert_all_invalid(
        418360,
        &[
            "matrix(1,0,0,1,0,0) first-valid(none)",
            "matrix(first-valid(1),0,0,1,0,0)",
        ],
    );

    // A CSS-wide keyword is a whole-value branch only.
    assert_all_invalid(
        418370,
        &["inherit matrix(1,0,0,1,0,0)", "matrix(1,0,0,1,0,inherit)"],
    );
}

#[test]
fn matrix_function_name_recognition_is_ascii_case_insensitive() {
    let result = qualify(
        418380,
        concat!(
            "a{transform:MATRIX(1,0,0,1,0,0);}",
            "b{transform:MaTrIx(1,0,0,1,0,0);}",
            "c{transform:m\\61 trix(1,0,0,1,0,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            argument_spellings(&result, index, 0),
            ["1", "0", "0", "1", "0", "0"],
            "matrix name recognition failed at {index}"
        );
    }

    // A name that merely starts with `matrix` is a different function and
    // stays outside selected-profile coverage.
    assert_all_unsupported(
        418390,
        CssTransformUnsupportedReason::NonMatrixTransformFunction,
        &["matrix3d(1,0,0,1,0,0)", "matrixx(1,0,0,1,0,0)"],
    );
}

#[test]
fn structurally_malformed_components_are_invalid() {
    assert_all_invalid(
        418400,
        &[
            "1",
            "matrix",
            "(1,0,0,1,0,0)",
            "[matrix(1,0,0,1,0,0)]",
            "matrix(1,0,0,1,0,0))",
            "matrix((1,0,0,1,0,0)",
            "1 matrix(1,0,0,1,0,0)",
        ],
    );
}

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        418410,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0);transform:none;}",
            "b{transform-box:border-box;}",
            "c{transform-origin:left top;}",
            "d{transform-style:flat;}",
            "e{scale:2;}",
            "f{rotate:45deg;}",
            "g{translate:10px;}",
            "h{counter-reset:reversed(foo);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(result.transform_observations()[0].occurrence_index(), 0);
    assert_eq!(result.transform_observations()[1].occurrence_index(), 1);
    assert_eq!(
        argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_whole_none(&result, 1);

    // The longer `transform-*` property names never enter this dispatch,
    // and the new matrix-body depth scoping never leaks into the accepted
    // single-value transform leaves or the accepted `reversed()` branch.
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.counter_reset_observations().len(), 1);
}
