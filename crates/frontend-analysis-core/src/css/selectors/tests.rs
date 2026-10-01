//! Focused tests for the public `CoreV1` selector facade (#857).
//!
//! These prove the new public projection only; the CSS tokenizer, parser,
//! and selector grammar suites remain the authority for grammar behavior.
//! Expected ranges and outcomes come from the independent selector gold
//! fixtures, the pinned Core analysis tests, or hand-counted source offsets,
//! never from facade output.

use super::*;
use crate::{SourceId, SourceText};

fn analyze(text: &str) -> CssSelectorReport {
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    analyze_core_v1(&source).expect("ordinary analysis must not be a Core failure")
}

fn range(anchor: &SourceAnchor) -> (usize, usize) {
    (anchor.range().start(), anchor.range().end())
}

fn assert_fully_complete(report: &CssSelectorReport) {
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
    assert_eq!(report.selector().completion(), CssStageCompletion::Complete);
    assert!(matches!(
        report.selector().termination(),
        CssSelectorTermination::AllRetainedQualifiedContextsProcessed
    ));
}

#[test]
fn qualified_context_projects_retained_header_profile_and_identity() {
    // CSS-SELECTOR-CORE-TYPE-001: header (0, 1), normal, qualified.
    let report = analyze("a{}");

    assert_eq!(report.source_id(), SourceId::new(0));
    assert_eq!(report.profile(), CssSelectorProfile::CoreV1);
    assert_fully_complete(&report);
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::SupportedForSelectedQuestion
    );
    assert_eq!(report.observations().len(), 1);
    let observation = &report.observations()[0];
    assert_eq!(range(observation.context()), (0, 1));
    assert_eq!(observation.context().fragment(), "a");
    assert_eq!(
        observation.grammar_context(),
        CssSelectorGrammarContext::NormalSelectorList
    );
    assert!(matches!(
        observation.outcome(),
        CssSelectorOutcome::QualifiedBySelectedGrammar
    ));
}

#[test]
fn invalid_outcome_keeps_reason_and_retained_subject() {
    // CSS-SELECTOR-INVALID-COMMA-001: header (0, 4), subject (2, 3).
    let report = analyze("a,,b{}");
    let observation = &report.observations()[0];

    assert_eq!(range(observation.context()), (0, 4));
    match observation.outcome() {
        CssSelectorOutcome::InvalidForSelectedGrammar { reason, subject } => {
            assert_eq!(*reason, CssSelectorInvalidReason::UnexpectedComma);
            assert_eq!(range(subject), (2, 3));
            assert_eq!(subject.fragment(), ",");
        }
        other => panic!("expected invalid outcome, got {other:?}"),
    }
}

#[test]
fn unsupported_outcome_is_not_collapsed_into_invalid() {
    // CSS-SELECTOR-PSEUDO-IDENT-UNSUPPORTED-001 and
    // CSS-SELECTOR-PSEUDO-ELEMENT-UNSUPPORTED-001.
    let report = analyze(":future-pseudo{}");
    match report.observations()[0].outcome() {
        CssSelectorOutcome::UnsupportedBySelectedGrammarProfile { feature, subject } => {
            assert_eq!(
                *feature,
                CssSelectorUnsupportedFeature::IdentifierPseudoClass
            );
            assert_eq!(range(subject), (1, 14));
        }
        other => panic!("expected unsupported outcome, got {other:?}"),
    }

    let report = analyze("::before{}");
    match report.observations()[0].outcome() {
        CssSelectorOutcome::UnsupportedBySelectedGrammarProfile { feature, subject } => {
            assert_eq!(*feature, CssSelectorUnsupportedFeature::PseudoElement);
            assert_eq!(range(subject), (2, 8));
        }
        other => panic!("expected unsupported outcome, got {other:?}"),
    }
}

#[test]
fn indeterminate_outcome_keeps_reason_and_optional_subject() {
    // CSS-SELECTOR-NS-NAMED-INDETERMINATE-001: header (0, 5), subject (0, 3).
    let report = analyze("svg|a{}");
    let observation = &report.observations()[0];

    assert_eq!(range(observation.context()), (0, 5));
    match observation.outcome() {
        CssSelectorOutcome::Indeterminate { reason, subject } => {
            assert_eq!(
                *reason,
                CssSelectorIndeterminateReason::MissingNamespaceEnvironment
            );
            let subject = subject.as_ref().expect("namespace subject is retained");
            assert_eq!(range(subject), (0, 3));
            assert_eq!(subject.fragment(), "svg");
        }
        other => panic!("expected indeterminate outcome, got {other:?}"),
    }
}

#[test]
fn observations_follow_retained_context_order_and_grammar_context() {
    // CSS-SELECTOR-NEST-IMPLICIT-001 places `.b` at (3, 5) in nested mode;
    // its `.a` parent header is (0, 2) in normal mode.
    let report = analyze(".a{.b{}}");
    let observed: Vec<_> = report
        .observations()
        .iter()
        .map(|observation| (range(observation.context()), observation.grammar_context()))
        .collect();
    assert_eq!(
        observed,
        [
            ((0, 2), CssSelectorGrammarContext::NormalSelectorList),
            (
                (3, 5),
                CssSelectorGrammarContext::NestedRelativeSelectorList
            ),
        ]
    );

    // CSS-SELECTOR-NEST-INVALID-AMP-TYPE-001: the second observation is an
    // invalid nesting placement inside header (3, 7). Its exact subject range
    // is checked by `facade_projects_internal_run_evidence_unchanged`.
    let report = analyze(".a{&Bar{}}");
    match report.observations()[1].outcome() {
        CssSelectorOutcome::InvalidForSelectedGrammar { reason, subject } => {
            assert_eq!(
                *reason,
                CssSelectorInvalidReason::InvalidNestingSelectorPlacement
            );
            assert!(subject.range().start() >= 3 && subject.range().end() <= 7);
        }
        other => panic!("expected invalid nesting outcome, got {other:?}"),
    }

    // CSS-SELECTOR-GROUP-SCOPE-MODE-001: `b` at (9, 10) is scoped.
    let report = analyze("a{@scope{b{}}}");
    let scoped = report
        .observations()
        .iter()
        .find(|observation| range(observation.context()) == (9, 10))
        .expect("scoped context is retained");
    assert_eq!(
        scoped.grammar_context(),
        CssSelectorGrammarContext::ScopedRelativeSelectorList
    );
}

/// The facade must copy the internal run's retained evidence unchanged: same
/// order, context ranges, grammar modes, outcome kinds, and subject ranges.
#[test]
fn facade_projects_internal_run_evidence_unchanged() {
    use super::super::selector::context::CssSelectorGrammarContext as InternalContext;
    use super::super::selector::result::CssSelectorQualificationOutcome as InternalOutcome;

    let text = "a,,b{} .a{&Bar{} > .b{}} svg|a{} ::before{} :nth-child(2n){} #1{} a{@scope{b{}}}";
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    let internal = analyze_css_selectors(
        &source,
        tokenizer_resource::CssTokenizerLimits::new(
            TOKENIZER_SOURCE_BYTES,
            TOKENIZER_ALGORITHM_STEPS,
            TOKENIZER_LEXICAL_ITEMS,
            TOKENIZER_DIAGNOSTICS,
            TOKENIZER_RETAINED_INTERPRETED_BYTES,
            TOKENIZER_TEMPORARY_BUFFER_BYTES,
        )
        .unwrap(),
        parser_resource::CssParserLimits::new(
            PARSER_ALGORITHM_STEPS,
            PARSER_PEAK_COMPONENT_DEPTH,
            PARSER_PEAK_CONTEXT_DEPTH,
            PARSER_DECLARATION_OCCURRENCES,
            PARSER_DIAGNOSTICS,
            PARSER_RECOVERY_RECORDS,
            PARSER_UNSUPPORTED_REGIONS,
            PARSER_DISCARD_RECORDS,
            PARSER_CONTEXT_RECORDS,
        )
        .unwrap(),
        selector_resource::CssSelectorLimits::new(
            SELECTOR_ALGORITHM_STEPS,
            SELECTOR_PEAK_SELECTOR_DEPTH,
            SELECTOR_OBSERVATIONS,
        )
        .unwrap(),
    )
    .unwrap();
    let report = analyze_core_v1(&source).unwrap();

    assert_eq!(report.observations().len(), internal.observations().len());
    assert!(report.observations().len() >= 9);
    for (public, private) in report.observations().iter().zip(internal.observations()) {
        let header =
            &internal.upstream_parser_result().context_records()[private.context_id().index()];
        assert_eq!(range(public.context()), range(header.header()));
        assert_eq!(range(public.context()), range(private.context_header()));
        assert_eq!(
            public.grammar_context(),
            match private.grammar_context() {
                InternalContext::NormalSelectorList => {
                    CssSelectorGrammarContext::NormalSelectorList
                }
                InternalContext::NestedRelativeSelectorList { .. } => {
                    CssSelectorGrammarContext::NestedRelativeSelectorList
                }
                InternalContext::ScopedRelativeSelectorList { .. } => {
                    CssSelectorGrammarContext::ScopedRelativeSelectorList
                }
            }
        );
        let public_subject = match public.outcome() {
            CssSelectorOutcome::QualifiedBySelectedGrammar => None,
            CssSelectorOutcome::InvalidForSelectedGrammar { subject, .. }
            | CssSelectorOutcome::UnsupportedBySelectedGrammarProfile { subject, .. } => {
                Some(range(subject))
            }
            CssSelectorOutcome::Indeterminate { subject, .. } => subject.as_ref().map(range),
        };
        let private_subject = match private.outcome() {
            InternalOutcome::QualifiedBySelectedGrammar => None,
            InternalOutcome::InvalidForSelectedGrammar { subject, .. }
            | InternalOutcome::UnsupportedBySelectedGrammarProfile { subject, .. } => {
                Some(range(subject))
            }
            InternalOutcome::Indeterminate { subject, .. } => subject.as_ref().map(range),
        };
        assert_eq!(public_subject, private_subject);
        let public_kind = match public.outcome() {
            CssSelectorOutcome::QualifiedBySelectedGrammar => "qualified",
            CssSelectorOutcome::InvalidForSelectedGrammar { .. } => "invalid",
            CssSelectorOutcome::UnsupportedBySelectedGrammarProfile { .. } => "unsupported",
            CssSelectorOutcome::Indeterminate { .. } => "indeterminate",
        };
        let private_kind = match private.outcome() {
            InternalOutcome::QualifiedBySelectedGrammar => "qualified",
            InternalOutcome::InvalidForSelectedGrammar { .. } => "invalid",
            InternalOutcome::UnsupportedBySelectedGrammarProfile { .. } => "unsupported",
            InternalOutcome::Indeterminate { .. } => "indeterminate",
        };
        assert_eq!(public_kind, private_kind);
    }
}

#[test]
fn empty_source_is_complete_with_zero_observations() {
    let report = analyze("");

    assert_fully_complete(&report);
    assert!(report.observations().is_empty());
}

#[test]
fn upstream_recovery_discard_and_unsupported_counts_are_preserved() {
    // Pinned by css::analysis::tests::recovery_propagates_through_core.
    let report = analyze("a{color red;background:blue;}");
    assert_eq!(report.parser().diagnostics(), 1);
    assert_eq!(report.parser().recovery_records(), 1);
    assert_eq!(report.observations().len(), 1);

    // Pinned by css::analysis::tests::discard_propagates_through_core.
    let report = analyze("--foo:bar{color:red;}");
    assert_eq!(report.parser().discard_records(), 1);

    // Pinned by css::analysis::tests::unsupported_propagates_through_core.
    let report = analyze("a{color:red;@unknown-rule{color:blue;}}");
    assert_eq!(report.parser().unsupported_regions(), 1);
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::ContainsUnsupportedContexts
    );
}

#[test]
fn source_bytes_at_policy_limit_is_accepted() {
    let text = format!("/*{}*/", "x".repeat(TOKENIZER_SOURCE_BYTES - 4));
    assert_eq!(text.len(), 32_768);

    let report = analyze(&text);
    assert_fully_complete(&report);
}

#[test]
fn tokenizer_source_bytes_refusal_is_reported_not_failed_and_not_upgraded() {
    let text = "a".repeat(32_769);
    let report = analyze(&text);

    assert_eq!(
        report.tokenizer().completion(),
        CssStageCompletion::Incomplete
    );
    match report.tokenizer().termination() {
        CssTokenizerTermination::ResourceLimit(refusal) => {
            assert_eq!(refusal.stage(), CssAnalysisStage::Tokenizer);
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Tokenizer(CssTokenizerResourceKind::SourceBytes)
            );
            assert_eq!(refusal.limit(), 32_768);
            assert_eq!(refusal.attempted(), 32_769);
            assert_eq!(range(refusal.location()), (0, 0));
        }
        other => panic!("expected tokenizer refusal, got {other:?}"),
    }
    // The downstream stages stay honest: the parser reports upstream
    // incompleteness, and a complete selector stage over zero retained
    // contexts is not a whole-source completeness claim.
    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    assert!(matches!(
        report.parser().termination(),
        CssParserTermination::UpstreamTokenizerIncomplete
    ));
    assert_eq!(report.selector().completion(), CssStageCompletion::Complete);
    assert!(report.observations().is_empty());
}

#[test]
fn parser_context_depth_refusal_is_reported_with_owning_stage() {
    // 33 nested qualified rules: the 33rd context entry exceeds the approved
    // PeakContextDepth = 32 at its preflight.
    let text = format!("{}{}", "a{".repeat(33), "}".repeat(33));
    let report = analyze(&text);

    assert_eq!(
        report.tokenizer().completion(),
        CssStageCompletion::Complete
    );
    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    match report.parser().termination() {
        CssParserTermination::ResourceLimit(refusal) => {
            assert_eq!(refusal.stage(), CssAnalysisStage::Parser);
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Parser(CssParserResourceKind::PeakContextDepth)
            );
            assert_eq!(refusal.limit(), 32);
            assert_eq!(refusal.attempted(), 33);
        }
        other => panic!("expected parser refusal, got {other:?}"),
    }
}

#[test]
fn selector_observation_refusal_preserves_ordered_prefix() {
    // 1025 top-level rules: the 1025th observation exceeds the approved
    // Observations = 1024 after 1024 committed observations.
    let text = "a{}".repeat(1_025);
    let report = analyze(&text);

    assert_eq!(report.parser().completion(), CssStageCompletion::Complete);
    assert_eq!(
        report.selector().completion(),
        CssStageCompletion::Incomplete
    );
    assert_eq!(report.observations().len(), 1_024);
    for (index, observation) in report.observations().iter().enumerate() {
        assert_eq!(range(observation.context()), (index * 3, index * 3 + 1));
    }
    match report.selector().termination() {
        CssSelectorTermination::ResourceLimit(refusal) => {
            assert_eq!(refusal.stage(), CssAnalysisStage::Selector);
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Selector(CssSelectorResourceKind::Observations)
            );
            assert_eq!(refusal.limit(), 1_024);
            assert_eq!(refusal.attempted(), 1_025);
            // Located inside the 1025th header, which spans (3072, 3073).
            let location = refusal.location().range().start();
            assert!((3_072..=3_073).contains(&location));
        }
        other => panic!("expected selector refusal, got {other:?}"),
    }
}

#[test]
fn selector_depth_refusal_reports_selector_stage() {
    // 17 nested `:is(` frames exceed the approved PeakSelectorDepth = 16.
    let text = format!("{}a{}{{}}", ":is(".repeat(17), ")".repeat(17));
    let report = analyze(&text);

    assert_eq!(report.parser().completion(), CssStageCompletion::Complete);
    assert!(report.observations().is_empty());
    match report.selector().termination() {
        CssSelectorTermination::ResourceLimit(refusal) => {
            assert_eq!(
                refusal.kind(),
                CssResourceKind::Selector(CssSelectorResourceKind::PeakSelectorDepth)
            );
            assert_eq!(refusal.limit(), 16);
            assert_eq!(refusal.attempted(), 17);
        }
        other => panic!("expected selector depth refusal, got {other:?}"),
    }
}

#[test]
fn report_evidence_outlives_the_caller_source_handle() {
    let source = SourceText::new(SourceId::new(0), "svg|a{}".to_owned());
    let report = analyze_core_v1(&source).unwrap();
    drop(source);

    let observation = &report.observations()[0];
    assert_eq!(observation.context().fragment(), "svg|a");
    assert_eq!(observation.context().source_id(), SourceId::new(0));
    match observation.outcome() {
        CssSelectorOutcome::Indeterminate {
            subject: Some(subject),
            ..
        } => assert_eq!(subject.fragment(), "svg"),
        other => panic!("expected indeterminate outcome, got {other:?}"),
    }
}

#[test]
fn authored_spelling_is_retained_not_reconstructed() {
    // CSS-SELECTOR-SOURCE-ESCAPED-IDENT-001 and
    // CSS-SELECTOR-SOURCE-MULTIBYTE-001 header ranges, plus an authored
    // leading BOM and CRLF that must remain exact source evidence.
    let report = analyze(r".\66 oo{}");
    assert_eq!(range(report.observations()[0].context()), (0, 7));
    assert_eq!(report.observations()[0].context().fragment(), r".\66 oo");

    let report = analyze(".é{}");
    assert_eq!(range(report.observations()[0].context()), (0, 3));
    assert_eq!(report.observations()[0].context().fragment(), ".é");

    // U+FEFF is 3 UTF-8 bytes; "a\r\n" then occupies (3, 6).
    let report = analyze("\u{feff}a\r\n{}");
    let context = report.observations()[0].context();
    assert_eq!(range(context), (3, 6));
    assert_eq!(context.fragment(), "a\r\n");
    assert_eq!(context.start_coordinate().line_index(), 0);
}

#[test]
fn repeated_analysis_is_deterministic() {
    let text = "a,,b{} .a{&Bar{}} svg|a{} ::before{}";
    let render = |report: &CssSelectorReport| -> Vec<String> {
        report
            .observations()
            .iter()
            .map(|observation| format!("{observation:?}"))
            .collect()
    };
    assert_eq!(render(&analyze(text)), render(&analyze(text)));
}

#[test]
fn approved_phase_1_envelope_constructs() {
    // The fixed policy must never surface as a Core failure.
    assert!(analyze_core_v1(&SourceText::new(SourceId::new(0), String::new())).is_ok());
}

#[test]
fn core_failure_display_names_boundary_without_source_content() {
    let failure = CssSelectorCoreFailure::execution_policy();
    assert_eq!(
        failure.to_string(),
        "CSS CoreV1 selector analysis returned a Core failure at the execution policy boundary"
    );
}

// ---- #860: source-locatable parser partial-coverage evidence ----
//
// Expected ranges come from the independent parser gold fixtures
// (CSS-PARSER-*, CSS-KEYFRAMES-*) or hand-counted offsets, never from facade
// output.

fn recoveries(
    report: &CssSelectorReport,
) -> Vec<((usize, usize), &str, CssParserRecoveryTermination)> {
    report
        .parser()
        .recovery()
        .iter()
        .map(|record| {
            assert_eq!(record.kind(), CssParserRecoveryKind::MalformedBlockItem);
            (
                range(record.region()),
                record.region().fragment(),
                record.termination(),
            )
        })
        .collect()
}

fn unsupported(
    report: &CssSelectorReport,
) -> Vec<((usize, usize), &str, CssParserUnsupportedRegionKind)> {
    report
        .parser()
        .unsupported()
        .iter()
        .map(|record| {
            (
                range(record.region()),
                record.region().fragment(),
                record.kind(),
            )
        })
        .collect()
}

fn discards(report: &CssSelectorReport) -> Vec<((usize, usize), &str)> {
    report
        .parser()
        .discard()
        .iter()
        .map(|record| {
            assert_eq!(
                record.kind(),
                CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule
            );
            (range(record.region()), record.region().fragment())
        })
        .collect()
}

fn assert_counts_match_records(report: &CssSelectorReport) {
    let parser = report.parser();
    assert_eq!(parser.recovery_records(), parser.recovery().len());
    assert_eq!(parser.unsupported_regions(), parser.unsupported().len());
    assert_eq!(parser.discard_records(), parser.discard().len());
}

#[test]
fn recovery_projects_region_kind_and_each_termination() {
    // CSS-PARSER-MALFORMED-MISSING-COLON-THEN-VALID-001: region (2, 12).
    let report = analyze("a{color red;background:blue;}");
    assert_eq!(
        recoveries(&report),
        [(
            (2, 12),
            "color red;",
            CssParserRecoveryTermination::AuthoredSemicolon
        )]
    );
    // The diagnostic count is preserved; no diagnostic detail is exposed.
    assert_eq!(report.parser().diagnostics(), 1);
    assert_counts_match_records(&report);

    // CSS-PARSER-MALFORMED-AT-TRUE-EOF-001: region (2, 11).
    let report = analyze("a{color red");
    assert_eq!(
        recoveries(&report),
        [(
            (2, 11),
            "color red",
            CssParserRecoveryTermination::EndOfInput
        )]
    );
    assert_counts_match_records(&report);

    // Hand-counted: `a{` 0..2, `color red` 2..11, authored `}` 11..12.
    let report = analyze("a{color red}");
    assert_eq!(
        recoveries(&report),
        [(
            (2, 11),
            "color red",
            CssParserRecoveryTermination::EnclosingBlockEnd
        )]
    );
    assert_counts_match_records(&report);
}

#[test]
fn recovery_records_keep_producer_source_order() {
    // `a{` 0..2, `x;` 2..4, `y;` 4..6, `z` 6..7.
    let report = analyze("a{x;y;z");
    assert_eq!(
        recoveries(&report),
        [
            (
                (2, 4),
                "x;",
                CssParserRecoveryTermination::AuthoredSemicolon
            ),
            (
                (4, 6),
                "y;",
                CssParserRecoveryTermination::AuthoredSemicolon
            ),
            ((6, 7), "z", CssParserRecoveryTermination::EndOfInput),
        ]
    );
    assert_counts_match_records(&report);
}

#[test]
fn unsupported_regions_project_canonical_region_and_kind() {
    // CSS-PARSER-UNSUPPORTED-UNKNOWN-AT-RULE-001: complete (0, 47).
    let text = "@futureabc spin{from{opacity:0;}to{opacity:1;}}";
    let report = analyze(text);
    assert_eq!(
        unsupported(&report),
        [(
            (0, 47),
            text,
            CssParserUnsupportedRegionKind::TopLevelAtRule
        )]
    );
    assert_eq!(
        report.parser().coverage(),
        CssParserCoverage::ContainsUnsupportedContexts
    );
    assert_counts_match_records(&report);

    // Pinned by css::analysis::tests::unsupported_propagates_through_core:
    // `a{color:red;` is 12 bytes, so the nested at-rule is (12, 38).
    let report = analyze("a{color:red;@unknown-rule{color:blue;}}");
    assert_eq!(
        unsupported(&report),
        [(
            (12, 38),
            "@unknown-rule{color:blue;}",
            CssParserUnsupportedRegionKind::NestedAtRule
        )]
    );

    // CSS-KEYFRAMES-INVALID-CHILD-001: complete (13, 24).
    let report = analyze("@keyframes x{bogus{x:y;}from{a:b;}}");
    assert_eq!(
        unsupported(&report),
        [(
            (13, 24),
            "bogus{x:y;}",
            CssParserUnsupportedRegionKind::UnqualifiedKeyframeBlock
        )]
    );
    assert_counts_match_records(&report);
}

#[test]
fn unsupported_regions_keep_producer_source_order() {
    // `@futureabc spin{}` 0..17 and `@other x;` 17..26.
    let report = analyze("@futureabc spin{}@other x;");
    assert_eq!(
        unsupported(&report),
        [
            (
                (0, 17),
                "@futureabc spin{}",
                CssParserUnsupportedRegionKind::TopLevelAtRule
            ),
            (
                (17, 26),
                "@other x;",
                CssParserUnsupportedRegionKind::TopLevelAtRule
            ),
        ]
    );
    assert_counts_match_records(&report);
}

#[test]
fn discard_projects_region_and_kind_in_producer_order() {
    // CSS-PARSER-DISCARD-TOP-LEVEL-CUSTOM-PROPERTY-LIKE-001: region (0, 21).
    let report = analyze("--foo:bar{color:red;}");
    assert_eq!(discards(&report), [((0, 21), "--foo:bar{color:red;}")]);
    assert_counts_match_records(&report);

    // `--a:b{}` is 7 bytes.
    let report = analyze("--a:b{}--c:d{}");
    assert_eq!(
        discards(&report),
        [((0, 7), "--a:b{}"), ((7, 14), "--c:d{}")]
    );
    assert_counts_match_records(&report);
}

#[test]
fn clean_source_has_no_partial_coverage_records() {
    let report = analyze("a{color:red;}");
    assert!(report.parser().recovery().is_empty());
    assert!(report.parser().unsupported().is_empty());
    assert!(report.parser().discard().is_empty());
    assert_counts_match_records(&report);
}

#[test]
fn nested_content_remainder_kind_is_projected_from_its_region() {
    // The parser does not currently produce this variant, so its mapping is
    // proven at the projection function with the internal constructor.
    let source = SourceText::new(SourceId::new(0), "a{x}".to_owned());
    let region = source.anchor(2, 3).expect("valid anchor");
    let internal =
        parser_evidence::CssParserUnsupportedRegion::new_nested_content_remainder(&source, region)
            .unwrap();
    let public = CssParserUnsupportedRegion::project(&internal);
    assert_eq!(
        public.kind(),
        CssParserUnsupportedRegionKind::NestedContentRemainder
    );
    assert_eq!(range(public.region()), (2, 3));
    assert_eq!(public.region().fragment(), "x");
}

#[test]
fn partial_coverage_evidence_outlives_the_caller_source_handle() {
    let source = SourceText::new(SourceId::new(0), "--a:b{}a{x;}@future{}".to_owned());
    let report = analyze_core_v1(&source).unwrap();
    drop(source);

    assert_eq!(report.parser().discard()[0].region().fragment(), "--a:b{}");
    assert_eq!(report.parser().recovery()[0].region().fragment(), "x;");
    assert_eq!(
        report.parser().unsupported()[0].region().fragment(),
        "@future{}"
    );
    assert_eq!(
        report.parser().discard()[0].region().source_id(),
        SourceId::new(0)
    );
}

#[test]
fn authored_spelling_of_partial_coverage_regions_is_retained() {
    // BOM `\u{feff}` is 3 bytes, so `a{` is 3..5 and `x\r\n;` 5..9.
    let report = analyze("\u{feff}a{x\r\n;}");
    let recovery = &report.parser().recovery()[0];
    assert_eq!(range(recovery.region()), (5, 9));
    assert_eq!(recovery.region().fragment(), "x\r\n;");

    // Escaped at-keyword spelling is authored spelling: `@\66 oo{}` is 0..9.
    let report = analyze(r"@\66 oo{}");
    let region = report.parser().unsupported()[0].region();
    assert_eq!(range(region), (0, 9));
    assert_eq!(region.fragment(), r"@\66 oo{}");
}

#[test]
fn parser_resource_refusal_exposes_only_committed_evidence() {
    // 65 top-level unsupported `@x;` rules: the 65th exceeds the approved
    // UnsupportedRegions = 64 after 64 committed regions of 3 bytes each.
    let text = "@x;".repeat(65);
    let report = analyze(&text);

    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    let refusal = match report.parser().termination() {
        CssParserTermination::ResourceLimit(refusal) => refusal,
        other => panic!("expected parser refusal, got {other:?}"),
    };
    assert_eq!(
        refusal.kind(),
        CssResourceKind::Parser(CssParserResourceKind::UnsupportedRegions)
    );
    assert_eq!(report.parser().unsupported().len(), 64);
    assert_counts_match_records(&report);
    for (index, record) in report.parser().unsupported().iter().enumerate() {
        assert_eq!(range(record.region()), (index * 3, index * 3 + 3));
        assert!(record.region().range().end() <= refusal.location().range().start());
    }
    // Nothing is fabricated for the refused or unprocessed source.
    assert!(report.parser().recovery().is_empty());
    assert!(report.parser().discard().is_empty());
}

#[test]
fn upstream_tokenizer_refusal_exposes_no_fabricated_records() {
    let text = "@x;".repeat(11_000);
    assert!(text.len() > TOKENIZER_SOURCE_BYTES);
    let report = analyze(&text);

    assert_eq!(report.parser().completion(), CssStageCompletion::Incomplete);
    assert!(report.parser().recovery().is_empty());
    assert!(report.parser().unsupported().is_empty());
    assert!(report.parser().discard().is_empty());
}
