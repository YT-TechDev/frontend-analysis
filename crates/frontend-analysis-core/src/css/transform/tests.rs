//! Focused tests for the public selected direct-authored `transform`
//! qualification facade (#914).
//!
//! These prove the new projection only; the CSS tokenizer, parser, and
//! authored-value/transform suites remain the authority for grammar behavior.
//! Every expected range and fragment below is hand-counted from the fixture
//! text (CSS Syntax boundary-trivia conventions follow the accepted parser
//! gold fixtures), never derived from facade output or a production range
//! helper.

use super::*;
use crate::css::analysis::analyze_css_source;
use crate::css::selectors::{
    CssParserCoverage, CssParserRecoveryKind, CssParserResourceKind, CssParserTermination,
    CssParserUnsupportedRegionKind, CssResourceKind, CssSelectorOutcome, CssStageCompletion,
    CssTokenizerResourceKind, CssTokenizerTermination, analyze_core_v1,
};
use crate::{SourceId, SourceText};

fn analyze(text: &str) -> CssTransformReport {
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    analyze_authored_transforms(&source).expect("ordinary analysis must not be a Core failure")
}

/// `(start, end, fragment)` of one retained anchor.
fn at(anchor: &SourceAnchor) -> (usize, usize, &str) {
    (
        anchor.range().start(),
        anchor.range().end(),
        anchor.fragment(),
    )
}

fn assert_fully_complete(report: &CssTransformReport) {
    assert_eq!(
        report.tokenizer().completion(),
        CssStageCompletion::Complete
    );
    assert!(matches!(
        report.tokenizer().termination(),
        CssTokenizerTermination::EndOfInput
    ));
    assert_eq!(report.parser().completion(), CssStageCompletion::Complete);
    assert!(matches!(
        report.parser().termination(),
        CssParserTermination::EndOfTokenizerInput
    ));
}

fn limits(
    source_bytes: usize,
    declaration_occurrences: usize,
) -> (
    tokenizer_resource::CssTokenizerLimits,
    parser_resource::CssParserLimits,
) {
    (
        tokenizer_resource::CssTokenizerLimits::new(
            source_bytes,
            1_000_000,
            8192,
            1024,
            65_536,
            65_536,
        )
        .unwrap(),
        parser_resource::CssParserLimits::new(
            1_000_000,
            256,
            256,
            declaration_occurrences,
            1024,
            1024,
            1024,
            1024,
            8192,
        )
        .unwrap(),
    )
}

fn analyze_limited(
    text: &str,
    source_bytes: usize,
    declaration_occurrences: usize,
) -> CssTransformReport {
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    let (tokenizer, parser) = limits(source_bytes, declaration_occurrences);
    analyze_with_limits(&source, tokenizer, parser).expect("refusal is a report, not a failure")
}

#[test]
fn qualified_observation_projects_exact_identity_profile_and_evidence() {
    // ".a{ transform: scale(1, 200%); }": header 0..2, `transform` 4..13,
    // value 15..29, `;` at 29, declaration 4..30.
    let source = SourceText::new(
        SourceId::new(7),
        ".a{ transform: scale(1, 200%); }".to_owned(),
    );
    let report = analyze_authored_transforms(&source).unwrap();

    assert_eq!(report.source_id(), SourceId::new(7));
    assert_eq!(
        report.profile(),
        CssTransformProfile::SelectedDirectAuthoredTransform
    );
    assert_fully_complete(&report);
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::SupportedForSelectedQuestion
    );
    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(observation.outcome(), CssTransformOutcome::Qualified);
    assert_eq!(
        at(observation.declaration()),
        (4, 30, "transform: scale(1, 200%);")
    );
    assert_eq!(at(observation.property()), (4, 13, "transform"));
    assert_eq!(at(observation.value()), (15, 29, "scale(1, 200%)"));
    assert!(observation.priority().is_none());
    assert_eq!(at(observation.context()), (0, 2, ".a"));
    for anchor in [
        observation.declaration(),
        observation.property(),
        observation.value(),
        observation.context(),
    ] {
        assert_eq!(anchor.source_id(), SourceId::new(7));
    }
}

#[test]
fn invalid_outcome_keeps_whole_value_as_subject_evidence() {
    // "a{transform:scale(1, 2, 3);}": `transform` 2..11, `:` 11,
    // value 12..26, `;` 26.
    let report = analyze("a{transform:scale(1, 2, 3);}");

    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(
        observation.outcome(),
        CssTransformOutcome::InvalidForSelectedValueGrammar
    );
    assert_eq!(at(observation.value()), (12, 26, "scale(1, 2, 3)"));
    assert_eq!(
        at(observation.declaration()),
        (2, 27, "transform:scale(1, 2, 3);")
    );
}

#[test]
fn empty_value_is_invalid_with_a_zero_length_value_anchor_at_colon_end() {
    // "a{transform:;}": `:` 11..12, empty value at 12, `;` 12..13.
    let report = analyze("a{transform:;}");

    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(
        observation.outcome(),
        CssTransformOutcome::InvalidForSelectedValueGrammar
    );
    assert_eq!(at(observation.value()), (12, 12, ""));
    assert_eq!(at(observation.declaration()), (2, 13, "transform:;"));
}

#[test]
fn every_unsupported_reason_has_a_distinct_public_representation() {
    // Declarations (declaration, value ranges), `a{` = 0..2:
    //   transform:inherit;                       (2, 20)  value (12, 19)
    //   transform:var(--x);                      (20, 39) value (30, 38)
    //   transform:cycle(none);                   (39, 61) value (49, 60)
    //   transform:unknownfunction(1px);          (61, 92) value (71, 91)
    //   transform:scale(calc(1));                (92, 117) value (102, 116)
    let report = analyze(
        "a{transform:inherit;transform:var(--x);transform:cycle(none);\
         transform:unknownfunction(1px);transform:scale(calc(1));}",
    );

    let expected = [
        (
            CssTransformUnsupportedReason::CssWideKeyword,
            (2, 20),
            (12, 19, "inherit"),
        ),
        (
            CssTransformUnsupportedReason::DeferredSubstitutionFunction,
            (20, 39),
            (30, 38, "var(--x)"),
        ),
        (
            CssTransformUnsupportedReason::WholeValueFunction,
            (39, 61),
            (49, 60, "cycle(none)"),
        ),
        (
            CssTransformUnsupportedReason::UnselectedTransformFunction,
            (61, 92),
            (71, 91, "unknownfunction(1px)"),
        ),
        (
            CssTransformUnsupportedReason::FunctionValuedTransformArgument,
            (92, 117),
            (102, 116, "scale(calc(1))"),
        ),
    ];
    assert_eq!(report.observations().len(), expected.len());
    for (observation, (reason, declaration, value)) in report.observations().iter().zip(expected) {
        assert_eq!(
            observation.outcome(),
            CssTransformOutcome::UnsupportedBySelectedValueProfile(reason)
        );
        assert_eq!(
            (
                observation.declaration().range().start(),
                observation.declaration().range().end()
            ),
            declaration
        );
        assert_eq!(at(observation.value()), value);
    }
}

#[test]
fn escaped_and_case_varied_property_identity_keeps_authored_spelling() {
    // "a{TR\41 NSFORM:none;Transform:none;}": the first property is
    // `TR\41 NSFORM` = 12 authored bytes (2..14) decoding to 9 characters;
    // `:` 14, `none` 15..19, `;` 19. The second property is 20..29,
    // `none` 30..34, `;` 34.
    let report = analyze("a{TR\\41 NSFORM:none;Transform:none;}");

    assert_eq!(report.observations().len(), 2);
    assert_eq!(
        at(report.observations()[0].property()),
        (2, 14, "TR\\41 NSFORM")
    );
    assert_eq!(
        at(report.observations()[0].declaration()),
        (2, 20, "TR\\41 NSFORM:none;")
    );
    assert_eq!(at(report.observations()[0].value()), (15, 19, "none"));
    assert_eq!(
        at(report.observations()[1].property()),
        (20, 29, "Transform")
    );
    assert_eq!(at(report.observations()[1].value()), (30, 34, "none"));
}

#[test]
fn boundary_trivia_stays_outside_value_and_priority_evidence() {
    // "a{ /*c*/ transform /*d*/ : /*e*/ scale(2) /*f*/ !important /*g*/ ;}":
    // `transform` 9..18, `:` 25, `scale(2)` 33..41, `!important` 48..58,
    // `;` 65..66; the declaration runs 9..66.
    let report = analyze("a{ /*c*/ transform /*d*/ : /*e*/ scale(2) /*f*/ !important /*g*/ ;}");

    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(observation.outcome(), CssTransformOutcome::Qualified);
    assert_eq!(at(observation.property()), (9, 18, "transform"));
    assert_eq!(at(observation.value()), (33, 41, "scale(2)"));
    assert_eq!(
        at(observation.priority().expect("priority is retained")),
        (48, 58, "!important")
    );
    assert_eq!(observation.declaration().range().start(), 9);
    assert_eq!(observation.declaration().range().end(), 66);
}

#[test]
fn priority_is_separate_evidence_and_does_not_change_the_outcome() {
    // "a{transform:none !important;}": `none` 12..16, `!important` 17..27,
    // `;` 27, declaration 2..28.
    let report = analyze("a{transform:none !important;}");

    let observation = &report.observations()[0];
    assert_eq!(observation.outcome(), CssTransformOutcome::Qualified);
    assert_eq!(at(observation.value()), (12, 16, "none"));
    assert_eq!(at(observation.priority().unwrap()), (17, 27, "!important"));
    assert_eq!(
        at(observation.declaration()),
        (2, 28, "transform:none !important;")
    );
}

#[test]
fn duplicates_and_mixed_properties_keep_every_transform_in_source_order() {
    // "a{" 0..2; transform:none; 2..17; color:red; 17..27;
    // transform:scale(1); 27..46 (value 37..45); transform:none; 46..61
    // (value 56..60); "}" 61; "b{" 62..64 (header 62..63);
    // transform:matrix(1,0,0,1,0,0); 64..94 (value 74..93); "}" 94.
    let report = analyze(
        "a{transform:none;color:red;transform:scale(1);transform:none;}\
         b{transform:matrix(1,0,0,1,0,0);}",
    );

    let seen: Vec<_> = report
        .observations()
        .iter()
        .map(|observation| {
            (
                (
                    observation.declaration().range().start(),
                    observation.declaration().range().end(),
                ),
                at(observation.value()),
                at(observation.context()),
                observation.outcome(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        vec![
            (
                (2, 17),
                (12, 16, "none"),
                (0, 1, "a"),
                CssTransformOutcome::Qualified
            ),
            (
                (27, 46),
                (37, 45, "scale(1)"),
                (0, 1, "a"),
                CssTransformOutcome::Qualified
            ),
            (
                (46, 61),
                (56, 60, "none"),
                (0, 1, "a"),
                CssTransformOutcome::Qualified
            ),
            (
                (64, 94),
                (74, 93, "matrix(1,0,0,1,0,0)"),
                (62, 63, "b"),
                CssTransformOutcome::Qualified
            ),
        ]
    );
}

#[test]
fn nested_qualified_and_group_contexts_report_their_direct_owning_header() {
    // "a{transform:none;b{transform:scale(1);}}": `a` 0..1; first
    // declaration 2..17; `b` 17..18; second declaration 19..38
    // (value 29..37).
    let report = analyze("a{transform:none;b{transform:scale(1);}}");
    let seen: Vec<_> = report
        .observations()
        .iter()
        .map(|observation| (at(observation.value()), at(observation.context())))
        .collect();
    assert_eq!(
        seen,
        vec![
            ((12, 16, "none"), (0, 1, "a")),
            ((29, 37, "scale(1)"), (17, 18, "b")),
        ]
    );

    // "a{@media screen{transform:none;}}": group header 2..15
    // ("@media screen"), declaration 16..31 (value 26..30).
    let report = analyze("a{@media screen{transform:none;}}");
    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(at(observation.context()), (2, 15, "@media screen"));
    assert_eq!(at(observation.value()), (26, 30, "none"));
    assert_eq!(at(observation.declaration()), (16, 31, "transform:none;"));
}

#[test]
fn selector_validity_does_not_gate_transform_observations() {
    // "a,,b{" header 0..4; transform:none; 5..20; "}" 20;
    // "::before{" header 21..29; transform:scale(2); 30..49; "}" 49.
    let text = "a,,b{transform:none;}::before{transform:scale(2);}";
    let report = analyze(text);

    assert_eq!(report.observations().len(), 2);
    assert_eq!(at(report.observations()[0].context()), (0, 4, "a,,b"));
    assert_eq!(at(report.observations()[1].context()), (21, 29, "::before"));
    for observation in report.observations() {
        assert_eq!(observation.outcome(), CssTransformOutcome::Qualified);
    }

    // The same source really has Invalid and Unsupported selector contexts
    // under the separate selector capability, so independence is exercised.
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    let selectors = analyze_core_v1(&source).unwrap();
    assert!(matches!(
        selectors.observations()[0].outcome(),
        CssSelectorOutcome::InvalidForSelectedGrammar { .. }
    ));
    assert!(matches!(
        selectors.observations()[1].outcome(),
        CssSelectorOutcome::UnsupportedBySelectedGrammarProfile { .. }
    ));
}

#[test]
fn nonordinary_declaration_categories_never_become_observations() {
    let text = "@font-face{transform:none;}@page{transform:none;@top-left{transform:none;}}\
                @keyframes k{from{transform:none;}}";
    let report = analyze(text);
    assert!(report.observations().is_empty());

    // The fixture genuinely retains transform-looking declarations in each
    // non-ordinary category, and none in the ordinary one.
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    let (tokenizer, parser) = limits(32_768, 512);
    let run = analyze_css_source(&source, tokenizer, parser).unwrap();
    assert_eq!(run.occurrences().len(), 0);
    assert_eq!(run.descriptor_occurrences().len(), 1);
    assert_eq!(run.page_occurrences().len(), 1);
    assert_eq!(run.page_margin_occurrences().len(), 1);
    assert_eq!(run.keyframe_occurrences().len(), 1);
}

#[test]
fn zero_observations_coexist_with_unsupported_coverage() {
    // A root `@media` is an unsupported top-level at-rule: its body holds
    // authored transform text that never becomes an observation.
    // "@media screen{transform:none;}" is 30 bytes.
    let report = analyze("@media screen{transform:none;}");

    assert!(report.observations().is_empty());
    assert_fully_complete(&report);
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::ContainsUnsupportedContexts
    );
    assert_eq!(report.parser().unsupported().len(), 1);
    let region = &report.parser().unsupported()[0];
    assert_eq!(
        region.kind(),
        CssParserUnsupportedRegionKind::TopLevelAtRule
    );
    assert_eq!(
        at(region.region()),
        (0, 30, "@media screen{transform:none;}")
    );
}

#[test]
fn zero_observations_for_a_nested_unsupported_region_and_for_comments_and_strings() {
    let report = analyze("a{@foo{transform:none;}}");
    assert!(report.observations().is_empty());
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::ContainsUnsupportedContexts
    );
    assert_eq!(report.parser().unsupported_regions(), 1);

    // Transform text inside a comment and a string is never an observation.
    let report = analyze("a{/*transform:none;*/content:\"transform:none;\";}");
    assert!(report.observations().is_empty());
    assert_fully_complete(&report);
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::SupportedForSelectedQuestion
    );
}

#[test]
fn a_recovered_malformed_item_with_transform_text_is_recovery_not_an_observation() {
    // "a{" 0..2; malformed "transform scale(1);" recovery 2..21;
    // "transform:none;" 21..36 (value 31..35).
    let report = analyze("a{transform scale(1);transform:none;}");

    assert_eq!(report.parser().recovery_records(), 1);
    let recovery = &report.parser().recovery()[0];
    assert_eq!(recovery.kind(), CssParserRecoveryKind::MalformedBlockItem);
    assert_eq!(at(recovery.region()), (2, 21, "transform scale(1);"));
    assert_eq!(report.observations().len(), 1);
    assert_eq!(
        at(report.observations()[0].declaration()),
        (21, 36, "transform:none;")
    );
    assert_eq!(at(report.observations()[0].value()), (31, 35, "none"));
}

#[test]
fn empty_source_is_complete_with_zero_observations() {
    let report = analyze("");

    assert!(report.observations().is_empty());
    assert_fully_complete(&report);
}

#[test]
fn true_end_of_input_keeps_a_committed_open_declaration() {
    // "a{transform:none": declaration 2..16, value 12..16, no authored
    // terminator.
    let report = analyze("a{transform:none");

    assert_fully_complete(&report);
    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(observation.outcome(), CssTransformOutcome::Qualified);
    assert_eq!(at(observation.declaration()), (2, 16, "transform:none"));
    assert_eq!(at(observation.value()), (12, 16, "none"));
    assert_eq!(at(observation.context()), (0, 1, "a"));
}

#[test]
fn multibyte_source_keeps_byte_exact_anchors() {
    // "a{/*é*/transform:none;}": the comment is 6 bytes (2..8);
    // declaration 8..23, value 18..22.
    let report = analyze("a{/*é*/transform:none;}");
    assert_eq!(
        at(report.observations()[0].declaration()),
        (8, 23, "transform:none;")
    );
    assert_eq!(at(report.observations()[0].value()), (18, 22, "none"));

    // "é{transform:none;}": `é` is 2 bytes (0..2); declaration 3..18.
    let report = analyze("é{transform:none;}");
    assert_eq!(at(report.observations()[0].context()), (0, 2, "é"));
    assert_eq!(
        at(report.observations()[0].declaration()),
        (3, 18, "transform:none;")
    );
}

#[test]
fn committed_prefix_survives_a_later_parser_refusal() {
    // Limit 2 declaration occurrences; the third declaration exceeds it.
    // First declaration 2..17 (value 12..16); second 17..36 (value 27..35).
    let report = analyze_limited(
        "a{transform:none;transform:scale(1);transform:scale(2);}",
        32_768,
        2,
    );

    assert_eq!(
        report.tokenizer().completion(),
        CssStageCompletion::Complete
    );
    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    match report.parser().termination() {
        CssParserTermination::ResourceLimit(refusal) => {
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Parser(CssParserResourceKind::DeclarationOccurrences)
            );
            assert_eq!(refusal.limit(), 2);
            assert_eq!(refusal.attempted(), 3);
        }
        other => panic!("expected parser resource refusal, got {other:?}"),
    }
    let seen: Vec<_> = report
        .observations()
        .iter()
        .map(|observation| at(observation.value()))
        .collect();
    assert_eq!(seen, vec![(12, 16, "none"), (27, 35, "scale(1)")]);
}

#[test]
fn tokenizer_refusal_is_reported_and_manufactures_no_observation() {
    let report = analyze_limited("a{transform:none;}", 4, 512);

    assert_eq!(
        report.tokenizer().completion(),
        CssStageCompletion::Incomplete
    );
    match report.tokenizer().termination() {
        CssTokenizerTermination::ResourceLimit(refusal) => {
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Tokenizer(CssTokenizerResourceKind::SourceBytes)
            );
            assert_eq!(refusal.limit(), 4);
            assert_eq!(refusal.attempted(), 18);
        }
        other => panic!("expected tokenizer resource refusal, got {other:?}"),
    }
    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    assert!(matches!(
        report.parser().termination(),
        CssParserTermination::UpstreamTokenizerIncomplete
    ));
    assert!(report.observations().is_empty());
}

#[test]
fn fixed_policy_admits_the_full_declaration_occurrence_envelope() {
    // 512 retained declarations are all reported, none truncated.
    let text = format!("a{{{}}}", "transform:none;".repeat(512));
    let report = analyze(&text);
    assert_fully_complete(&report);
    assert_eq!(report.observations().len(), 512);
    assert_eq!(
        at(report.observations()[511].declaration()),
        (2 + 15 * 511, 2 + 15 * 512, "transform:none;")
    );

    // The 513th exceeds the policy: the 512 committed observations remain
    // and the parser reports the refusal.
    let text = format!("a{{{}}}", "transform:none;".repeat(513));
    let report = analyze(&text);
    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    match report.parser().termination() {
        CssParserTermination::ResourceLimit(refusal) => {
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Parser(CssParserResourceKind::DeclarationOccurrences)
            );
            assert_eq!(refusal.limit(), 512);
            assert_eq!(refusal.attempted(), 513);
        }
        other => panic!("expected parser resource refusal, got {other:?}"),
    }
    assert_eq!(report.observations().len(), 512);
}

#[test]
fn long_repeated_context_headers_publish_completely_without_selector_work() {
    // A 10_000-byte header with 100 declarations: every observation keeps the
    // same header range; selector qualification would otherwise refuse or
    // charge work for it.
    let header = "a".repeat(10_000);
    let text = format!("{header}{{{}}}", "transform:none;".repeat(100));
    let report = analyze(&text);

    assert_fully_complete(&report);
    assert_eq!(report.observations().len(), 100);
    for observation in report.observations() {
        assert_eq!(observation.context().range().start(), 0);
        assert_eq!(observation.context().range().end(), 10_000);
        assert_eq!(observation.outcome(), CssTransformOutcome::Qualified);
    }
}

#[test]
fn report_evidence_outlives_the_caller_source_handle() {
    let report = {
        let source = SourceText::new(SourceId::new(9), "a{transform:none !important;}".to_owned());
        analyze_authored_transforms(&source).unwrap()
    };

    let observation = &report.observations()[0];
    assert_eq!(at(observation.property()), (2, 11, "transform"));
    assert_eq!(at(observation.value()), (12, 16, "none"));
    assert_eq!(at(observation.priority().unwrap()), (17, 27, "!important"));
    assert_eq!(at(observation.context()), (0, 1, "a"));
}

#[test]
fn distinct_source_ids_are_preserved_exactly() {
    for id in [0, 1, 4_000_000_000] {
        let source = SourceText::new(SourceId::new(id), "a{transform:none;}".to_owned());
        let report = analyze_authored_transforms(&source).unwrap();
        assert_eq!(report.source_id(), SourceId::new(id));
        assert_eq!(
            report.observations()[0].declaration().source_id(),
            SourceId::new(id)
        );
    }
}

#[test]
fn repeated_analysis_is_deterministic() {
    let text = "a{transform:none;b{transform:scale(1,2,3);}}@x;";
    let first = analyze(text);
    let second = analyze(text);

    let key = |report: &CssTransformReport| {
        report
            .observations()
            .iter()
            .map(|observation| (at(observation.declaration()).0, observation.outcome()))
            .collect::<Vec<_>>()
    };
    assert_eq!(key(&first), key(&second));
    assert_eq!(first.parser().unsupported_regions(), 1);
}

#[test]
fn fixed_execution_policy_constructs() {
    assert!(analyze_authored_transforms(&SourceText::new(SourceId::new(0), String::new())).is_ok());
}

#[test]
fn core_failure_display_names_boundary_without_source_content() {
    for (failure, boundary) in [
        (
            CssTransformCoreFailure::execution_policy(),
            "execution policy",
        ),
        (
            CssTransformCoreFailure::projection(Projection::OccurrenceMissing),
            "transform report projection",
        ),
        (
            CssTransformCoreFailure::projection(Projection::PlacementMismatch),
            "transform report projection",
        ),
        (
            CssTransformCoreFailure::projection(Projection::ContextMissing),
            "transform report projection",
        ),
    ] {
        assert_eq!(
            failure.to_string(),
            format!("CSS transform analysis returned a Core failure at the {boundary} boundary")
        );
    }
}
