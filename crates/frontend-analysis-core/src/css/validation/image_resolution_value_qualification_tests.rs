use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssImageResolutionQualificationOutcome, CssImageResolutionUnsupportedReason,
    CssImageResolutionValue, CssValueQualificationRunResult, run,
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

fn qualified(value: CssImageResolutionValue) -> CssImageResolutionQualificationOutcome {
    CssImageResolutionQualificationOutcome::Qualified(value)
}

fn unsupported(
    reason: CssImageResolutionUnsupportedReason,
) -> CssImageResolutionQualificationOutcome {
    CssImageResolutionQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssImageResolutionQualificationOutcome],
) {
    let actual: Vec<_> = result
        .image_resolution_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(actual, expected);
}

// #418 comment 5601753463 / #596: current focused WPT (`image-resolution-valid.html`
// @ web-platform-tests/wpt@3a1177e6c76f1a7f854f3ae4b93a00835e171fad) still asserts
// `image-resolution: snap -8dpcm from-image;` as valid. That corroboration is
// stale relative to `w3c/csswg-drafts#8532`, which the Working Group resolved
// with "negative <resolution> values are invalid", and current Values 4 says
// the allowed range of `<resolution>` always excludes negative values. This
// candidate intentionally diverges from that stale WPT case; do not "fix" it
// back to match without a fresh maintainer/authority resolution.
#[test]
fn all_twelve_direct_valid_shapes_qualify_with_correct_shape() {
    use CssImageResolutionValue::{
        DirectResolution, DirectResolutionAndSnap, FromImage, FromImageAndResolution,
        FromImageAndResolutionAndSnap, FromImageAndSnap,
    };

    let result = qualify(
        59600,
        concat!(
            "a{image-resolution:from-image;}",
            "b{image-resolution:1dpi;}",
            "c{image-resolution:from-image 2dpcm;}",
            "d{image-resolution:2dpcm from-image;}",
            "e{image-resolution:from-image snap;}",
            "f{image-resolution:snap from-image;}",
            "g{image-resolution:3dppx snap;}",
            "h{image-resolution:snap 3dppx;}",
            "i{image-resolution:from-image 4dpi snap;}",
            "j{image-resolution:4dpi from-image snap;}",
            "k{image-resolution:snap from-image 4dpi;}",
            "l{image-resolution:snap 4dpi from-image;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(FromImage),
            qualified(DirectResolution),
            qualified(FromImageAndResolution),
            qualified(FromImageAndResolution),
            qualified(FromImageAndSnap),
            qualified(FromImageAndSnap),
            qualified(DirectResolutionAndSnap),
            qualified(DirectResolutionAndSnap),
            qualified(FromImageAndResolutionAndSnap),
            qualified(FromImageAndResolutionAndSnap),
            qualified(FromImageAndResolutionAndSnap),
            qualified(FromImageAndResolutionAndSnap),
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn snap_splitting_the_inner_group_is_decisive_invalid() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        59601,
        concat!(
            "a{image-resolution:from-image snap 4dppx;}",
            "b{image-resolution:3dpi snap from-image;}",
        ),
    );

    let expected = vec![Invalid; 2];
    assert_expected(&result, &expected);
}

#[test]
fn snap_alone_is_invalid() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(59602, "a{image-resolution:snap;}");
    assert_expected(&result, &[Invalid]);
}

#[test]
fn duplicate_operand_shapes_are_invalid() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        59603,
        concat!(
            "a{image-resolution:from-image from-image;}",
            "b{image-resolution:1dpi 2dpi;}",
            "c{image-resolution:snap snap;}",
            "d{image-resolution:from-image 1dpi from-image;}",
            "e{image-resolution:1dpi snap 2dpi;}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn all_supported_units_qualify_ascii_case_insensitively_including_x_alias() {
    use CssImageResolutionValue::DirectResolution;

    let result = qualify(
        59604,
        concat!(
            "a{image-resolution:1dpi;}",
            "b{image-resolution:1DPI;}",
            "c{image-resolution:1dpcm;}",
            "d{image-resolution:1DPCM;}",
            "e{image-resolution:1dppx;}",
            "f{image-resolution:1DPPX;}",
            "g{image-resolution:1x;}",
            "h{image-resolution:1X;}",
        ),
    );

    let expected = vec![qualified(DirectResolution); 8];
    assert_expected(&result, &expected);
}

#[test]
fn zero_including_signed_zero_qualifies_and_negative_non_zero_is_invalid() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssImageResolutionValue::{DirectResolution, FromImageAndResolutionAndSnap};

    let zero = qualify(
        59605,
        concat!(
            "a{image-resolution:0dpi;}",
            "b{image-resolution:0dpcm;}",
            "c{image-resolution:0dppx;}",
            "d{image-resolution:0x;}",
            "e{image-resolution:+0dppx;}",
            "f{image-resolution:-0dppx;}",
        ),
    );
    let expected = vec![qualified(DirectResolution); 6];
    assert_expected(&zero, &expected);

    let negative = qualify(
        59606,
        concat!(
            "a{image-resolution:-1dpi;}",
            "b{image-resolution:-8dpcm;}",
            "c{image-resolution:-0.5dppx;}",
            "d{image-resolution:-2x;}",
        ),
    );
    let expected = vec![Invalid; 4];
    assert_expected(&negative, &expected);

    // Intentional stale-WPT regression: see the module-level comment above.
    let stale_wpt_divergence = qualify(59607, "a{image-resolution:snap -8dpcm from-image;}");
    assert_expected(&stale_wpt_divergence, &[Invalid]);

    // Sanity check that the equivalent positive-magnitude shape does qualify,
    // so the divergence above is isolated to the sign, not the grouping.
    let positive_equivalent = qualify(59608, "a{image-resolution:snap 8dpcm from-image;}");
    assert_expected(
        &positive_equivalent,
        &[qualified(FromImageAndResolutionAndSnap)],
    );
}

#[test]
fn unitless_numbers_and_wrong_token_classes_are_invalid() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        59609,
        concat!(
            "a{image-resolution:0;}",
            "b{image-resolution:2;}",
            "c{image-resolution:100%;}",
            r#"d{image-resolution:"1dpi";}"#,
            "e{image-resolution:#foo;}",
            "f{image-resolution:1px;}",
            "g{image-resolution:1deg;}",
            "h{image-resolution:1s;}",
            "i{image-resolution:from-image, 1dpi;}",
        ),
    );

    let expected = vec![Invalid; 9];
    assert_expected(&result, &expected);
}

#[test]
fn residual_function_in_feasible_resolution_slot_is_unsupported() {
    use CssImageResolutionUnsupportedReason::FunctionValue;

    let result = qualify(
        59610,
        concat!(
            "a{image-resolution:calc(1dppx);}",
            "b{image-resolution:foo();}",
            "c{image-resolution:from-image calc(1dppx);}",
            "d{image-resolution:calc(1dppx) from-image;}",
            "e{image-resolution:calc(1dppx) snap;}",
            "f{image-resolution:snap calc(1dppx);}",
            "g{image-resolution:from-image calc(1dppx) snap;}",
            "h{image-resolution:snap from-image calc(1dppx);}",
            "i{image-resolution:from-image foo() snap;}",
        ),
    );

    let expected = vec![unsupported(FunctionValue); 9];
    assert_expected(&result, &expected);
}

#[test]
fn decisive_structural_invalidity_outranks_residual_function_unsupported() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        59611,
        concat!(
            "a{image-resolution:from-image snap calc(1dppx);}",
            "b{image-resolution:calc(1dppx) snap from-image;}",
            "c{image-resolution:calc(1dppx) 2dppx;}",
            "d{image-resolution:2dppx calc(1dppx);}",
            "e{image-resolution:calc(1dppx) calc(2dppx);}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn deferred_substitution_precedes_structural_recognition_everywhere() {
    use CssImageResolutionUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        59612,
        concat!(
            "a{image-resolution:var(--x);}",
            "b{image-resolution:from-image var(--x);}",
            "c{image-resolution:var(--x) snap;}",
            "d{image-resolution:from-image snap var(--x);}",
        ),
    );

    let expected = vec![unsupported(DeferredSubstitutionFunction); 4];
    assert_expected(&result, &expected);
}

#[test]
fn generic_whole_value_function_is_unsupported_only_at_whole_value_placement() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssImageResolutionUnsupportedReason::WholeValueFunction;

    let whole = qualify(59613, "a{image-resolution:first-valid(from-image, 1dpi);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let embedded = qualify(
        59614,
        concat!(
            "a{image-resolution:from-image first-valid(1dpi);}",
            "b{image-resolution:snap first-valid(1dpi);}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&embedded, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssImageResolutionUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        59615,
        concat!(
            "a{image-resolution:initial;}",
            "b{image-resolution:inherit;}",
            "c{image-resolution:unset;}",
            "d{image-resolution:revert;}",
            "e{image-resolution:revert-layer;}",
        ),
    );
    let expected = vec![unsupported(CssWideKeyword); 5];
    assert_expected(&whole, &expected);

    let embedded = qualify(
        59616,
        concat!(
            "a{image-resolution:from-image initial;}",
            "b{image-resolution:1dpi inherit;}",
            "c{image-resolution:snap revert;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&embedded, &expected);
}

// Pinned to current #596 authority only: CSS Images 4 has an open downstream
// design issue that could later add an `auto` branch if the initial value
// changes. That would be a future, separately bounded authority delta.
#[test]
fn auto_is_invalid_under_current_pinned_grammar() {
    use CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(59617, "a{image-resolution:auto;}");
    assert_expected(&result, &[Invalid]);
}

#[test]
fn important_priority_does_not_alter_value_qualification() {
    use CssImageResolutionValue::FromImageAndResolutionAndSnap;

    let result = qualify(
        59618,
        concat!(
            "a{image-resolution:from-image 300dpi snap;}",
            "b{image-resolution:from-image 300dpi snap !important;}",
        ),
    );

    let expected = vec![qualified(FromImageAndResolutionAndSnap); 2];
    assert_expected(&result, &expected);
    assert!(
        result.upstream_parser_result().occurrences()[1]
            .priority()
            .is_some()
    );
}

#[test]
fn duplicate_occurrences_preserve_authored_order_without_cascade_selection() {
    use CssImageResolutionValue::{DirectResolution, FromImage, FromImageAndSnap};

    let result = qualify(
        59619,
        concat!(
            "a{image-resolution:from-image;image-resolution:1dpi;image-resolution:from-image snap;}",
            "b{image-resolution:snap from-image;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(FromImage),
            qualified(DirectResolution),
            qualified(FromImageAndSnap),
            qualified(FromImageAndSnap),
        ],
    );
    assert_eq!(result.image_resolution_observations().len(), 4);
    assert_eq!(
        result.image_resolution_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.image_resolution_observations()[2].occurrence_index(),
        2
    );
}

#[test]
fn nonordinary_contexts_do_not_enter_image_resolution_dispatch() {
    for (source_id, css) in [
        (59620, "@font-face{image-resolution:1dpi;}"),
        (59621, "@page{image-resolution:1dpi;}"),
        (59622, "@keyframes k{from{image-resolution:1dpi;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.image_resolution_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssImageResolutionValue::FromImage;

    let incomplete = qualify_with_limits(
        59623,
        "a{image-resolution:from-image;}b{image-resolution:1dpi;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(FromImage)]);

    let css = concat!(
        "a{image-resolution:from-image snap;}",
        "b{image-resolution:var(--x);}",
        "c{image-resolution:-8dpcm;}",
    );
    let first = qualify(59624, css);
    let repeated = qualify(59624, css);
    let another_source = qualify(59625, css);

    assert_eq!(
        first.image_resolution_observations(),
        repeated.image_resolution_observations()
    );
    assert_eq!(
        first.image_resolution_observations(),
        another_source.image_resolution_observations()
    );
}

#[test]
fn existing_leaf_dispatch_remains_isolated_from_image_resolution() {
    use crate::css::value_qualification::{
        CssAspectRatioQualificationOutcome, CssAspectRatioValue,
    };

    let result = qualify(
        59626,
        concat!(
            "a{image-resolution:from-image;aspect-ratio:auto;}",
            "b{offset-rotate:45deg;}",
        ),
    );

    assert_eq!(result.image_resolution_observations().len(), 1);
    assert_eq!(result.aspect_ratio_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
    assert_eq!(
        result.aspect_ratio_observations()[0].outcome(),
        CssAspectRatioQualificationOutcome::Qualified(CssAspectRatioValue::Auto)
    );
}
