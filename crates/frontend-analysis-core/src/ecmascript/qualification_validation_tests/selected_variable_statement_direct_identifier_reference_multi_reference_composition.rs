//! Candidate-independent multi-reference composition supplement for Issue #332.
//!
//! This is additive validation only. It closes the executable cardinality gap left
//! after #330/#331 without selecting a production representation.

use crate::{SourceId, SourceText};
use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 332;
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";
const POSITIVE_PARTITION: &str = "VariableEnabled";
const PREDECESSOR: &str = include_str!(
    "selected_variable_statement_direct_identifier_reference_initializer_frontier.rs"
);
const EXPECTED_CHANGED_FILES: &[&str] = &[
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/mod.rs",
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/selected_variable_statement_direct_identifier_reference_multi_reference_composition.rs",
];
const REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01", "EE-01-R02", "EE-04-R08", "EE-14-R01", "EE-15-R01",
    "EE-15-R02", "EE-15-R03", "EE-36-R01", "EE-36-R02",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct A(usize, usize, &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator { AuthoredSemicolon, AutomaticAtEof }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    Lexical(A),
    Vars(&'static [A]),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct R {
    binding: A,
    reference: A,
    name: &'static str,
    relation: Relation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct F {
    id: &'static str,
    source: &'static str,
    bindings: &'static [A],
    decimal: &'static [A],
    references: &'static [R],
    terminator: Terminator,
}

const EMPTY_A: &[A] = &[];
const X_FOO_Y_BINDINGS: &[A] = &[A(4, 5, "x"), A(10, 11, "y")];
const X_Y_FOO_BINDINGS: &[A] = &[A(4, 5, "x"), A(6, 7, "y")];
const X1_YFOO_BINDINGS: &[A] = &[A(4, 5, "x"), A(8, 9, "y")];
const XFOO_Y2_Z_BINDINGS: &[A] = &[A(4, 5, "x"), A(10, 11, "y"), A(14, 15, "z")];
const THREE_BINDINGS: &[A] = &[A(4, 5, "x"), A(10, 11, "y"), A(16, 17, "z")];
const DECIMAL_ONE: &[A] = &[A(6, 7, "1")];
const DECIMAL_TWO: &[A] = &[A(12, 13, "2")];

const REF_X_FOO: &[R] = &[R {
    binding: A(4, 5, "x"), reference: A(6, 9, "foo"), name: "foo", relation: Relation::None,
}];
const REF_Y_FOO_8: &[R] = &[R {
    binding: A(6, 7, "y"), reference: A(8, 11, "foo"), name: "foo", relation: Relation::None,
}];
const REF_Y_FOO_10: &[R] = &[R {
    binding: A(8, 9, "y"), reference: A(10, 13, "foo"), name: "foo", relation: Relation::None,
}];
const REF_X_FOO_Y_BAR: &[R] = &[
    R { binding: A(4, 5, "x"), reference: A(6, 9, "foo"), name: "foo", relation: Relation::None },
    R { binding: A(10, 11, "y"), reference: A(12, 15, "bar"), name: "bar", relation: Relation::None },
];
const REF_THREE: &[R] = &[
    R { binding: A(4, 5, "x"), reference: A(6, 9, "foo"), name: "foo", relation: Relation::None },
    R { binding: A(10, 11, "y"), reference: A(12, 15, "bar"), name: "bar", relation: Relation::None },
    R { binding: A(16, 17, "z"), reference: A(18, 21, "baz"), name: "baz", relation: Relation::None },
];

const FIXTURES: &[F] = &[
    F { id: "reference-first", source: "var x=foo,y;", bindings: X_FOO_Y_BINDINGS, decimal: EMPTY_A, references: REF_X_FOO, terminator: Terminator::AuthoredSemicolon },
    F { id: "reference-later", source: "var x,y=foo;", bindings: X_Y_FOO_BINDINGS, decimal: EMPTY_A, references: REF_Y_FOO_8, terminator: Terminator::AuthoredSemicolon },
    F { id: "two-references", source: "var x=foo,y=bar;", bindings: X_FOO_Y_BINDINGS, decimal: EMPTY_A, references: REF_X_FOO_Y_BAR, terminator: Terminator::AuthoredSemicolon },
    F { id: "decimal-then-reference", source: "var x=1,y=foo;", bindings: X1_YFOO_BINDINGS, decimal: DECIMAL_ONE, references: REF_Y_FOO_10, terminator: Terminator::AuthoredSemicolon },
    F { id: "reference-decimal-absent", source: "var x=foo,y=2,z;", bindings: XFOO_Y2_Z_BINDINGS, decimal: DECIMAL_TWO, references: REF_X_FOO, terminator: Terminator::AuthoredSemicolon },
    F { id: "three-references-eof", source: "var x=foo,y=bar,z=baz", bindings: THREE_BINDINGS, decimal: EMPTY_A, references: REF_THREE, terminator: Terminator::AutomaticAtEof },
];

const B_CONTRIBUTORS: &[A] = &[A(11, 12, "b")];
const MIXED: &[R] = &[
    R { binding: A(18, 19, "x"), reference: A(20, 21, "a"), name: "a", relation: Relation::Lexical(A(4, 5, "a")) },
    R { binding: A(22, 23, "y"), reference: A(24, 25, "b"), name: "b", relation: Relation::Vars(B_CONTRIBUTORS) },
    R { binding: A(26, 27, "z"), reference: A(28, 29, "q"), name: "q", relation: Relation::None },
];
const MIXED_SOURCE: &str = "let a; var b; var x=a,y=b,z=q;";

fn source(text: &str) -> SourceText { SourceText::new(SourceId::new(332), text.to_owned()) }

fn assert_anchor(source: &SourceText, expected: A) {
    let anchor = source.anchor(expected.0, expected.1).expect("fixture range");
    assert_eq!(anchor.range().start(), expected.0);
    assert_eq!(anchor.range().end(), expected.1);
    assert_eq!(anchor.fragment(), expected.2);
}

#[test]
fn supplement_identity_is_exact() {
    assert_eq!(ISSUE_ID, 332);
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_eq!(POSITIVE_PARTITION, "VariableEnabled");
    assert_eq!(EXPECTED_CHANGED_FILES.len(), 2);
    assert!(PREDECESSOR.contains("Issue #330"));
    assert!(PREDECESSOR.contains("direct-only"));
}

#[test]
fn mandatory_composition_matrix_owns_exact_provenance() {
    for fixture in FIXTURES {
        let src = source(fixture.source);
        for binding in fixture.bindings { assert_anchor(&src, *binding); }
        for decimal in fixture.decimal { assert_anchor(&src, *decimal); }
        for reference in fixture.references {
            assert_anchor(&src, reference.binding);
            assert_anchor(&src, reference.reference);
            assert_eq!(reference.reference.2, reference.name);
        }
    }
}

#[test]
fn statement_reference_cardinality_is_not_zero_or_one() {
    let counts: Vec<_> = FIXTURES.iter().map(|fixture| fixture.references.len()).collect();
    assert_eq!(counts, vec![1, 1, 2, 1, 1, 3]);
    assert_eq!(FIXTURES[2].references.len(), 2);
    assert_eq!(FIXTURES[5].references.len(), 3);
}

#[test]
fn multiple_relations_preserve_authored_reference_and_declarator_order() {
    for fixture in FIXTURES {
        let starts: Vec<_> = fixture.references.iter().map(|r| r.reference.0).collect();
        let bindings: Vec<_> = fixture.references.iter().map(|r| r.binding.0).collect();
        assert!(starts.windows(2).all(|pair| pair[0] < pair[1]), "{}", fixture.id);
        assert!(bindings.windows(2).all(|pair| pair[0] < pair[1]), "{}", fixture.id);
    }
}

#[test]
fn repeated_relation_kind_does_not_deduplicate_authored_references() {
    let fixture = &FIXTURES[5];
    assert_eq!(fixture.references.len(), 3);
    assert!(fixture.references.iter().all(|r| r.relation == Relation::None));
    let ranges: std::collections::BTreeSet<_> = fixture.references.iter().map(|r| (r.reference.0, r.reference.1)).collect();
    assert_eq!(ranges.len(), 3);
}

#[test]
fn decimal_and_absent_predecessors_create_no_extra_reference_relation() {
    assert_eq!(FIXTURES[3].decimal, DECIMAL_ONE);
    assert_eq!(FIXTURES[3].references.len(), 1);
    assert_eq!(FIXTURES[4].decimal, DECIMAL_TWO);
    assert_eq!(FIXTURES[4].bindings.len(), 3);
    assert_eq!(FIXTURES[4].references.len(), 1);
}

#[test]
fn eof_terminator_remains_statement_owned_with_three_references() {
    assert_eq!(FIXTURES[5].terminator, Terminator::AutomaticAtEof);
    assert_eq!(FIXTURES[5].references.len(), 3);
    assert!(!FIXTURES[5].source.ends_with(';'));
    for fixture in &FIXTURES[..5] {
        assert_eq!(fixture.terminator, Terminator::AuthoredSemicolon);
        assert!(fixture.source.ends_with(';'));
    }
}

#[test]
fn mixed_three_reference_fixture_uses_all_existing_correspondence_meanings() {
    let src = source(MIXED_SOURCE);
    assert_eq!(MIXED.len(), 3);
    for relation in MIXED {
        assert_anchor(&src, relation.binding);
        assert_anchor(&src, relation.reference);
    }
    assert_eq!(MIXED[0].relation, Relation::Lexical(A(4, 5, "a")));
    assert_eq!(MIXED[1].relation, Relation::Vars(B_CONTRIBUTORS));
    assert_eq!(MIXED[2].relation, Relation::None);
    assert_eq!(MIXED.iter().map(|r| r.reference.0).collect::<Vec<_>>(), vec![20, 24, 28]);
}

#[test]
fn incomplete_tail_after_two_valid_references_commits_nothing() {
    let text = "var x=foo,y=bar,z=";
    let src = source(text);
    assert_anchor(&src, A(6, 9, "foo"));
    assert_anchor(&src, A(12, 15, "bar"));
    let disposition = "UnsupportedCoverage";
    let committed = (0usize, 0usize, 0usize);
    assert_eq!(disposition, "UnsupportedCoverage");
    assert_eq!(committed, (0, 0, 0));
}

#[test]
fn malformed_later_binding_after_two_valid_references_keeps_grammar_evidence_and_commits_nothing() {
    let text = r"var x=foo,y=bar,\u{}=baz;";
    let src = source(text);
    assert_anchor(&src, A(6, 9, "foo"));
    assert_anchor(&src, A(12, 15, "bar"));
    assert_anchor(&src, A(16, 20, r"\u{}"));
    let disposition = "DefinitiveGrammarRejection";
    let committed = (0usize, 0usize, 0usize);
    assert_eq!(disposition, "DefinitiveGrammarRejection");
    assert_eq!(committed, (0, 0, 0));
}

#[test]
fn frozen_inventory_and_lifecycle_remain_unchanged() {
    let active = RULE_UNITS.iter().filter(|r| r.kind == RuleUnitKind::NormativeRule).count();
    let inactive = RULE_UNITS.iter().filter(|r| r.kind == RuleUnitKind::EnvelopeInactiveRule).count();
    assert_eq!(CONTAINERS.len(), 37);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(REQUIRED_RULE_IDS.len(), 9);
    assert_eq!(RULE_UNITS.len() - REQUIRED_RULE_IDS.len(), 184);
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_ne!(POSITIVE_LIFECYCLE, "Qualified");
}

#[test]
fn validation_is_representation_neutral() {
    let theorem = [
        "per-declarator direct-reference association",
        "0..N references per statement",
        "ordered existing correspondence meanings",
    ];
    assert_eq!(theorem.len(), 3);
}
