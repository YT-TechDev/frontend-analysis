#[path = "qualification_validation_tests/gold.rs"]
mod gold;
#[path = "qualification_validation_tests/model.rs"]
mod model;

use super::qualification::{
    EvidenceOrigin, EvidenceSourceMismatch, EvidenceSubject, FIRST_QUALIFICATION_ENVELOPE,
    ProcessingStatus, QualificationOutcome, QualificationVerdictKind, RejectionFamily,
    test_complete_qualification_witness,
};
use crate::{SourceId, SourceText};
use model::{
    ExpectedProcessing, ExpectedQualification, ImplementationCoverage, RequestApplicability,
};

fn gold_fixture(id: &str) -> model::GoldFixture {
    gold::fixtures()
        .into_iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing candidate-independent gold fixture {id}"))
}

#[test]
fn fixed_envelope_identity_matches_candidate_independent_validation_authority() {
    let envelope = FIRST_QUALIFICATION_ENVELOPE;

    assert_eq!(envelope.ecma_262_edition(), model::ECMA_262_EDITION);
    assert_eq!(envelope.ecma_262_snapshot(), model::ECMA_262_SNAPSHOT);
    assert_eq!(envelope.unicode_version(), model::UNICODE_VERSION);
    assert_eq!(envelope.source_authority(), "Core UTF-8 SourceText");
    assert_eq!(envelope.source_context(), "Independent Source Unit");
    assert_eq!(envelope.parse_goal(), "Script");
    assert!(envelope.includes_mandatory_legacy());
    assert!(!envelope.includes_normative_optional_source_static());
}

#[test]
fn normal_processing_completion_does_not_imply_qualified() {
    let source = SourceText::new(SourceId::new(199), "let x; let x;".to_owned());
    let anchor = source.anchor(11, 12).unwrap();
    let subject = EvidenceSubject::authored(&source, anchor).unwrap();
    let rejected = QualificationOutcome::static_semantics_rejected(subject);
    let qualified = QualificationOutcome::qualified(test_complete_qualification_witness());

    assert_eq!(rejected.processing(), ProcessingStatus::Complete);
    assert_eq!(qualified.processing(), ProcessingStatus::Complete);
    assert_eq!(
        rejected.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );
    assert_eq!(
        qualified.verdict(),
        Some(QualificationVerdictKind::Qualified)
    );
    assert!(!rejected.equivalent_meaning(&qualified));
}

#[test]
fn resource_and_internal_incompleteness_never_fabricate_a_verdict() {
    let resource_gold = gold_fixture("JS-GOLD-LIFECYCLE-RESOURCE-LIMIT-001");
    let resource = QualificationOutcome::resource_limited();
    assert_eq!(
        resource_gold.processing,
        ExpectedProcessing::ResourceLimited
    );
    assert_eq!(resource_gold.qualification, None);
    assert_eq!(resource.processing(), ProcessingStatus::ResourceLimited);
    assert_eq!(resource.verdict(), None);
    assert!(resource.rejection_evidence().is_none());

    let internal_gold = gold_fixture("JS-GOLD-LIFECYCLE-INTERNAL-FAILURE-001");
    let internal = QualificationOutcome::internal_failure();
    assert_eq!(
        internal_gold.processing,
        ExpectedProcessing::InternalFailure
    );
    assert_eq!(internal_gold.qualification, None);
    assert_eq!(internal.processing(), ProcessingStatus::InternalFailure);
    assert_eq!(internal.verdict(), None);
    assert!(internal.rejection_evidence().is_none());
}

#[test]
fn implementation_pending_gold_remains_normatively_qualified_not_rejected() {
    let fixture = gold_fixture("JS-GOLD-SCRIPT-VALID-001");

    assert_eq!(fixture.applicability, RequestApplicability::Supported);
    assert_eq!(fixture.processing, ExpectedProcessing::Complete);
    assert_eq!(
        fixture.implementation_coverage,
        ImplementationCoverage::Pending
    );
    assert_eq!(
        fixture.qualification,
        Some(ExpectedQualification::Qualified)
    );

    let qualified = QualificationOutcome::qualified(test_complete_qualification_witness());
    assert_eq!(
        qualified.verdict(),
        Some(QualificationVerdictKind::Qualified)
    );
}

#[test]
fn authored_evidence_preserves_exact_source_identity_and_range() {
    let source = SourceText::new(SourceId::new(200), "let π = 1;".to_owned());
    let anchor = source.anchor(4, 6).unwrap();
    let subject = EvidenceSubject::authored(&source, anchor).unwrap();

    assert_eq!(subject.origin(), EvidenceOrigin::Authored);
    let retained = subject
        .authored_anchor()
        .expect("authored anchor must exist");
    assert_eq!(retained.source_id(), SourceId::new(200));
    assert_eq!(retained.range().start(), 4);
    assert_eq!(retained.range().end(), 6);
    assert_eq!(retained.fragment(), "π");
}

#[test]
fn authored_evidence_rejects_same_id_with_different_source_bytes() {
    let authoritative = SourceText::new(SourceId::new(201), "same".to_owned());
    let different = SourceText::new(SourceId::new(201), "diff".to_owned());
    let foreign_anchor = different.anchor(0, 4).unwrap();

    assert_eq!(
        EvidenceSubject::authored(&authoritative, foreign_anchor).unwrap_err(),
        EvidenceSourceMismatch
    );
}

#[test]
fn derived_evidence_never_fabricates_an_authored_range() {
    let subject = EvidenceSubject::derived();
    assert_eq!(subject.origin(), EvidenceOrigin::Derived);
    assert!(subject.authored_anchor().is_none());

    let rejected = QualificationOutcome::profile_boundary_rejected(subject);
    let evidence = rejected
        .rejection_evidence()
        .expect("profile rejection must retain evidence scope");
    assert_eq!(evidence.family(), RejectionFamily::ProfileBoundary);
    assert_eq!(evidence.subject().origin(), EvidenceOrigin::Derived);
    assert!(evidence.subject().authored_anchor().is_none());
}

#[test]
fn rejection_constructors_preserve_distinct_semantic_families() {
    let syntax = QualificationOutcome::syntax_rejected(EvidenceSubject::derived());
    assert_eq!(syntax.processing(), ProcessingStatus::Complete);
    assert_eq!(
        syntax.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    assert_eq!(
        syntax.rejection_evidence().unwrap().family(),
        RejectionFamily::Grammar
    );

    let static_semantics =
        QualificationOutcome::static_semantics_rejected(EvidenceSubject::derived());
    assert_eq!(
        static_semantics.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );
    assert_eq!(
        static_semantics.rejection_evidence().unwrap().family(),
        RejectionFamily::StaticSemantics
    );

    let profile = QualificationOutcome::profile_boundary_rejected(EvidenceSubject::derived());
    assert_eq!(
        profile.verdict(),
        Some(QualificationVerdictKind::ProfileBoundaryRejected)
    );
    assert_eq!(
        profile.rejection_evidence().unwrap().family(),
        RejectionFamily::ProfileBoundary
    );
}

#[test]
fn equivalent_meaning_is_independent_of_allocation_identity() {
    let first_source = SourceText::new(SourceId::new(202), "let x; let x;".to_owned());
    let second_source = SourceText::new(SourceId::new(202), "let x; let x;".to_owned());
    let first_subject =
        EvidenceSubject::authored(&first_source, first_source.anchor(11, 12).unwrap()).unwrap();
    let second_subject =
        EvidenceSubject::authored(&second_source, second_source.anchor(11, 12).unwrap()).unwrap();

    let first = QualificationOutcome::static_semantics_rejected(first_subject);
    let second = QualificationOutcome::static_semantics_rejected(second_subject);
    assert!(first.equivalent_meaning(&second));

    let different_source = SourceText::new(SourceId::new(202), "let y; let y;".to_owned());
    let different_subject =
        EvidenceSubject::authored(&different_source, different_source.anchor(11, 12).unwrap())
            .unwrap();
    let different = QualificationOutcome::static_semantics_rejected(different_subject);
    assert!(!first.equivalent_meaning(&different));
}

#[test]
fn every_outcome_recovers_the_same_fixed_envelope() {
    for outcome in [
        QualificationOutcome::resource_limited(),
        QualificationOutcome::internal_failure(),
        QualificationOutcome::syntax_rejected(EvidenceSubject::derived()),
        QualificationOutcome::static_semantics_rejected(EvidenceSubject::derived()),
        QualificationOutcome::profile_boundary_rejected(EvidenceSubject::derived()),
        QualificationOutcome::qualified(test_complete_qualification_witness()),
    ] {
        assert_eq!(outcome.envelope(), FIRST_QUALIFICATION_ENVELOPE);
    }
}

#[test]
fn qualification_contract_remains_crate_private() {
    let lib_source = include_str!("../lib.rs");
    assert!(!lib_source.contains("pub use ecmascript"));
    assert!(!lib_source.contains("QualificationOutcome"));

    let ecmascript_module = include_str!("mod.rs");
    assert!(ecmascript_module.contains("mod qualification;"));
    assert!(!ecmascript_module.contains("pub mod qualification;"));
}
