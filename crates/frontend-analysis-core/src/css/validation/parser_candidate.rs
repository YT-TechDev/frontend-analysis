//! Test-only production parser candidate observation adapter (#139).
//!
//! Projects the production `CssParserRunResult` into a shape comparable
//! against the independent #138 parser gold model. Gold is never generated
//! from or rewritten by production output; this module only reads
//! production results to compare them against fixtures authored in
//! `parser_fixtures.rs`. Production code must never import parser gold.

use crate::css::declaration::{CssDeclarationOccurrence, CssDeclarationTermination};
use crate::css::parser::evidence::{
    CssParserRecoveryEvidence, CssParserRecoveryTermination, CssParserUnsupportedRegion,
};
use crate::css::parser::producer::run;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::{
    CssParserCoverage, CssParserExecutionCompletion, CssParserRunResult, CssParserTermination,
};
use crate::css::token::{CssLexicalItem, CssToken, CssTokenKind};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::{CssTokenizerLimits, CssTokenizerResourceUsage};
use crate::css::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use crate::{SourceId, SourceText};

use super::gold::GoldRange;
use super::parser_gold::{
    ParserGoldCoverage, ParserGoldDiagnosticCode, ParserGoldExecutionCompletion, ParserGoldFixture,
    ParserGoldRecoveryTermination, ParserGoldTermination, ParserGoldTerminationLifecycle,
    ParserGoldUnsupportedRegion,
};

pub(super) fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

pub(super) fn generous_parser_limits() -> CssParserLimits {
    CssParserLimits::new(1 << 20, 1 << 12, 1 << 16, 1 << 16, 1 << 16, 1 << 16).unwrap()
}

/// Runs the production tokenizer then the production parser over `fixture`'s
/// source under generous finite limits.
pub(super) fn run_fixture(fixture: &ParserGoldFixture) -> CssParserRunResult {
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .unwrap_or_else(|error| panic!("{}: production tokenizer error: {error:?}", fixture.id));
    run(&source, tokenizer_result, generous_parser_limits())
        .unwrap_or_else(|error| panic!("{}: production parser error: {error:?}", fixture.id))
}

/// Lifecycle-specific synthetic upstream input: constructs a contract-valid
/// `CssTokenizerRunResult` matching the `CSS-PARSER-LIFECYCLE-TOKENIZER-
/// INCOMPLETE-001` fixture's specified upstream-incomplete terminal, then
/// feeds it into the production parser. A normal generous tokenizer run over
/// this fixture's source would complete; this path exists only to exercise
/// production parser lifecycle propagation against a deliberately
/// constructed incomplete upstream result, not as parser-recognition output
/// from a crippled tokenizer.
pub(super) fn run_fixture_with_synthetic_upstream_incomplete(
    fixture: &ParserGoldFixture,
) -> CssParserRunResult {
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let terminal_offset = fixture.terminal.start;
    assert_eq!(
        fixture.terminal.end, terminal_offset,
        "{}: synthetic upstream terminal must be an empty anchor",
        fixture.id
    );

    let mut lexical_items = Vec::new();
    let mut cursor = 0usize;
    for boundary in synthetic_lexical_boundaries(fixture.source, terminal_offset) {
        let fragment = &fixture.source[cursor..boundary];
        let kind = if fragment == "{" {
            CssTokenKind::LeftCurlyBracket
        } else if fragment == "}" {
            CssTokenKind::RightCurlyBracket
        } else {
            CssTokenKind::Ident(fragment.to_owned())
        };
        let anchor = source.anchor(cursor, boundary).unwrap();
        lexical_items.push(CssLexicalItem::SemanticToken(
            CssToken::new(&source, anchor, kind).unwrap(),
        ));
        cursor = boundary;
    }
    assert_eq!(
        cursor, terminal_offset,
        "{}: synthetic lexical coverage must reach the terminal exactly",
        fixture.id
    );

    let item_count = lexical_items.len();
    let resource_limit = crate::css::tokenizer::resource::CssTokenizerResourceLimitEvidence::new(
        &source,
        crate::css::tokenizer::resource::CssTokenizerResourceKind::AlgorithmSteps,
        1,
        2,
        source.anchor(terminal_offset, terminal_offset).unwrap(),
    )
    .unwrap();
    let tokenizer_result = CssTokenizerRunResult::new(
        &source,
        None,
        lexical_items,
        Vec::new(),
        source.anchor(0, terminal_offset).unwrap(),
        source
            .anchor(terminal_offset, fixture.source.len())
            .unwrap(),
        source.anchor(terminal_offset, terminal_offset).unwrap(),
        CssTokenizerCompletion::Incomplete,
        CssTokenizerTermination::ResourceLimit(resource_limit),
        CssTokenizerResourceUsage::new(fixture.source.len(), 1, item_count, 0, 0, 0),
    )
    .unwrap();

    run(&source, tokenizer_result, generous_parser_limits()).unwrap_or_else(|error| {
        panic!(
            "{}: production parser error on synthetic upstream: {error:?}",
            fixture.id
        )
    })
}

/// Produces synthetic single-byte-run token boundaries covering
/// `[0, terminal_offset)` so the synthetic upstream result satisfies the
/// tokenizer's gapless lexical-coverage contract without claiming any real
/// tokenization semantics past the fixture's declared truncation point.
fn synthetic_lexical_boundaries(source: &str, terminal_offset: usize) -> Vec<usize> {
    let mut boundaries = Vec::new();
    let mut position = 0usize;
    while position < terminal_offset {
        let mut next = position + 1;
        while next < terminal_offset && !source.is_char_boundary(next) {
            next += 1;
        }
        boundaries.push(next);
        position = next;
    }
    boundaries
}

pub(super) fn assert_matches_gold(fixture: &ParserGoldFixture, result: &CssParserRunResult) {
    let id = fixture.id;

    assert_eq!(
        result.occurrences().len(),
        fixture.declarations.len(),
        "{id}: declaration count"
    );
    for (index, (actual, expected)) in result
        .occurrences()
        .iter()
        .zip(&fixture.declarations)
        .enumerate()
    {
        assert_occurrence_matches(id, index, actual, expected);
    }

    assert_eq!(
        result.parser_diagnostics().len(),
        fixture.diagnostics.len(),
        "{id}: diagnostic count"
    );
    for (index, (actual, expected)) in result
        .parser_diagnostics()
        .iter()
        .zip(&fixture.diagnostics)
        .enumerate()
    {
        assert_eq!(
            map_diagnostic_code(actual.code()),
            expected.code,
            "{id}: diagnostic {index} code"
        );
        assert_range(
            id,
            "diagnostic location",
            index,
            actual.location().range(),
            expected.location,
        );
    }

    assert_eq!(
        result.recovery_records().len(),
        fixture.recovery.len(),
        "{id}: recovery count"
    );
    for (index, (actual, expected)) in result
        .recovery_records()
        .iter()
        .zip(&fixture.recovery)
        .enumerate()
    {
        assert_recovery_matches(id, index, actual, expected);
    }

    assert_eq!(
        result.unsupported_regions().len(),
        fixture.unsupported.len(),
        "{id}: unsupported region count"
    );
    for (index, (actual, expected)) in result
        .unsupported_regions()
        .iter()
        .zip(&fixture.unsupported)
        .enumerate()
    {
        assert_unsupported_matches(id, index, actual, *expected);
    }

    match fixture.execution_completion {
        ParserGoldExecutionCompletion::Complete => {
            assert_eq!(
                result.execution_completion(),
                CssParserExecutionCompletion::Complete,
                "{id}: execution completion"
            );
        }
        ParserGoldExecutionCompletion::Incomplete => {
            assert_eq!(
                result.execution_completion(),
                CssParserExecutionCompletion::Incomplete,
                "{id}: execution completion"
            );
        }
    }

    match fixture.coverage {
        ParserGoldCoverage::SupportedForSelectedQuestion => {
            assert_eq!(
                result.coverage(),
                CssParserCoverage::SupportedForSelectedQuestion,
                "{id}: coverage"
            );
        }
        ParserGoldCoverage::ContainsUnsupportedContexts => {
            assert_eq!(
                result.coverage(),
                CssParserCoverage::ContainsUnsupportedContexts,
                "{id}: coverage"
            );
        }
    }

    match fixture.termination {
        ParserGoldTerminationLifecycle::EndOfTokenizerInput => {
            assert!(
                matches!(
                    result.termination(),
                    CssParserTermination::EndOfTokenizerInput
                ),
                "{id}: termination"
            );
        }
        ParserGoldTerminationLifecycle::UpstreamTokenizerIncomplete => {
            assert!(
                matches!(
                    result.termination(),
                    CssParserTermination::UpstreamTokenizerIncomplete
                ),
                "{id}: termination"
            );
        }
        ParserGoldTerminationLifecycle::ParserResourceLimit => {
            panic!("{id}: candidate wiring only compares non-resource-limited normative fixtures");
        }
    }

    assert_range(
        id,
        "terminal",
        0,
        result.terminal().range(),
        fixture.terminal,
    );
}

fn assert_occurrence_matches(
    id: &str,
    index: usize,
    actual: &CssDeclarationOccurrence,
    expected: &super::parser_gold::ParserGoldDeclaration,
) {
    assert_range(
        id,
        "declaration complete",
        index,
        actual.complete().range(),
        expected.complete,
    );
    assert_range(
        id,
        "declaration property_name",
        index,
        actual.property_name().range(),
        expected.property_name,
    );
    assert_range(
        id,
        "declaration colon",
        index,
        actual.colon().range(),
        expected.colon,
    );
    assert_range(
        id,
        "declaration value",
        index,
        actual.value().range(),
        expected.value,
    );

    match (actual.priority(), expected.priority) {
        (None, None) => {}
        (Some(priority), Some(expected_priority)) => {
            assert_range(
                id,
                "priority complete",
                index,
                priority.complete().range(),
                expected_priority.complete,
            );
            assert_range(
                id,
                "priority bang",
                index,
                priority.bang().range(),
                expected_priority.bang,
            );
            assert_range(
                id,
                "priority important_ident",
                index,
                priority.important_ident().range(),
                expected_priority.important_ident,
            );
        }
        _ => panic!("{id}: declaration {index} priority presence mismatch"),
    }

    match (actual.termination(), expected.termination) {
        (
            CssDeclarationTermination::AuthoredSemicolon { semicolon },
            ParserGoldTermination::AuthoredSemicolon(range),
        ) => {
            assert_range(id, "termination semicolon", index, semicolon.range(), range);
        }
        (
            CssDeclarationTermination::OmittedBeforeRightCurly { right_curly },
            ParserGoldTermination::OmittedBeforeRightCurly(range),
        ) => {
            assert_range(
                id,
                "termination right_curly",
                index,
                right_curly.range(),
                range,
            );
        }
        (
            CssDeclarationTermination::OmittedAtEndOfInput { terminal },
            ParserGoldTermination::OmittedAtEndOfInput(range),
        ) => {
            assert_range(
                id,
                "termination eof terminal",
                index,
                terminal.range(),
                range,
            );
        }
        _ => panic!("{id}: declaration {index} termination kind mismatch"),
    }
}

fn assert_recovery_matches(
    id: &str,
    index: usize,
    actual: &CssParserRecoveryEvidence,
    expected: &super::parser_gold::ParserGoldRecovery,
) {
    assert_range(
        id,
        "recovery region",
        index,
        actual.region().range(),
        expected.region,
    );
    match (actual.termination(), expected.termination) {
        (
            CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
            ParserGoldRecoveryTermination::AuthoredSemicolon(range),
        ) => {
            assert_range(id, "recovery semicolon", index, semicolon.range(), range);
        }
        (
            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly },
            ParserGoldRecoveryTermination::EnclosingBlockEnd(range),
        ) => {
            assert_range(
                id,
                "recovery right_curly",
                index,
                right_curly.range(),
                range,
            );
        }
        _ => panic!("{id}: recovery {index} termination kind mismatch"),
    }
}

fn assert_unsupported_matches(
    id: &str,
    index: usize,
    actual: &CssParserUnsupportedRegion,
    expected: ParserGoldUnsupportedRegion,
) {
    match (actual, expected) {
        (
            CssParserUnsupportedRegion::TopLevelAtRule {
                complete,
                at_keyword,
            },
            ParserGoldUnsupportedRegion::TopLevelAtRule {
                complete: expected_complete,
                at_keyword: expected_at_keyword,
            },
        ) => {
            assert_range(
                id,
                "unsupported at-rule complete",
                index,
                complete.range(),
                expected_complete,
            );
            assert_range(
                id,
                "unsupported at-rule at_keyword",
                index,
                at_keyword.range(),
                expected_at_keyword,
            );
        }
        (
            CssParserUnsupportedRegion::NestedContentRemainder { region },
            ParserGoldUnsupportedRegion::NestedContentRemainder {
                region: expected_region,
            },
        ) => {
            assert_range(
                id,
                "unsupported nested remainder",
                index,
                region.range(),
                expected_region,
            );
        }
        _ => panic!("{id}: unsupported region {index} kind mismatch"),
    }
}

fn map_diagnostic_code(
    code: crate::css::parser::diagnostic::CssParserDiagnosticCode,
) -> ParserGoldDiagnosticCode {
    use crate::css::parser::diagnostic::CssParserDiagnosticCode as Code;
    match code {
        Code::InvalidStylesheetQualifiedRule => {
            ParserGoldDiagnosticCode::InvalidStylesheetQualifiedRule
        }
        Code::InvalidBlockItem => ParserGoldDiagnosticCode::InvalidBlockItem,
    }
}

fn assert_range(
    id: &str,
    label: &str,
    index: usize,
    actual: crate::SourceRange,
    expected: GoldRange,
) {
    assert_eq!(
        (actual.start(), actual.end()),
        (expected.start, expected.end),
        "{id}: {label} {index}"
    );
}

/// A canonical, order-preserving textual signature of a parser run result,
/// used only to compare two independent production runs for determinism.
/// Deliberately excludes `SourceId` and never formats authored source.
pub(super) fn canonical_signature(result: &CssParserRunResult) -> String {
    use std::fmt::Write;

    let mut signature = String::new();
    for occurrence in result.occurrences() {
        let range = occurrence.complete().range();
        let _ = write!(signature, "D[{},{})", range.start(), range.end());
        if let Some(priority) = occurrence.priority() {
            let priority_range = priority.complete().range();
            let _ = write!(
                signature,
                "!P[{},{})",
                priority_range.start(),
                priority_range.end()
            );
        }
        let _ = write!(
            signature,
            "T{:?}",
            termination_shape(occurrence.termination())
        );
    }
    for diagnostic in result.parser_diagnostics() {
        let range = diagnostic.location().range();
        let _ = write!(
            signature,
            "|Diag{:?}@[{},{})",
            diagnostic.code(),
            range.start(),
            range.end()
        );
    }
    for recovery in result.recovery_records() {
        let range = recovery.region().range();
        let _ = write!(signature, "|Rec@[{},{})", range.start(), range.end());
    }
    for unsupported in result.unsupported_regions() {
        let range = unsupported.region().range();
        let _ = write!(signature, "|Unsup@[{},{})", range.start(), range.end());
    }
    let terminal = result.terminal().range();
    let _ = write!(
        signature,
        "|{:?}|{:?}|{:?}|term[{},{})",
        result.execution_completion(),
        result.coverage(),
        termination_lifecycle_shape(result.termination()),
        terminal.start(),
        terminal.end()
    );
    signature
}

fn termination_shape(termination: &CssDeclarationTermination) -> (&'static str, usize, usize) {
    match termination {
        CssDeclarationTermination::AuthoredSemicolon { semicolon } => (
            "AuthoredSemicolon",
            semicolon.range().start(),
            semicolon.range().end(),
        ),
        CssDeclarationTermination::OmittedBeforeRightCurly { right_curly } => (
            "OmittedBeforeRightCurly",
            right_curly.range().start(),
            right_curly.range().end(),
        ),
        CssDeclarationTermination::OmittedAtEndOfInput { terminal } => (
            "OmittedAtEndOfInput",
            terminal.range().start(),
            terminal.range().end(),
        ),
    }
}

fn termination_lifecycle_shape(termination: &CssParserTermination) -> &'static str {
    match termination {
        CssParserTermination::EndOfTokenizerInput => "EndOfTokenizerInput",
        CssParserTermination::UpstreamTokenizerIncomplete => "UpstreamTokenizerIncomplete",
        CssParserTermination::ParserResourceLimit(_) => "ParserResourceLimit",
    }
}
