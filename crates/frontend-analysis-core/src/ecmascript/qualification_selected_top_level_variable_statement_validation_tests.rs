//! Candidate-independent top-level VariableStatement / Script lexical-var
//! collision validation for Issue #306.
//!
//! This oracle owns fixture-level authored ranges, semantic identifier names,
//! Script lexical/var name domains, EE-36-R02 collision expectations, primary
//! evidence precedence, and unsupported-boundary expectations. It deliberately
//! does not call production recognition, static semantics, qualification, or
//! Binding / Scope analyzers to derive expected meaning.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 306;
const SELECTED_ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const SELECTED_ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const SELECTED_UNICODE_VERSION: &str = "17.0.0";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;
const SELECTED_YIELD: bool = false;
const SELECTED_AWAIT: bool = false;
const SELECTED_VARIABLE_STATEMENT_GRAMMAR: &str =
    "VariableStatement ::= var SelectedBindingIdentifier ;";
const SELECTED_VARIABLE_TOP_LEVEL_ONLY: bool = true;
const SELECTED_VARIABLE_SINGLE_BINDING_ONLY: bool = true;
const SELECTED_VARIABLE_INITIALIZER: bool = false;
const SELECTED_VARIABLE_DESTRUCTURING: bool = false;
const SELECTED_VARIABLE_EOF_ASI: bool = false;
const SELECTED_BLOCK_VAR: bool = false;

const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");
const CURRENT_COMPLETION_SOURCE: &str =
    include_str!("qualification_validation_tests/selected_one_level_block_slice_completion.rs");
const BLOCK_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_one_level_block_validation_tests.rs");
const QUALIFICATION_SOURCE: &str = include_str!("selected_qualification_integration.rs");
const FLAT_BINDING_SCOPE_SOURCE: &str = include_str!("selected_binding_scope.rs");
const HIERARCHICAL_BINDING_SCOPE_SOURCE: &str =
    include_str!("selected_one_level_block_binding_scope.rs");
const THIS_SOURCE: &str =
    include_str!("qualification_selected_top_level_variable_statement_validation_tests.rs");

const X: &[u32] = &[0x78];
const Y: &[u32] = &[0x79];
const A: &[u32] = &[0x61];
const E_ACUTE: &[u32] = &[0x00e9];
const E_COMBINING_ACUTE: &[u32] = &[0x65, 0x0301];

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
enum NameDomain {
    ScriptLexical,
    BlockLexical,
    ScriptVar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedName {
    domain: NameDomain,
    authored: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedCollision {
    lexical_binding: ExpectedAnchor,
    var_binding: ExpectedAnchor,
    semantic_name: &'static str,
    completing_binding: ExpectedAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedPrimaryEvidence {
    None,
    Rule {
        rule_id: &'static str,
        subject: ExpectedAnchor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedProcessing {
    Complete {
        primary: ExpectedPrimaryEvidence,
        ee36_r02_collision: Option<ExpectedCollision>,
    },
    UnsupportedCoverage(UnsupportedReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    MultipleVarDeclarators,
    VarInitializer,
    VarDestructuring,
    ForVar,
    BlockVar,
    EofAsiDeferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fixture {
    id: &'static str,
    source: &'static str,
    names: &'static [ExpectedName],
    processing: ExpectedProcessing,
}

const LET_X_VAR_X_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(11, 12, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const VAR_X_LET_X_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(11, 12, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const LET_X_VAR_Y_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(11, 12, "y"),
        semantic_name: "y",
        semantic_code_points: Y,
    },
];

const REPEATED_VAR_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(11, 12, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const LET_A_ESCAPED_VAR_A_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 5, "a"),
        semantic_name: "a",
        semantic_code_points: A,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(11, 17, r"\u0061"),
        semantic_name: "a",
        semantic_code_points: A,
    },
];

const ESCAPED_VAR_A_LET_A_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(4, 10, r"\u0061"),
        semantic_name: "a",
        semantic_code_points: A,
    },
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(16, 17, "a"),
        semantic_name: "a",
        semantic_code_points: A,
    },
];

const NON_NORMALIZED_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 6, "é"),
        semantic_name: "é",
        semantic_code_points: E_ACUTE,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(12, 19, r"e\u0301"),
        semantic_name: "e\u{301}",
        semantic_code_points: E_COMBINING_ACUTE,
    },
];

const BLOCK_LOCAL_ONLY_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::BlockLexical,
        authored: ExpectedAnchor::new(6, 7, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(15, 16, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const TOP_AND_BLOCK_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::BlockLexical,
        authored: ExpectedAnchor::new(13, 14, "y"),
        semantic_name: "y",
        semantic_code_points: Y,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(22, 23, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const DUPLICATE_SCRIPT_LEXICAL_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(11, 12, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(18, 19, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const DUPLICATE_DECLARATION_BINDING_NAMES: &[ExpectedName] = &[
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(4, 5, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptLexical,
        authored: ExpectedAnchor::new(6, 7, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
    ExpectedName {
        domain: NameDomain::ScriptVar,
        authored: ExpectedAnchor::new(13, 14, "x"),
        semantic_name: "x",
        semantic_code_points: X,
    },
];

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "lexical-before-var-collision",
        source: "let x; var x;",
        names: LET_X_VAR_X_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-36-R02",
                subject: ExpectedAnchor::new(11, 12, "x"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(4, 5, "x"),
                var_binding: ExpectedAnchor::new(11, 12, "x"),
                semantic_name: "x",
                completing_binding: ExpectedAnchor::new(11, 12, "x"),
            }),
        },
    },
    Fixture {
        id: "var-before-lexical-collision",
        source: "var x; let x;",
        names: VAR_X_LET_X_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-36-R02",
                subject: ExpectedAnchor::new(11, 12, "x"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(11, 12, "x"),
                var_binding: ExpectedAnchor::new(4, 5, "x"),
                semantic_name: "x",
                completing_binding: ExpectedAnchor::new(11, 12, "x"),
            }),
        },
    },
    Fixture {
        id: "non-colliding-lexical-and-var",
        source: "let x; var y;",
        names: LET_X_VAR_Y_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::None,
            ee36_r02_collision: None,
        },
    },
    Fixture {
        id: "repeated-var-alone-is-not-lexical-var-collision",
        source: "var x; var x;",
        names: REPEATED_VAR_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::None,
            ee36_r02_collision: None,
        },
    },
    Fixture {
        id: "escaped-var-equals-direct-lexical",
        source: r"let a; var \u0061;",
        names: LET_A_ESCAPED_VAR_A_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-36-R02",
                subject: ExpectedAnchor::new(11, 17, r"\u0061"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(4, 5, "a"),
                var_binding: ExpectedAnchor::new(11, 17, r"\u0061"),
                semantic_name: "a",
                completing_binding: ExpectedAnchor::new(11, 17, r"\u0061"),
            }),
        },
    },
    Fixture {
        id: "escaped-var-before-direct-lexical",
        source: r"var \u0061; let a;",
        names: ESCAPED_VAR_A_LET_A_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-36-R02",
                subject: ExpectedAnchor::new(16, 17, "a"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(16, 17, "a"),
                var_binding: ExpectedAnchor::new(4, 10, r"\u0061"),
                semantic_name: "a",
                completing_binding: ExpectedAnchor::new(16, 17, "a"),
            }),
        },
    },
    Fixture {
        id: "canonical-equivalence-does-not-normalize",
        source: "let é; var e\\u0301;",
        names: NON_NORMALIZED_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::None,
            ee36_r02_collision: None,
        },
    },
    Fixture {
        id: "block-local-lexical-does-not-enter-script-domain",
        source: "{ let x; } var x;",
        names: BLOCK_LOCAL_ONLY_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::None,
            ee36_r02_collision: None,
        },
    },
    Fixture {
        id: "top-level-lexical-still-collides-across-block-item",
        source: "let x; { let y; } var x;",
        names: TOP_AND_BLOCK_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-36-R02",
                subject: ExpectedAnchor::new(22, 23, "x"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(4, 5, "x"),
                var_binding: ExpectedAnchor::new(22, 23, "x"),
                semantic_name: "x",
                completing_binding: ExpectedAnchor::new(22, 23, "x"),
            }),
        },
    },
    Fixture {
        id: "existing-script-duplicate-precedes-lexical-var-collision",
        source: "let x; let x; var x;",
        names: DUPLICATE_SCRIPT_LEXICAL_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-36-R01",
                subject: ExpectedAnchor::new(11, 12, "x"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(4, 5, "x"),
                var_binding: ExpectedAnchor::new(18, 19, "x"),
                semantic_name: "x",
                completing_binding: ExpectedAnchor::new(18, 19, "x"),
            }),
        },
    },
    Fixture {
        id: "existing-declaration-duplicate-precedes-lexical-var-collision",
        source: "let x,x; var x;",
        names: DUPLICATE_DECLARATION_BINDING_NAMES,
        processing: ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::Rule {
                rule_id: "EE-15-R02",
                subject: ExpectedAnchor::new(6, 7, "x"),
            },
            ee36_r02_collision: Some(ExpectedCollision {
                lexical_binding: ExpectedAnchor::new(4, 5, "x"),
                var_binding: ExpectedAnchor::new(13, 14, "x"),
                semantic_name: "x",
                completing_binding: ExpectedAnchor::new(13, 14, "x"),
            }),
        },
    },
];

const UNSUPPORTED_FIXTURES: &[Fixture] = &[
    Fixture {
        id: "multiple-var-declarators",
        source: "var x, y;",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(
            UnsupportedReason::MultipleVarDeclarators,
        ),
    },
    Fixture {
        id: "numeric-var-initializer",
        source: "var x = 1;",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::VarInitializer),
    },
    Fixture {
        id: "identifier-var-initializer",
        source: "var x = y;",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::VarInitializer),
    },
    Fixture {
        id: "var-destructuring",
        source: "var {x} = y;",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::VarDestructuring),
    },
    Fixture {
        id: "for-var",
        source: "for (var x;;) {}",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::ForVar),
    },
    Fixture {
        id: "block-var",
        source: "{ var x; }",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::BlockVar),
    },
    Fixture {
        id: "var-eof-asi-deferred",
        source: "var x",
        names: &[],
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::EofAsiDeferred),
    },
];

fn names_in_domain(fixture: &Fixture, domain: NameDomain) -> BTreeSet<&'static str> {
    fixture
        .names
        .iter()
        .filter(|name| name.domain == domain)
        .map(|name| name.semantic_name)
        .collect()
}

fn script_lexical_var_intersection(fixture: &Fixture) -> BTreeSet<&'static str> {
    let lexical = names_in_domain(fixture, NameDomain::ScriptLexical);
    let vars = names_in_domain(fixture, NameDomain::ScriptVar);
    lexical.intersection(&vars).copied().collect()
}

#[test]
fn fixed_authority_and_refined_research_frontier_are_explicit() {
    assert_eq!(ISSUE_ID, 306);
    assert_eq!(SELECTED_ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        SELECTED_ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(SELECTED_UNICODE_VERSION, "17.0.0");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_YIELD);
    assert!(!SELECTED_AWAIT);
    assert_eq!(
        SELECTED_VARIABLE_STATEMENT_GRAMMAR,
        "VariableStatement ::= var SelectedBindingIdentifier ;"
    );
    assert!(SELECTED_VARIABLE_TOP_LEVEL_ONLY);
    assert!(SELECTED_VARIABLE_SINGLE_BINDING_ONLY);
    assert!(!SELECTED_VARIABLE_INITIALIZER);
    assert!(!SELECTED_VARIABLE_DESTRUCTURING);
    assert!(!SELECTED_VARIABLE_EOF_ASI);
    assert!(!SELECTED_BLOCK_VAR);

    assert!(INVENTORY_SOURCE.contains("\"EE-36-R02\""));
    assert!(
        INVENTORY_SOURCE
            .contains("Script : ScriptBody / LexicallyDeclaredNames intersects VarDeclaredNames")
    );
    assert!(INVENTORY_SOURCE.contains("&[\"LexicallyDeclaredNames\", \"VarDeclaredNames\"]"));

    assert!(CURRENT_COMPLETION_SOURCE.contains(
        "const CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR: &str = \"LexicalDeclaration | SelectedBlock\";"
    ));
    assert!(CURRENT_COMPLETION_SOURCE.contains("assert_eq!(required.len(), 8);"));
    assert!(CURRENT_COMPLETION_SOURCE.contains("assert_eq!(structurally_non_triggering, 185);"));

    assert!(BLOCK_ORACLE_SOURCE.contains("const SELECTED_VAR_DECLARATION: bool = false;"));
    assert!(BLOCK_ORACLE_SOURCE.contains("EE-14-R02"));
    assert!(
        BLOCK_ORACLE_SOURCE.contains("selected Block contains no VarDeclaredNames contributor")
    );
}

#[test]
fn fixture_authored_ranges_and_semantic_code_points_are_exact_utf8() {
    for fixture in FIXTURES {
        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());
        assert_eq!(
            source.as_str(),
            fixture.source,
            "{} exact source",
            fixture.id
        );

        for name in fixture.names {
            let anchor = source
                .anchor(name.authored.start, name.authored.end)
                .unwrap_or_else(|error| panic!("{} invalid anchor: {error}", fixture.id));
            assert_eq!(
                anchor.fragment(),
                name.authored.fragment,
                "{} fragment",
                fixture.id
            );
            assert_eq!(anchor.range().start(), name.authored.start);
            assert_eq!(anchor.range().end(), name.authored.end);

            let code_points: Vec<u32> = name.semantic_name.chars().map(u32::from).collect();
            assert_eq!(
                code_points.as_slice(),
                name.semantic_code_points,
                "{} semantic code points for {:?}",
                fixture.id,
                name.authored
            );
        }
    }
}

#[test]
fn ee36_r02_collision_matrix_uses_only_script_level_name_domains() {
    for fixture in FIXTURES {
        let intersection = script_lexical_var_intersection(fixture);
        let ExpectedProcessing::Complete {
            ee36_r02_collision, ..
        } = fixture.processing
        else {
            panic!("positive fixture {} must be complete", fixture.id);
        };

        match ee36_r02_collision {
            Some(collision) => {
                assert_eq!(intersection, BTreeSet::from([collision.semantic_name]));
                assert!(fixture.names.iter().any(|name| {
                    name.domain == NameDomain::ScriptLexical
                        && name.authored == collision.lexical_binding
                        && name.semantic_name == collision.semantic_name
                }));
                assert!(fixture.names.iter().any(|name| {
                    name.domain == NameDomain::ScriptVar
                        && name.authored == collision.var_binding
                        && name.semantic_name == collision.semantic_name
                }));
            }
            None => assert!(
                intersection.is_empty(),
                "{} must remain an explicit EE-36-R02 non-collision control",
                fixture.id
            ),
        }
    }
}

#[test]
fn ee36_r02_primary_subject_is_the_binding_that_completes_the_collision() {
    for fixture in FIXTURES {
        let ExpectedProcessing::Complete {
            primary,
            ee36_r02_collision,
        } = fixture.processing
        else {
            continue;
        };
        let ExpectedPrimaryEvidence::Rule { rule_id, subject } = primary else {
            continue;
        };
        if rule_id != "EE-36-R02" {
            continue;
        }

        let collision = ee36_r02_collision
            .unwrap_or_else(|| panic!("{} must retain its collision", fixture.id));
        assert_eq!(subject, collision.completing_binding);
        assert_eq!(
            collision.completing_binding.start,
            collision
                .lexical_binding
                .start
                .max(collision.var_binding.start),
            "{} uses project-owned source-order collision completion, not normative diagnostic ordering",
            fixture.id
        );
    }
}

#[test]
fn existing_static_evidence_remains_primary_when_it_precedes_ee36_r02() {
    let expected = [
        (
            "existing-script-duplicate-precedes-lexical-var-collision",
            "EE-36-R01",
            ExpectedAnchor::new(11, 12, "x"),
        ),
        (
            "existing-declaration-duplicate-precedes-lexical-var-collision",
            "EE-15-R02",
            ExpectedAnchor::new(6, 7, "x"),
        ),
    ];

    for (fixture_id, expected_rule, expected_subject) in expected {
        let fixture = FIXTURES
            .iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
        let ExpectedProcessing::Complete {
            primary,
            ee36_r02_collision,
        } = fixture.processing
        else {
            panic!("{fixture_id} must be complete");
        };
        assert!(ee36_r02_collision.is_some());
        assert_eq!(
            primary,
            ExpectedPrimaryEvidence::Rule {
                rule_id: expected_rule,
                subject: expected_subject,
            }
        );
    }
}

#[test]
fn block_lexical_names_stay_outside_script_lexically_declared_names() {
    let block_only = FIXTURES
        .iter()
        .find(|fixture| fixture.id == "block-local-lexical-does-not-enter-script-domain")
        .expect("block-only control must exist");
    assert_eq!(
        names_in_domain(block_only, NameDomain::BlockLexical),
        BTreeSet::from(["x"])
    );
    assert!(names_in_domain(block_only, NameDomain::ScriptLexical).is_empty());
    assert_eq!(
        names_in_domain(block_only, NameDomain::ScriptVar),
        BTreeSet::from(["x"])
    );
    assert!(script_lexical_var_intersection(block_only).is_empty());

    let top_and_block = FIXTURES
        .iter()
        .find(|fixture| fixture.id == "top-level-lexical-still-collides-across-block-item")
        .expect("mixed top/block control must exist");
    assert_eq!(
        names_in_domain(top_and_block, NameDomain::BlockLexical),
        BTreeSet::from(["y"])
    );
    assert_eq!(
        script_lexical_var_intersection(top_and_block),
        BTreeSet::from(["x"])
    );
}

#[test]
fn repeated_var_names_alone_do_not_become_a_lexical_var_collision() {
    let fixture = FIXTURES
        .iter()
        .find(|fixture| fixture.id == "repeated-var-alone-is-not-lexical-var-collision")
        .expect("repeated-var control must exist");
    assert!(names_in_domain(fixture, NameDomain::ScriptLexical).is_empty());
    assert_eq!(
        names_in_domain(fixture, NameDomain::ScriptVar),
        BTreeSet::from(["x"])
    );
    assert!(script_lexical_var_intersection(fixture).is_empty());
    assert_eq!(
        fixture.processing,
        ExpectedProcessing::Complete {
            primary: ExpectedPrimaryEvidence::None,
            ee36_r02_collision: None,
        }
    );
}

#[test]
fn direct_escaped_equality_and_non_normalization_are_distinct_controls() {
    for fixture_id in [
        "escaped-var-equals-direct-lexical",
        "escaped-var-before-direct-lexical",
    ] {
        let fixture = FIXTURES
            .iter()
            .find(|fixture| fixture.id == fixture_id)
            .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
        assert_eq!(
            script_lexical_var_intersection(fixture),
            BTreeSet::from(["a"])
        );
    }

    let non_normalized = FIXTURES
        .iter()
        .find(|fixture| fixture.id == "canonical-equivalence-does-not-normalize")
        .expect("non-normalization control must exist");
    assert_eq!(
        names_in_domain(non_normalized, NameDomain::ScriptLexical),
        BTreeSet::from(["é"])
    );
    assert_eq!(
        names_in_domain(non_normalized, NameDomain::ScriptVar),
        BTreeSet::from(["e\u{301}"])
    );
    assert!(script_lexical_var_intersection(non_normalized).is_empty());
}

#[test]
fn richer_var_forms_and_block_var_remain_explicit_unsupported_controls() {
    assert_eq!(UNSUPPORTED_FIXTURES.len(), 7);
    let expected_reasons = [
        UnsupportedReason::MultipleVarDeclarators,
        UnsupportedReason::VarInitializer,
        UnsupportedReason::VarInitializer,
        UnsupportedReason::VarDestructuring,
        UnsupportedReason::ForVar,
        UnsupportedReason::BlockVar,
        UnsupportedReason::EofAsiDeferred,
    ];

    for (fixture, expected_reason) in UNSUPPORTED_FIXTURES.iter().zip(expected_reasons) {
        assert!(
            !fixture.source.is_empty(),
            "{} source must be explicit",
            fixture.id
        );
        assert!(fixture.names.is_empty());
        assert_eq!(
            fixture.processing,
            ExpectedProcessing::UnsupportedCoverage(expected_reason)
        );
    }
}

#[test]
fn frozen_completion_and_qualification_remain_fail_closed() {
    assert!(CURRENT_COMPLETION_SOURCE.contains("!aggregate_gold_coverage_complete(RULE_UNITS)"));
    assert!(CURRENT_COMPLETION_SOURCE.contains("assert_eq!(required.len(), 8);"));
    assert!(CURRENT_COMPLETION_SOURCE.contains("assert_eq!(structurally_non_triggering, 185);"));
    assert!(QUALIFICATION_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(
        !QUALIFICATION_SOURCE.contains("QualificationOutcome::qualified("),
        "research evidence must not create a production Qualified path"
    );
}

#[test]
fn binding_scope_and_block_var_semantics_remain_external_to_this_oracle() {
    assert!(FLAT_BINDING_SCOPE_SOURCE.contains("NoSameSourceSelectedLexicalBinding"));
    assert!(FLAT_BINDING_SCOPE_SOURCE.contains("SelectedLexicalBindingOrder"));
    assert!(
        HIERARCHICAL_BINDING_SCOPE_SOURCE
            .contains("NoSelectedLexicalBindingTargetInCoveredRegions")
    );
    assert!(
        HIERARCHICAL_BINDING_SCOPE_SOURCE.contains("SelectedOneLevelBlockStaticSemanticsAccepted")
    );
    assert!(BLOCK_ORACLE_SOURCE.contains("const SELECTED_VAR_DECLARATION: bool = false;"));
    assert!(
        BLOCK_ORACLE_SOURCE.contains("selected Block contains no VarDeclaredNames contributor")
    );
}

#[test]
fn oracle_source_is_candidate_independent_and_does_not_call_production() {
    for (left, right) in [
        ("recognize_selected_lexical_", "slice("),
        ("evaluate_selected_static_", "semantics("),
        ("evaluate_selected_one_level_block_static_", "semantics("),
        ("attempt_selected_", "qualification("),
        ("analyze_selected_binding_", "scope("),
        ("analyze_selected_one_level_block_binding_", "scope("),
    ] {
        let forbidden = format!("{left}{right}");
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "#306 oracle must not call production owner {forbidden}"
        );
    }

    for forbidden_import in [
        "use super::selected_lexical_slice",
        "use super::selected_static_semantics",
        "use super::selected_binding_scope",
        "use super::selected_one_level_block_binding_scope",
        "use super::selected_qualification_integration",
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden_import),
            "#306 expected results must stay fixture-owned: {forbidden_import}"
        );
    }
}
