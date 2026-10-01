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
