//! Candidate-independent successor oracle for selected top-level `VariableStatement`
//! escaped non-ReservedWord `IdentifierReference` initializer source/correspondence
//! semantics (Issue #336).
//!
//! This module composes immutable escaped-IdentifierReference authority from #241,
//! escaped-ReservedWord authority from #263, direct var RHS authority from #330,
//! and multi-reference authority from #332. It does not call production recognition,
//! static-semantics, qualification, correspondence, Binding / Scope, or runtime code
//! to derive the expected result.
//!
//! The theorem is representation-neutral: authored source provenance and decoded
//! semantic identity are observable obligations; later production placement decides
//! how, or whether, those obligations are retained in production data.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::super::unicode::{is_id_continue, is_id_start};
use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 336;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";
const POSITIVE_PARTITION: &str = "VariableEnabled";
const UNCONDITIONALLY_RESERVED_NAME_COUNT: usize = 35;

const ESCAPED_IDENTIFIER_ORACLE: &str = include_str!(
    "../qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const ESCAPED_RESERVED_ORACLE: &str = include_str!(
    "../qualification_selected_escaped_reserved_identifier_initializer_validation_tests.rs"
);
const DIRECT_VAR_ORACLE: &str =
    include_str!("selected_variable_statement_direct_identifier_reference_initializer_frontier.rs");
const MULTI_REFERENCE_ORACLE: &str = include_str!(
    "selected_variable_statement_direct_identifier_reference_multi_reference_composition.rs"
);

const EXPECTED_CHANGED_FILES: &[&str] = &[
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/mod.rs",
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/selected_variable_statement_escaped_identifier_reference_initializer_frontier.rs",
];

const CURRENT_REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-14-R01",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
    "EE-36-R02",
];

const EE01_C1_RHS_ROUTE: &str = "absent; EE-01 remains LHS BindingIdentifier-route-only";
const EE04_R08_C1_RHS_ROUTE: &str =
    "absent; decoded unconditionally-reserved var RHS remains frontier-scoped UnsupportedCoverage";
const REPRESENTATION_POLICY: &str = "candidate-independent authored provenance plus decoded semantic identity; production representation deferred";
const RUNTIME_NEGATIVE_BOUNDARY: &str = "same-source source-name correspondence is not runtime ResolveBinding or runtime target identity";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct A(usize, usize, &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    AuthoredSemicolon,
    AutomaticAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Correspondence {
    VisibleLexical(A),
    SameSourceVarContributors(&'static [A]),
    NoSelectedSameSourceContributor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RhsRoute {
    DirectPredecessor,
    EscapedSelected,
    EscapedDecodedReserved,
    EscapedInvalidStart,
    EscapedInvalidPart,
    EscapedInvalidScalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RhsExpectation {
    binding: A,
    rhs: A,
    decoded_code_points: &'static [u32],
    semantic_name: Option<&'static str>,
    route: RhsRoute,
    correspondence: Option<Correspondence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticRejected { rule_id: &'static str, subject: A },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fixture {
    id: &'static str,
    source: &'static str,
    disposition: Disposition,
    rhs: [Option<RhsExpectation>; 2],
    terminator: Option<Terminator>,
    committed_relations: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FailedWholeSourceCommitExpectation {
    fixture_id: &'static str,
    committed_selected_success_statements: usize,
    committed_selected_success_references: usize,
    committed_correspondence_relations: usize,
}

const FOO: &[u32] = &[0x66, 0x6f, 0x6f];
const BAR: &[u32] = &[0x62, 0x61, 0x72];
const A_NAME: &[u32] = &[0x61];
const X_NAME: &[u32] = &[0x78];
const A_ZERO: &[u32] = &[0x61, 0x30];
const YIELD: &[u32] = &[0x79, 0x69, 0x65, 0x6c, 0x64];
const LET: &[u32] = &[0x6c, 0x65, 0x74];
const AWAIT: &[u32] = &[0x61, 0x77, 0x61, 0x69, 0x74];
const PRECOMPOSED_E: &[u32] = &[0x00e9];
const DECOMPOSED_E: &[u32] = &[0x65, 0x0301];
const INVALID_START_ZERO: &[u32] = &[0x30];
const INVALID_PART_DASH: &[u32] = &[0x61, 0x2d, 0x62];
const INVALID_SURROGATE: &[u32] = &[0xd800];

const M6_CONTRIBUTORS: &[A] = &[A(4, 5, "a")];
const M7_CONTRIBUTORS: &[A] = &[A(4, 10, r"\u{61}")];
const M9_CONTRIBUTORS: &[A] = &[A(4, 5, "x")];
const M10_CONTRIBUTORS: &[A] = &[A(18, 19, "a")];

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "M1",
        source: r"var x=\u0066oo;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"\u0066oo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M2",
        source: r"var x=f\u006Fo;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"f\u006Fo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M3",
        source: r"var x=\u{66}oo;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"\u{66}oo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M4",
        source: r"var x=\u{1D49C};",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 15, r"\u{1D49C}"),
                decoded_code_points: &[0x1d49c],
                semantic_name: Some("𝒜"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M5",
        source: r"var \u00E9=1; var x=e\u0301;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(18, 19, "x"),
                rhs: A(20, 27, r"e\u0301"),
                decoded_code_points: DECOMPOSED_E,
                semantic_name: Some("e\u{301}"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M6",
        source: r"var a=1; var x=\u0061;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(13, 14, "x"),
                rhs: A(15, 21, r"\u0061"),
                decoded_code_points: A_NAME,
                semantic_name: Some("a"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::SameSourceVarContributors(M6_CONTRIBUTORS)),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M7",
        source: r"var \u{61}=1, x=\u0061;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(14, 15, "x"),
                rhs: A(16, 22, r"\u0061"),
                decoded_code_points: A_NAME,
                semantic_name: Some("a"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::SameSourceVarContributors(M7_CONTRIBUTORS)),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M8",
        source: r"let a; var x=\u0061;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(11, 12, "x"),
                rhs: A(13, 19, r"\u0061"),
                decoded_code_points: A_NAME,
                semantic_name: Some("a"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::VisibleLexical(A(4, 5, "a"))),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M9",
        source: r"var x=\u0078;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 12, r"\u0078"),
                decoded_code_points: X_NAME,
                semantic_name: Some("x"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::SameSourceVarContributors(M9_CONTRIBUTORS)),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M10",
        source: r"var x=\u0061; var a;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 12, r"\u0061"),
                decoded_code_points: A_NAME,
                semantic_name: Some("a"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::SameSourceVarContributors(M10_CONTRIBUTORS)),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M11",
        source: r"var x=a\u0030;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 13, r"a\u0030"),
                decoded_code_points: A_ZERO,
                semantic_name: Some("a0"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M12",
        source: r"var x=\u0069f;",
        disposition: Disposition::UnsupportedCoverage,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 13, r"\u0069f"),
                decoded_code_points: &[0x69, 0x66],
                semantic_name: Some("if"),
                route: RhsRoute::EscapedDecodedReserved,
                correspondence: None,
            }),
            None,
        ],
        terminator: None,
        committed_relations: 0,
    },
    Fixture {
        id: "M13",
        source: r"var x=\u0079ield;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 16, r"\u0079ield"),
                decoded_code_points: YIELD,
                semantic_name: Some("yield"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M14",
        source: r"var x=\u006Cet;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"\u006Cet"),
                decoded_code_points: LET,
                semantic_name: Some("let"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M15",
        source: r"var x=a\u0077ait;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 16, r"a\u0077ait"),
                decoded_code_points: AWAIT,
                semantic_name: Some("await"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 1,
    },
    Fixture {
        id: "M16",
        source: r"var x=\u0030;",
        disposition: Disposition::UnsupportedCoverage,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 12, r"\u0030"),
                decoded_code_points: INVALID_START_ZERO,
                semantic_name: None,
                route: RhsRoute::EscapedInvalidStart,
                correspondence: None,
            }),
            None,
        ],
        terminator: None,
        committed_relations: 0,
    },
    Fixture {
        id: "M17",
        source: r"var x=a\u002Db;",
        disposition: Disposition::UnsupportedCoverage,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"a\u002Db"),
                decoded_code_points: INVALID_PART_DASH,
                semantic_name: None,
                route: RhsRoute::EscapedInvalidPart,
                correspondence: None,
            }),
            None,
        ],
        terminator: None,
        committed_relations: 0,
    },
    Fixture {
        id: "M18",
        source: r"var x=\uD800;",
        disposition: Disposition::UnsupportedCoverage,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 12, r"\uD800"),
                decoded_code_points: INVALID_SURROGATE,
                semantic_name: None,
                route: RhsRoute::EscapedInvalidScalar,
                correspondence: None,
            }),
            None,
        ],
        terminator: None,
        committed_relations: 0,
    },
    Fixture {
        id: "M19",
        source: r"var x=\u0066oo, y=\u0062ar;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"\u0066oo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            Some(RhsExpectation {
                binding: A(16, 17, "y"),
                rhs: A(18, 26, r"\u0062ar"),
                decoded_code_points: BAR,
                semantic_name: Some("bar"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 2,
    },
    Fixture {
        id: "M20",
        source: r"var x=foo, y=\u0062ar, z=1, w;",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 9, "foo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::DirectPredecessor,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            Some(RhsExpectation {
                binding: A(11, 12, "y"),
                rhs: A(13, 21, r"\u0062ar"),
                decoded_code_points: BAR,
                semantic_name: Some("bar"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 2,
    },
    Fixture {
        id: "M21",
        source: r"var x=\u0061, y=\u{61};",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 12, r"\u0061"),
                decoded_code_points: A_NAME,
                semantic_name: Some("a"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            Some(RhsExpectation {
                binding: A(14, 15, "y"),
                rhs: A(16, 22, r"\u{61}"),
                decoded_code_points: A_NAME,
                semantic_name: Some("a"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 2,
    },
    Fixture {
        id: "M22",
        source: r"var x=\u0066oo, y=;",
        disposition: Disposition::UnsupportedCoverage,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"\u0066oo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::EscapedSelected,
                correspondence: None,
            }),
            None,
        ],
        terminator: None,
        committed_relations: 0,
    },
    Fixture {
        id: "M23",
        source: r"var x=\u0066oo, \u0030y;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-01-R01",
            subject: A(16, 22, r"\u0030"),
        },
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 14, r"\u0066oo"),
                decoded_code_points: FOO,
                semantic_name: Some("foo"),
                route: RhsRoute::EscapedSelected,
                correspondence: None,
            }),
            None,
        ],
        terminator: Some(Terminator::AuthoredSemicolon),
        committed_relations: 0,
    },
    Fixture {
        id: "M24",
        source: r"var x=\u0066",
        disposition: Disposition::SelectedAcceptedIncomplete,
        rhs: [
            Some(RhsExpectation {
                binding: A(4, 5, "x"),
                rhs: A(6, 12, r"\u0066"),
                decoded_code_points: &[0x66],
                semantic_name: Some("f"),
                route: RhsRoute::EscapedSelected,
                correspondence: Some(Correspondence::NoSelectedSameSourceContributor),
            }),
            None,
        ],
        terminator: Some(Terminator::AutomaticAtEof),
        committed_relations: 1,
    },
];

const M5_PRECOMPOSED_BINDING: A = A(4, 10, r"\u00E9");
const M20_DECIMAL: A = A(25, 26, "1");
const M20_ABSENT_BINDING: A = A(28, 29, "w");
const FAILED_WHOLE_SOURCE_COMMIT_EXPECTATIONS: &[FailedWholeSourceCommitExpectation] = &[
    FailedWholeSourceCommitExpectation {
        fixture_id: "M22",
        committed_selected_success_statements: 0,
        committed_selected_success_references: 0,
        committed_correspondence_relations: 0,
    },
    FailedWholeSourceCommitExpectation {
        fixture_id: "M23",
        committed_selected_success_statements: 0,
        committed_selected_success_references: 0,
        committed_correspondence_relations: 0,
    },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing fixture {id}"))
}

fn source_for(fixture: &Fixture, ordinal: u64) -> SourceText {
    SourceText::new(
        SourceId::new(ISSUE_ID * 10_000 + ordinal),
        fixture.source.to_owned(),
    )
}

fn assert_anchor(source: &SourceText, expected: A) {
    let anchor = source
        .anchor(expected.0, expected.1)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error:?}"));
    assert_eq!(anchor.range().start(), expected.0);
    assert_eq!(anchor.range().end(), expected.1);
    assert_eq!(anchor.fragment(), expected.2);
}

fn assert_correspondence(source: &SourceText, correspondence: Correspondence) {
    match correspondence {
        Correspondence::VisibleLexical(binding) => assert_anchor(source, binding),
        Correspondence::SameSourceVarContributors(contributors) => {
            assert!(!contributors.is_empty());
            for contributor in contributors {
                assert_anchor(source, *contributor);
            }
            assert!(
                contributors.windows(2).all(|pair| pair[0].0 <= pair[1].0),
                "contributors must remain in authored order"
            );
        }
        Correspondence::NoSelectedSameSourceContributor => {}
    }
}

fn assert_rhs(source: &SourceText, rhs: RhsExpectation) {
    assert_anchor(source, rhs.binding);
    assert_anchor(source, rhs.rhs);

    match rhs.semantic_name {
        Some(name) => {
            assert!(
                name.chars()
                    .map(u32::from)
                    .eq(rhs.decoded_code_points.iter().copied()),
                "decoded semantic code points for {:?}",
                rhs.rhs
            );
        }
        None => assert!(
            matches!(
                rhs.route,
                RhsRoute::EscapedInvalidStart
                    | RhsRoute::EscapedInvalidPart
                    | RhsRoute::EscapedInvalidScalar
            ),
            "only invalid escaped routes may lack a semantic IdentifierReference name"
        ),
    }

    match rhs.route {
        RhsRoute::DirectPredecessor => assert!(!rhs.rhs.2.contains("\\u")),
        RhsRoute::EscapedSelected
        | RhsRoute::EscapedDecodedReserved
        | RhsRoute::EscapedInvalidStart
        | RhsRoute::EscapedInvalidPart
        | RhsRoute::EscapedInvalidScalar => assert!(rhs.rhs.2.contains("\\u")),
    }

    if let Some(correspondence) = rhs.correspondence {
        assert_correspondence(source, correspondence);
    }
}

fn assert_fixture(fixture: &Fixture, ordinal: u64) {
    let source = source_for(fixture, ordinal);
    let mut actual_relation_count = 0usize;

    for rhs in fixture.rhs.into_iter().flatten() {
        assert_rhs(&source, rhs);
        if rhs.correspondence.is_some() {
            actual_relation_count += 1;
        }
    }

    if let Disposition::StaticRejected { subject, .. } = fixture.disposition {
        assert_anchor(&source, subject);
    }

    assert_eq!(
        actual_relation_count, fixture.committed_relations,
        "{}",
        fixture.id
    );

    match fixture.disposition {
        Disposition::SelectedAcceptedIncomplete => {
            assert!(fixture.terminator.is_some(), "{}", fixture.id);
            assert_eq!(
                fixture.committed_relations,
                fixture.rhs.into_iter().flatten().count(),
                "{}",
                fixture.id
            );
        }
        Disposition::UnsupportedCoverage | Disposition::StaticRejected { .. } => {
            assert_eq!(fixture.committed_relations, 0, "{}", fixture.id);
        }
    }
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn validation_identity_envelope_and_constituent_authority_are_exact() {
    assert_eq!(ISSUE_ID, 336);
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_eq!(POSITIVE_PARTITION, "VariableEnabled");
    assert_eq!(UNCONDITIONALLY_RESERVED_NAME_COUNT, 35);
    assert_eq!(EXPECTED_CHANGED_FILES.len(), 2);

    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("Issue #241"));
    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("escaped `IdentifierReference`"));
    assert!(ESCAPED_RESERVED_ORACLE.contains("Issue #263"));
    assert!(ESCAPED_RESERVED_ORACLE.contains("EE-04-R08"));
    assert!(DIRECT_VAR_ORACLE.contains("Issue #330"));
    assert!(DIRECT_VAR_ORACLE.contains("direct-only"));
    assert!(MULTI_REFERENCE_ORACLE.contains("Issue #332"));
    assert!(MULTI_REFERENCE_ORACLE.contains("cardinality"));
}

#[test]
fn all_twenty_four_mandatory_fixtures_are_present_unique_and_source_backed() {
    assert_eq!(FIXTURES.len(), 24);

    let ids: BTreeSet<_> = FIXTURES.iter().map(|fixture| fixture.id).collect();
    let expected: BTreeSet<_> = [
        "M1", "M2", "M3", "M4", "M5", "M6", "M7", "M8", "M9", "M10", "M11", "M12", "M13", "M14",
        "M15", "M16", "M17", "M18", "M19", "M20", "M21", "M22", "M23", "M24",
    ]
    .into_iter()
    .collect();
    assert_eq!(ids, expected);

    for (index, fixture) in FIXTURES.iter().enumerate() {
        assert_fixture(fixture, index as u64);
    }

    let selected = FIXTURES
        .iter()
        .filter(|fixture| fixture.disposition == Disposition::SelectedAcceptedIncomplete)
        .count();
    let unsupported = FIXTURES
        .iter()
        .filter(|fixture| fixture.disposition == Disposition::UnsupportedCoverage)
        .count();
    let static_rejected = FIXTURES
        .iter()
        .filter(|fixture| matches!(fixture.disposition, Disposition::StaticRejected { .. }))
        .count();
    assert_eq!((selected, unsupported, static_rejected), (18, 5, 1));
}

#[test]
fn authored_provenance_and_decoded_identity_are_independent() {
    let m1 = fixture("M1");
    let rhs = m1.rhs[0].unwrap();
    assert_eq!(rhs.rhs, A(6, 14, r"\u0066oo"));
    assert_eq!(rhs.semantic_name, Some("foo"));
    assert_ne!(rhs.rhs.2, rhs.semantic_name.unwrap());

    let m2 = fixture("M2");
    assert_eq!(m2.rhs[0].unwrap().rhs, A(6, 14, r"f\u006Fo"));

    let m3 = fixture("M3");
    assert_eq!(m3.rhs[0].unwrap().rhs.2, r"\u{66}oo");

    let m4 = fixture("M4");
    assert_eq!(m4.rhs[0].unwrap().decoded_code_points, &[0x1d49c]);
    assert!(is_id_start('𝒜' as u32));

    let m5 = fixture("M5");
    let source = source_for(m5, 5);
    assert_anchor(&source, M5_PRECOMPOSED_BINDING);
    assert_eq!(PRECOMPOSED_E, &[0x00e9]);
    assert_eq!(m5.rhs[0].unwrap().decoded_code_points, DECOMPOSED_E);
    assert_ne!(PRECOMPOSED_E, DECOMPOSED_E);
    assert_eq!(
        m5.rhs[0].unwrap().correspondence,
        Some(Correspondence::NoSelectedSameSourceContributor)
    );
}

#[test]
fn decoded_names_compose_with_the_existing_three_correspondence_meanings() {
    let m6 = fixture("M6");
    assert_eq!(
        m6.rhs[0].unwrap().correspondence,
        Some(Correspondence::SameSourceVarContributors(M6_CONTRIBUTORS))
    );

    let m7 = fixture("M7");
    assert_eq!(
        m7.rhs[0].unwrap().correspondence,
        Some(Correspondence::SameSourceVarContributors(M7_CONTRIBUTORS))
    );

    let m8 = fixture("M8");
    assert_eq!(
        m8.rhs[0].unwrap().correspondence,
        Some(Correspondence::VisibleLexical(A(4, 5, "a")))
    );

    let m9 = fixture("M9");
    assert_eq!(
        m9.rhs[0].unwrap().correspondence,
        Some(Correspondence::SameSourceVarContributors(M9_CONTRIBUTORS))
    );

    let m10 = fixture("M10");
    assert_eq!(
        m10.rhs[0].unwrap().correspondence,
        Some(Correspondence::SameSourceVarContributors(M10_CONTRIBUTORS))
    );
    assert!(M10_CONTRIBUTORS[0].0 > m10.rhs[0].unwrap().rhs.0);

    let none = fixture("M11").rhs[0].unwrap().correspondence;
    assert_eq!(none, Some(Correspondence::NoSelectedSameSourceContributor));
}

#[test]
fn c1_c6_firewall_and_name_policy_corrections_are_load_bearing() {
    let m12 = fixture("M12");
    assert_eq!(m12.disposition, Disposition::UnsupportedCoverage);
    assert_eq!(m12.rhs[0].unwrap().route, RhsRoute::EscapedDecodedReserved);
    assert_eq!(m12.rhs[0].unwrap().semantic_name, Some("if"));
    assert_eq!(m12.committed_relations, 0);

    for (id, expected_name) in [("M13", "yield"), ("M14", "let"), ("M15", "await")] {
        let fixture = fixture(id);
        assert_eq!(
            fixture.disposition,
            Disposition::SelectedAcceptedIncomplete,
            "{id}"
        );
        assert_eq!(fixture.rhs[0].unwrap().semantic_name, Some(expected_name));
        assert_eq!(fixture.rhs[0].unwrap().route, RhsRoute::EscapedSelected);
    }

    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("ESCAPED-IDREF-NAME-BINDING-CONTRAST-EVAL"));
    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("ESCAPED-IDREF-NAME-BINDING-CONTRAST-ARGUMENTS"));
    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("ESCAPED-IDREF-NAME-STRICT-ONLY-STATIC"));
    assert!(EE04_R08_C1_RHS_ROUTE.contains("frontier-scoped UnsupportedCoverage"));
}

#[test]
fn position_invalid_and_invalid_scalar_rhs_never_gain_lhs_ee01_ownership() {
    let m16 = fixture("M16");
    assert_eq!(m16.disposition, Disposition::UnsupportedCoverage);
    assert_eq!(m16.rhs[0].unwrap().route, RhsRoute::EscapedInvalidStart);
    assert!(!is_id_start('0' as u32));
    assert!(is_id_continue('0' as u32));

    let m17 = fixture("M17");
    assert_eq!(m17.disposition, Disposition::UnsupportedCoverage);
    let rhs = m17.rhs[0].unwrap();
    assert_eq!(rhs.route, RhsRoute::EscapedInvalidPart);
    assert_eq!(rhs.rhs, A(6, 14, r"a\u002Db"));
    assert!(rhs.rhs.2.starts_with('a'));
    assert!(!is_id_continue('-' as u32));

    let m18 = fixture("M18");
    assert_eq!(m18.disposition, Disposition::UnsupportedCoverage);
    assert_eq!(m18.rhs[0].unwrap().route, RhsRoute::EscapedInvalidScalar);
    assert!(char::from_u32(0xd800).is_none());

    assert!(EE01_C1_RHS_ROUTE.contains("LHS BindingIdentifier-route-only"));
}

#[test]
fn multiple_escaped_and_mixed_reference_relations_preserve_cardinality_order_and_association() {
    let m19 = fixture("M19");
    assert_eq!(m19.committed_relations, 2);
    assert_eq!(m19.rhs[0].unwrap().binding, A(4, 5, "x"));
    assert_eq!(m19.rhs[1].unwrap().binding, A(16, 17, "y"));
    assert!(
        m19.rhs[0].unwrap().rhs.0 < m19.rhs[1].unwrap().rhs.0,
        "authored reference order"
    );

    let m20 = fixture("M20");
    assert_eq!(m20.committed_relations, 2);
    assert_eq!(m20.rhs[0].unwrap().route, RhsRoute::DirectPredecessor);
    assert_eq!(m20.rhs[1].unwrap().route, RhsRoute::EscapedSelected);
    let source = source_for(m20, 20);
    assert_anchor(&source, M20_DECIMAL);
    assert_anchor(&source, M20_ABSENT_BINDING);

    let m21 = fixture("M21");
    assert_eq!(m21.committed_relations, 2);
    let first = m21.rhs[0].unwrap();
    let second = m21.rhs[1].unwrap();
    assert_eq!(first.semantic_name, Some("a"));
    assert_eq!(second.semantic_name, Some("a"));
    assert_ne!(first.rhs, second.rhs);
    assert_eq!(
        first.correspondence,
        Some(Correspondence::NoSelectedSameSourceContributor)
    );
    assert_eq!(
        second.correspondence,
        Some(Correspondence::NoSelectedSameSourceContributor)
    );

    assert!(MULTI_REFERENCE_ORACLE.contains("statement_reference_cardinality_is_not_zero_or_one"));
}

#[test]
fn whole_source_failure_and_static_rejection_suppress_all_correspondence_relations() {
    let m22 = fixture("M22");
    assert_eq!(m22.disposition, Disposition::UnsupportedCoverage);
    assert_eq!(m22.rhs[0].unwrap().semantic_name, Some("foo"));
    assert_eq!(m22.committed_relations, 0);
    assert_eq!(m22.rhs[0].unwrap().correspondence, None);

    let m23 = fixture("M23");
    let Disposition::StaticRejected { rule_id, subject } = m23.disposition else {
        panic!("M23 must remain the existing LHS EE-01 rejection");
    };
    assert_eq!(rule_id, "EE-01-R01");
    assert_eq!(subject, A(16, 22, r"\u0030"));
    assert_ne!(subject, m23.rhs[0].unwrap().rhs);
    assert_eq!(m23.committed_relations, 0);
    assert_eq!(m23.rhs[0].unwrap().correspondence, None);

    assert_eq!(FAILED_WHOLE_SOURCE_COMMIT_EXPECTATIONS.len(), 2);
    for expected in FAILED_WHOLE_SOURCE_COMMIT_EXPECTATIONS {
        let failed = fixture(expected.fixture_id);
        assert_ne!(
            failed.disposition,
            Disposition::SelectedAcceptedIncomplete,
            "{}",
            expected.fixture_id
        );
        assert_eq!(
            expected.committed_selected_success_statements, 0,
            "{}",
            expected.fixture_id
        );
        assert_eq!(
            expected.committed_selected_success_references, 0,
            "{}",
            expected.fixture_id
        );
        assert_eq!(
            expected.committed_correspondence_relations, 0,
            "{}",
            expected.fixture_id
        );
        assert_eq!(
            failed.committed_relations, expected.committed_correspondence_relations,
            "{}",
            expected.fixture_id
        );
    }

    let source = source_for(m23, 23);
    assert_anchor(&source, subject);
    assert_anchor(&source, A(16, 23, r"\u0030y"));
}

#[test]
fn authored_semicolon_and_eof_only_asi_remain_statement_owned() {
    let m1 = fixture("M1");
    assert_eq!(m1.terminator, Some(Terminator::AuthoredSemicolon));
    assert!(m1.source.ends_with(';'));

    let m24 = fixture("M24");
    assert_eq!(m24.terminator, Some(Terminator::AutomaticAtEof));
    assert!(!m24.source.ends_with(';'));
    let rhs = m24.rhs[0].unwrap().rhs;
    assert_eq!(rhs.1, m24.source.len());
    assert_eq!(rhs, A(6, 12, r"\u0066"));

    assert!(DIRECT_VAR_ORACLE.contains("non-eof-asi"));
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn frozen_rule_closure_lifecycle_partition_and_source_routes_remain_unchanged() {
    let active = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .count();
    let inactive = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
        .count();

    assert_eq!(CONTAINERS.len(), 37);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(CURRENT_REQUIRED_RULE_IDS.len(), 9);
    assert_eq!(RULE_UNITS.len() - CURRENT_REQUIRED_RULE_IDS.len(), 184);
    assert_eq!(POSITIVE_PARTITION, "VariableEnabled");
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_ne!(POSITIVE_LIFECYCLE, "Qualified");

    let active_ids: BTreeSet<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .map(|rule| rule.id)
        .collect();
    for rule_id in CURRENT_REQUIRED_RULE_IDS {
        assert!(active_ids.contains(rule_id), "{rule_id}");
    }

    assert!(EE01_C1_RHS_ROUTE.contains("LHS BindingIdentifier-route-only"));
    assert!(EE04_R08_C1_RHS_ROUTE.contains("absent"));
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn validation_remains_representation_neutral_and_runtime_negative() {
    assert!(REPRESENTATION_POLICY.contains("production representation deferred"));
    assert!(RUNTIME_NEGATIVE_BOUNDARY.contains("not runtime ResolveBinding"));
    assert_eq!(EXPECTED_CHANGED_FILES.len(), 2);
}
