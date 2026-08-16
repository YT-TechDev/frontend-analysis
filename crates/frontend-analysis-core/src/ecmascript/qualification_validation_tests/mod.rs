#![allow(dead_code)]

mod escaped_identifier;
mod focused;
mod gold;
mod inventory;
mod model;
mod test262;

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};
use focused::FOCUSED_RULE_EVIDENCE;
use gold::fixtures;
use inventory::{
    CONTAINERS, RULE_UNITS, RuleUnitKind, active_rule_container_count,
    aggregate_gold_coverage_complete, aggregate_rule_expansion_complete, rule_units_by_container,
    validate_inventory,
};
use model::{
    ECMA_262_EDITION, ECMA_262_SNAPSHOT, ExpectedProcessing, GoldRange, ImplementationCoverage,
    REQUIRED_DIMENSIONS, RequestApplicability, RequestContext, SyntheticKind, TEST262_REVISION,
    UNICODE_VERSION, ValidationDimension,
};
use test262::{
    EffectiveVariant, EvidenceInput, ExclusionReason, FrontmatterIdentity, MaterializedSource,
    Metadata, NegativePhase, NegativeType, PINNED_CASES, STRICT_PREFIX, select_effective_sources,
    select_evidence,
};

pub(super) fn gold_source(fixture_id: &str) -> Option<&'static str> {
    fixtures()
        .into_iter()
        .find(|fixture| fixture.id == fixture_id)
        .map(|fixture| fixture.source)
}

pub(super) fn gold_subject_range(fixture_id: &str) -> Option<(usize, usize)> {
    fixtures()
        .into_iter()
        .find(|fixture| fixture.id == fixture_id)
        .and_then(|fixture| fixture.subject.map(|range| (range.start, range.end)))
}

// Independent literal expectations recovered from the pinned ECMA extractor
// checkpoint. Do not derive this table from RULE_UNITS.
const EXPECTED_RULE_COUNTS: &[(&str, usize, usize)] = &[
    ("EE-01", 2, 0),
    ("EE-02", 1, 0),
    ("EE-03", 1, 0),
    ("EE-04", 8, 2),
    ("EE-05", 4, 0),
    ("EE-06", 1, 0),
    ("EE-07", 5, 0),
    ("EE-08", 1, 0),
    ("EE-09", 2, 0),
    ("EE-10", 2, 0),
    ("EE-11", 2, 0),
    ("EE-12", 4, 0),
    ("EE-13", 4, 0),
    ("EE-14", 2, 0),
    ("EE-15", 3, 0),
    ("EE-16", 0, 3),
    ("EE-17", 0, 1),
    ("EE-18", 0, 1),
    ("EE-19", 1, 1),
    ("EE-20", 5, 1),
    ("EE-21", 1, 0),
    ("EE-22", 1, 0),
    ("EE-23", 1, 1),
    ("EE-24", 2, 0),
    ("EE-25", 1, 0),
    ("EE-26", 3, 0),
    ("EE-27", 2, 0),
    ("EE-28", 13, 0),
    ("EE-29", 5, 0),
    ("EE-30", 5, 0),
    ("EE-31", 13, 0),
    ("EE-32", 15, 0),
    ("EE-33", 20, 0),
    ("EE-34", 13, 0),
    ("EE-35", 6, 0),
    ("EE-36", 8, 0),
    ("EE-37", 26, 0),
];

const EXPECTED_ACTIVE_RULE_UNITS: usize = 183;
const EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS: usize = 10;
const EXPECTED_TOTAL_RULE_UNITS: usize = 193;
const EXPECTED_CONTAINERS_WITH_ACTIVE_RULES: usize = 34;

#[test]
fn frozen_external_identities_are_exact() {
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(TEST262_REVISION, "3655e7464de3d52643ecddd4b5f9f4f3e7f62398");
}

#[test]
fn frozen_early_error_container_set_is_complete_and_unique() {
    validate_inventory(&CONTAINERS, RULE_UNITS).expect("frozen inventory structure must validate");
    assert_eq!(CONTAINERS.len(), 37);

    let ids: BTreeSet<_> = CONTAINERS.iter().map(|container| container.id).collect();
    assert_eq!(ids.len(), 37);

    let by_container = rule_units_by_container(RULE_UNITS);
    for container in CONTAINERS {
        assert!(
            by_container.contains_key(container.id),
            "{} must remain visible in rule-level coverage tracking",
            container.id
        );
    }
}

#[test]
fn audited_containers_distinguish_active_and_envelope_inactive_rules() {
    let by_container = rule_units_by_container(RULE_UNITS);

    for container_id in ["EE-16", "EE-17", "EE-18"] {
        let units = by_container
            .get(container_id)
            .unwrap_or_else(|| panic!("{container_id} must remain in the audited inventory"));
        assert!(
            units
                .iter()
                .any(|unit| unit.kind == RuleUnitKind::EnvelopeInactiveRule),
            "{container_id} must retain explicit envelope-inactive audit evidence"
        );
        assert!(
            units
                .iter()
                .all(|unit| unit.kind != RuleUnitKind::NormativeRule),
            "{container_id} must not contribute an active rule in the frozen first envelope"
        );
        assert!(
            units
                .iter()
                .all(|unit| unit.kind != RuleUnitKind::ExpansionSentinel),
            "{container_id} applicability audit is complete and needs no expansion sentinel"
        );
    }

    assert_eq!(
        by_container["EE-16"]
            .iter()
            .filter(|unit| unit.kind == RuleUnitKind::EnvelopeInactiveRule)
            .count(),
        3,
        "the if-statement clause has three separately audited labelled-function bullets"
    );
    assert_eq!(active_rule_container_count(RULE_UNITS), 34);
}

#[test]
fn pinned_rule_expansion_is_complete_but_executable_gold_remains_fail_closed() {
    assert!(
        aggregate_rule_expansion_complete(RULE_UNITS),
        "all audited active and envelope-inactive rules must be explicit"
    );
    assert!(
        !aggregate_gold_coverage_complete(RULE_UNITS),
        "initial executable gold intentionally covers only a subset of active rules"
    );
}

#[test]
fn frozen_rule_counts_match_the_independent_per_container_universe() {
    assert_eq!(EXPECTED_RULE_COUNTS.len(), 37);

    let expected_container_ids: BTreeSet<_> = EXPECTED_RULE_COUNTS
        .iter()
        .map(|(container_id, _, _)| *container_id)
        .collect();
    let by_container = rule_units_by_container(RULE_UNITS);
    let actual_container_ids: BTreeSet<_> = by_container.keys().copied().collect();
    assert_eq!(actual_container_ids, expected_container_ids);

    let expected_active: usize = EXPECTED_RULE_COUNTS
        .iter()
        .map(|(_, active, _)| active)
        .sum();
    let expected_inactive: usize = EXPECTED_RULE_COUNTS
        .iter()
        .map(|(_, _, inactive)| inactive)
        .sum();
    let expected_total: usize = EXPECTED_RULE_COUNTS
        .iter()
        .map(|(_, active, inactive)| active + inactive)
        .sum();
    let expected_active_containers = EXPECTED_RULE_COUNTS
        .iter()
        .filter(|(_, active, _)| *active > 0)
        .count();

    assert_eq!(expected_active, EXPECTED_ACTIVE_RULE_UNITS);
    assert_eq!(expected_inactive, EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS);
    assert_eq!(expected_total, EXPECTED_TOTAL_RULE_UNITS);
    assert_eq!(
        expected_active_containers,
        EXPECTED_CONTAINERS_WITH_ACTIVE_RULES
    );

    let mut actual_active = 0;
    let mut actual_inactive = 0;
    let mut actual_sentinels = 0;

    for &(container_id, expected_active, expected_inactive) in EXPECTED_RULE_COUNTS {
        let units = by_container
            .get(container_id)
            .unwrap_or_else(|| panic!("{container_id} must remain inventoried"));
        let active = units
            .iter()
            .filter(|unit| unit.kind == RuleUnitKind::NormativeRule)
            .count();
        let inactive = units
            .iter()
            .filter(|unit| unit.kind == RuleUnitKind::EnvelopeInactiveRule)
            .count();
        let sentinels = units
            .iter()
            .filter(|unit| unit.kind == RuleUnitKind::ExpansionSentinel)
            .count();

        assert_eq!(active, expected_active, "{container_id} active count");
        assert_eq!(
            inactive, expected_inactive,
            "{container_id} envelope-inactive count"
        );
        assert_eq!(sentinels, 0, "{container_id} expansion sentinels");
        assert_eq!(
            units.len(),
            expected_active + expected_inactive,
            "{container_id} total count"
        );

        let expected_rule_ids: BTreeSet<_> = (1..=expected_active)
            .map(|index| format!("{container_id}-R{index:02}"))
            .chain((1..=expected_inactive).map(|index| format!("{container_id}-I{index:02}")))
            .collect();
        let actual_rule_ids: BTreeSet<_> = units.iter().map(|unit| unit.id.to_owned()).collect();
        assert_eq!(
            actual_rule_ids, expected_rule_ids,
            "{container_id} rule identity shape"
        );

        actual_active += active;
        actual_inactive += inactive;
        actual_sentinels += sentinels;
    }

    assert_eq!(actual_active, EXPECTED_ACTIVE_RULE_UNITS);
    assert_eq!(actual_inactive, EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS);
    assert_eq!(actual_active + actual_inactive, EXPECTED_TOTAL_RULE_UNITS);
    assert_eq!(actual_sentinels, 0);
    assert_eq!(
        active_rule_container_count(RULE_UNITS),
        EXPECTED_CONTAINERS_WITH_ACTIVE_RULES
    );
}

#[test]
fn missing_container_fails_closed() {
    let truncated = &CONTAINERS[..CONTAINERS.len() - 1];
    assert!(validate_inventory(truncated, RULE_UNITS).is_err());
}

#[test]
fn inventoried_gold_references_resolve_and_uncovered_rules_remain_explicit() {
    let fixture_ids: BTreeSet<_> = fixtures().into_iter().map(|fixture| fixture.id).collect();

    for rule_unit in RULE_UNITS {
        for fixture_id in rule_unit.gold_fixture_ids {
            assert!(
                fixture_ids.contains(fixture_id),
                "{} references missing gold fixture {}",
                rule_unit.id,
                fixture_id
            );
        }

        if rule_unit.kind == RuleUnitKind::EnvelopeInactiveRule {
            assert!(
                rule_unit.gold_fixture_ids.is_empty(),
                "inactive rule {} must remain applicability evidence, not active conformance coverage",
                rule_unit.id
            );
        }
    }

    let uncovered_active = RULE_UNITS
        .iter()
        .filter(|rule_unit| {
            rule_unit.kind == RuleUnitKind::NormativeRule && rule_unit.gold_fixture_ids.is_empty()
        })
        .count();
    assert!(
        uncovered_active > 0,
        "the initial foundation must not pretend broad rule-level gold is already complete"
    );
}

#[test]
fn gold_fixture_ids_ranges_and_dimensions_are_self_consistent() {
    let fixtures = fixtures();
    assert_eq!(fixtures.len(), 46);
    let mut ids = BTreeSet::new();

    for fixture in &fixtures {
        assert!(
            ids.insert(fixture.id),
            "duplicate fixture id: {}",
            fixture.id
        );
        if let Some(subject) = fixture.subject {
            assert!(
                subject.is_valid_for(fixture.source),
                "invalid UTF-8 byte range in {}",
                fixture.id
            );
        }
        assert!(
            !fixture.dimensions.is_empty(),
            "{} must state at least one validation dimension",
            fixture.id
        );
    }

    let observed_dimensions: BTreeSet<_> = fixtures
        .iter()
        .flat_map(|fixture| fixture.dimensions.iter().copied())
        .collect();

    for required in REQUIRED_DIMENSIONS {
        if matches!(required, ValidationDimension::Test262Challenge) {
            // Test262 is exercised by dedicated transformation/selection tests.
            continue;
        }
        assert!(
            observed_dimensions.contains(&required),
            "initial gold foundation must visibly own {required:?}"
        );
    }
}

#[test]
fn core_source_contract_validates_multibyte_gold_range() {
    let source = SourceText::new(SourceId::new(197), "let π = 1;".to_owned());
    let anchor = source.anchor(4, 6).expect("π occupies UTF-8 bytes 4..6");

    assert_eq!(anchor.source_id(), SourceId::new(197));
    assert_eq!(anchor.range().start(), 4);
    assert_eq!(anchor.range().end(), 6);
    assert_eq!(anchor.fragment(), "π");
}

#[test]
fn synthetic_gold_never_fabricates_an_authored_range() {
    let fixture = fixtures()
        .into_iter()
        .find(|fixture| fixture.id == "JS-GOLD-ASI-NO-FABRICATED-RANGE-001")
        .expect("ASI provenance fixture must exist");

    assert_eq!(fixture.synthetic.len(), 1);
    assert_eq!(
        fixture.synthetic[0].kind,
        SyntheticKind::AutomaticSemicolonInsertion
    );
    assert_eq!(fixture.synthetic[0].authored_range, None);
}

#[test]
fn valid_gold_can_remain_implementation_pending_without_becoming_rejected() {
    let fixture = fixtures()
        .into_iter()
        .find(|fixture| fixture.id == "JS-GOLD-SCRIPT-VALID-001")
        .expect("baseline valid fixture must exist");

    assert_eq!(fixture.processing, ExpectedProcessing::Complete);
    assert_eq!(fixture.applicability, RequestApplicability::Supported);
    assert_eq!(
        fixture.implementation_coverage,
        ImplementationCoverage::Pending
    );
    assert_eq!(
        fixture.qualification,
        Some(model::ExpectedQualification::Qualified)
    );
}

#[test]
fn incomplete_processing_never_fabricates_a_whole_source_verdict() {
    for fixture_id in [
        "JS-GOLD-LIFECYCLE-RESOURCE-LIMIT-001",
        "JS-GOLD-LIFECYCLE-INTERNAL-FAILURE-001",
    ] {
        let fixture = fixtures()
            .into_iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("{fixture_id} must exist"));
        assert_eq!(fixture.applicability, RequestApplicability::Supported);
        assert_eq!(fixture.qualification, None);
        assert_ne!(fixture.processing, ExpectedProcessing::Complete);
    }
}

#[test]
fn unsupported_requests_are_inapplicable_without_a_qualification_verdict() {
    let expected = [
        (
            "JS-GOLD-UNSUPPORTED-MODULE-GOAL-001",
            RequestContext::Module,
            RequestApplicability::UnsupportedGoal,
        ),
        (
            "JS-GOLD-UNSUPPORTED-DIRECT-EVAL-001",
            RequestContext::DirectEval,
            RequestApplicability::UnsupportedSourceContext,
        ),
        (
            "JS-GOLD-UNSUPPORTED-FUNCTION-CONSTRUCTOR-001",
            RequestContext::FunctionConstructor,
            RequestApplicability::UnsupportedSourceContext,
        ),
    ];
    let fixtures = fixtures();

    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.applicability != RequestApplicability::Supported)
            .count(),
        expected.len()
    );

    for (fixture_id, request_context, applicability) in expected {
        let fixture = fixtures
            .iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("{fixture_id} must exist"));
        assert_eq!(fixture.request_context, request_context);
        assert_eq!(fixture.applicability, applicability);
        assert_eq!(fixture.qualification, None);
        assert_eq!(fixture.processing, ExpectedProcessing::Complete);
        assert_eq!(
            fixture.implementation_coverage,
            ImplementationCoverage::Pending
        );
    }
}

#[test]
fn ordinary_test262_script_has_non_strict_and_exact_strict_variants() {
    let source = "var x = 1;\r\n";
    let selected = select_effective_sources(source, Metadata::ordinary_parse_negative(), true)
        .expect("ordinary mapped Script parse-negative evidence must be selectable");

    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].variant, EffectiveVariant::NonStrict);
    assert_eq!(selected[0].source, source);
    assert_eq!(selected[1].variant, EffectiveVariant::Strict);
    assert_eq!(selected[1].source, format!("{STRICT_PREFIX}{source}"));
    assert!(selected[1].source.ends_with("\r\n"));
}

#[test]
fn selected_test262_evidence_preserves_pinned_identity_and_mapping() {
    let input = EvidenceInput {
        revision: TEST262_REVISION,
        path: "test/language/example.js",
        source_blob_sha: "test-blob",
        source: "var x = 1;\n",
        metadata: Metadata::ordinary_parse_negative(),
        normative_mapping: Some("EE-36-R01"),
    };

    let selected = select_evidence(input).expect("pinned mapped evidence must be selectable");
    assert_eq!(selected.len(), 2);
    for evidence in &selected {
        assert_eq!(evidence.revision, TEST262_REVISION);
        assert_eq!(evidence.path, "test/language/example.js");
        assert_eq!(evidence.source_blob_sha, "test-blob");
        assert_eq!(evidence.original_source, "var x = 1;\n");
        assert_eq!(evidence.normative_mapping, "EE-36-R01");
        assert_eq!(evidence.metadata, input.metadata);
    }

    let wrong_revision = EvidenceInput {
        revision: "moving-main",
        ..input
    };
    assert_eq!(
        select_evidence(wrong_revision),
        Err(ExclusionReason::WrongRevision)
    );

    let missing_mapping = EvidenceInput {
        normative_mapping: None,
        ..input
    };
    assert_eq!(
        select_evidence(missing_mapping),
        Err(ExclusionReason::MissingIndependentNormativeMapping)
    );
}

#[test]
fn pinned_test262_seed_manifest_preserves_real_revision_path_blob_and_mapping() {
    assert_eq!(PINNED_CASES.len(), 4);

    let expected = [
        (
            "test/language/statements/with/strict-script.js",
            "9514530c03705c8461a7aa42aeac52a0c027ed26",
            "EE-23-R01",
            1,
        ),
        (
            "test/language/statements/class/syntax/early-errors/class-definition-evaluation-scriptbody-duplicate-binding.js",
            "c5f3dff7cc6e23dfd729954698301e156a76985b",
            "EE-36-R01",
            2,
        ),
        (
            "test/language/global-code/new.target.js",
            "e8c55b1ce90046bda2201a51a49662a24b845430",
            "EE-36-R04",
            2,
        ),
        (
            "test/language/literals/regexp/early-err-modifiers-code-point-repeat-i-1.js",
            "055cebc72a993126baf3517420fbc85d929d1a49",
            "EE-37-R04",
            2,
        ),
    ];

    for (case, (path, blob_sha, mapping, variant_count)) in PINNED_CASES.iter().zip(expected) {
        assert!(case.path.starts_with("test/language/"));
        assert_eq!(case.path, path);
        assert_eq!(case.source_blob_sha, blob_sha);
        assert_eq!(case.normative_mapping, mapping);

        let mapped_rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == mapping)
            .unwrap_or_else(|| panic!("{path} maps to missing rule {mapping}"));
        assert_eq!(
            mapped_rule.kind,
            RuleUnitKind::NormativeRule,
            "{path} must map only to an active first-envelope rule"
        );

        let materialized_source =
            "/* exact source bytes supplied by the evidence materializer */\n";
        let selected = case
            .materialize(MaterializedSource {
                revision: TEST262_REVISION,
                path,
                source_blob_sha: blob_sha,
                source: materialized_source,
                frontmatter: case.frontmatter,
            })
            .unwrap_or_else(|reason| panic!("{path} must remain selectable: {reason:?}"));
        assert_eq!(selected.len(), variant_count);
        for evidence in selected {
            assert_eq!(evidence.revision, TEST262_REVISION);
            assert_eq!(evidence.path, path);
            assert_eq!(evidence.source_blob_sha, blob_sha);
            assert_eq!(evidence.original_source, materialized_source);
            assert_eq!(evidence.normative_mapping, mapping);
        }
    }

    let regexp_modifiers = PINNED_CASES[3];
    assert_eq!(
        regexp_modifiers.frontmatter.esid,
        Some("sec-patterns-static-semantics-early-errors")
    );
    assert_eq!(regexp_modifiers.frontmatter.features, ["regexp-modifiers"]);
}

#[test]
fn pinned_test262_materialization_rejects_identity_mismatch_before_transformation() {
    let case = PINNED_CASES[3];
    let materialized = MaterializedSource {
        revision: TEST262_REVISION,
        path: case.path,
        source_blob_sha: case.source_blob_sha,
        source: "/(?ii:a)/;\r\n",
        frontmatter: case.frontmatter,
    };

    assert_eq!(
        case.materialize(MaterializedSource {
            source_blob_sha: "wrong-blob",
            ..materialized
        }),
        Err(ExclusionReason::SourceBlobMismatch)
    );
    assert_eq!(
        case.materialize(MaterializedSource {
            frontmatter: FrontmatterIdentity {
                metadata: Metadata {
                    no_strict: true,
                    ..case.frontmatter.metadata
                },
                ..case.frontmatter
            },
            ..materialized
        }),
        Err(ExclusionReason::RelevantFrontmatterMismatch)
    );
    assert_eq!(
        case.materialize(MaterializedSource {
            revision: "moving-main",
            ..materialized
        }),
        Err(ExclusionReason::WrongRevision)
    );
    assert_eq!(
        case.materialize(MaterializedSource {
            path: "test/language/literals/regexp/another-test.js",
            ..materialized
        }),
        Err(ExclusionReason::PathMismatch)
    );
}

#[test]
fn test262_first_profile_rejects_non_language_and_fixture_paths() {
    let base = EvidenceInput {
        revision: TEST262_REVISION,
        path: "test/language/example.js",
        source_blob_sha: "test-blob",
        source: "x;\n",
        metadata: Metadata::ordinary_parse_negative(),
        normative_mapping: Some("EE-01-R01"),
    };

    assert_eq!(
        select_evidence(EvidenceInput {
            path: "test/built-ins/example.js",
            ..base
        }),
        Err(ExclusionReason::OutsideLanguageTests)
    );
    assert_eq!(
        select_evidence(EvidenceInput {
            path: "test/language/example_FIXTURE.js",
            ..base
        }),
        Err(ExclusionReason::FixtureFile)
    );
}

#[test]
fn test262_manifest_does_not_vendor_selected_source_bodies() {
    let source = include_str!("test262.rs");
    for forbidden in [
        concat!("$D", "ONOTEVALUATE"),
        concat!("All rights", " reserved."),
        concat!("with ({})", " {}"),
        concat!("new.", "target;"),
    ] {
        assert!(
            !source.contains(forbidden),
            "manifest-only Test262 evidence must not vendor source body marker {forbidden}"
        );
    }
}

#[test]
fn test262_strict_flags_and_raw_are_deterministic() {
    let base = Metadata::ordinary_parse_negative();

    let only_strict = Metadata {
        only_strict: true,
        ..base
    };
    let selected = select_effective_sources("x;", only_strict, true).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].variant, EffectiveVariant::Strict);
    assert_eq!(selected[0].source, "\"use strict\";\nx;");

    let no_strict = Metadata {
        no_strict: true,
        ..base
    };
    let selected = select_effective_sources("x;", no_strict, true).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].variant, EffectiveVariant::NonStrict);
    assert_eq!(selected[0].source, "x;");

    let raw = Metadata { raw: true, ..base };
    let selected = select_effective_sources("'use strict'\n[0]", raw, true).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].variant, EffectiveVariant::Raw);
    assert_eq!(selected[0].source, "'use strict'\n[0]");
}

#[test]
fn test262_invalid_metadata_combinations_fail_closed() {
    let base = Metadata::ordinary_parse_negative();

    for metadata in [
        Metadata {
            only_strict: true,
            no_strict: true,
            ..base
        },
        Metadata {
            raw: true,
            only_strict: true,
            ..base
        },
        Metadata {
            raw: true,
            no_strict: true,
            ..base
        },
    ] {
        assert_eq!(
            select_effective_sources("x;", metadata, true),
            Err(ExclusionReason::InvalidFlagCombination)
        );
    }

    assert_eq!(
        select_effective_sources(
            "x;",
            Metadata {
                negative_type: None,
                ..base
            },
            true
        ),
        Err(ExclusionReason::InvalidNegativeMetadata)
    );
    assert_eq!(
        select_effective_sources(
            "x;",
            Metadata {
                negative_phase: None,
                negative_type: Some(NegativeType::SyntaxError),
                ..base
            },
            true
        ),
        Err(ExclusionReason::InvalidNegativeMetadata)
    );
}

#[test]
fn test262_non_static_or_out_of_profile_evidence_is_excluded() {
    let base = Metadata::ordinary_parse_negative();

    assert_eq!(
        select_effective_sources(
            "export {};",
            Metadata {
                module: true,
                ..base
            },
            true
        ),
        Err(ExclusionReason::Module)
    );
    assert_eq!(
        select_effective_sources(
            "x;",
            Metadata {
                negative_phase: Some(NegativePhase::Resolution),
                ..base
            },
            true
        ),
        Err(ExclusionReason::NotParseNegative)
    );
    assert_eq!(
        select_effective_sources(
            "x;",
            Metadata {
                negative_phase: Some(NegativePhase::Runtime),
                ..base
            },
            true
        ),
        Err(ExclusionReason::NotParseNegative)
    );
    assert_eq!(
        select_effective_sources(
            "x;",
            Metadata {
                proposal_or_post_es2026: true,
                ..base
            },
            true
        ),
        Err(ExclusionReason::ProposalOrPostEs2026)
    );
    assert_eq!(
        select_effective_sources("x;", base, false),
        Err(ExclusionReason::MissingIndependentNormativeMapping)
    );
}

#[test]
fn second_lexical_slice_focused_evidence_is_candidate_independent_and_consistent() {
    let fixtures = fixtures();
    let mut focused_fixture_ids = BTreeSet::new();

    for evidence in FOCUSED_RULE_EVIDENCE {
        assert!(
            focused_fixture_ids.insert(evidence.fixture_id),
            "focused evidence must have one primary ownership record per fixture"
        );

        let fixture = fixtures
            .iter()
            .find(|fixture| fixture.id == evidence.fixture_id)
            .unwrap_or_else(|| {
                panic!(
                    "focused evidence references missing fixture {}",
                    evidence.fixture_id
                )
            });
        assert_eq!(
            fixture.qualification,
            Some(model::ExpectedQualification::StaticSemanticsRejected),
            "focused Early Error evidence must refer to whole-source static rejection gold"
        );
        assert_eq!(fixture.subject, Some(evidence.primary_subject));
        assert!(evidence.primary_subject.is_valid_for(fixture.source));
        for supporting in evidence.supporting_subjects {
            assert!(supporting.is_valid_for(fixture.source));
        }

        let primary_rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == evidence.primary_rule_id)
            .unwrap_or_else(|| panic!("missing focused primary rule {}", evidence.primary_rule_id));
        assert_eq!(primary_rule.kind, RuleUnitKind::NormativeRule);
        assert!(
            primary_rule.gold_fixture_ids.contains(&evidence.fixture_id),
            "{} must map its focused primary fixture {}",
            evidence.primary_rule_id,
            evidence.fixture_id
        );

        let mut co_triggers = BTreeSet::new();
        for co_trigger in evidence.co_trigger_rule_ids {
            assert_ne!(*co_trigger, evidence.primary_rule_id);
            assert!(
                co_triggers.insert(*co_trigger),
                "co-trigger identities must be unique for {}",
                evidence.fixture_id
            );
            let rule = RULE_UNITS
                .iter()
                .find(|rule| rule.id == *co_trigger)
                .unwrap_or_else(|| panic!("missing focused co-trigger rule {co_trigger}"));
            assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
        }
    }
}

#[test]
fn r02_and_r03_inventory_coverage_requires_matching_focused_primary_evidence() {
    for rule_id in ["EE-15-R02", "EE-15-R03"] {
        let rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == rule_id)
            .unwrap_or_else(|| panic!("{rule_id} must remain inventoried"));
        let inventory_fixtures: BTreeSet<_> = rule.gold_fixture_ids.iter().copied().collect();
        let focused_fixtures: BTreeSet<_> = FOCUSED_RULE_EVIDENCE
            .iter()
            .filter(|evidence| evidence.primary_rule_id == rule_id)
            .map(|evidence| evidence.fixture_id)
            .collect();
        assert_eq!(
            inventory_fixtures, focused_fixtures,
            "{rule_id} inventory coverage must not outrun focused rule ownership evidence"
        );
    }
}

#[test]
fn second_lexical_slice_focused_evidence_pins_ownership_precedence_and_ranges() {
    let evidence = |fixture_id| {
        FOCUSED_RULE_EVIDENCE
            .iter()
            .find(|evidence| evidence.fixture_id == fixture_id)
            .unwrap_or_else(|| panic!("missing focused evidence for {fixture_id}"))
    };

    let r02 = evidence("JS-GOLD-LEXDECL-DUPBOUNDNAMES-001");
    assert_eq!(r02.primary_rule_id, "EE-15-R02");
    assert_eq!(r02.primary_subject, GoldRange::new(7, 8));
    assert_eq!(r02.supporting_subjects, &[GoldRange::new(4, 5)]);
    assert_eq!(r02.co_trigger_rule_ids, &["EE-36-R01"]);

    let r03 = evidence("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001");
    assert_eq!(r03.primary_rule_id, "EE-15-R03");
    assert_eq!(r03.primary_subject, GoldRange::new(6, 7));
    assert!(r03.supporting_subjects.is_empty());
    assert!(r03.co_trigger_rule_ids.is_empty());

    let ee36 = evidence("JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001");
    assert_eq!(ee36.primary_rule_id, "EE-36-R01");
    assert_eq!(ee36.primary_subject, GoldRange::new(14, 15));
    assert_eq!(ee36.supporting_subjects, &[GoldRange::new(7, 8)]);
    assert!(ee36.co_trigger_rule_ids.is_empty());

    let r01_before_r03 = evidence("JS-GOLD-LEXDECL-CONST-LET-MISSING-INIT-001");
    assert_eq!(r01_before_r03.primary_rule_id, "EE-15-R01");
    assert_eq!(r01_before_r03.primary_subject, GoldRange::new(6, 9));
    assert_eq!(r01_before_r03.co_trigger_rule_ids, &["EE-15-R03"]);

    let r02_before_r03_ee36 = evidence("JS-GOLD-LEXDECL-CONST-DUP-MISSING-INIT-001");
    assert_eq!(r02_before_r03_ee36.primary_rule_id, "EE-15-R02");
    assert_eq!(r02_before_r03_ee36.primary_subject, GoldRange::new(13, 14));
    assert_eq!(
        r02_before_r03_ee36.supporting_subjects,
        &[GoldRange::new(6, 7)]
    );
    assert_eq!(
        r02_before_r03_ee36.co_trigger_rule_ids,
        &["EE-15-R03", "EE-36-R01"]
    );
}

#[test]
fn ee_15_r03_retains_is_constant_declaration_dependency() {
    let rule = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-15-R03")
        .expect("EE-15-R03 must remain inventoried");
    assert_eq!(rule.dependencies, &["IsConstantDeclaration"]);
    assert!(
        rule.gold_fixture_ids
            .contains(&"JS-GOLD-LEXDECL-CONST-MISSING-INIT-001")
    );
}

#[test]
fn second_lexical_positive_gold_stays_whole_standard_qualified_and_pending() {
    for fixture_id in [
        "JS-GOLD-LEXDECL-CONST-VALID-001",
        "JS-GOLD-LEXDECL-MULTIBIND-VALID-001",
        "JS-GOLD-LEXDECL-CONST-MULTIBIND-VALID-001",
    ] {
        let fixture = fixtures()
            .into_iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("{fixture_id} must exist"));
        assert_eq!(fixture.processing, ExpectedProcessing::Complete);
        assert_eq!(fixture.applicability, RequestApplicability::Supported);
        assert_eq!(
            fixture.qualification,
            Some(model::ExpectedQualification::Qualified)
        );
        assert_eq!(
            fixture.implementation_coverage,
            ImplementationCoverage::Pending
        );
    }
}

#[test]
fn initializer_presence_and_malformed_syntax_remain_distinct_from_missing_initializer() {
    let fixture = |fixture_id| {
        fixtures()
            .into_iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("{fixture_id} must exist"))
    };

    assert_eq!(
        fixture("JS-GOLD-LEXDECL-CONST-IDENTIFIER-INIT-001").qualification,
        Some(model::ExpectedQualification::Qualified)
    );
    assert_eq!(
        fixture("JS-GOLD-LEXDECL-CONST-MALFORMED-INIT-001").qualification,
        Some(model::ExpectedQualification::SyntaxRejected)
    );
    let missing = fixture("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001");
    assert_eq!(
        missing.qualification,
        Some(model::ExpectedQualification::StaticSemanticsRejected)
    );
    assert_eq!(missing.subject, Some(GoldRange::new(6, 7)));
    assert!(missing.synthetic.is_empty());
}

#[test]
fn widened_binding_shape_preserves_selected_ee04_non_trigger_context() {
    for fixture_id in [
        "JS-GOLD-LEXDECL-EE04-AWAIT-YIELD-001",
        "JS-GOLD-LEXDECL-EE04-FUTURE-RESERVED-001",
        "JS-GOLD-LEXDECL-EE04-EVAL-ARGUMENTS-001",
    ] {
        let fixture = fixtures()
            .into_iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("{fixture_id} must exist"));
        assert_eq!(
            fixture.request_context,
            RequestContext::ScriptIndependentSource
        );
        assert_eq!(
            fixture.qualification,
            Some(model::ExpectedQualification::Qualified)
        );
        assert_eq!(
            fixture.implementation_coverage,
            ImplementationCoverage::Pending
        );
        assert!(
            fixture
                .dimensions
                .contains(&ValidationDimension::EarlyErrorRule)
        );
        assert!(
            fixture
                .dimensions
                .contains(&ValidationDimension::ValidityDependency)
        );
    }
}

#[test]
fn multi_binding_name_identity_does_not_normalize_unicode() {
    let fixture = fixtures()
        .into_iter()
        .find(|fixture| fixture.id == "JS-GOLD-LEXDECL-MULTIBIND-CANONICAL-DISTINCT-001")
        .expect("canonical-distinct multi-binding fixture must exist");
    assert_eq!(
        fixture.qualification,
        Some(model::ExpectedQualification::Qualified)
    );

    let source = SourceText::new(SourceId::new(216), fixture.source.to_owned());
    let composed = source.anchor(4, 6).expect("é occupies UTF-8 bytes 4..6");
    let decomposed = source
        .anchor(8, 11)
        .expect("e + combining acute occupies UTF-8 bytes 8..11");
    assert_eq!(composed.fragment(), "é");
    assert_eq!(decomposed.fragment(), "é");
    assert_ne!(composed.fragment(), decomposed.fragment());
}

#[test]
fn validation_gold_source_does_not_import_ecmascript_production_contracts() {
    for (name, source) in [
        ("gold", include_str!("gold.rs")),
        ("focused", include_str!("focused.rs")),
    ] {
        for forbidden in [
            "frontend_analysis_core::ecmascript",
            "crate::ecmascript",
            "css::selector",
            "SelectedStaticSemanticsOutcome",
            "FirstLexicalSliceOutcome",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} validation authority must remain candidate-independent: found {forbidden}"
            );
        }
    }
}
