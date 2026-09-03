use crate::{SourceId, SourceText};

use super::super::parser;
use super::super::tokenizer;
use super::super::value_qualification::{
    self, CssFontSynthesisSmallCapsQualificationOutcome,
    CssFontSynthesisSmallCapsUnsupportedReason, CssFontSynthesisSmallCapsValue,
};
use super::super::{CssAnalysisBudget, CssDeclarationPlacement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Qualified(CssFontSynthesisSmallCapsValue),
    Invalid,
    Unsupported(CssFontSynthesisSmallCapsUnsupportedReason),
}

#[derive(Debug, Clone, Copy)]
struct Case {
    source: &'static str,
    expected: ExpectedOutcome,
}

fn run(source: &'static str, source_id: SourceId) -> value_qualification::CssValueQualificationRunResult {
    let source = SourceText::new(source_id, source);
    let tokenizer_result = tokenizer::run(&source, CssAnalysisBudget::default())
        .expect("tokenizer should succeed");
    let parser_result = parser::run(tokenizer_result, CssAnalysisBudget::default())
        .expect("parser should succeed");
    value_qualification::run(parser_result).expect("qualification should succeed")
}

fn assert_outcome(source: &'static str, expected: ExpectedOutcome) {
    let result = run(source, SourceId::new(1));
    let observations = result.font_synthesis_small_caps_observations();
    assert_eq!(observations.len(), 1, "{source}");
    let actual = observations[0].outcome();
    match expected {
        ExpectedOutcome::Qualified(value) => {
            assert_eq!(
                actual,
                CssFontSynthesisSmallCapsQualificationOutcome::Qualified(value),
                "{source}"
            );
        }
        ExpectedOutcome::Invalid => {
            assert_eq!(
                actual,
                CssFontSynthesisSmallCapsQualificationOutcome::InvalidForSelectedValueGrammar,
                "{source}"
            );
        }
        ExpectedOutcome::Unsupported(reason) => {
            assert_eq!(
                actual,
                CssFontSynthesisSmallCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                    reason
                ),
                "{source}"
            );
        }
    }
}

#[test]
fn qualifies_direct_keywords_case_and_escapes() {
    for case in [
        Case {
            source: "a { font-synthesis-small-caps: auto; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::Auto),
        },
        Case {
            source: "a { font-synthesis-small-caps: none; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::None),
        },
        Case {
            source: "a { FONT-SYNTHESIS-SMALL-CAPS: AuTo; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::Auto),
        },
        Case {
            source: r"a { font-synthesis-small-caps: n\6f ne; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::None),
        },
        Case {
            source: r"a { font-synthesis-small-caps: \61 uto; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::Auto),
        },
    ] {
        assert_outcome(case.source, case.expected);
    }
}

#[test]
fn invalidates_direct_grammar_mismatches() {
    for source in [
        "a { font-synthesis-small-caps: normal; }",
        "a { font-synthesis-small-caps: auto none; }",
        "a { font-synthesis-small-caps: none auto; }",
        "a { font-synthesis-small-caps: 1; }",
        "a { font-synthesis-small-caps: 1px; }",
        "a { font-synthesis-small-caps: \"auto\"; }",
        "a { font-synthesis-small-caps: ; }",
        "a { font-synthesis-small-caps: foo(); }",
        "a { font-synthesis-small-caps: calc(1); }",
    ] {
        assert_outcome(source, ExpectedOutcome::Invalid);
    }
}

#[test]
fn comments_and_priority_do_not_change_value_grammar() {
    for case in [
        Case {
            source: "a { font-synthesis-small-caps: /*x*/ auto /*y*/; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::Auto),
        },
        Case {
            source: "a { font-synthesis-small-caps: none !important; }",
            expected: ExpectedOutcome::Qualified(CssFontSynthesisSmallCapsValue::None),
        },
    ] {
        assert_outcome(case.source, case.expected);
    }
}

#[test]
fn css_wide_keywords_are_profile_unsupported() {
    for keyword in [
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "revert-rule",
    ] {
        let source = Box::leak(
            format!("a {{ font-synthesis-small-caps: {keyword}; }}").into_boxed_str(),
        );
        assert_outcome(
            source,
            ExpectedOutcome::Unsupported(CssFontSynthesisSmallCapsUnsupportedReason::CssWideKeyword),
        );
    }
}

#[test]
fn deferred_and_entire_whole_value_functions_are_profile_unsupported() {
    for source in [
        "a { font-synthesis-small-caps: var(--x); }",
        "a { font-synthesis-small-caps: env(x); }",
        "a { font-synthesis-small-caps: attr(data-x); }",
        "a { font-synthesis-small-caps: if(style(--x): auto; else: none); }",
        "a { font-synthesis-small-caps: inherit(auto); }",
        "a { font-synthesis-small-caps: ident(auto); }",
        "a { font-synthesis-small-caps: random-item(auto, none); }",
        "a { font-synthesis-small-caps: --custom(auto); }",
        "a { font-synthesis-small-caps: first-valid(auto, none); }",
        "a { font-synthesis-small-caps: cycle(auto, none); }",
        "a { font-synthesis-small-caps: interpolate(auto, none); }",
    ] {
        assert_outcome(
            source,
            ExpectedOutcome::Unsupported(
                CssFontSynthesisSmallCapsUnsupportedReason::FunctionValue,
            ),
        );
    }
}

#[test]
fn function_placement_boundaries_remain_distinct() {
    for source in [
        "a { font-synthesis-small-caps: auto first-valid(auto, none); }",
        "a { font-synthesis-small-caps: first-valid(auto, none) auto; }",
        "a { font-synthesis-small-caps: auto cycle(auto, none); }",
        "a { font-synthesis-small-caps: interpolate(auto, none) none; }",
    ] {
        assert_outcome(source, ExpectedOutcome::Invalid);
    }

    for source in [
        "a { font-synthesis-small-caps: auto var(--x); }",
        "a { font-synthesis-small-caps: var(--x) none; }",
        "a { font-synthesis-small-caps: auto --custom(none); }",
    ] {
        assert_outcome(
            source,
            ExpectedOutcome::Unsupported(
                CssFontSynthesisSmallCapsUnsupportedReason::FunctionValue,
            ),
        );
    }
}

#[test]
fn interleaves_with_all_accepted_value_qualification_leaves() {
    let source = r#"
a {
  direction: rtl;
  box-sizing: border-box;
  isolation: isolate;
  backface-visibility: hidden;
  order: 2;
  column-count: 3;
  flex-grow: 1;
  flex-shrink: 1;
  opacity: 0.5;
  shape-image-threshold: 50%;
  shape-margin: 1px;
  line-height: normal;
  word-spacing: 1px;
  text-underline-offset: auto;
  scroll-margin-top: 1px;
  border-top-width: thin;
  perspective: none;
  z-index: auto;
  scroll-snap-align: center;
  scroll-snap-stop: always;
  empty-cells: hide;
  text-decoration-style: wavy;
  table-layout: fixed;
  border-collapse: collapse;
  box-decoration-break: clone;
  font-kerning: normal;
  font-variant-position: super;
  font-synthesis-weight: none;
  font-synthesis-small-caps: auto;
}
"#;
    let result = run(source, SourceId::new(2));

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.backface_visibility_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.column_count_observations().len(), 1);
    assert_eq!(result.flex_grow_observations().len(), 1);
    assert_eq!(result.flex_shrink_observations().len(), 1);
    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.shape_image_threshold_observations().len(), 1);
    assert_eq!(result.shape_margin_observations().len(), 1);
    assert_eq!(result.line_height_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.text_underline_offset_observations().len(), 1);
    assert_eq!(result.scroll_margin_top_observations().len(), 1);
    assert_eq!(result.border_top_width_observations().len(), 1);
    assert_eq!(result.perspective_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.scroll_snap_stop_observations().len(), 1);
    assert_eq!(result.empty_cells_observations().len(), 1);
    assert_eq!(result.text_decoration_style_observations().len(), 1);
    assert_eq!(result.table_layout_observations().len(), 1);
    assert_eq!(result.border_collapse_observations().len(), 1);
    assert_eq!(result.box_decoration_break_observations().len(), 1);
    assert_eq!(result.font_kerning_observations().len(), 1);
    assert_eq!(result.font_variant_position_observations().len(), 1);
    assert_eq!(result.font_synthesis_weight_observations().len(), 1);

    let observation = result.font_synthesis_small_caps_observations()[0];
    assert_eq!(observation.occurrence_index(), 28);
    assert_eq!(
        observation.placement(),
        CssDeclarationPlacement::OrdinaryDeclaration
    );
    assert_eq!(
        observation.outcome(),
        CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
            CssFontSynthesisSmallCapsValue::Auto
        )
    );
}

#[test]
fn duplicate_occurrences_preserve_run_local_identity() {
    let source = "a { font-synthesis-small-caps: auto; font-synthesis-small-caps: none; }";
    let result = run(source, SourceId::new(3));
    let observations = result.font_synthesis_small_caps_observations();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].occurrence_index(), 0);
    assert_eq!(observations[1].occurrence_index(), 1);
    assert_eq!(
        observations[0].outcome(),
        CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
            CssFontSynthesisSmallCapsValue::Auto
        )
    );
    assert_eq!(
        observations[1].outcome(),
        CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
            CssFontSynthesisSmallCapsValue::None
        )
    );
}

#[test]
fn excludes_nonordinary_declaration_shaped_contexts() {
    for source in [
        "@font-face { font-synthesis-small-caps: auto; }",
        "@page { font-synthesis-small-caps: auto; }",
        "@keyframes x { from { font-synthesis-small-caps: auto; } }",
    ] {
        let result = run(source, SourceId::new(4));
        assert!(result.font_synthesis_small_caps_observations().is_empty());
    }
}

#[test]
fn preserves_upstream_incomplete_committed_prefix() {
    let source = SourceText::new(
        SourceId::new(5),
        "a { font-synthesis-small-caps: auto; font-synthesis-small-caps: none; }",
    );
    let tokenizer_result = tokenizer::run(
        &source,
        CssAnalysisBudget {
            max_lexical_items: 12,
            ..CssAnalysisBudget::default()
        },
    )
    .expect("tokenizer should return committed prefix");
    let parser_result = parser::run(tokenizer_result, CssAnalysisBudget::default())
        .expect("parser should preserve committed prefix");
    assert!(matches!(
        parser_result.execution_completion(),
        super::super::parser::result::CssParserExecutionCompletion::Incomplete(_)
    ));

    let result = value_qualification::run(parser_result).expect("qualification should succeed");
    assert!(matches!(
        result.execution_completion(),
        super::super::parser::result::CssParserExecutionCompletion::Incomplete(_)
    ));
    assert!(result.font_synthesis_small_caps_observations().len() <= 1);
}

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let source = "a { font-synthesis-small-caps: none; }";
    let first = run(source, SourceId::new(6));
    let second = run(source, SourceId::new(6));
    let other_source = run(source, SourceId::new(7));

    assert_eq!(
        first.font_synthesis_small_caps_observations(),
        second.font_synthesis_small_caps_observations()
    );
    assert_eq!(
        first.font_synthesis_small_caps_observations(),
        other_source.font_synthesis_small_caps_observations()
    );
}
