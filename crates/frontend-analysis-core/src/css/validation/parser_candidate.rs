//! Test-only production parser candidate observation adapter (#139).
//!
//! Projects the production `CssParserRunResult` into a shape comparable
//! against the independent #138 parser gold model. Gold is never generated
//! from or rewritten by production output; this module only reads
//! production results to compare them against fixtures authored in
//! `parser_fixtures.rs`. Production code must never import parser gold.

use crate::css::declaration::{CssDeclarationOccurrence, CssDeclarationTermination};
use crate::css::parser::evidence::{
    CssParserDiscardEvidence, CssParserDiscardKind, CssParserRecoveryEvidence,
    CssParserRecoveryTermination, CssParserUnsupportedRegion,
};
use crate::css::parser::producer::run;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
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
    ParserGoldCoverage, ParserGoldDiagnosticCode, ParserGoldDiscard, ParserGoldExecutionCompletion,
    ParserGoldFixture, ParserGoldRecoveryTermination, ParserGoldTermination,
    ParserGoldTerminationLifecycle, ParserGoldUnsupportedRegion,
};

pub(super) fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

pub(super) fn generous_parser_limits() -> CssParserLimits {
    CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
    )
    .unwrap()
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

    assert_eq!(
        result.discard_records().len(),
        fixture.discard.len(),
        "{id}: discard count"
    );
    for (index, (actual, expected)) in result
        .discard_records()
        .iter()
        .zip(&fixture.discard)
        .enumerate()
    {
        assert_discard_matches(id, index, actual, *expected);
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
        (
            CssParserRecoveryTermination::EndOfInput { terminal },
            ParserGoldRecoveryTermination::EndOfInput(range),
        ) => {
            assert_range(id, "recovery eof terminal", index, terminal.range(), range);
        }
        _ => panic!("{id}: recovery {index} termination kind mismatch"),
    }
}

fn assert_discard_matches(
    id: &str,
    index: usize,
    actual: &CssParserDiscardEvidence,
    expected: ParserGoldDiscard,
) {
    match expected {
        ParserGoldDiscard::TopLevelCustomPropertyLikeQualifiedRule {
            region,
            property_name,
            colon,
        } => {
            assert_eq!(
                actual.kind(),
                CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
                "{id}: discard {index} kind"
            );
            assert_range(id, "discard region", index, actual.region().range(), region);
            assert_range(
                id,
                "discard property_name",
                index,
                actual.property_name().range(),
                property_name,
            );
            assert_range(id, "discard colon", index, actual.colon().range(), colon);
        }
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
/// Deliberately excludes `SourceId` and never formats authored or decoded
/// source strings, but otherwise covers every parser-owned deterministic
/// field: for declarations, the complete/property_name/colon/value ranges,
/// placement (owning context id, item ordinal, run ordinal; #167), priority
/// presence with its complete/bang/important_ident ranges, and termination
/// kind with evidence range; for #169 descriptor occurrences, the same
/// complete/name/colon/value/priority/termination shape under a distinct
/// `Desc` prefix and placement (owning context id, item ordinal; no run
/// ordinal); for diagnostics, code and location; for recovery, kind,
/// region, and termination kind with evidence range; for unsupported
/// regions, the variant with its complete range and, for `TopLevelAtRule`,
/// its exact `at_keyword` range; for discard records, kind, region,
/// property_name, and colon; for context records (#167/#168/#169), id,
/// parent, implicit-root-or-parent-scoped item ordinal, kind, at_keyword
/// range, descriptor property_name range, header/block_opener/body ranges,
/// and termination kind with evidence range; and for the run itself,
/// execution completion, coverage, termination, terminal, and all nine
/// `CssParserResourceUsage` dimensions.
pub(super) fn canonical_signature(result: &CssParserRunResult) -> String {
    use std::fmt::Write;

    let mut signature = String::new();
    for occurrence in result.occurrences() {
        let range = occurrence.complete().range();
        let property_name = occurrence.property_name().range();
        let colon = occurrence.colon().range();
        let value = occurrence.value().range();
        let placement = occurrence.placement();
        let _ = write!(
            signature,
            "D[{},{})Name[{},{})Colon[{},{})Value[{},{})Ctx{}Item{}Run{}",
            range.start(),
            range.end(),
            property_name.start(),
            property_name.end(),
            colon.start(),
            colon.end(),
            value.start(),
            value.end(),
            placement.context_id().index(),
            placement.item_ordinal().value(),
            placement.run_ordinal().value(),
        );
        match occurrence.priority() {
            Some(priority) => {
                let complete = priority.complete().range();
                let bang = priority.bang().range();
                let important_ident = priority.important_ident().range();
                let _ = write!(
                    signature,
                    "!P[{},{})Bang[{},{})Important[{},{})",
                    complete.start(),
                    complete.end(),
                    bang.start(),
                    bang.end(),
                    important_ident.start(),
                    important_ident.end()
                );
            }
            None => {
                let _ = write!(signature, "!NoPriority");
            }
        }
        let _ = write!(
            signature,
            "T{:?}",
            termination_shape(occurrence.termination())
        );
    }
    // #169: empty for every run without a supported descriptor context,
    // populated with real `CssDescriptorOccurrence` evidence for runs that
    // retain at least one `DescriptorRuleBlock`. Deliberately distinct from
    // the `occurrences` signature above -- a `D` prefix versus `Desc` -- so
    // two runs differing only in occurrence kind never collide.
    for occurrence in result.descriptor_occurrences() {
        let range = occurrence.complete().range();
        let name = occurrence.name().range();
        let colon = occurrence.colon().range();
        let value = occurrence.value().range();
        let placement = occurrence.placement();
        let _ = write!(
            signature,
            "|Desc[{},{})Name[{},{})Colon[{},{})Value[{},{})Ctx{}Item{}",
            range.start(),
            range.end(),
            name.start(),
            name.end(),
            colon.start(),
            colon.end(),
            value.start(),
            value.end(),
            placement.context_id().index(),
            placement.item_ordinal().value(),
        );
        match occurrence.priority() {
            Some(priority) => {
                let complete = priority.complete().range();
                let bang = priority.bang().range();
                let important_ident = priority.important_ident().range();
                let _ = write!(
                    signature,
                    "!P[{},{})Bang[{},{})Important[{},{})",
                    complete.start(),
                    complete.end(),
                    bang.start(),
                    bang.end(),
                    important_ident.start(),
                    important_ident.end()
                );
            }
            None => {
                let _ = write!(signature, "!NoPriority");
            }
        }
        let _ = write!(
            signature,
            "T{:?}",
            termination_shape(occurrence.termination())
        );
    }
    for occurrence in result.page_occurrences() {
        let complete = occurrence.complete().range();
        let name = occurrence.name().range();
        let colon = occurrence.colon().range();
        let value = occurrence.value().range();
        let placement = occurrence.placement();
        let _ = write!(
            signature,
            "|Page[{},{})Name[{},{})Colon[{},{})Value[{},{})Ctx{}Item{}",
            complete.start(),
            complete.end(),
            name.start(),
            name.end(),
            colon.start(),
            colon.end(),
            value.start(),
            value.end(),
            placement.context_id().index(),
            placement.item_ordinal().value(),
        );
        match occurrence.priority() {
            Some(priority) => {
                let complete = priority.complete().range();
                let bang = priority.bang().range();
                let important = priority.important_ident().range();
                let _ = write!(
                    signature,
                    "!P[{},{})Bang[{},{})Important[{},{})",
                    complete.start(),
                    complete.end(),
                    bang.start(),
                    bang.end(),
                    important.start(),
                    important.end()
                );
            }
            None => {
                let _ = write!(signature, "!NoPriority");
            }
        }
        let _ = write!(
            signature,
            "T{:?}",
            termination_shape(occurrence.termination())
        );
    }
    for occurrence in result.page_margin_occurrences() {
        let complete = occurrence.complete().range();
        let name = occurrence.name().range();
        let colon = occurrence.colon().range();
        let value = occurrence.value().range();
        let placement = occurrence.placement();
        let _ = write!(
            signature,
            "|PageMargin[{},{})Name[{},{})Colon[{},{})Value[{},{})Ctx{}Item{}",
            complete.start(),
            complete.end(),
            name.start(),
            name.end(),
            colon.start(),
            colon.end(),
            value.start(),
            value.end(),
            placement.context_id().index(),
            placement.item_ordinal().value(),
        );
        match occurrence.priority() {
            Some(priority) => {
                let complete = priority.complete().range();
                let bang = priority.bang().range();
                let important = priority.important_ident().range();
                let _ = write!(
                    signature,
                    "!P[{},{})Bang[{},{})Important[{},{})",
                    complete.start(),
                    complete.end(),
                    bang.start(),
                    bang.end(),
                    important.start(),
                    important.end()
                );
            }
            None => {
                let _ = write!(signature, "!NoPriority");
            }
        }
        let _ = write!(
            signature,
            "T{:?}",
            termination_shape(occurrence.termination())
        );
    }
    for occurrence in result.keyframe_occurrences() {
        let complete = occurrence.complete().range();
        let name = occurrence.name().range();
        let colon = occurrence.colon().range();
        let value = occurrence.value().range();
        let placement = occurrence.placement();
        let _ = write!(
            signature,
            "|Keyframe[{},{})Name[{},{})Colon[{},{})Value[{},{})Ctx{}Item{}",
            complete.start(),
            complete.end(),
            name.start(),
            name.end(),
            colon.start(),
            colon.end(),
            value.start(),
            value.end(),
            placement.context_id().index(),
            placement.item_ordinal().value(),
        );
        match occurrence.priority() {
            Some(priority) => {
                let complete = priority.complete().range();
                let bang = priority.bang().range();
                let important = priority.important_ident().range();
                let _ = write!(
                    signature,
                    "!P[{},{})Bang[{},{})Important[{},{})",
                    complete.start(),
                    complete.end(),
                    bang.start(),
                    bang.end(),
                    important.start(),
                    important.end()
                );
            }
            None => {
                let _ = write!(signature, "!NoPriority");
            }
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
        let _ = write!(
            signature,
            "|Rec{:?}@[{},{}){:?}",
            recovery.kind(),
            range.start(),
            range.end(),
            recovery_termination_shape(recovery.termination())
        );
    }
    for unsupported in result.unsupported_regions() {
        let _ = write!(signature, "|Unsup{}", unsupported_shape(unsupported));
    }
    for discard in result.discard_records() {
        let region = discard.region().range();
        let property_name = discard.property_name().range();
        let colon = discard.colon().range();
        let _ = write!(
            signature,
            "|Discard{:?}@[{},{})Name[{},{})Colon[{},{})",
            discard.kind(),
            region.start(),
            region.end(),
            property_name.start(),
            property_name.end(),
            colon.start(),
            colon.end()
        );
    }
    // #166 contract, #167/#168/#169 production: empty for a #166-only run,
    // populated with real QualifiedRuleBlock/GroupRuleBlock/
    // DescriptorRuleBlock records for #167/#168/#169 runs. Included so the
    // signature stays honest about every result-owned field, not merely
    // whichever ones a given run happens to populate.
    for context in result.context_records() {
        let header = context.header().range();
        let block_opener = context.block_opener().range();
        let body = context.body().range();
        let at_keyword = context.at_keyword().map(|anchor| anchor.range());
        let descriptor_property_name = context
            .descriptor_property_name()
            .map(|anchor| anchor.range());
        let page_selector_list = context.page_selector_list().map(|anchor| anchor.range());
        let keyframes_name = context.keyframes_name().map(|anchor| anchor.range());
        let keyframe_selector_list = context
            .keyframe_selector_list()
            .map(|anchor| anchor.range());
        let _ = write!(
            signature,
            "|Ctx{:?}Id{}Parent{:?}Ordinal{}AtKeyword{:?}PropertyName{:?}PageSelector{:?}KeyframesName{:?}KeyframeSelector{:?}Nearest{:?}Header[{},{})Opener[{},{})Body[{},{}){:?}",
            context.kind(),
            context.id().index(),
            context.parent().map(|parent_id| parent_id.index()),
            context.item_ordinal().value(),
            at_keyword.map(|range| (range.start(), range.end())),
            descriptor_property_name.map(|range| (range.start(), range.end())),
            page_selector_list.map(|range| (range.start(), range.end())),
            keyframes_name.map(|range| (range.start(), range.end())),
            keyframe_selector_list.map(|range| (range.start(), range.end())),
            context
                .nearest_qualified_ancestor()
                .map(|ancestor_id| ancestor_id.index()),
            header.start(),
            header.end(),
            block_opener.start(),
            block_opener.end(),
            body.start(),
            body.end(),
            context_termination_shape(context.termination()),
        );
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
    let resources = result.resources();
    let _ = write!(
        signature,
        "|Res(AlgorithmSteps={},PeakComponentDepth={},PeakContextDepth={},DeclarationOccurrences={},ParserDiagnostics={},RecoveryRecords={},UnsupportedRegions={},DiscardRecords={},ContextRecords={})",
        resources.value(CssParserResourceKind::AlgorithmSteps),
        resources.value(CssParserResourceKind::PeakComponentDepth),
        resources.value(CssParserResourceKind::PeakContextDepth),
        resources.value(CssParserResourceKind::DeclarationOccurrences),
        resources.value(CssParserResourceKind::ParserDiagnostics),
        resources.value(CssParserResourceKind::RecoveryRecords),
        resources.value(CssParserResourceKind::UnsupportedRegions),
        resources.value(CssParserResourceKind::DiscardRecords),
        resources.value(CssParserResourceKind::ContextRecords),
    );
    signature
}

fn context_termination_shape(
    termination: &crate::css::parser::context::CssParserContextTermination,
) -> (&'static str, usize, usize) {
    use crate::css::parser::context::CssParserContextTermination;
    match termination {
        CssParserContextTermination::AuthoredRightCurly { right_curly } => (
            "AuthoredRightCurly",
            right_curly.range().start(),
            right_curly.range().end(),
        ),
        CssParserContextTermination::EndOfInput { terminal } => (
            "EndOfInput",
            terminal.range().start(),
            terminal.range().end(),
        ),
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => (
            "UpstreamTokenizerIncomplete",
            terminal.range().start(),
            terminal.range().end(),
        ),
        CssParserContextTermination::ParserResourceLimit { terminal } => (
            "ParserResourceLimit",
            terminal.range().start(),
            terminal.range().end(),
        ),
    }
}

fn unsupported_shape(unsupported: &CssParserUnsupportedRegion) -> String {
    match unsupported {
        CssParserUnsupportedRegion::TopLevelAtRule {
            complete,
            at_keyword,
        } => {
            let complete = complete.range();
            let at_keyword = at_keyword.range();
            format!(
                "TopLevelAtRule@[{},{})At[{},{})",
                complete.start(),
                complete.end(),
                at_keyword.start(),
                at_keyword.end()
            )
        }
        CssParserUnsupportedRegion::NestedContentRemainder { region } => {
            let region = region.range();
            format!(
                "NestedContentRemainder@[{},{})",
                region.start(),
                region.end()
            )
        }
        CssParserUnsupportedRegion::NestedAtRule {
            complete,
            at_keyword,
            context_id,
            item_ordinal,
        } => {
            let complete = complete.range();
            let at_keyword = at_keyword.range();
            format!(
                "NestedAtRule@[{},{})At[{},{})Ctx{}Item{}",
                complete.start(),
                complete.end(),
                at_keyword.start(),
                at_keyword.end(),
                context_id.index(),
                item_ordinal.value(),
            )
        }
        CssParserUnsupportedRegion::UnqualifiedKeyframeBlock {
            complete,
            context_id,
            item_ordinal,
        } => {
            let complete = complete.range();
            format!(
                "UnqualifiedKeyframeBlock@[{},{})Ctx{}Item{}",
                complete.start(),
                complete.end(),
                context_id.index(),
                item_ordinal.value(),
            )
        }
    }
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

fn recovery_termination_shape(
    termination: &CssParserRecoveryTermination,
) -> (&'static str, usize, usize) {
    match termination {
        CssParserRecoveryTermination::AuthoredSemicolon { semicolon } => (
            "AuthoredSemicolon",
            semicolon.range().start(),
            semicolon.range().end(),
        ),
        CssParserRecoveryTermination::EnclosingBlockEnd { right_curly } => (
            "EnclosingBlockEnd",
            right_curly.range().start(),
            right_curly.range().end(),
        ),
        CssParserRecoveryTermination::EndOfInput { terminal } => (
            "EndOfInput",
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
