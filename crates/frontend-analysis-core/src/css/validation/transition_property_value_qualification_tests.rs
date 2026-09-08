use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransitionPropertyItemValue, CssTransitionPropertyQualificationObservation,
    CssTransitionPropertyQualificationOutcome, CssTransitionPropertyUnsupportedReason,
    CssTransitionPropertyValue, CssValueQualificationRunResult, run,
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

fn qualified_none() -> CssTransitionPropertyQualificationOutcome {
    CssTransitionPropertyQualificationOutcome::Qualified(CssTransitionPropertyValue::None)
}

fn qualified_items(
    kinds: &[CssTransitionPropertyItemValue],
) -> CssTransitionPropertyQualificationOutcome {
    CssTransitionPropertyQualificationOutcome::Qualified(CssTransitionPropertyValue::Items(
        kinds.to_vec(),
    ))
}

fn unsupported(
    reason: CssTransitionPropertyUnsupportedReason,
) -> CssTransitionPropertyQualificationOutcome {
    CssTransitionPropertyQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssTransitionPropertyQualificationOutcome],
) {
    let actual: Vec<_> = result
        .transition_property_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

/// Resolves one observation's ordered custom-ident items to their
/// tokenizer-owned decoded identity, `None` at each `All` position.
fn custom_ident_values<'a>(
    result: &'a CssValueQualificationRunResult,
    observation: &CssTransitionPropertyQualificationObservation,
) -> Vec<Option<&'a str>> {
    observation
        .custom_ident_evidence()
        .iter()
        .map(|evidence| {
            evidence.and_then(|evidence| result.transition_property_custom_ident_value(evidence))
        })
        .collect()
}

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_creates_no_item_evidence() {
    let result = qualify(
        97700,
        concat!(
            "a{transition-property:none;}",
            "b{transition-property:NONE;}",
            "c{transition-property:NoNe;}",
            "d{transition-property:\\6e one;}",
        ),
    );

    assert_expected(&result, &vec![qualified_none(); 4]);
    for observation in result.transition_property_observations() {
        assert!(observation.custom_ident_evidence().is_empty());
    }
}

#[test]
fn predefined_all_item_is_ascii_case_insensitive_and_ordered_with_custom_ident() {
    use CssTransitionPropertyItemValue::{All, CustomIdent};

    let result = qualify(
        97701,
        concat!(
            "a{transition-property:all;}",
            "b{transition-property:ALL;}",
            "c{transition-property:\\61 ll;}",
            "d{transition-property:width,all;}",
            "e{transition-property:all,width;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[All]),
            qualified_items(&[All]),
            qualified_items(&[All]),
            qualified_items(&[CustomIdent, All]),
            qualified_items(&[All, CustomIdent]),
        ],
    );
    for observation in &result.transition_property_observations()[0..3] {
        assert_eq!(observation.custom_ident_evidence(), [None]);
    }
}

#[test]
fn unknown_identifiers_qualify_without_property_registry_lookup() {
    use CssTransitionPropertyItemValue::{All, CustomIdent};

    let result = qualify(
        97702,
        concat!(
            "a{transition-property:made-up-property;}",
            "b{transition-property:totally-unknown;}",
            "c{transition-property:INVALID,SYNTAX,SRC;}",
            "d{transition-property:ALL,INVALID,SYNTAX,SRC;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent, CustomIdent, CustomIdent]),
            qualified_items(&[All, CustomIdent, CustomIdent, CustomIdent]),
        ],
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[2]),
        [Some("INVALID"), Some("SYNTAX"), Some("SRC")]
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[3]),
        [None, Some("INVALID"), Some("SYNTAX"), Some("SRC")]
    );
}

#[test]
fn custom_ident_identity_is_case_sensitive() {
    use CssTransitionPropertyItemValue::CustomIdent;

    let result = qualify(
        97703,
        concat!(
            "a{transition-property:Foo;}",
            "b{transition-property:foo;}",
            "c{transition-property:FOO;}",
            "d{transition-property:Foo,foo,Foo;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent, CustomIdent, CustomIdent]),
        ],
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[0]),
        [Some("Foo")]
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[1]),
        [Some("foo")]
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[2]),
        [Some("FOO")]
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[3]),
        [Some("Foo"), Some("foo"), Some("Foo")]
    );
}

#[test]
fn escape_equivalent_custom_ident_spellings_share_interpreted_identity() {
    use CssTransitionPropertyItemValue::CustomIdent;

    let result = qualify(
        97704,
        concat!(
            "a{transition-property:Foo;}",
            "b{transition-property:\\46 oo;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent]),
        ],
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[0]),
        [Some("Foo")]
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[1]),
        [Some("Foo")]
    );
}

#[test]
fn ordered_evidence_refs_point_to_the_exact_retained_ident_tokens() {
    use CssTransitionPropertyItemValue::{All, CustomIdent};

    let result = qualify(97705, "a{transition-property:Foo,all,bar,Foo;}");

    assert_expected(
        &result,
        &[qualified_items(&[
            CustomIdent,
            All,
            CustomIdent,
            CustomIdent,
        ])],
    );

    let observation = &result.transition_property_observations()[0];
    assert_eq!(
        custom_ident_values(&result, observation),
        [Some("Foo"), None, Some("bar"), Some("Foo")]
    );

    let evidence = observation.custom_ident_evidence();
    assert_eq!(evidence.len(), 4);
    assert!(evidence[1].is_none());
    let indices: Vec<_> = [0usize, 2, 3]
        .iter()
        .map(|&position| evidence[position].unwrap().lexical_item_index())
        .collect();
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn trivia_around_separators_does_not_shift_custom_ident_evidence() {
    use CssTransitionPropertyItemValue::{All, CustomIdent};

    let result = qualify(
        97706,
        concat!(
            "a{transition-property:width,opacity;}",
            "b{transition-property: width , opacity ;}",
            "c{transition-property:width/*a*/,/*b*/opacity;}",
            "d{transition-property:/*x*/width/*y*/,/*z*/all/*w*/;}",
            "e{transition-property:/*a*/ Foo /*b*/,/*c*/ all /*d*/,/*e*/ bar;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[CustomIdent, CustomIdent]),
            qualified_items(&[CustomIdent, CustomIdent]),
            qualified_items(&[CustomIdent, CustomIdent]),
            qualified_items(&[CustomIdent, All]),
            qualified_items(&[CustomIdent, All, CustomIdent]),
        ],
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[0]),
        [Some("width"), Some("opacity")]
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[4]),
        [Some("Foo"), None, Some("bar")]
    );
}

#[test]
fn duplicate_items_preserve_multiplicity_and_authored_order() {
    use CssTransitionPropertyItemValue::{All, CustomIdent};

    let result = qualify(
        97707,
        concat!(
            "a{transition-property:width,width;}",
            "b{transition-property:all,all;}",
            "c{transition-property:Foo,foo,Foo;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[CustomIdent, CustomIdent]),
            qualified_items(&[All, All]),
            qualified_items(&[CustomIdent, CustomIdent, CustomIdent]),
        ],
    );
    assert_eq!(
        custom_ident_values(&result, &result.transition_property_observations()[2]),
        [Some("Foo"), Some("foo"), Some("Foo")]
    );
}

#[test]
fn none_is_invalid_in_list_item_position() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97708,
        concat!(
            "a{transition-property:none,width;}",
            "b{transition-property:width,none;}",
            "c{transition-property:none,all;}",
            "d{transition-property:all,none;}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 4]);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value_and_default_is_always_invalid() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssTransitionPropertyUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        97709,
        concat!(
            "a{transition-property:initial;}",
            "b{transition-property:inherit;}",
            "c{transition-property:unset;}",
            "d{transition-property:revert;}",
            "e{transition-property:revert-layer;}",
        ),
    );
    assert_expected(&whole, &vec![unsupported(CssWideKeyword); 5]);

    let list_placement = qualify(
        97710,
        concat!(
            "a{transition-property:initial,top;}",
            "b{transition-property:top,initial;}",
            "c{transition-property:inherit,top;}",
            "d{transition-property:top,inherit;}",
            "e{transition-property:unset,top;}",
            "f{transition-property:top,unset;}",
            "g{transition-property:revert,top;}",
            "h{transition-property:top,revert;}",
            "i{transition-property:revert-layer,top;}",
            "j{transition-property:top,revert-layer;}",
        ),
    );
    assert_expected(&list_placement, &vec![Invalid; 10]);

    let default_boundary = qualify(
        97711,
        concat!(
            "a{transition-property:default;}",
            "b{transition-property:default,top;}",
            "c{transition-property:top,default;}",
        ),
    );
    assert_expected(&default_boundary, &vec![Invalid; 3]);
}

#[test]
fn ordinary_function_items_are_invalid_never_softened_to_unsupported() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97712,
        concat!(
            "a{transition-property:foo();}",
            "b{transition-property:calc(1);}",
            "c{transition-property:width,foo();}",
            "d{transition-property:foo(),width;}",
            "e{transition-property:width,calc(1);}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 5]);
}

#[test]
fn whole_value_generic_function_boundary_does_not_legalize_function_items() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssTransitionPropertyUnsupportedReason::WholeValueFunction;

    let whole = qualify(97713, "a{transition-property:first-valid(width,opacity);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        97714,
        concat!(
            "a{transition-property:width,first-valid(opacity);}",
            "b{transition-property:first-valid(width),opacity;}",
        ),
    );
    assert_expected(&items, &vec![Invalid; 2]);
}

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssTransitionPropertyUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        97715,
        concat!(
            "a{transition-property:var(--x);}",
            "b{transition-property:width,var(--x);}",
            "c{transition-property:var(--x),width;}",
            "d{transition-property:var(--x,width,opacity);}",
        ),
    );

    assert_expected(&result, &vec![unsupported(DeferredSubstitutionFunction); 4]);
}

#[test]
fn separator_and_missing_comma_cases_are_invalid() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97716,
        concat!(
            "a{transition-property:;}",
            "b{transition-property:,width;}",
            "c{transition-property:width,;}",
            "d{transition-property:width,,opacity;}",
            "e{transition-property:one two three;}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 5]);
}

#[test]
fn wrong_token_classes_are_invalid() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97717,
        concat!(
            "a{transition-property:1;}",
            "b{transition-property:1px;}",
            "c{transition-property:50%;}",
            r#"d{transition-property:"width";}"#,
        ),
    );

    assert_expected(&result, &vec![Invalid; 4]);
}

#[test]
fn important_priority_is_outside_the_semantic_list_window() {
    use CssTransitionPropertyItemValue::CustomIdent;

    let result = qualify(97718, "a{transition-property:width,opacity !important;}");
    assert_expected(&result, &[qualified_items(&[CustomIdent, CustomIdent])]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    use CssTransitionPropertyItemValue::CustomIdent;

    let result = qualify(
        97719,
        concat!(
            "a{transition-property:width;transition-property:opacity,all;}",
            "b{page:Foo;}",
            "c{transition-duration:1s;}",
            "d{animation-delay:1s;}",
        ),
    );

    assert_eq!(result.transition_property_observations().len(), 2);
    assert_eq!(
        result.transition_property_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.transition_property_observations()[1].occurrence_index(),
        1
    );
    assert_expected(
        &result,
        &[
            qualified_items(&[CustomIdent]),
            qualified_items(&[CustomIdent, CssTransitionPropertyItemValue::All]),
        ],
    );
    assert_eq!(result.page_observations().len(), 1);
    assert_eq!(result.transition_duration_observations().len(), 1);
    assert_eq!(result.animation_delay_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_transition_property_dispatch() {
    for (source_id, css) in [
        (97720, "@font-face{transition-property:width;}"),
        (97721, "@page{transition-property:width;}"),
        (97722, "@keyframes k{from{transition-property:width;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transition_property_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97723,
        concat!(
            "a{transition-property:foo(bar,baz),width;}",
            "b{transition-property:(width,opacity),top;}",
            "c{transition-property:width),opacity;}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 3]);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssTransitionPropertyItemValue::CustomIdent;

    let incomplete = qualify_with_limits(
        97724,
        "a{transition-property:width;}b{transition-property:width,opacity;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified_items(&[CustomIdent])]);

    let css = concat!(
        "a{transition-property:width,opacity;}",
        "b{transition-property:none;}",
        "c{transition-property:var(--x);}",
        "d{transition-property:none,width;}",
    );
    let first = qualify(97725, css);
    let repeated = qualify(97725, css);
    let another_source = qualify(97726, css);

    assert_eq!(
        first.transition_property_observations(),
        repeated.transition_property_observations()
    );
    assert_eq!(
        first.transition_property_observations(),
        another_source.transition_property_observations()
    );
}
