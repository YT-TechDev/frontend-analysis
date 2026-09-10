use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssLetterSpacingQualificationOutcome, CssLetterSpacingUnsupportedReason, CssLetterSpacingValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    DirectLength,
    DirectPercentage,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
    UnsupportedFunction,
}

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

fn expected_outcome(expected: ExpectedOutcome) -> CssLetterSpacingQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssLetterSpacingQualificationOutcome::Qualified(CssLetterSpacingValue::Normal)
        }
        ExpectedOutcome::DirectLength => CssLetterSpacingQualificationOutcome::Qualified(
            CssLetterSpacingValue::DirectLengthLiteral,
        ),
        ExpectedOutcome::DirectPercentage => CssLetterSpacingQualificationOutcome::Qualified(
            CssLetterSpacingValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssLetterSpacingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLetterSpacingUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLetterSpacingUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLetterSpacingUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLetterSpacingUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .letter_spacing_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_normal_unitless_zero_and_signed_percentage_boundaries_are_explicit() {
    let result = qualify(
        1600,
        concat!(
            "a{letter-spacing:normal;}",
            "b{letter-spacing:NORMAL;}",
            "c{letter-spacing:\\6eormal;}",
            "d{letter-spacing:0;}",
            "e{letter-spacing:+0;}",
            "f{letter-spacing:-0;}",
            "g{letter-spacing:.0;}",
            "h{letter-spacing:-.0;}",
            "i{letter-spacing:0.0;}",
            "j{letter-spacing:-0.0;}",
            "k{letter-spacing:0e100;}",
            "l{letter-spacing:-0e100;}",
            "m{letter-spacing:1;}",
            "n{letter-spacing:-1;}",
            "o{letter-spacing:.5;}",
            "p{letter-spacing:37.5%;}",
            "q{letter-spacing:0%;}",
            "r{letter-spacing:+0%;}",
            "s{letter-spacing:-0%;}",
            "t{letter-spacing:.0%;}",
            "u{letter-spacing:-.0%;}",
            "v{letter-spacing:0e100%;}",
            "w{letter-spacing:-0e100%;}",
            "x{letter-spacing:100%;}",
            "y{letter-spacing:1e100%;}",
            "z{letter-spacing:-1%;}",
            "aa{letter-spacing:-.5%;}",
            "ab{letter-spacing:-1e-999%;}",
            "ac{letter-spacing:auto;}",
            "ad{letter-spacing:none;}",
            "ae{letter-spacing:\"1px\";}",
            "af{letter-spacing:;}",
            "ag{color:1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Normal,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn handwritten_current_css_length_unit_inventory_is_qualified_case_insensitively() {
    let units = [
        "cm", "mm", "q", "in", "pt", "pc", "px", "em", "rem", "ex", "rex", "cap", "rcap", "ch",
        "rch", "ic", "ric", "lh", "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh",
        "svi", "svb", "svmin", "svmax", "lvw", "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh",
        "dvi", "dvb", "dvmin", "dvmax", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ];

    let mut css = String::new();
    for (index, unit) in units.iter().enumerate() {
        css.push_str(&format!(".u{index}{{letter-spacing:1{unit};}}"));
    }
    css.push_str(".upper{letter-spacing:1Q;}.escaped{letter-spacing:1p\\78;}");

    let result = qualify(1601, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn signed_dimension_values_are_qualified_without_range_ordering() {
    let result = qualify(
        1602,
        concat!(
            "a{letter-spacing:0px;}",
            "b{letter-spacing:+0px;}",
            "c{letter-spacing:-0px;}",
            "d{letter-spacing:-0e100px;}",
            "e{letter-spacing:1px;}",
            "f{letter-spacing:.5em;}",
            "g{letter-spacing:1e100cqi;}",
            "h{letter-spacing:-1px;}",
            "i{letter-spacing:-.5em;}",
            "j{letter-spacing:-1e-999px;}",
            "k{letter-spacing:1deg;}",
            "l{letter-spacing:-1s;}",
            "m{letter-spacing:0fr;}",
            "n{letter-spacing:-1foo;}",
            "o{letter-spacing:0deg;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn branch_identity_distinguishes_zero_zero_px_and_zero_percent() {
    let result = qualify(
        1603,
        concat!(
            "a{letter-spacing:0;}",
            "b{letter-spacing:0px;}",
            "c{letter-spacing:0%;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_ne!(
        result.letter_spacing_observations()[0].outcome(),
        result.letter_spacing_observations()[2].outcome()
    );
    assert_eq!(
        result.letter_spacing_observations()[0].outcome(),
        result.letter_spacing_observations()[1].outcome()
    );
}

#[test]
fn comments_priority_separated_signs_and_cardinality_preserve_token_boundaries() {
    let result = qualify(
        1604,
        concat!(
            "a{letter-spacing:/**/normal/**/!important;}",
            "b{letter-spacing:/**/-20px/**/!important;}",
            "c{letter-spacing:/**/-10%/**/!important;}",
            "d{letter-spacing:+ 0;}",
            "e{letter-spacing:+/**/0;}",
            "f{letter-spacing:- 1px;}",
            "g{letter-spacing:-/**/1px;}",
            "h{letter-spacing:1px 2px;}",
            "i{letter-spacing:10% 10px;}",
            "j{letter-spacing:normal 10px;}",
            "k{letter-spacing:10px normal;}",
            "l{letter-spacing:10% 20%;}",
            "m{letter-spacing:(1px);}",
            "n{letter-spacing:10px,;}",
            "o{letter-spacing:10px, 20px;}",
            "p{letter-spacing:normal, 10px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    for index in 0..3 {
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some()
        );
    }
}

#[test]
fn sole_functions_are_unsupported_without_length_percentage_evaluation() {
    let result = qualify(
        1605,
        concat!(
            "a{letter-spacing:calc(2em + 3ex);}",
            "b{letter-spacing:calc(2ch - 30%);}",
            "c{letter-spacing:calc(40% + 50px);}",
            "d{letter-spacing:min(-1px,20%);}",
            "e{letter-spacing:max(-10%,2px);}",
            "f{letter-spacing:clamp(-2em,10%,3px);}",
            "g{letter-spacing:foo();}",
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);

    let mixed = qualify(
        1606,
        concat!(
            "a{letter-spacing:calc(1px) 2px;}",
            "b{letter-spacing:normal foo();}",
            "c{letter-spacing:foo() -10%;}",
            "d{letter-spacing:10px foo();}",
            "e{letter-spacing:foo() 10%;}",
            "f{letter-spacing:normal calc(1px);}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 6]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1607,
        concat!(
            "a{letter-spacing:initial;}",
            "b{letter-spacing:inherit;}",
            "c{letter-spacing:unset;}",
            "d{letter-spacing:revert;}",
            "e{letter-spacing:revert-layer;}",
            "f{letter-spacing:revert-rule;}",
            "g{letter-spacing:-1px initial;}",
            "h{letter-spacing:initial normal;}",
            "i{letter-spacing:1px inherit;}",
            "j{letter-spacing:initial 10%;}",
        ),
    );
    assert_expected(
        &css_wide,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );

    let deferred = qualify(
        1608,
        concat!(
            "a{letter-spacing:var(--s);}",
            "b{letter-spacing:env(s);}",
            "c{letter-spacing:attr(data-s);}",
            "d{letter-spacing:--s();}",
            "e{letter-spacing:-1px var(--s);}",
            "f{letter-spacing:normal var(--s);}",
            "g{letter-spacing:calc(var(--s));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1609,
        concat!(
            "a{letter-spacing:first-valid(-1px,-10%);}",
            "b{letter-spacing:cycle(-1px,2px);}",
            "c{letter-spacing:interpolate(1,0:-1px,1:2px);}",
            "d{letter-spacing:first-valid(-1px,-10%) 2px;}",
            "e{letter-spacing:-1px first-valid(-10%,1px);}",
            "f{letter-spacing:normal first-valid(1px);}",
            "g{letter-spacing:1px first-valid(2px);}",
        ),
    );
    assert_expected(
        &whole,
        &[
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn one_run_owns_upstream_evidence_for_every_selected_value_leaf() {
    let result = qualify(
        1610,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{z-index:auto;}",
            "g{column-count:2;}",
            "h{flex-grow:.5;}",
            "i{flex-shrink:.5;}",
            "j{opacity:120%;}",
            "k{shape-image-threshold:-50%;}",
            "l{perspective:1px;}",
            "m{border-top-width:thick;}",
            "n{shape-margin:37.5%;}",
            "o{line-height:1.2;}",
            "p{word-spacing:-10%;}",
            "q{text-underline-offset:auto;}",
            "r{text-indent:10px;}",
            "s{letter-spacing:-10%;}",
            "t{direction:rtl;}",
        ),
    );

    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.text_underline_offset_observations().len(), 1);
    assert_eq!(result.text_indent_observations().len(), 1);
    assert_eq!(result.letter_spacing_observations().len(), 1);

    assert_eq!(
        result.letter_spacing_observations()[0].occurrence_index(),
        18
    );
    assert_eq!(result.direction_observations()[1].occurrence_index(), 19);
    assert_eq!(
        result.letter_spacing_observations()[0].outcome(),
        CssLetterSpacingQualificationOutcome::Qualified(
            CssLetterSpacingValue::DirectPercentageLiteral
        )
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1611, "a{letter-spacing:normal;}b{letter-spacing:normal;}");
    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(
        result.letter_spacing_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.letter_spacing_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.letter_spacing_observations()[0]
            .placement()
            .context_id(),
        result.letter_spacing_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1612, "@font-face{letter-spacing:-1px;}"),
        (1613, "@page{letter-spacing:-1px;}"),
        (1614, "@page{@top-left{letter-spacing:-1px;}}"),
        (1615, "@keyframes k{from{letter-spacing:-1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.letter_spacing_observations().is_empty(),
            "nonordinary declaration context produced a letter-spacing observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1616,
        "a{letter-spacing:-1px;letter-spacing:-10%;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectLength]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{letter-spacing:normal;}",
        "b{letter-spacing:-1px;}",
        "c{letter-spacing:-10%;}",
        "d{letter-spacing:calc(2em + 3ex);}",
        "e{letter-spacing:var(--s);}",
        "f{word-spacing:1px;}",
        "g{text-underline-offset:auto;}",
        "h{text-indent:1px;}",
        "i{line-height:1.2;}",
        "j{shape-margin:1px;}",
        "k{perspective:1px;}",
        "l{opacity:-50%;}",
        "m{flex-grow:.5;}",
        "n{direction:ltr;}",
        "o{column-count:2;}",
        "p{z-index:auto;}",
        "q{border-top-width:thin;}",
    );
    let first = qualify(1617, css);
    let repeated = qualify(1617, css);
    let another_source = qualify(1618, css);

    assert_eq!(
        first.letter_spacing_observations(),
        repeated.letter_spacing_observations()
    );
    assert_eq!(
        first.letter_spacing_observations(),
        another_source.letter_spacing_observations()
    );
    assert_eq!(
        first.word_spacing_observations(),
        repeated.word_spacing_observations()
    );
    assert_eq!(
        first.text_underline_offset_observations(),
        repeated.text_underline_offset_observations()
    );
    assert_eq!(
        first.text_indent_observations(),
        repeated.text_indent_observations()
    );
    assert_eq!(
        first.line_height_observations(),
        repeated.line_height_observations()
    );
    assert_eq!(
        first.shape_margin_observations(),
        repeated.shape_margin_observations()
    );
    assert_eq!(
        first.perspective_observations(),
        repeated.perspective_observations()
    );
    assert_eq!(
        first.opacity_observations(),
        repeated.opacity_observations()
    );
    assert_eq!(
        first.flex_grow_observations(),
        repeated.flex_grow_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(
        first.column_count_observations(),
        repeated.column_count_observations()
    );
    assert_eq!(
        first.z_index_observations(),
        repeated.z_index_observations()
    );
    assert_eq!(
        first.border_top_width_observations(),
        repeated.border_top_width_observations()
    );
}
