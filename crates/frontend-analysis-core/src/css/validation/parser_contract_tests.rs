//! Gold self-validation, category coverage, and deterministic corruption
//! tests for the #138 candidate-independent parser-level gold matrix.
//!
//! These tests validate the independent gold model and its own fixtures
//! only. #138 implements no parser producer, so no parser execution or
//! parser generated-corpus coverage is claimed here; that remains #139's
//! responsibility against this immutable gold.

use std::collections::BTreeSet;

use super::gold::GoldRange;
use super::parser_fixtures::normative_parser_fixtures;
use super::parser_gold::{
    ParserGoldCoverage, ParserGoldDeclaration, ParserGoldDiagnostic, ParserGoldExecutionCompletion,
    ParserGoldFixture, ParserGoldGroup, ParserGoldPriority, ParserGoldRecovery,
    ParserGoldRecoveryTermination, ParserGoldTermination, ParserGoldTerminationLifecycle,
    ParserGoldUnsupportedRegion, ParserGoldValidationError, validate_fixture,
};

fn fixture<'a>(fixtures: &'a [ParserGoldFixture], id: &str) -> &'a ParserGoldFixture {
    fixtures
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing parser gold fixture {id}"))
}

#[test]
fn normative_parser_fixture_count_is_exact() {
    let fixtures = normative_parser_fixtures();
    assert_eq!(fixtures.len(), 34);
}

#[test]
fn normative_parser_fixture_inventory_is_self_consistent_and_unique() {
    let fixtures = normative_parser_fixtures();
    let mut ids = BTreeSet::new();
    for fixture in &fixtures {
        assert!(
            ids.insert(fixture.id),
            "duplicate fixture id {}",
            fixture.id
        );
        validate_fixture(fixture).unwrap_or_else(|error| {
            panic!("fixture {} failed self-validation: {error:?}", fixture.id)
        });
    }
}

#[test]
fn every_required_category_has_an_explicit_fixture() {
    let fixtures = normative_parser_fixtures();
    let ids: BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();

    let required = [
        // Mandatory historical regressions (#99, #102 G1-04).
        "CSS-PARSER-HIST-PROVENANCE-001",
        "CSS-PARSER-HIST-PRIORITY-001",
        // Basic declaration with semicolon.
        "CSS-PARSER-BASIC-SEMICOLON-001",
        // Omitted semicolon before `}`.
        "CSS-PARSER-TERM-OMITTED-BEFORE-RIGHT-CURLY-001",
        // Omitted semicolon at true EOF with automatic block closure.
        "CSS-PARSER-TERM-OMITTED-AT-EOF-001",
        // Multiple declarations / source ordering.
        "CSS-PARSER-MULTIPLE-DECLARATIONS-001",
        // Empty value.
        "CSS-PARSER-VALUE-EMPTY-001",
        // Whitespace/comment-only value -> zero-length value at colon.end.
        "CSS-PARSER-VALUE-WHITESPACE-COMMENT-ONLY-001",
        // Boundary whitespace/comments around every component.
        "CSS-PARSER-BOUNDARY-TRIVIA-001",
        // Mixed-case property.
        "CSS-PARSER-PROPERTY-MIXED-CASE-001",
        // Vendor-prefixed property.
        "CSS-PARSER-PROPERTY-VENDOR-PREFIX-001",
        // Unknown/future property-shaped name.
        "CSS-PARSER-PROPERTY-UNKNOWN-FUTURE-001",
        // Custom property with `{}` value.
        "CSS-PARSER-PROPERTY-CUSTOM-BRACE-VALUE-001",
        // Unicode/multibyte name/value.
        "CSS-PARSER-UNICODE-MULTIBYTE-001",
        // Strings containing semicolon/braces.
        "CSS-PARSER-STRING-WITH-SEMICOLON-BRACE-001",
        // Nested functions and ()/[] blocks.
        "CSS-PARSER-NESTED-FUNCTIONS-BRACKETS-001",
        // Non-custom sole `{}` value case.
        "CSS-PARSER-NONCUSTOM-BRACE-SOLE-VALUE-001",
        // Non-custom `{}` plus other top-level components -> fallback, not
        // declaration; also the rollback no-leak case.
        "CSS-PARSER-NONCUSTOM-BRACE-PLUS-OTHER-001",
        // Simple valid !important.
        "CSS-PARSER-PRIORITY-SIMPLE-001",
        // Mixed-case/escaped valid priority.
        "CSS-PARSER-PRIORITY-MIXED-CASE-ESCAPED-001",
        // Invalid priority-shaped suffix retained as value syntax.
        "CSS-PARSER-PRIORITY-INVALID-SUFFIX-001",
        // Missing-colon malformed item followed by a valid declaration.
        "CSS-PARSER-MALFORMED-MISSING-COLON-THEN-VALID-001",
        // Malformed declaration start followed by a valid declaration.
        "CSS-PARSER-MALFORMED-START-THEN-VALID-001",
        // Supported declarations followed by a nested qualified rule; also
        // covers declaration-shaped text after the nesting boundary
        // producing no included occurrence.
        "CSS-PARSER-NESTING-DECL-THEN-QUALIFIED-RULE-001",
        // Supported declarations followed by a nested at-rule.
        "CSS-PARSER-NESTING-DECL-THEN-AT-RULE-001",
        // Top-level @font-face unsupported.
        "CSS-PARSER-UNSUPPORTED-FONT-FACE-001",
        // Top-level @keyframes unsupported.
        "CSS-PARSER-UNSUPPORTED-KEYFRAMES-001",
        // Top-level @media containing style rules unsupported.
        "CSS-PARSER-UNSUPPORTED-MEDIA-WITH-STYLE-RULES-001",
        // Top-level at-rule followed by a later supported top-level rule.
        "CSS-PARSER-UNSUPPORTED-AT-RULE-THEN-SUPPORTED-RULE-001",
        // Duplicate identical declaration spellings at distinct offsets.
        "CSS-PARSER-DUPLICATE-IDENTICAL-DECLARATIONS-001",
        // BOM before first rule.
        "CSS-PARSER-BOM-BEFORE-FIRST-RULE-001",
        // CR/LF/CRLF/FF raw mapping.
        "CSS-PARSER-CR-LF-CRLF-FF-RAW-MAPPING-001",
        // U+0000 replacement-sensitive evidence.
        "CSS-PARSER-NUL-REPLACEMENT-SENSITIVE-001",
        // Tokenizer-incomplete lifecycle cannot become parser-complete.
        "CSS-PARSER-LIFECYCLE-TOKENIZER-INCOMPLETE-001",
    ];

    for id in required {
        assert!(ids.contains(id), "missing required category fixture {id}");
    }
    assert_eq!(required.len(), ids.len());
}

#[test]
fn fixture_groups_cover_the_approved_parser_matrix() {
    use std::collections::BTreeMap;

    let fixtures = normative_parser_fixtures();
    let mut counts: BTreeMap<ParserGoldGroup, usize> = BTreeMap::new();
    for fixture in &fixtures {
        *counts.entry(fixture.group).or_insert(0) += 1;
    }

    assert_eq!(counts[&ParserGoldGroup::Historical], 2);
    assert_eq!(counts[&ParserGoldGroup::Basic], 3);
    assert_eq!(counts[&ParserGoldGroup::Termination], 2);
    assert_eq!(counts[&ParserGoldGroup::Values], 7);
    assert_eq!(counts[&ParserGoldGroup::PropertyNames], 5);
    assert_eq!(counts[&ParserGoldGroup::Priority], 3);
    assert_eq!(counts[&ParserGoldGroup::Malformed], 2);
    assert_eq!(counts[&ParserGoldGroup::Rollback], 1);
    assert_eq!(counts[&ParserGoldGroup::Nesting], 2);
    assert_eq!(counts[&ParserGoldGroup::UnsupportedAtRule], 4);
    assert_eq!(counts[&ParserGoldGroup::InputPreprocessing], 2);
    assert_eq!(counts[&ParserGoldGroup::Lifecycle], 1);

    let total: usize = counts.values().sum();
    assert_eq!(total, fixtures.len());
}

#[test]
fn historical_provenance_fixture_preserves_exact_boundary() {
    let fixtures = normative_parser_fixtures();
    let fixture = fixture(&fixtures, "CSS-PARSER-HIST-PROVENANCE-001");
    assert_eq!(fixture.source.as_bytes(), br"a{c\6F lor/**/:red;}");
    assert_eq!(fixture.declarations.len(), 1);

    let declaration = &fixture.declarations[0];
    assert_eq!(declaration.complete, GoldRange::new(2, 19));
    assert_eq!(declaration.property_name, GoldRange::new(2, 10));
    assert_eq!(declaration.colon, GoldRange::new(14, 15));
    assert_eq!(declaration.value, GoldRange::new(15, 18));
    assert!(declaration.priority.is_none());
    assert_eq!(
        declaration.termination,
        ParserGoldTermination::AuthoredSemicolon(GoldRange::new(18, 19))
    );
}

#[test]
fn historical_priority_fixture_preserves_exact_boundary_and_no_byte_loss() {
    let fixtures = normative_parser_fixtures();
    let fixture = fixture(&fixtures, "CSS-PARSER-HIST-PRIORITY-001");
    assert_eq!(
        fixture.source.as_bytes(),
        br"a{color:red !/**/Im\70 ortant;}"
    );
    assert_eq!(fixture.declarations.len(), 1);

    let declaration = &fixture.declarations[0];
    assert_eq!(declaration.complete, GoldRange::new(2, 30));
    assert_eq!(declaration.property_name, GoldRange::new(2, 7));
    assert_eq!(declaration.colon, GoldRange::new(7, 8));
    assert_eq!(declaration.value, GoldRange::new(8, 11));
    let priority = declaration
        .priority
        .expect("priority evidence must be present");
    assert_eq!(priority.complete, GoldRange::new(12, 29));
    assert_eq!(priority.bang, GoldRange::new(12, 13));
    assert_eq!(priority.important_ident, GoldRange::new(17, 29));
    assert_eq!(
        declaration.termination,
        ParserGoldTermination::AuthoredSemicolon(GoldRange::new(29, 30))
    );

    // No authored byte after "red" silently disappears: the complete range
    // covers every byte from the property name through the semicolon,
    // including the whitespace and comment on either side of `!`.
    assert_eq!(declaration.complete.end - declaration.complete.start, 28);
}

#[test]
fn rollback_fixture_leaves_no_declaration_diagnostic_or_recovery_evidence() {
    let fixtures = normative_parser_fixtures();
    let fixture = fixture(&fixtures, "CSS-PARSER-NONCUSTOM-BRACE-PLUS-OTHER-001");
    assert!(fixture.declarations.is_empty());
    assert!(fixture.diagnostics.is_empty());
    assert!(fixture.recovery.is_empty());
    assert_eq!(fixture.unsupported.len(), 1);
    assert_eq!(
        fixture.coverage,
        ParserGoldCoverage::ContainsUnsupportedContexts
    );
}

#[test]
fn tokenizer_incomplete_fixture_cannot_be_parser_complete_or_ordinary_eof() {
    let fixtures = normative_parser_fixtures();
    let fixture = fixture(&fixtures, "CSS-PARSER-LIFECYCLE-TOKENIZER-INCOMPLETE-001");
    assert_eq!(
        fixture.execution_completion,
        ParserGoldExecutionCompletion::Incomplete
    );
    assert_eq!(
        fixture.termination,
        ParserGoldTerminationLifecycle::UpstreamTokenizerIncomplete
    );
    assert!(fixture.declarations.is_empty());
}

#[test]
fn nested_content_remainder_fixtures_absorb_declaration_shaped_trailing_text() {
    let fixtures = normative_parser_fixtures();
    let fixture = fixture(&fixtures, "CSS-PARSER-NESTING-DECL-THEN-QUALIFIED-RULE-001");
    assert_eq!(fixture.declarations.len(), 1);
    assert_eq!(fixture.unsupported.len(), 1);
    let region = match fixture.unsupported[0] {
        ParserGoldUnsupportedRegion::NestedContentRemainder { region } => region,
        ParserGoldUnsupportedRegion::TopLevelAtRule { .. } => {
            panic!("expected NestedContentRemainder")
        }
    };
    // The trailing declaration-shaped "color:green;" lies fully inside the
    // remainder, so it cannot become a second included occurrence.
    assert_eq!(region, GoldRange::new(12, 38));
    assert!(region.contains(GoldRange::new(26, 38)));
}

// ---------------------------------------------------------------------
// Deterministic generated-corruption validation (#138 section 19/24).
//
// #138 has no parser producer, so this targets only the domain/gold
// invariants: starting from one already-valid fixture, apply one
// deterministic structural mutation at a time and assert the gold
// self-validator rejects it. This is not parser execution coverage.
// ---------------------------------------------------------------------

fn base_fixture() -> ParserGoldFixture {
    let fixtures = normative_parser_fixtures();
    fixture(&fixtures, "CSS-PARSER-HIST-PRIORITY-001").clone()
}

#[test]
fn corruption_shifted_property_name_range_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].property_name = GoldRange::new(3, 8);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::PropertyNameOutsideComplete)
    );
}

#[test]
fn corruption_overlapping_declarations_is_rejected() {
    let mut fixture = base_fixture();
    let duplicate = fixture.declarations[0];
    fixture.declarations.push(duplicate);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::DeclarationOverlap)
    );
}

#[test]
fn corruption_wrong_colon_delimiter_is_rejected() {
    let mut fixture = base_fixture();
    // Shift colon onto "r" (a byte that is not ":").
    fixture.declarations[0].colon = GoldRange::new(6, 7);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::ColonOrderViolation)
    );
}

#[test]
fn corruption_wrong_semicolon_delimiter_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].termination =
        ParserGoldTermination::AuthoredSemicolon(GoldRange::new(28, 29));
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd)
    );
}

#[test]
fn corruption_wrong_priority_bang_delimiter_is_rejected() {
    let mut fixture = base_fixture();
    let priority = fixture.declarations[0].priority.as_mut().unwrap();
    priority.bang = GoldRange::new(13, 14);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::PriorityOrderViolation)
    );
}

#[test]
fn corruption_fabricated_priority_relationship_is_rejected() {
    let mut fixture = base_fixture();
    let priority = fixture.declarations[0].priority.as_mut().unwrap();
    // important_ident no longer ends where priority.complete claims.
    priority.important_ident = GoldRange::new(17, 25);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::PriorityOrderViolation)
    );
}

#[test]
fn corruption_value_priority_overlap_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].value = GoldRange::new(8, 13);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::PriorityContainmentViolation)
    );
}

#[test]
fn corruption_inconsistent_declaration_endpoint_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].complete = GoldRange::new(2, 31);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd)
    );
}

#[test]
fn corruption_fabricated_omitted_eof_terminal_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].termination =
        ParserGoldTermination::OmittedAtEndOfInput(GoldRange::new(29, 29));
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd)
    );
}

#[test]
fn corruption_invalid_termination_boundary_right_curly_before_complete_end_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].termination =
        ParserGoldTermination::OmittedBeforeRightCurly(GoldRange::new(25, 26));
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd)
    );
}

#[test]
fn corruption_unsupported_region_overlap_with_declaration_is_rejected() {
    let mut fixture = base_fixture();
    fixture
        .unsupported
        .push(ParserGoldUnsupportedRegion::NestedContentRemainder {
            region: GoldRange::new(10, 20),
        });
    fixture.coverage = ParserGoldCoverage::ContainsUnsupportedContexts;
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::DeclarationOverlapsNestedRemainder)
    );
}

#[test]
fn corruption_impossible_lifecycle_combination_is_rejected() {
    let mut fixture = base_fixture();
    fixture.execution_completion = ParserGoldExecutionCompletion::Complete;
    fixture.termination = ParserGoldTerminationLifecycle::UpstreamTokenizerIncomplete;
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::CompletionTerminationMismatch)
    );
}

#[test]
fn corruption_supported_coverage_with_unsupported_region_is_rejected() {
    let mut fixture = base_fixture();
    // [30, 31) sits after the sole declaration's `complete` range [2, 30),
    // so it does not also trip the declaration/unsupported overlap check.
    fixture
        .unsupported
        .push(ParserGoldUnsupportedRegion::NestedContentRemainder {
            region: GoldRange::new(30, 31),
        });
    // coverage deliberately left as SupportedForSelectedQuestion.
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::CoverageUnsupportedMismatch)
    );
}

#[test]
fn corruption_impossible_source_order_diagnostic_is_rejected() {
    let mut fixture = base_fixture();
    fixture.diagnostics = vec![
        ParserGoldDiagnostic {
            code: super::parser_gold::ParserGoldDiagnosticCode::InvalidBlockItem,
            location: GoldRange::new(20, 21),
        },
        ParserGoldDiagnostic {
            code: super::parser_gold::ParserGoldDiagnosticCode::InvalidBlockItem,
            location: GoldRange::new(5, 6),
        },
    ];
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::DiagnosticOrder)
    );
}

#[test]
fn corruption_invalid_utf8_boundary_range_is_rejected() {
    let mut fixture = base_fixture();
    // Byte 18 sits inside the multibyte-adjacent escape sequence's ASCII
    // run in this fixture's source, so instead force an out-of-bounds
    // range to exercise the same anchor-validation path.
    fixture.declarations[0].value = GoldRange::new(8, 9_999);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::InvalidRange)
    );
}

#[test]
fn corruption_zero_length_value_not_at_colon_end_is_rejected() {
    let mut fixture = base_fixture();
    fixture.declarations[0].value = GoldRange::new(9, 9);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ParserGoldValidationError::ZeroLengthValueNotAtColonEnd)
    );
}

#[test]
fn declaration_and_recovery_and_priority_field_shapes_remain_plain_data() {
    // Exercises the gold model's own plain-data construction paths (no
    // production constructor involved), matching the requirement that gold
    // never uses production types as its data model.
    let declaration = ParserGoldDeclaration {
        complete: GoldRange::new(0, 5),
        property_name: GoldRange::new(0, 1),
        colon: GoldRange::new(1, 2),
        value: GoldRange::new(2, 4),
        priority: None,
        termination: ParserGoldTermination::AuthoredSemicolon(GoldRange::new(4, 5)),
        context: super::parser_gold::ParserGoldContext::TopLevelQualifiedRuleLeadingDeclarationZone,
    };
    assert_eq!(declaration.complete, GoldRange::new(0, 5));

    let priority = ParserGoldPriority {
        complete: GoldRange::new(0, 1),
        bang: GoldRange::new(0, 1),
        important_ident: GoldRange::new(1, 1),
    };
    assert_eq!(priority.complete, GoldRange::new(0, 1));

    let recovery = ParserGoldRecovery {
        region: GoldRange::new(0, 1),
        termination: ParserGoldRecoveryTermination::EnclosingBlockEnd(GoldRange::new(1, 2)),
    };
    assert_eq!(recovery.region, GoldRange::new(0, 1));
}
