//! Candidate-independent #172 gate for the Core-integrated CSS context graph.

use crate::css::analysis::analyze_css_source;
use crate::css::parser::producer::run as run_parser;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::{CssTokenizerLimits, CssTokenizerResourceKind};
use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};
use crate::{SourceId, SourceText};

use super::context_candidate::{CONTRACT_ONLY_FIXTURE_IDS, assert_matches_context_gold};
use super::context_fixtures::independent_context_fixtures;
use super::context_gold::{ContextGoldFixture, ContextGoldResourceKind};
use super::descriptor_candidate::{
    DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS, assert_matches_descriptor_gold,
};
use super::descriptor_fixtures::independent_descriptor_fixtures;
use super::descriptor_gold::{DescriptorGoldFixture, DescriptorGoldResourceKind};
use super::group_context_fixtures::independent_group_context_fixtures;
use super::keyframe_candidate::assert_matches_keyframe_gold;
use super::keyframe_fixtures::independent_keyframe_fixtures;
use super::page_candidate::{assert_matches_page_gold, tight_limits_for as page_tight_limits_for};
use super::page_fixtures::independent_page_fixtures;
use super::parser_candidate::{
    canonical_signature, generous_parser_limits, generous_tokenizer_limits,
};

fn tokenizer_limits() -> CssTokenizerLimits {
    generous_tokenizer_limits()
}

fn context_limits(fixture: &ContextGoldFixture) -> CssParserLimits {
    let Some(expectation) = fixture.resource_expectation else {
        return generous_parser_limits();
    };
    let mut peak_context = 1 << 12;
    let mut unsupported = 1 << 16;
    let mut contexts = 1 << 16;
    match expectation.kind {
        ContextGoldResourceKind::PeakContextDepth => peak_context = expectation.limit,
        ContextGoldResourceKind::ContextRecords => contexts = expectation.limit,
        ContextGoldResourceKind::UnsupportedRegions => unsupported = expectation.limit,
    }
    CssParserLimits::new(
        1 << 20,
        1 << 12,
        peak_context,
        1 << 16,
        1 << 16,
        1 << 16,
        unsupported,
        1 << 16,
        contexts,
    )
    .unwrap()
}

fn descriptor_limits(fixture: &DescriptorGoldFixture) -> CssParserLimits {
    let Some(expectation) = fixture.resource_expectation else {
        return generous_parser_limits();
    };
    let mut algorithm = 1 << 20;
    let mut peak_context = 1 << 12;
    let mut declarations = 1 << 16;
    let mut unsupported = 1 << 16;
    let mut contexts = 1 << 16;
    match expectation.kind {
        DescriptorGoldResourceKind::AlgorithmSteps => algorithm = expectation.limit.max(1),
        DescriptorGoldResourceKind::PeakContextDepth => peak_context = expectation.limit,
        DescriptorGoldResourceKind::ContextRecords => contexts = expectation.limit,
        DescriptorGoldResourceKind::DeclarationOccurrences => declarations = expectation.limit,
        DescriptorGoldResourceKind::UnsupportedRegions => unsupported = expectation.limit,
    }
    CssParserLimits::new(
        algorithm,
        1 << 12,
        peak_context,
        declarations,
        1 << 16,
        1 << 16,
        unsupported,
        1 << 16,
        contexts,
    )
    .unwrap()
}

fn core_run(
    source_id: u64,
    source: &str,
    limits: CssParserLimits,
) -> crate::css::parser::result::CssParserRunResult {
    let text = SourceText::new(SourceId::new(source_id), source.to_owned());
    analyze_css_source(&text, tokenizer_limits(), limits)
        .unwrap_or_else(|error| panic!("Core context gate failure: {error:?}"))
}

#[test]
fn core_matches_fixed_context_gold_for_every_source_driven_fixture() {
    for fixture in independent_context_fixtures() {
        if CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id)
            || fixture.id.contains("UPSTREAM-INCOMPLETE")
        {
            continue;
        }
        let result = core_run(fixture.source_id, fixture.source, context_limits(&fixture));
        assert_matches_context_gold(&fixture, &result);
    }
}

#[test]
fn core_matches_group_context_gold_for_all_28_fixtures() {
    let fixtures = independent_group_context_fixtures();
    assert_eq!(fixtures.len(), 28);
    for fixture in fixtures {
        let result = core_run(fixture.source_id, fixture.source, context_limits(&fixture));
        assert_matches_context_gold(&fixture, &result);
    }
}

#[test]
fn core_matches_descriptor_gold_for_every_executable_fixture() {
    for fixture in independent_descriptor_fixtures() {
        if DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
            continue;
        }
        let result = core_run(
            fixture.source_id,
            fixture.source,
            descriptor_limits(&fixture),
        );
        assert_matches_descriptor_gold(&fixture, &result);
    }
}

#[test]
fn core_matches_page_gold_for_all_fixtures() {
    let fixtures = independent_page_fixtures();
    for fixture in fixtures {
        let limits = fixture
            .resource_expectation
            .map(page_tight_limits_for)
            .unwrap_or_else(generous_parser_limits);
        let result = core_run(fixture.source_id, fixture.source, limits);
        assert_matches_page_gold(&fixture, &result);
    }
}

#[test]
fn core_matches_keyframe_gold_for_all_fixtures() {
    let fixtures = independent_keyframe_fixtures();
    assert_eq!(fixtures.len(), 10);
    for fixture in fixtures {
        let result = core_run(fixture.source_id, fixture.source, generous_parser_limits());
        assert_matches_keyframe_gold(&fixture, &result);
    }
}

#[test]
fn direct_pipeline_and_core_operation_remain_structurally_equivalent_for_context_matrix() {
    let sources = [
        "a{p:0;b{q:1;}r:2;}",
        "a{@media screen{p:0;b{q:1;}r:2;}}",
        "@font-face{font-family:x;src:y;}",
        "@page :first{size:auto;@top-center{content:'x';}}",
        "@keyframes spin{from{opacity:0;}50%,to{opacity:1;}}",
    ];
    for source in sources {
        let direct_source = SourceText::new(SourceId::new(172_500), source.to_owned());
        let tokenizer = run_tokenizer(&direct_source, tokenizer_limits()).unwrap();
        let direct = run_parser(&direct_source, tokenizer, generous_parser_limits()).unwrap();
        let core = core_run(172_500, source, generous_parser_limits());
        assert_eq!(
            canonical_signature(&direct),
            canonical_signature(&core),
            "{source}"
        );
    }
}

#[test]
fn core_context_matrix_is_repeat_deterministic_and_source_id_independent() {
    let sources = [
        "a{p:0;b{q:1;}r:2;}",
        "a{@media{p:0;}}",
        "@property --x{syntax:'*';inherits:false;}",
        "@page{size:auto;@bottom-center{content:'x';}}",
        "@keyframes x{from{x:y;}to{x:z;}}",
    ];
    for source in sources {
        let first = core_run(172_601, source, generous_parser_limits());
        let repeat = core_run(172_601, source, generous_parser_limits());
        let other_id = core_run(172_602, source, generous_parser_limits());
        assert_eq!(
            canonical_signature(&first),
            canonical_signature(&repeat),
            "repeat {source}"
        );
        assert_eq!(
            canonical_signature(&first),
            canonical_signature(&other_id),
            "SourceId {source}"
        );
    }
}

#[test]
fn real_tokenizer_incomplete_retains_entered_context_through_core() {
    let source = SourceText::new(SourceId::new(172_650), "a{p:v;}".to_owned());
    let tokenizer_limits =
        CssTokenizerLimits::new(1 << 20, 1 << 20, 2, 1 << 16, 1 << 20, 1 << 20).unwrap();
    let result = analyze_css_source(&source, tokenizer_limits, generous_parser_limits())
        .expect("real tokenizer incompleteness remains a successful partial parser result");

    assert_eq!(
        result.upstream_tokenizer_result().completion(),
        CssTokenizerCompletion::Incomplete
    );
    match result.upstream_tokenizer_result().termination() {
        CssTokenizerTermination::ResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssTokenizerResourceKind::LexicalItems);
            assert_eq!(evidence.limit(), 2);
            assert_eq!(evidence.attempted(), 3);
        }
        other => panic!("expected lexical-item resource stop, got {other:?}"),
    }
    assert_eq!(result.context_records().len(), 1);
    assert_eq!(
        result.context_records()[0].kind(),
        crate::css::parser::context::CssParserContextKind::QualifiedRuleBlock
    );
    assert!(matches!(
        result.context_records()[0].termination(),
        crate::css::parser::context::CssParserContextTermination::UpstreamTokenizerIncomplete { .. }
    ));
    assert!(matches!(
        result.termination(),
        crate::css::parser::result::CssParserTermination::UpstreamTokenizerIncomplete
    ));
}

#[test]
fn real_context_resource_limits_remain_successful_incomplete_core_results() {
    let source = "a{@media{p:v;}}";
    let limits = CssParserLimits::new(
        1 << 20,
        1 << 12,
        1,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
    )
    .unwrap();
    let result = core_run(172_700, source, limits);
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::PeakContextDepth),
        1
    );
    assert!(matches!(
        result.termination(),
        crate::css::parser::result::CssParserTermination::ParserResourceLimit(_)
    ));
}
