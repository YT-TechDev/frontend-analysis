//! Candidate-independent validation for the first selected Binding / Scope
//! source-relation capability accepted by Issue #270.
//!
//! This oracle owns expected source ranges, semantic identifier names, relation
//! targets, and prerequisite lifecycle independently from any future production
//! Binding / Scope implementation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const THIS_SOURCE: &str = include_str!("selected_binding_scope_validation_tests.rs");
const SELECTED_SLICE_COMPLETION_SOURCE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");
const ESCAPED_RESERVED_RHS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_reserved_identifier_initializer_validation_tests.rs"
);

const SOURCE_RELATION_BOUNDARY: &str = "SameSourceSelectedLexicalBinding != runtime ResolveBinding != Environment Record identity != GlobalDeclarationInstantiation success != TDZ safety != initializer evaluation success != initialized value != value flow";
const NO_MATCH_BOUNDARY: &str = "NoSameSourceSelectedLexicalBinding != runtime unbound != ReferenceError != SyntaxRejected != StaticSemanticsRejected";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedAnchor {
    start: usize,
    end: usize,
    fragment: &'static str,
}

impl ExpectedAnchor {
    const fn new(start: usize, end: usize, fragment: &'static str) -> Self {
        Self {
            start,
            end,
            fragment,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedTarget {
    SameSourceSelectedLexicalBinding(ExpectedAnchor),
    NoSameSourceSelectedLexicalBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedRelation {
    reference: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
    target: ExpectedTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedProcessing {
    Complete,
    PrerequisiteStaticRejected { authority: &'static str },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RelationFixture {
    id: &'static str,
    source: &'static str,
    processing: ExpectedProcessing,
    relations: &'static [ExpectedRelation],
}

const A_CODE_POINTS: &[u32] = &[0x61];
const B_CODE_POINTS: &[u32] = &[0x62];
const X_CODE_POINTS: &[u32] = &[0x78];
const Y_CODE_POINTS: &[u32] = &[0x79];
const E_COMBINING_ACUTE_CODE_POINTS: &[u32] = &[0x65, 0x0301];

const BACKWARD_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(15, 16, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(4, 5, "a")),
}];

const FORWARD_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(6, 7, "y"),
    semantic_name: "y",
    semantic_code_points: Y_CODE_POINTS,
    target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(13, 14, "y")),
}];

const SELF_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(6, 7, "x"),
    semantic_name: "x",
    semantic_code_points: X_CODE_POINTS,
    target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(4, 5, "x")),
}];

const NO_MATCH_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(6, 7, "y"),
    semantic_name: "y",
    semantic_code_points: Y_CODE_POINTS,
    target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
}];

const ESCAPED_DIRECT_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(15, 21, "\\u0061"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(4, 5, "a")),
}];

const CANONICAL_DISTINCT_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(16, 23, "e\\u0301"),
    semantic_name: "e\u{301}",
    semantic_code_points: E_COMBINING_ACUTE_CODE_POINTS,
    target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
}];

const SAME_DECLARATION_FORWARD_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    reference: ExpectedAnchor::new(6, 7, "y"),
    semantic_name: "y",
    semantic_code_points: Y_CODE_POINTS,
    target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(9, 10, "y")),
}];

const MULTIPLE_RELATIONS: &[ExpectedRelation] = &[
    ExpectedRelation {
        reference: ExpectedAnchor::new(19, 20, "a"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
        target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(4, 5, "a")),
    },
    ExpectedRelation {
        reference: ExpectedAnchor::new(23, 24, "b"),
        semantic_name: "b",
        semantic_code_points: B_CODE_POINTS,
        target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(8, 9, "b")),
    },
];

const FIXTURES: &[RelationFixture] = &[
    RelationFixture {
        id: "backward",
        source: "let a=1; let x=a;",
        processing: ExpectedProcessing::Complete,
        relations: BACKWARD_RELATIONS,
    },
    RelationFixture {
        id: "forward",
        source: "let x=y; let y=1;",
        processing: ExpectedProcessing::Complete,
        relations: FORWARD_RELATIONS,
    },
    RelationFixture {
        id: "self",
        source: "let x=x;",
        processing: ExpectedProcessing::Complete,
        relations: SELF_RELATIONS,
    },
    RelationFixture {
        id: "no-match",
        source: "let x=y;",
        processing: ExpectedProcessing::Complete,
        relations: NO_MATCH_RELATIONS,
    },
    RelationFixture {
        id: "escaped-direct-equality",
        source: "let a=1; let x=\\u0061;",
        processing: ExpectedProcessing::Complete,
        relations: ESCAPED_DIRECT_RELATIONS,
    },
    RelationFixture {
        id: "canonical-distinct",
        source: "let é=1; let x=e\\u0301;",
        processing: ExpectedProcessing::Complete,
        relations: CANONICAL_DISTINCT_RELATIONS,
    },
    RelationFixture {
        id: "same-declaration-forward",
        source: "let x=y, y=1;",
        processing: ExpectedProcessing::Complete,
        relations: SAME_DECLARATION_FORWARD_RELATIONS,
    },
    RelationFixture {
        id: "multiple-source-order",
        source: "let a=1,b=2; let x=a,y=b;",
        processing: ExpectedProcessing::Complete,
        relations: MULTIPLE_RELATIONS,
    },
    RelationFixture {
        id: "no-reference-atom",
        source: "let x=1;",
        processing: ExpectedProcessing::Complete,
        relations: &[],
    },
    RelationFixture {
        id: "duplicate-prerequisite",
        source: "let a=1; let a=2; let x=a;",
        processing: ExpectedProcessing::PrerequisiteStaticRejected {
            authority: "EE-36-R01",
        },
        relations: &[],
    },
    RelationFixture {
        id: "escaped-reserved-prerequisite",
        source: "let x=\\u0069f;",
        processing: ExpectedProcessing::PrerequisiteStaticRejected {
            authority: "EE-04-R08",
        },
        relations: &[],
    },
    RelationFixture {
        id: "unsupported-parenthesized-initializer",
        source: "let x=(y);",
        processing: ExpectedProcessing::UnsupportedCoverage,
        relations: &[],
    },
];

fn fixture(id: &str) -> &'static RelationFixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing Binding / Scope fixture {id}"))
}

fn decoded_expected_name(code_points: &[u32]) -> String {
    let mut decoded = String::new();
    for code_point in code_points {
        let scalar = char::from_u32(*code_point)
            .unwrap_or_else(|| panic!("fixture contains non-scalar code point U+{code_point:04X}"));
        decoded.push(scalar);
    }
    decoded
}

fn validate_anchor(source: &SourceText, expected: ExpectedAnchor) {
    let anchor = source
        .anchor(expected.start, expected.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error}"));
    assert_eq!(anchor.fragment(), expected.fragment);
}

#[test]
fn fixture_ids_and_sources_are_stable_and_unique() {
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for fixture in FIXTURES {
        assert!(
            ids.insert(fixture.id),
            "duplicate fixture id {}",
            fixture.id
        );
        assert!(
            sources.insert(fixture.source),
            "duplicate fixture source {}",
            fixture.source
        );
    }

    assert_eq!(ids.len(), 12);
    assert_eq!(sources.len(), 12);
}

#[test]
fn all_expected_authored_ranges_are_valid_utf8_source_anchors() {
    for (index, fixture) in FIXTURES.iter().enumerate() {
        let source = SourceText::new(SourceId::new(index as u64 + 1), fixture.source.to_owned());

        for relation in fixture.relations {
            validate_anchor(&source, relation.reference);
            if let ExpectedTarget::SameSourceSelectedLexicalBinding(target) = relation.target {
                validate_anchor(&source, target);
            }

            assert_eq!(
                decoded_expected_name(relation.semantic_code_points),
                relation.semantic_name
            );
        }
    }
}

#[test]
fn backward_forward_and_self_relations_are_source_relations_only() {
    let backward = fixture("backward");
    assert_eq!(backward.processing, ExpectedProcessing::Complete);
    assert_eq!(backward.relations, BACKWARD_RELATIONS);

    let forward = fixture("forward");
    assert_eq!(forward.processing, ExpectedProcessing::Complete);
    assert_eq!(forward.relations, FORWARD_RELATIONS);

    let self_relation = fixture("self");
    assert_eq!(self_relation.processing, ExpectedProcessing::Complete);
    assert_eq!(self_relation.relations, SELF_RELATIONS);

    assert!(SOURCE_RELATION_BOUNDARY.contains("runtime ResolveBinding"));
    assert!(SOURCE_RELATION_BOUNDARY.contains("TDZ safety"));
    assert!(SOURCE_RELATION_BOUNDARY.contains("initialized value"));
}

#[test]
fn no_same_source_binding_is_explicitly_non_error() {
    let no_match = fixture("no-match");
    assert_eq!(no_match.processing, ExpectedProcessing::Complete);
    assert_eq!(no_match.relations, NO_MATCH_RELATIONS);
    assert!(matches!(
        no_match.relations[0].target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));

    assert!(NO_MATCH_BOUNDARY.contains("runtime unbound"));
    assert!(NO_MATCH_BOUNDARY.contains("ReferenceError"));
    assert!(NO_MATCH_BOUNDARY.contains("StaticSemanticsRejected"));
}

#[test]
fn escaped_reference_matches_by_semantic_name_without_losing_authored_spelling() {
    let escaped = fixture("escaped-direct-equality");
    let relation = escaped.relations[0];

    assert_eq!(relation.reference, ExpectedAnchor::new(15, 21, "\\u0061"));
    assert_eq!(relation.semantic_name, "a");
    assert_eq!(relation.semantic_code_points, &[0x61]);
    assert_eq!(
        relation.target,
        ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(4, 5, "a"))
    );
}

#[test]
fn canonical_equivalence_does_not_create_a_same_source_relation() {
    let canonical = fixture("canonical-distinct");
    let relation = canonical.relations[0];
    let source = SourceText::new(SourceId::new(100), canonical.source.to_owned());

    let composed = source
        .anchor(4, 6)
        .expect("composed binding range must be valid");
    assert_eq!(composed.fragment(), "é");
    assert_eq!(relation.reference, ExpectedAnchor::new(16, 23, "e\\u0301"));
    assert_eq!(relation.semantic_name, "e\u{301}");
    assert_eq!(relation.semantic_code_points, &[0x65, 0x0301]);
    assert_ne!(composed.fragment(), relation.semantic_name);
    assert!(matches!(
        relation.target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));
}

#[test]
fn same_declaration_forward_relation_is_not_source_order_resolution() {
    let fixture = fixture("same-declaration-forward");
    assert_eq!(fixture.relations, SAME_DECLARATION_FORWARD_RELATIONS);
    assert_eq!(fixture.relations[0].reference.start, 6);
    let ExpectedTarget::SameSourceSelectedLexicalBinding(target) = fixture.relations[0].target
    else {
        panic!("same-declaration forward fixture must have a same-source target");
    };
    assert_eq!(target.start, 9);
    assert!(target.start > fixture.relations[0].reference.start);
}

#[test]
fn multiple_relations_are_ordered_by_reference_source_order() {
    let multiple = fixture("multiple-source-order");
    assert_eq!(multiple.relations, MULTIPLE_RELATIONS);
    assert_eq!(multiple.relations.len(), 2);

    let starts: Vec<_> = multiple
        .relations
        .iter()
        .map(|relation| relation.reference.start)
        .collect();
    assert_eq!(starts, vec![19, 23]);
    assert!(starts.windows(2).all(|window| window[0] < window[1]));
}

#[test]
fn non_identifier_reference_initializer_has_zero_relations() {
    let no_reference = fixture("no-reference-atom");
    assert_eq!(no_reference.processing, ExpectedProcessing::Complete);
    assert!(no_reference.relations.is_empty());
}

#[test]
fn static_rejection_prerequisites_never_choose_a_relation_target() {
    let duplicate = fixture("duplicate-prerequisite");
    assert_eq!(
        duplicate.processing,
        ExpectedProcessing::PrerequisiteStaticRejected {
            authority: "EE-36-R01"
        }
    );
    assert!(duplicate.relations.is_empty());

    let reserved = fixture("escaped-reserved-prerequisite");
    assert_eq!(
        reserved.processing,
        ExpectedProcessing::PrerequisiteStaticRejected {
            authority: "EE-04-R08"
        }
    );
    assert!(reserved.relations.is_empty());

    assert!(SELECTED_SLICE_COMPLETION_SOURCE.contains("EE-36-R01"));
    assert!(ESCAPED_RESERVED_RHS_ORACLE_SOURCE.contains("EE-04-R08"));
    assert!(ESCAPED_RESERVED_RHS_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
}

#[test]
fn unsupported_grammar_is_not_a_no_match_relation() {
    let unsupported = fixture("unsupported-parenthesized-initializer");
    assert_eq!(
        unsupported.processing,
        ExpectedProcessing::UnsupportedCoverage
    );
    assert!(unsupported.relations.is_empty());
    assert!(unsupported.source.contains("(y)"));
}

#[test]
fn validation_oracle_does_not_import_future_binding_scope_production() {
    for forbidden in [
        concat!("use super::selected_", "binding_scope"),
        concat!("selected_", "binding_scope::"),
        concat!("analyze_selected_", "binding_scope"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "candidate-independent Binding / Scope oracle must not import future production: {forbidden}"
        );
    }
}
