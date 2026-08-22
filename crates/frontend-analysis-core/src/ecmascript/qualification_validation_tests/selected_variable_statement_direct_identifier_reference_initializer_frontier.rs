//! Candidate-independent successor oracle for selected top-level `VariableStatement`
//! direct-authored `IdentifierReference` initializer source/correspondence composition
//! (Issue #330).
//!
//! This validation composes immutable atom/source authority from #237, current
//! selected top-level var source authority through #329, and source-name
//! correspondence authority from #314 without calling production recognition,
//! static semantics, qualification, correspondence, Binding / Scope, or runtime
//! behavior to derive expected meaning.
//!
//! The successor is intentionally direct-only. Authored UnicodeEscapeSequence
//! spelling on the RHS remains outside this frontier even though a broader
//! production helper exists for other selected source owners.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 330;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";

const DIRECT_IDENTIFIER_ORACLE: &str =
    include_str!("../qualification_selected_identifier_reference_initializer_validation_tests.rs");
const VAR_CORRESPONDENCE_ORACLE: &str =
    include_str!("../selected_variable_statement_name_correspondence_validation_tests.rs");
const DECIMAL_VAR_ORACLE: &str =
    include_str!("selected_variable_statement_decimal_initializer_frontier.rs");
const MULTI_DECLARATOR_VAR_ORACLE: &str =
    include_str!("selected_variable_statement_multi_declarator_frontier.rs");
const THIS_SOURCE: &str =
    include_str!("selected_variable_statement_direct_identifier_reference_initializer_frontier.rs");

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

const EXPECTED_CHANGED_FILES: &[&str] = &[
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/mod.rs",
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/selected_variable_statement_direct_identifier_reference_initializer_frontier.rs",
];

const RUNTIME_NEGATIVE_BOUNDARY: &str =
    "same-source selected var contributor is authored source provenance, not runtime binding identity";
const LOOKUP_NEGATIVE_BOUNDARY: &str =
    "same-source selected correspondence is not runtime environment lookup or runtime unresolvability";

fn aggregate_qualified_available() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: usize,
    end: usize,
}

impl Range {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedAnchor {
    range: Range,
    fragment: &'static str,
}

impl ExpectedAnchor {
    const fn new(start: usize, end: usize, fragment: &'static str) -> Self {
        Self {
            range: Range::new(start, end),
            fragment,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    AuthoredSemicolon,
    AutomaticAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositivePartition {
    VariableEnabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedCorrespondence {
    VisibleSelectedLexicalBinding {
        binding: ExpectedAnchor,
    },
    SameSourceSelectedVarNameContributors {
        contributors: &'static [ExpectedAnchor],
    },
    NoSelectedSameSourceContributor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RelationFixture {
    id: &'static str,
    source: &'static str,
    containing_binding: ExpectedAnchor,
    reference: ExpectedAnchor,
    semantic_name: &'static str,
    correspondence: ExpectedCorrespondence,
    terminator: Terminator,
    partition: PositivePartition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum NamePolicyCategory {
    Ordinary,
    StrictOnlyRestricted,
    YieldAwaitParameterSpecial,
    BindingContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirectRhsFixture {
    id: &'static str,
    source: &'static str,
    rhs: ExpectedAnchor,
    semantic_name: &'static str,
    category: NamePolicyCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    SelectedAcceptedIncomplete,
    StaticRejected {
        rule_id: &'static str,
        subject: ExpectedAnchor,
    },
    DefinitiveGrammarRejection {
        subject: ExpectedAnchor,
    },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundaryFixture {
    id: &'static str,
    source: &'static str,
    disposition: Disposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FailedTransactionFixture {
    id: &'static str,
    source: &'static str,
    disposition: Disposition,
    committed_relations: usize,
    committed_contributors: usize,
}

const SELF_CONTRIBUTORS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const BEFORE_CONTRIBUTORS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const AFTER_CONTRIBUTORS: &[ExpectedAnchor] = &[ExpectedAnchor::new(13, 14, "a")];
const REPEATED_CONTRIBUTORS: &[ExpectedAnchor] = &[
    ExpectedAnchor::new(4, 5, "a"),
    ExpectedAnchor::new(11, 12, "a"),
];
const SAME_LIST_CONTRIBUTORS: &[ExpectedAnchor] = &[
    ExpectedAnchor::new(4, 5, "a"),
    ExpectedAnchor::new(6, 7, "a"),
];
const LATER_LIST_CONTRIBUTORS: &[ExpectedAnchor] = &[ExpectedAnchor::new(8, 9, "a")];
const ESCAPED_LHS_CONTRIBUTORS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(4, 10, r"\u0061")];
const DECIMAL_MIX_CONTRIBUTORS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];

const RELATION_FIXTURES: &[RelationFixture] = &[
    RelationFixture {
        id: "self-contributor",
        source: "var a=a;",
        containing_binding: ExpectedAnchor::new(4, 5, "a"),
        reference: ExpectedAnchor::new(6, 7, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: SELF_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "contributor-before-reference",
        source: "var a; var x=a;",
        containing_binding: ExpectedAnchor::new(11, 12, "x"),
        reference: ExpectedAnchor::new(13, 14, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: BEFORE_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "contributor-after-reference",
        source: "var x=a; var a;",
        containing_binding: ExpectedAnchor::new(4, 5, "x"),
        reference: ExpectedAnchor::new(6, 7, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: AFTER_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "repeated-whole-source-contributors",
        source: "var a; var a; var x=a;",
        containing_binding: ExpectedAnchor::new(18, 19, "x"),
        reference: ExpectedAnchor::new(20, 21, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: REPEATED_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "same-list-self-and-earlier",
        source: "var a,a=a;",
        containing_binding: ExpectedAnchor::new(6, 7, "a"),
        reference: ExpectedAnchor::new(8, 9, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: SAME_LIST_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "same-list-later-contributor",
        source: "var x=a,a;",
        containing_binding: ExpectedAnchor::new(4, 5, "x"),
        reference: ExpectedAnchor::new(6, 7, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: LATER_LIST_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "top-level-lexical-target",
        source: "let a; var x=a;",
        containing_binding: ExpectedAnchor::new(11, 12, "x"),
        reference: ExpectedAnchor::new(13, 14, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::VisibleSelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "no-selected-same-source-contributor",
        source: "var x=z;",
        containing_binding: ExpectedAnchor::new(4, 5, "x"),
        reference: ExpectedAnchor::new(6, 7, "z"),
        semantic_name: "z",
        correspondence: ExpectedCorrespondence::NoSelectedSameSourceContributor,
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "direct-reference-eof",
        source: "var x=foo",
        containing_binding: ExpectedAnchor::new(4, 5, "x"),
        reference: ExpectedAnchor::new(6, 9, "foo"),
        semantic_name: "foo",
        correspondence: ExpectedCorrespondence::NoSelectedSameSourceContributor,
        terminator: Terminator::AutomaticAtEof,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "escaped-lhs-direct-rhs-equality",
        source: r"var \u0061; var x=a;",
        containing_binding: ExpectedAnchor::new(16, 17, "x"),
        reference: ExpectedAnchor::new(18, 19, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: ESCAPED_LHS_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "canonical-distinct-no-normalization",
        source: "var é; var x=e\u{301};",
        containing_binding: ExpectedAnchor::new(12, 13, "x"),
        reference: ExpectedAnchor::new(14, 17, "e\u{301}"),
        semantic_name: "e\u{301}",
        correspondence: ExpectedCorrespondence::NoSelectedSameSourceContributor,
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "decimal-predecessor-mixed-with-reference",
        source: "var a=1,x=a;",
        containing_binding: ExpectedAnchor::new(8, 9, "x"),
        reference: ExpectedAnchor::new(10, 11, "a"),
        semantic_name: "a",
        correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: DECIMAL_MIX_CONTRIBUTORS,
        },
        terminator: Terminator::AuthoredSemicolon,
        partition: PositivePartition::VariableEnabled,
    },
    RelationFixture {
        id: "direct-reference-with-final-decimal-eof",
        source: "var x=foo,a=2",
        containing_binding: ExpectedAnchor::new(4, 5, "x"),
        reference: ExpectedAnchor::new(6, 9, "foo"),
        semantic_name: "foo",
        correspondence: ExpectedCorrespondence::NoSelectedSameSourceContributor,
        terminator: Terminator::AutomaticAtEof,
        partition: PositivePartition::VariableEnabled,
    },
];

const DIRECT_RHS_FIXTURES: &[DirectRhsFixture] = &[
    DirectRhsFixture {
        id: "ordinary-ascii",
        source: "var x=foo;",
        rhs: ExpectedAnchor::new(6, 9, "foo"),
        semantic_name: "foo",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-dollar",
        source: "var x=$;",
        rhs: ExpectedAnchor::new(6, 7, "$"),
        semantic_name: "$",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-low-line",
        source: "var x=_;",
        rhs: ExpectedAnchor::new(6, 7, "_"),
        semantic_name: "_",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-digit-part",
        source: "var x=a0;",
        rhs: ExpectedAnchor::new(6, 8, "a0"),
        semantic_name: "a0",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-multibyte-pi",
        source: "var x=π;",
        rhs: ExpectedAnchor::new(6, 8, "π"),
        semantic_name: "π",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-multibyte-script-a",
        source: "var x=𝒜;",
        rhs: ExpectedAnchor::new(6, 10, "𝒜"),
        semantic_name: "𝒜",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-combining-part",
        source: "var x=a\u{301};",
        rhs: ExpectedAnchor::new(6, 9, "a\u{301}"),
        semantic_name: "a\u{301}",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-zwnj-part",
        source: "var x=a\u{200c}b;",
        rhs: ExpectedAnchor::new(6, 11, "a\u{200c}b"),
        semantic_name: "a\u{200c}b",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "ordinary-zwj-part",
        source: "var x=a\u{200d}b;",
        rhs: ExpectedAnchor::new(6, 11, "a\u{200d}b"),
        semantic_name: "a\u{200d}b",
        category: NamePolicyCategory::Ordinary,
    },
    DirectRhsFixture {
        id: "strict-only-let",
        source: "var x=let;",
        rhs: ExpectedAnchor::new(6, 9, "let"),
        semantic_name: "let",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-static",
        source: "var x=static;",
        rhs: ExpectedAnchor::new(6, 12, "static"),
        semantic_name: "static",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-implements",
        source: "var x=implements;",
        rhs: ExpectedAnchor::new(6, 16, "implements"),
        semantic_name: "implements",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-interface",
        source: "var x=interface;",
        rhs: ExpectedAnchor::new(6, 15, "interface"),
        semantic_name: "interface",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-package",
        source: "var x=package;",
        rhs: ExpectedAnchor::new(6, 13, "package"),
        semantic_name: "package",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-private",
        source: "var x=private;",
        rhs: ExpectedAnchor::new(6, 13, "private"),
        semantic_name: "private",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-protected",
        source: "var x=protected;",
        rhs: ExpectedAnchor::new(6, 15, "protected"),
        semantic_name: "protected",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "strict-only-public",
        source: "var x=public;",
        rhs: ExpectedAnchor::new(6, 12, "public"),
        semantic_name: "public",
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    DirectRhsFixture {
        id: "parameter-yield",
        source: "var x=yield;",
        rhs: ExpectedAnchor::new(6, 11, "yield"),
        semantic_name: "yield",
        category: NamePolicyCategory::YieldAwaitParameterSpecial,
    },
    DirectRhsFixture {
        id: "parameter-await",
        source: "var x=await;",
        rhs: ExpectedAnchor::new(6, 11, "await"),
        semantic_name: "await",
        category: NamePolicyCategory::YieldAwaitParameterSpecial,
    },
    DirectRhsFixture {
        id: "binding-contrast-eval",
        source: "var x=eval;",
        rhs: ExpectedAnchor::new(6, 10, "eval"),
        semantic_name: "eval",
        category: NamePolicyCategory::BindingContrast,
    },
    DirectRhsFixture {
        id: "binding-contrast-arguments",
        source: "var x=arguments;",
        rhs: ExpectedAnchor::new(6, 15, "arguments"),
        semantic_name: "arguments",
        category: NamePolicyCategory::BindingContrast,
    },
];

const STATIC_BOUNDARIES: &[BoundaryFixture] = &[
    BoundaryFixture {
        id: "tier-one-before-ee36-r02",
        source: r"let a; var a=foo, \u0069f=bar;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-04-R08",
            subject: ExpectedAnchor::new(18, 25, r"\u0069f"),
        },
    },
    BoundaryFixture {
        id: "ee36-r02-binding-not-rhs",
        source: "let b; var a=foo,b=bar;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-36-R02",
            subject: ExpectedAnchor::new(17, 18, "b"),
        },
    },
    BoundaryFixture {
        id: "static-rejection-prevents-correspondence",
        source: "let a; var a; var x=a;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-36-R02",
            subject: ExpectedAnchor::new(11, 12, "a"),
        },
    },
    BoundaryFixture {
        id: "escaped-reserved-var-binding-stays-static",
        source: r"var \u0069f=foo;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-04-R08",
            subject: ExpectedAnchor::new(4, 11, r"\u0069f"),
        },
    },
];

const UNSUPPORTED_BOUNDARIES: &[BoundaryFixture] = &[
    BoundaryFixture {
        id: "escaped-rhs",
        source: r"var x=\u0066oo;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "member-expression",
        source: "var x=foo.bar;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "call-expression",
        source: "var x=foo();",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "binary-expression",
        source: "var x=foo + 1;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "parenthesized-expression",
        source: "var x=(foo);",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "assignment-expression",
        source: "var x=foo=bar;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "conditional-expression",
        source: "var x=foo?bar:baz;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "comment",
        source: "var x=foo/*comment*/;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "non-eof-asi",
        source: "var x=foo\nvar y;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "reserved-if",
        source: "var x=if;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "reserved-class",
        source: "var x=class;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "reserved-return",
        source: "var x=return;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "reserved-var",
        source: "var x=var;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "malformed-rhs-escape",
        source: r"var x=\u{};",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "missing-rhs",
        source: "var x=;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "boolean-true",
        source: "var x=true;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "boolean-false",
        source: "var x=false;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "null-literal",
        source: "var x=null;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "this-expression",
        source: "var x=this;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        id: "string-literal",
        source: "var x=\"foo\";",
        disposition: Disposition::UnsupportedCoverage,
    },
];

const FAILED_TRANSACTION_FIXTURES: &[FailedTransactionFixture] = &[
    FailedTransactionFixture {
        id: "trailing-comma",
        source: "var x=foo,",
        disposition: Disposition::UnsupportedCoverage,
        committed_relations: 0,
        committed_contributors: 0,
    },
    FailedTransactionFixture {
        id: "missing-later-rhs",
        source: "var x=foo,y=",
        disposition: Disposition::UnsupportedCoverage,
        committed_relations: 0,
        committed_contributors: 0,
    },
    FailedTransactionFixture {
        id: "unselected-later-rhs",
        source: "var x=foo,y=true;",
        disposition: Disposition::UnsupportedCoverage,
        committed_relations: 0,
        committed_contributors: 0,
    },
    FailedTransactionFixture {
        id: "later-malformed-binding",
        source: r"var x=foo,\u{}=bar;",
        disposition: Disposition::DefinitiveGrammarRejection {
            subject: ExpectedAnchor::new(10, 14, r"\u{}"),
        },
        committed_relations: 0,
        committed_contributors: 0,
    },
    FailedTransactionFixture {
        id: "requires-non-eof-asi",
        source: "var x=foo\nvar y;",
        disposition: Disposition::UnsupportedCoverage,
        committed_relations: 0,
        committed_contributors: 0,
    },
];

const TERMINATOR_BOUNDARIES: &[BoundaryFixture] = &[
    BoundaryFixture {
        id: "authored-semicolon",
        source: "var x=foo;",
        disposition: Disposition::SelectedAcceptedIncomplete,
    },
    BoundaryFixture {
        id: "automatic-at-eof",
        source: "var x=foo",
        disposition: Disposition::SelectedAcceptedIncomplete,
    },
    BoundaryFixture {
        id: "newline-before-comma",
        source: "var x=foo\n, a=1;",
        disposition: Disposition::SelectedAcceptedIncomplete,
    },
    BoundaryFixture {
        id: "newline-after-comma-eof",
        source: "var x=foo,\n a=1",
        disposition: Disposition::SelectedAcceptedIncomplete,
    },
];

fn assert_anchor(source: &str, expected: ExpectedAnchor, source_id: u64) {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    let anchor = source
        .anchor(expected.range.start, expected.range.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error:?}"));
    assert_eq!(anchor.fragment(), expected.fragment);
    assert_eq!(anchor.range().start(), expected.range.start);
    assert_eq!(anchor.range().end(), expected.range.end);
}

fn assert_correspondence_anchors(fixture: &RelationFixture, source_id: u64) {
    assert_anchor(fixture.source, fixture.containing_binding, source_id);
    assert_anchor(fixture.source, fixture.reference, source_id + 1);

    match fixture.correspondence {
        ExpectedCorrespondence::VisibleSelectedLexicalBinding { binding } => {
            assert_anchor(fixture.source, binding, source_id + 2);
        }
        ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } => {
            let mut previous_start = None;
            for (index, contributor) in contributors.iter().enumerate() {
                assert_anchor(
                    fixture.source,
                    *contributor,
                    source_id + 10 + index as u64,
                );
                if let Some(previous) = previous_start {
                    assert!(
                        contributor.range.start >= previous,
                        "contributors must preserve authored order in {}",
                        fixture.id
                    );
                }
                previous_start = Some(contributor.range.start);
            }
        }
        ExpectedCorrespondence::NoSelectedSameSourceContributor => {}
    }
}

#[test]
fn live_validation_contract_pins_the_current_envelope_and_predecessor_authority() {
    assert_eq!(ISSUE_ID, 330);
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");

    assert!(DIRECT_IDENTIFIER_ORACLE.contains("const ISSUE_ID: u64 = 237;"));
    assert!(DIRECT_IDENTIFIER_ORACLE.contains("EscapedIdentifierReference"));
    assert!(VAR_CORRESPONDENCE_ORACLE.contains("const ISSUE_ID: u64 = 314;"));
    assert!(VAR_CORRESPONDENCE_ORACLE.contains("SameSourceSelectedVarNameContributors"));
    assert!(DECIMAL_VAR_ORACLE.contains("const ISSUE_ID: u64 = 326;"));
    assert!(DECIMAL_VAR_ORACLE.contains("0 | [1-9][0-9]*"));
    assert!(MULTI_DECLARATOR_VAR_ORACLE.contains("Issue #322"));
}

#[test]
fn direct_rhs_fixture_ranges_and_name_policy_categories_are_exact() {
    let mut ids = BTreeSet::new();
    let mut categories = BTreeSet::new();

    for (index, fixture) in DIRECT_RHS_FIXTURES.iter().enumerate() {
        assert!(ids.insert(fixture.id), "duplicate fixture id {}", fixture.id);
        assert_anchor(fixture.source, fixture.rhs, ISSUE_ID * 100 + index as u64);
        assert_eq!(fixture.rhs.fragment, fixture.semantic_name);
        categories.insert(fixture.category);
    }

    for expected in [
        NamePolicyCategory::Ordinary,
        NamePolicyCategory::StrictOnlyRestricted,
        NamePolicyCategory::YieldAwaitParameterSpecial,
        NamePolicyCategory::BindingContrast,
    ] {
        assert!(categories.contains(&expected));
    }
}

#[test]
fn direct_only_maximality_keeps_authored_rhs_unicode_escape_outside_the_frontier() {
    assert!(
        DIRECT_RHS_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var x=foo;" && fixture.semantic_name == "foo")
    );

    let escaped = UNSUPPORTED_BOUNDARIES
        .iter()
        .find(|fixture| fixture.source == r"var x=\u0066oo;")
        .expect("escaped RHS control must remain explicit");
    assert_eq!(escaped.disposition, Disposition::UnsupportedCoverage);
    assert!(DIRECT_IDENTIFIER_ORACLE.contains("EscapedIdentifierReference"));
}

#[test]
fn self_contributor_is_explicit_source_provenance_not_a_runtime_target() {
    let fixture = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "self-contributor")
        .expect("self contributor fixture must exist");
    assert_correspondence_anchors(fixture, ISSUE_ID * 1_000);

    assert_eq!(fixture.reference, ExpectedAnchor::new(6, 7, "a"));
    assert_eq!(
        fixture.correspondence,
        ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: SELF_CONTRIBUTORS,
        }
    );
    assert!(RUNTIME_NEGATIVE_BOUNDARY.contains("runtime binding identity"));
}

#[test]
fn var_contributors_are_whole_source_authored_provenance_not_previous_only_lookup() {
    for id in [
        "contributor-before-reference",
        "contributor-after-reference",
        "repeated-whole-source-contributors",
    ] {
        let fixture = RELATION_FIXTURES
            .iter()
            .find(|fixture| fixture.id == id)
            .unwrap_or_else(|| panic!("{id} must exist"));
        assert_correspondence_anchors(fixture, ISSUE_ID * 2_000 + fixture.reference.range.start as u64);
    }

    let after = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "contributor-after-reference")
        .unwrap();
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        after.correspondence
    else {
        panic!("later contributor fixture must have var contributors");
    };
    assert!(contributors[0].range.start > after.reference.range.start);

    let repeated = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "repeated-whole-source-contributors")
        .unwrap();
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        repeated.correspondence
    else {
        panic!("repeated fixture must have var contributors");
    };
    assert_eq!(contributors.len(), 2);
}

#[test]
fn same_statement_list_includes_self_and_later_contributors_without_order_labels() {
    let self_and_earlier = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "same-list-self-and-earlier")
        .unwrap();
    assert_correspondence_anchors(self_and_earlier, ISSUE_ID * 3_000);
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        self_and_earlier.correspondence
    else {
        panic!("same-list fixture must have var contributors");
    };
    assert_eq!(contributors, SAME_LIST_CONTRIBUTORS);
    assert_eq!(contributors[1], self_and_earlier.containing_binding);

    let later = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "same-list-later-contributor")
        .unwrap();
    assert_correspondence_anchors(later, ISSUE_ID * 3_100);
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        later.correspondence
    else {
        panic!("later-list fixture must have var contributors");
    };
    assert_eq!(contributors, LATER_LIST_CONTRIBUTORS);
    assert!(contributors[0].range.start > later.reference.range.start);
    assert!(!RUNTIME_NEGATIVE_BOUNDARY.contains("Before"));
    assert!(!RUNTIME_NEGATIVE_BOUNDARY.contains("After"));
}

#[test]
fn correspondence_phase_has_three_existing_meanings_only_after_static_acceptance() {
    let lexical = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "top-level-lexical-target")
        .unwrap();
    assert_eq!(
        lexical.correspondence,
        ExpectedCorrespondence::VisibleSelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
        }
    );

    let var = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "contributor-before-reference")
        .unwrap();
    assert!(matches!(
        var.correspondence,
        ExpectedCorrespondence::SameSourceSelectedVarNameContributors { .. }
    ));

    let none = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "no-selected-same-source-contributor")
        .unwrap();
    assert_eq!(
        none.correspondence,
        ExpectedCorrespondence::NoSelectedSameSourceContributor
    );

    let rejected = STATIC_BOUNDARIES
        .iter()
        .find(|fixture| fixture.id == "static-rejection-prevents-correspondence")
        .unwrap();
    assert!(matches!(
        rejected.disposition,
        Disposition::StaticRejected {
            rule_id: "EE-36-R02",
            ..
        }
    ));
}

#[test]
fn direct_rhs_semantic_names_do_not_become_var_declared_names_or_static_subjects() {
    let tier_one = STATIC_BOUNDARIES
        .iter()
        .find(|fixture| fixture.id == "tier-one-before-ee36-r02")
        .unwrap();
    let Disposition::StaticRejected { rule_id, subject } = tier_one.disposition else {
        panic!("tier-one fixture must be statically rejected");
    };
    assert_eq!(rule_id, "EE-04-R08");
    assert_anchor(tier_one.source, subject, ISSUE_ID * 4_000);

    let collision = STATIC_BOUNDARIES
        .iter()
        .find(|fixture| fixture.id == "ee36-r02-binding-not-rhs")
        .unwrap();
    let Disposition::StaticRejected { rule_id, subject } = collision.disposition else {
        panic!("collision fixture must be statically rejected");
    };
    assert_eq!(rule_id, "EE-36-R02");
    assert_eq!(subject.fragment, "b");
    assert_ne!(subject.fragment, "foo");
    assert_ne!(subject.fragment, "bar");
    assert_anchor(collision.source, subject, ISSUE_ID * 4_100);
}

#[test]
fn escaped_lhs_matches_direct_rhs_but_unicode_normalization_remains_forbidden() {
    let equal = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "escaped-lhs-direct-rhs-equality")
        .unwrap();
    assert_correspondence_anchors(equal, ISSUE_ID * 5_000);
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        equal.correspondence
    else {
        panic!("escaped LHS fixture must have var contributors");
    };
    assert_eq!(contributors[0].fragment, r"\u0061");
    assert_eq!(equal.semantic_name, "a");

    let distinct = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "canonical-distinct-no-normalization")
        .unwrap();
    assert_correspondence_anchors(distinct, ISSUE_ID * 5_100);
    assert_eq!(
        distinct.correspondence,
        ExpectedCorrespondence::NoSelectedSameSourceContributor
    );
    assert_ne!("é", distinct.semantic_name);
}

#[test]
fn decimal_predecessor_and_direct_reference_successor_compose_in_one_list() {
    let mixed = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "decimal-predecessor-mixed-with-reference")
        .unwrap();
    assert_correspondence_anchors(mixed, ISSUE_ID * 6_000);
    assert_eq!(mixed.terminator, Terminator::AuthoredSemicolon);

    let eof = RELATION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "direct-reference-with-final-decimal-eof")
        .unwrap();
    assert_correspondence_anchors(eof, ISSUE_ID * 6_100);
    assert_eq!(eof.terminator, Terminator::AutomaticAtEof);
}

#[test]
fn terminator_and_list_trivia_preserve_authored_vs_eof_without_non_eof_asi() {
    assert!(TERMINATOR_BOUNDARIES.iter().any(|fixture| {
        fixture.source == "var x=foo;"
            && fixture.disposition == Disposition::SelectedAcceptedIncomplete
    }));
    assert!(TERMINATOR_BOUNDARIES.iter().any(|fixture| {
        fixture.source == "var x=foo"
            && fixture.disposition == Disposition::SelectedAcceptedIncomplete
    }));
    assert!(TERMINATOR_BOUNDARIES.iter().any(|fixture| {
        fixture.source == "var x=foo\n, a=1;"
            && fixture.disposition == Disposition::SelectedAcceptedIncomplete
    }));
    assert!(TERMINATOR_BOUNDARIES.iter().any(|fixture| {
        fixture.source == "var x=foo,\n a=1"
            && fixture.disposition == Disposition::SelectedAcceptedIncomplete
    }));
    assert!(UNSUPPORTED_BOUNDARIES.iter().any(|fixture| {
        fixture.source == "var x=foo\nvar y;"
            && fixture.disposition == Disposition::UnsupportedCoverage
    }));
}

#[test]
fn richer_expression_reserved_and_other_atom_neighbors_remain_coverage_honest() {
    assert!(
        UNSUPPORTED_BOUNDARIES
            .iter()
            .all(|fixture| fixture.disposition == Disposition::UnsupportedCoverage)
    );
    for required_id in [
        "escaped-rhs",
        "member-expression",
        "call-expression",
        "binary-expression",
        "parenthesized-expression",
        "assignment-expression",
        "conditional-expression",
        "comment",
        "non-eof-asi",
        "reserved-if",
        "reserved-class",
        "reserved-return",
        "reserved-var",
        "malformed-rhs-escape",
        "missing-rhs",
        "boolean-true",
        "boolean-false",
        "null-literal",
        "this-expression",
        "string-literal",
    ] {
        assert!(
            UNSUPPORTED_BOUNDARIES
                .iter()
                .any(|fixture| fixture.id == required_id),
            "missing unsupported control {required_id}"
        );
    }
}

#[test]
fn incomplete_unselected_and_later_grammar_failure_commit_no_partial_relation_state() {
    for fixture in FAILED_TRANSACTION_FIXTURES {
        assert_ne!(
            fixture.disposition,
            Disposition::SelectedAcceptedIncomplete,
            "{} must fail the whole transaction",
            fixture.id
        );
        assert_eq!(fixture.committed_relations, 0);
        assert_eq!(fixture.committed_contributors, 0);
    }

    let malformed = FAILED_TRANSACTION_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "later-malformed-binding")
        .unwrap();
    let Disposition::DefinitiveGrammarRejection { subject } = malformed.disposition else {
        panic!("malformed later binding must retain Grammar evidence");
    };
    assert_eq!(subject, ExpectedAnchor::new(10, 14, r"\u{}"));
    assert_anchor(malformed.source, subject, ISSUE_ID * 7_000);
}

#[test]
fn positive_relation_fixtures_have_exact_source_provenance_and_incomplete_lifecycle() {
    let mut ids = BTreeSet::new();
    for (index, fixture) in RELATION_FIXTURES.iter().enumerate() {
        assert!(ids.insert(fixture.id), "duplicate relation fixture {}", fixture.id);
        assert_correspondence_anchors(fixture, ISSUE_ID * 10_000 + index as u64 * 100);
        assert_eq!(fixture.partition, PositivePartition::VariableEnabled);
    }
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_ne!(POSITIVE_LIFECYCLE, "Qualified");
    assert!(!aggregate_qualified_available());
}

#[test]
fn frozen_rule_reachability_and_three_way_partition_pressure_remain_unchanged() {
    let active_ids: BTreeSet<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .map(|rule| rule.id)
        .collect();
    let required_ids: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();

    assert_eq!(CONTAINERS.len(), 37);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(
        RULE_UNITS
            .iter()
            .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
            .count(),
        183
    );
    assert_eq!(
        RULE_UNITS
            .iter()
            .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
            .count(),
        10
    );
    assert_eq!(
        RULE_UNITS
            .iter()
            .filter(|rule| rule.kind == RuleUnitKind::ExpansionSentinel)
            .count(),
        0
    );

    assert_eq!(required_ids.len(), 9);
    for required in &required_ids {
        assert!(active_ids.contains(required), "missing required rule {required}");
    }

    let mut reachable = 0;
    let mut non_triggering = 0;
    for rule in RULE_UNITS {
        if required_ids.contains(rule.id) {
            reachable += 1;
        } else {
            non_triggering += 1;
        }
    }
    assert_eq!(reachable, 9);
    assert_eq!(non_triggering, 184);
    assert_eq!(reachable + non_triggering, 193);
    assert!(!required_ids.contains("EE-02-R01"));
    assert!(!required_ids.contains("EE-14-R02"));
}

#[test]
fn validation_remains_candidate_independent_and_representation_neutral() {
    let forbidden = [
        ["selected", "lexical", "slice", "::"].join("_").replace("_::", "::"),
        ["selected", "static", "semantics", "::"].join("_").replace("_::", "::"),
        ["selected", "qualification", "integration", "::"]
            .join("_")
            .replace("_::", "::"),
        ["selected", "variable", "statement", "name", "correspondence", "::"]
            .join("_")
            .replace("_::", "::"),
        ["selected", "binding", "scope", "::"].join("_").replace("_::", "::"),
        ["recognize", "selected", "lexical", "slice"].join("_"),
        ["attempt", "selected", "qualification"].join("_"),
    ];

    for forbidden in forbidden {
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "candidate-independent oracle must not import or call production owner {forbidden}"
        );
    }

    assert!(RUNTIME_NEGATIVE_BOUNDARY.contains("source provenance"));
    assert!(LOOKUP_NEGATIVE_BOUNDARY.contains("not runtime"));
    assert!(!aggregate_qualified_available());
}

#[test]
fn exact_validation_handoff_is_two_files_and_production_placement_remains_open() {
    assert_eq!(EXPECTED_CHANGED_FILES.len(), 2);
    assert!(EXPECTED_CHANGED_FILES.iter().all(|path| {
        path.starts_with(
            "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/",
        )
    }));
    assert!(EXPECTED_CHANGED_FILES[0].ends_with("mod.rs"));
    assert!(
        EXPECTED_CHANGED_FILES[1]
            .ends_with("selected_variable_statement_direct_identifier_reference_initializer_frontier.rs")
    );
}

#[test]
fn fixture_evaluation_is_deterministic_and_utf8_fail_closed() {
    for (index, fixture) in RELATION_FIXTURES.iter().enumerate() {
        assert_correspondence_anchors(fixture, ISSUE_ID * 20_000 + index as u64 * 100);
        assert_correspondence_anchors(fixture, ISSUE_ID * 20_000 + index as u64 * 100);
    }

    let source = SourceText::new(SourceId::new(ISSUE_ID * 30_000), "var x=π;".to_owned());
    assert!(source.anchor(6, 7).is_err(), "mid-code-point range must fail");
    let exact = source.anchor(6, 8).expect("π exact UTF-8 range must be valid");
    assert_eq!(exact.fragment(), "π");
}
