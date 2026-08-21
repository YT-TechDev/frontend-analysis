//! Candidate-independent composition supplement for Issue #308.
//!
//! This validation-only oracle pins two project-policy boundaries left open by
//! the first top-level `VariableStatement` / EE-36-R02 oracle: deterministic
//! multi-collision evidence selection and `var` keyword-adjacent malformed UES
//! routing. It does not call production recognition, static semantics,
//! qualification, or Binding / Scope analyzers to derive expected meaning.

use std::collections::{BTreeMap, BTreeSet};

use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 308;
const SELECTED_ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;
const SELECTED_YIELD: bool = false;
const SELECTED_AWAIT: bool = false;

const PRIOR_VAR_ORACLE: &str =
    include_str!("qualification_selected_top_level_variable_statement_validation_tests.rs");
const TERMINAL_COMPOSITION_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_terminal_composition_validation_tests.rs");
const THIS_SOURCE: &str = include_str!(
    "qualification_selected_top_level_variable_statement_composition_validation_tests.rs"
);

const _: () = {
    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_YIELD);
    assert!(!SELECTED_AWAIT);
};

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
    ScriptVar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedNameEvent {
    domain: NameDomain,
    semantic_name: &'static str,
    authored: ExpectedAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedCollision {
    lexical_binding: ExpectedAnchor,
    var_binding: ExpectedAnchor,
    semantic_name: &'static str,
    completing_binding: ExpectedAnchor,
}

const fn event(
    domain: NameDomain,
    semantic_name: &'static str,
    start: usize,
    end: usize,
    fragment: &'static str,
) -> ExpectedNameEvent {
    ExpectedNameEvent {
        domain,
        semantic_name,
        authored: ExpectedAnchor::new(start, end, fragment),
    }
}

const MULTI_COLLISION_EVENTS: &[ExpectedNameEvent] = &[
    event(NameDomain::ScriptVar, "y", 4, 5, "y"),
    event(NameDomain::ScriptVar, "x", 11, 12, "x"),
    event(NameDomain::ScriptLexical, "x", 18, 19, "x"),
    event(NameDomain::ScriptLexical, "y", 25, 26, "y"),
];

const REPEATED_VAR_EVENTS: &[ExpectedNameEvent] = &[
    event(NameDomain::ScriptVar, "x", 4, 5, "x"),
    event(NameDomain::ScriptVar, "x", 11, 12, "x"),
    event(NameDomain::ScriptLexical, "x", 18, 19, "x"),
];

const INVERSE_ORDER_EVENTS: &[ExpectedNameEvent] = &[
    event(NameDomain::ScriptLexical, "y", 4, 5, "y"),
    event(NameDomain::ScriptVar, "y", 11, 12, "y"),
    event(NameDomain::ScriptVar, "x", 18, 19, "x"),
    event(NameDomain::ScriptLexical, "x", 25, 26, "x"),
];

fn colliding_names(events: &[ExpectedNameEvent]) -> BTreeSet<&'static str> {
    let lexical: BTreeSet<_> = events
        .iter()
        .filter(|event| event.domain == NameDomain::ScriptLexical)
        .map(|event| event.semantic_name)
        .collect();
    let vars: BTreeSet<_> = events
        .iter()
        .filter(|event| event.domain == NameDomain::ScriptVar)
        .map(|event| event.semantic_name)
        .collect();
    lexical.intersection(&vars).copied().collect()
}

fn first_completed_collision(events: &[ExpectedNameEvent]) -> Option<ExpectedCollision> {
    let mut first_lexical_by_name: BTreeMap<&str, ExpectedAnchor> = BTreeMap::new();
    let mut first_var_by_name: BTreeMap<&str, ExpectedAnchor> = BTreeMap::new();

    for event in events {
        match event.domain {
            NameDomain::ScriptLexical => {
                if let Some(var_binding) = first_var_by_name.get(event.semantic_name).copied() {
                    return Some(ExpectedCollision {
                        lexical_binding: event.authored,
                        var_binding,
                        semantic_name: event.semantic_name,
                        completing_binding: event.authored,
                    });
                }
                first_lexical_by_name
                    .entry(event.semantic_name)
                    .or_insert(event.authored);
            }
            NameDomain::ScriptVar => {
                if let Some(lexical_binding) =
                    first_lexical_by_name.get(event.semantic_name).copied()
                {
                    return Some(ExpectedCollision {
                        lexical_binding,
                        var_binding: event.authored,
                        semantic_name: event.semantic_name,
                        completing_binding: event.authored,
                    });
                }
                first_var_by_name
                    .entry(event.semantic_name)
                    .or_insert(event.authored);
            }
        }
    }

    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedCompositionDisposition {
    UnsupportedCoverage,
    DefinitiveGrammarRejection {
        subject: ExpectedAnchor,
    },
    AcceptedWithTrailingLexicalEofAsi {
        var_semicolon: ExpectedAnchor,
        trailing_lexical_binding: ExpectedAnchor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CompositionFixture {
    id: &'static str,
    source: &'static str,
    disposition: ExpectedCompositionDisposition,
}

const COMPOSITION_FIXTURES: &[CompositionFixture] = &[
    CompositionFixture {
        id: "keyword-adjacent-malformed-var-remains-unsupported",
        source: r"var\u{};",
        disposition: ExpectedCompositionDisposition::UnsupportedCoverage,
    },
    CompositionFixture {
        id: "trivia-separated-malformed-binding-uses-general-grammar-evidence",
        source: r"var \u{};",
        disposition: ExpectedCompositionDisposition::DefinitiveGrammarRejection {
            subject: ExpectedAnchor::new(4, 8, r"\u{}"),
        },
    },
    CompositionFixture {
        id: "formed-escape-continues-maximal-identifier-name",
        source: r"var\u0061;",
        disposition: ExpectedCompositionDisposition::UnsupportedCoverage,
    },
    CompositionFixture {
        id: "var-eof-asi-remains-deferred",
        source: "var x",
        disposition: ExpectedCompositionDisposition::UnsupportedCoverage,
    },
    CompositionFixture {
        id: "authored-var-semicolon-composes-with-existing-final-lexical-eof-asi",
        source: "var x; let y",
        disposition: ExpectedCompositionDisposition::AcceptedWithTrailingLexicalEofAsi {
            var_semicolon: ExpectedAnchor::new(5, 6, ";"),
            trailing_lexical_binding: ExpectedAnchor::new(11, 12, "y"),
        },
    },
];

fn assert_anchor(source: &str, anchor: ExpectedAnchor) {
    let source = SourceText::new(SourceId::new(ISSUE_ID), source.to_owned());
    let actual = source
        .anchor(anchor.start, anchor.end)
        .expect("fixture anchor must reconcile through Core SourceText");
    assert_eq!(actual.fragment(), anchor.fragment);
    assert_eq!(actual.range().start(), anchor.start);
    assert_eq!(actual.range().end(), anchor.end);
}

#[test]
fn authority_and_candidate_independence_are_explicit() {
    assert_eq!(ISSUE_ID, 308);
    assert_eq!(
        SELECTED_ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(SELECTED_PARSE_GOAL, "Script");

    assert!(PRIOR_VAR_ORACLE.contains("SELECTED_VARIABLE_TOP_LEVEL_ONLY: bool = true"));
    assert!(PRIOR_VAR_ORACLE.contains("SELECTED_VARIABLE_SINGLE_BINDING_ONLY: bool = true"));
    assert!(PRIOR_VAR_ORACLE.contains("SELECTED_VARIABLE_EOF_ASI: bool = false"));
    assert!(PRIOR_VAR_ORACLE.contains("SELECTED_BLOCK_VAR: bool = false"));
    assert!(PRIOR_VAR_ORACLE.contains("var-eof-asi-deferred"));
    assert!(PRIOR_VAR_ORACLE.contains("repeated-var-alone-is-not-lexical-var-collision"));

    assert!(TERMINAL_COMPOSITION_ORACLE.contains("KEYWORD_ADJACENT_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains(r#"source: r"{ let\u{}; }""#));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains(r#"source: r"{ const\u{}; }""#));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("EOF_COMPOSITION_FIXTURES"));
    assert!(
        TERMINAL_COMPOSITION_ORACLE
            .contains("only_the_final_declaration_may_use_the_eof_only_policy")
    );

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
            "#308 supplement must not call production owner {forbidden}"
        );
    }
}

#[test]
fn multi_collision_primary_is_first_collision_completed_in_authored_order() {
    let source = "var y; var x; let x; let y;";
    for event in MULTI_COLLISION_EVENTS {
        assert_anchor(source, event.authored);
    }
    assert_eq!(
        colliding_names(MULTI_COLLISION_EVENTS),
        BTreeSet::from(["x", "y"])
    );

    let collision = first_completed_collision(MULTI_COLLISION_EVENTS)
        .expect("multi-collision fixture must contain a collision");
    assert_eq!(
        collision,
        ExpectedCollision {
            lexical_binding: ExpectedAnchor::new(18, 19, "x"),
            var_binding: ExpectedAnchor::new(11, 12, "x"),
            semantic_name: "x",
            completing_binding: ExpectedAnchor::new(18, 19, "x"),
        }
    );
}

#[test]
fn repeated_var_preserves_first_var_provenance_until_lexical_collision_completes() {
    let source = "var x; var x; let x;";
    for event in REPEATED_VAR_EVENTS {
        assert_anchor(source, event.authored);
    }
    assert_eq!(colliding_names(REPEATED_VAR_EVENTS), BTreeSet::from(["x"]));

    let collision = first_completed_collision(REPEATED_VAR_EVENTS)
        .expect("repeated-var fixture must eventually contain a lexical/var collision");
    assert_eq!(
        collision,
        ExpectedCollision {
            lexical_binding: ExpectedAnchor::new(18, 19, "x"),
            var_binding: ExpectedAnchor::new(4, 5, "x"),
            semantic_name: "x",
            completing_binding: ExpectedAnchor::new(18, 19, "x"),
        }
    );
}

#[test]
fn inverse_order_control_falsifies_always_lexical_or_hash_iteration_primary_selection() {
    let source = "let y; var y; var x; let x;";
    for event in INVERSE_ORDER_EVENTS {
        assert_anchor(source, event.authored);
    }
    assert_eq!(
        colliding_names(INVERSE_ORDER_EVENTS),
        BTreeSet::from(["x", "y"])
    );

    let collision = first_completed_collision(INVERSE_ORDER_EVENTS)
        .expect("inverse-order fixture must contain a collision");
    assert_eq!(
        collision,
        ExpectedCollision {
            lexical_binding: ExpectedAnchor::new(4, 5, "y"),
            var_binding: ExpectedAnchor::new(11, 12, "y"),
            semantic_name: "y",
            completing_binding: ExpectedAnchor::new(11, 12, "y"),
        }
    );
}

#[test]
fn var_keyword_adjacent_and_general_malformed_ues_routes_are_distinct() {
    for fixture in COMPOSITION_FIXTURES {
        match fixture.disposition {
            ExpectedCompositionDisposition::UnsupportedCoverage => {}
            ExpectedCompositionDisposition::DefinitiveGrammarRejection { subject } => {
                assert_anchor(fixture.source, subject);
            }
            ExpectedCompositionDisposition::AcceptedWithTrailingLexicalEofAsi { .. } => {}
        }
    }

    assert_eq!(COMPOSITION_FIXTURES[0].source, r"var\u{};");
    assert_eq!(
        COMPOSITION_FIXTURES[0].disposition,
        ExpectedCompositionDisposition::UnsupportedCoverage
    );
    assert_eq!(COMPOSITION_FIXTURES[1].source, r"var \u{};");
    assert_eq!(
        COMPOSITION_FIXTURES[1].disposition,
        ExpectedCompositionDisposition::DefinitiveGrammarRejection {
            subject: ExpectedAnchor::new(4, 8, r"\u{}"),
        }
    );
    assert_eq!(COMPOSITION_FIXTURES[2].source, r"var\u0061;");
    assert_eq!(
        COMPOSITION_FIXTURES[2].disposition,
        ExpectedCompositionDisposition::UnsupportedCoverage
    );
}

#[test]
fn authored_var_semicolon_does_not_widen_var_eof_asi() {
    let deferred = COMPOSITION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "var-eof-asi-remains-deferred")
        .expect("deferred var EOF fixture");
    assert_eq!(
        deferred.disposition,
        ExpectedCompositionDisposition::UnsupportedCoverage
    );

    let composed = COMPOSITION_FIXTURES
        .iter()
        .find(|fixture| {
            fixture.id == "authored-var-semicolon-composes-with-existing-final-lexical-eof-asi"
        })
        .expect("terminal composition fixture");
    let ExpectedCompositionDisposition::AcceptedWithTrailingLexicalEofAsi {
        var_semicolon,
        trailing_lexical_binding,
    } = composed.disposition
    else {
        panic!("terminal composition fixture must be accepted");
    };
    assert_anchor(composed.source, var_semicolon);
    assert_anchor(composed.source, trailing_lexical_binding);
}
