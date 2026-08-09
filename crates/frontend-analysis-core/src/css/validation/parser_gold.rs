//! Candidate-independent CSS parser-level gold data model (#138).
//!
//! Test-only and structurally independent from the production
//! `CssDeclarationOccurrence` / `CssParserRunResult` contracts: fixtures are
//! authored here from the pinned normative specs and historical evidence
//! records, never from production constructors, a future production parser,
//! an external parser, CSSOM, or browser output.

use super::gold::GoldRange;
use crate::{SourceId, SourceText};

fn anchor_ok(source: &SourceText, range: GoldRange) -> bool {
    source.anchor(range.start, range.end).is_ok()
}

fn fragment_is(source: &SourceText, range: GoldRange, expected: &str) -> bool {
    source
        .anchor(range.start, range.end)
        .is_ok_and(|anchor| anchor.fragment() == expected)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ParserGoldGroup {
    Basic,
    Termination,
    Values,
    PropertyNames,
    Priority,
    Malformed,
    Rollback,
    Nesting,
    UnsupportedAtRule,
    InputPreprocessing,
    Lifecycle,
    Historical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldContext {
    TopLevelQualifiedRuleLeadingDeclarationZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldTermination {
    AuthoredSemicolon(GoldRange),
    OmittedBeforeRightCurly(GoldRange),
    OmittedAtEndOfInput(GoldRange),
}

impl ParserGoldTermination {
    const fn evidence_range(self) -> GoldRange {
        match self {
            Self::AuthoredSemicolon(range)
            | Self::OmittedBeforeRightCurly(range)
            | Self::OmittedAtEndOfInput(range) => range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParserGoldPriority {
    pub(super) complete: GoldRange,
    pub(super) bang: GoldRange,
    pub(super) important_ident: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParserGoldDeclaration {
    pub(super) complete: GoldRange,
    pub(super) property_name: GoldRange,
    pub(super) colon: GoldRange,
    pub(super) value: GoldRange,
    pub(super) priority: Option<ParserGoldPriority>,
    pub(super) termination: ParserGoldTermination,
    pub(super) context: ParserGoldContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ParserGoldDiagnosticCode {
    InvalidStylesheetQualifiedRule,
    InvalidBlockItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParserGoldDiagnostic {
    pub(super) code: ParserGoldDiagnosticCode,
    pub(super) location: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldRecoveryTermination {
    AuthoredSemicolon(GoldRange),
    EnclosingBlockEnd(GoldRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParserGoldRecovery {
    pub(super) region: GoldRange,
    pub(super) termination: ParserGoldRecoveryTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldUnsupportedRegion {
    TopLevelAtRule {
        complete: GoldRange,
        at_keyword: GoldRange,
    },
    NestedContentRemainder {
        region: GoldRange,
    },
}

impl ParserGoldUnsupportedRegion {
    const fn region(self) -> GoldRange {
        match self {
            Self::TopLevelAtRule { complete, .. } => complete,
            Self::NestedContentRemainder { region } => region,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldExecutionCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldCoverage {
    SupportedForSelectedQuestion,
    ContainsUnsupportedContexts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldTerminationLifecycle {
    EndOfTokenizerInput,
    UpstreamTokenizerIncomplete,
    ParserResourceLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParserGoldFixture {
    pub(super) id: &'static str,
    pub(super) group: ParserGoldGroup,
    pub(super) source_id: u64,
    pub(super) source: &'static str,
    pub(super) byte_len: usize,
    pub(super) declarations: Vec<ParserGoldDeclaration>,
    pub(super) diagnostics: Vec<ParserGoldDiagnostic>,
    pub(super) recovery: Vec<ParserGoldRecovery>,
    pub(super) unsupported: Vec<ParserGoldUnsupportedRegion>,
    pub(super) terminal: GoldRange,
    pub(super) execution_completion: ParserGoldExecutionCompletion,
    pub(super) coverage: ParserGoldCoverage,
    pub(super) termination: ParserGoldTerminationLifecycle,
}

impl ParserGoldFixture {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn complete(
        id: &'static str,
        group: ParserGoldGroup,
        source_id: u64,
        source: &'static str,
        byte_len: usize,
        declarations: Vec<ParserGoldDeclaration>,
        diagnostics: Vec<ParserGoldDiagnostic>,
        recovery: Vec<ParserGoldRecovery>,
        unsupported: Vec<ParserGoldUnsupportedRegion>,
    ) -> Self {
        let terminal = GoldRange::new(byte_len, byte_len);
        let coverage = if unsupported.is_empty() {
            ParserGoldCoverage::SupportedForSelectedQuestion
        } else {
            ParserGoldCoverage::ContainsUnsupportedContexts
        };
        Self {
            id,
            group,
            source_id,
            source,
            byte_len,
            declarations,
            diagnostics,
            recovery,
            unsupported,
            terminal,
            execution_completion: ParserGoldExecutionCompletion::Complete,
            coverage,
            termination: ParserGoldTerminationLifecycle::EndOfTokenizerInput,
        }
    }

    pub(super) fn upstream_incomplete(
        id: &'static str,
        group: ParserGoldGroup,
        source_id: u64,
        source: &'static str,
        byte_len: usize,
        terminal: GoldRange,
    ) -> Self {
        Self {
            id,
            group,
            source_id,
            source,
            byte_len,
            declarations: Vec::new(),
            diagnostics: Vec::new(),
            recovery: Vec::new(),
            unsupported: Vec::new(),
            terminal,
            execution_completion: ParserGoldExecutionCompletion::Incomplete,
            coverage: ParserGoldCoverage::SupportedForSelectedQuestion,
            termination: ParserGoldTerminationLifecycle::UpstreamTokenizerIncomplete,
        }
    }

    pub(super) fn parser_resource_limited(
        id: &'static str,
        source_id: u64,
        source: &'static str,
        byte_len: usize,
        declarations: Vec<ParserGoldDeclaration>,
        terminal: GoldRange,
    ) -> Self {
        Self {
            id,
            group: ParserGoldGroup::Lifecycle,
            source_id,
            source,
            byte_len,
            declarations,
            diagnostics: Vec::new(),
            recovery: Vec::new(),
            unsupported: Vec::new(),
            terminal,
            execution_completion: ParserGoldExecutionCompletion::Incomplete,
            coverage: ParserGoldCoverage::SupportedForSelectedQuestion,
            termination: ParserGoldTerminationLifecycle::ParserResourceLimit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserGoldValidationError {
    ByteLengthMismatch,
    InvalidRange,
    InvalidTerminal,
    CompletionTerminationMismatch,
    DeclarationOrderViolation,
    DeclarationOverlap,
    DeclarationBeyondTerminal,
    PropertyNameOutsideComplete,
    ColonOrderViolation,
    ValueOrderViolation,
    ZeroLengthValueNotAtColonEnd,
    PriorityOrderViolation,
    PriorityContainmentViolation,
    TerminationInconsistentWithCompleteEnd,
    OmittedEofTerminalNotAtSourceEnd,
    DiagnosticOrder,
    RecoveryOrder,
    UnsupportedOrder,
    DeclarationOverlapsUnsupportedRegion,
    CoverageUnsupportedMismatch,
    WrongFixedDelimiter,
    OmittedAtEndOfInputRequiresUpstreamComplete,
}

pub(super) fn validate_fixture(
    fixture: &ParserGoldFixture,
) -> Result<(), ParserGoldValidationError> {
    if fixture.source.len() != fixture.byte_len {
        return Err(ParserGoldValidationError::ByteLengthMismatch);
    }
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());

    if !fixture.terminal.is_empty() || !anchor_ok(&source, fixture.terminal) {
        return Err(ParserGoldValidationError::InvalidTerminal);
    }

    match (fixture.execution_completion, fixture.termination) {
        (
            ParserGoldExecutionCompletion::Complete,
            ParserGoldTerminationLifecycle::EndOfTokenizerInput,
        ) => {
            if fixture.terminal.start != fixture.byte_len {
                return Err(ParserGoldValidationError::InvalidTerminal);
            }
        }
        (
            ParserGoldExecutionCompletion::Incomplete,
            ParserGoldTerminationLifecycle::UpstreamTokenizerIncomplete
            | ParserGoldTerminationLifecycle::ParserResourceLimit,
        ) => {}
        _ => return Err(ParserGoldValidationError::CompletionTerminationMismatch),
    }

    let upstream_ended_at_true_eof = fixture.execution_completion
        == ParserGoldExecutionCompletion::Complete
        && fixture.termination == ParserGoldTerminationLifecycle::EndOfTokenizerInput;

    let mut previous_end: Option<usize> = None;
    for declaration in &fixture.declarations {
        validate_declaration(&source, declaration, fixture.terminal.start)?;
        if declaration.complete.end > fixture.terminal.start {
            return Err(ParserGoldValidationError::DeclarationBeyondTerminal);
        }
        if let Some(previous_end) = previous_end
            && declaration.complete.start < previous_end
        {
            return Err(ParserGoldValidationError::DeclarationOverlap);
        }
        previous_end = Some(declaration.complete.end);

        for unsupported in &fixture.unsupported {
            if ranges_overlap(declaration.complete, unsupported.region()) {
                return Err(ParserGoldValidationError::DeclarationOverlapsUnsupportedRegion);
            }
        }

        // `OmittedAtEndOfInput` is valid declaration-termination evidence
        // only when the upstream tokenizer completed at true `EndOfInput`;
        // it must never stand in for a resource-limited or otherwise
        // incomplete upstream terminal.
        if matches!(
            declaration.termination,
            ParserGoldTermination::OmittedAtEndOfInput(_)
        ) && !upstream_ended_at_true_eof
        {
            return Err(ParserGoldValidationError::OmittedAtEndOfInputRequiresUpstreamComplete);
        }
    }

    let mut previous_key = None;
    for diagnostic in &fixture.diagnostics {
        if !anchor_ok(&source, diagnostic.location)
            || diagnostic.location.end > fixture.terminal.start
        {
            return Err(ParserGoldValidationError::InvalidRange);
        }
        let key = (diagnostic.location.start, diagnostic.location.end);
        if previous_key.is_some_and(|previous| previous > key) {
            return Err(ParserGoldValidationError::DiagnosticOrder);
        }
        previous_key = Some(key);
    }

    let mut previous_key = None;
    for record in &fixture.recovery {
        if !anchor_ok(&source, record.region) || record.region.is_empty() {
            return Err(ParserGoldValidationError::InvalidRange);
        }
        match record.termination {
            ParserGoldRecoveryTermination::AuthoredSemicolon(semicolon) => {
                if !anchor_ok(&source, semicolon)
                    || semicolon.end != record.region.end
                    || semicolon.start < record.region.start
                {
                    return Err(ParserGoldValidationError::InvalidRange);
                }
                if !fragment_is(&source, semicolon, ";") {
                    return Err(ParserGoldValidationError::WrongFixedDelimiter);
                }
            }
            ParserGoldRecoveryTermination::EnclosingBlockEnd(right_curly) => {
                if !anchor_ok(&source, right_curly) || right_curly.start != record.region.end {
                    return Err(ParserGoldValidationError::InvalidRange);
                }
                if !fragment_is(&source, right_curly, "}") {
                    return Err(ParserGoldValidationError::WrongFixedDelimiter);
                }
            }
        }
        let key = (record.region.start, record.region.end);
        if previous_key.is_some_and(|previous| previous > key) {
            return Err(ParserGoldValidationError::RecoveryOrder);
        }
        previous_key = Some(key);
    }

    let mut previous_key = None;
    for region in &fixture.unsupported {
        let range = region.region();
        if !anchor_ok(&source, range) || range.is_empty() {
            return Err(ParserGoldValidationError::InvalidRange);
        }
        if let ParserGoldUnsupportedRegion::TopLevelAtRule {
            complete,
            at_keyword,
        } = region
        {
            if !anchor_ok(&source, *at_keyword)
                || at_keyword.is_empty()
                || at_keyword.start != complete.start
                || at_keyword.end > complete.end
            {
                return Err(ParserGoldValidationError::InvalidRange);
            }
            let starts_with_at = source
                .anchor(at_keyword.start, at_keyword.end)
                .is_ok_and(|anchor| anchor.fragment().starts_with('@'));
            if !starts_with_at {
                return Err(ParserGoldValidationError::WrongFixedDelimiter);
            }
        }
        let key = (range.start, range.end);
        if previous_key.is_some_and(|previous| previous > key) {
            return Err(ParserGoldValidationError::UnsupportedOrder);
        }
        previous_key = Some(key);
    }

    let has_unsupported = !fixture.unsupported.is_empty();
    let coverage_ok = match fixture.coverage {
        ParserGoldCoverage::SupportedForSelectedQuestion => !has_unsupported,
        ParserGoldCoverage::ContainsUnsupportedContexts => has_unsupported,
    };
    if !coverage_ok {
        return Err(ParserGoldValidationError::CoverageUnsupportedMismatch);
    }

    Ok(())
}

fn validate_declaration(
    source: &SourceText,
    declaration: &ParserGoldDeclaration,
    terminal: usize,
) -> Result<(), ParserGoldValidationError> {
    let complete = declaration.complete;
    let property_name = declaration.property_name;
    let colon = declaration.colon;
    let value = declaration.value;

    if !anchor_ok(source, complete) || complete.is_empty() {
        return Err(ParserGoldValidationError::InvalidRange);
    }
    if !anchor_ok(source, property_name) || property_name.is_empty() {
        return Err(ParserGoldValidationError::InvalidRange);
    }
    if !anchor_ok(source, colon) || colon.is_empty() {
        return Err(ParserGoldValidationError::InvalidRange);
    }
    if !anchor_ok(source, value) {
        return Err(ParserGoldValidationError::InvalidRange);
    }

    if complete.start != property_name.start || !complete.contains(property_name) {
        return Err(ParserGoldValidationError::PropertyNameOutsideComplete);
    }
    if !complete.contains(colon) || property_name.end > colon.start {
        return Err(ParserGoldValidationError::ColonOrderViolation);
    }
    if !fragment_is(source, colon, ":") {
        return Err(ParserGoldValidationError::WrongFixedDelimiter);
    }
    if value.is_empty() {
        if value.start != colon.end {
            return Err(ParserGoldValidationError::ZeroLengthValueNotAtColonEnd);
        }
    } else {
        if colon.end > value.start || !complete.contains(value) {
            return Err(ParserGoldValidationError::ValueOrderViolation);
        }
    }

    if let Some(priority) = declaration.priority {
        if !anchor_ok(source, priority.complete)
            || !anchor_ok(source, priority.bang)
            || !anchor_ok(source, priority.important_ident)
            || priority.complete.is_empty()
            || priority.bang.is_empty()
            || priority.important_ident.is_empty()
        {
            return Err(ParserGoldValidationError::InvalidRange);
        }
        if priority.complete.start != priority.bang.start
            || priority.complete.end != priority.important_ident.end
            || priority.bang.end > priority.important_ident.start
            || !priority.complete.contains(priority.important_ident)
        {
            return Err(ParserGoldValidationError::PriorityOrderViolation);
        }
        if !complete.contains(priority.complete) {
            return Err(ParserGoldValidationError::PropertyNameOutsideComplete);
        }
        if !fragment_is(source, priority.bang, "!") {
            return Err(ParserGoldValidationError::WrongFixedDelimiter);
        }
        let value_end_or_point = if value.is_empty() {
            value.start
        } else {
            value.end
        };
        if value_end_or_point > priority.complete.start {
            return Err(ParserGoldValidationError::PriorityContainmentViolation);
        }
    }

    let expected_omitted_end = declaration.priority.map_or_else(
        || {
            if value.is_empty() {
                colon.end
            } else {
                value.end
            }
        },
        |priority| priority.complete.end,
    );

    match declaration.termination {
        ParserGoldTermination::AuthoredSemicolon(semicolon) => {
            if !anchor_ok(source, semicolon) || semicolon.is_empty() {
                return Err(ParserGoldValidationError::InvalidRange);
            }
            if complete.end != semicolon.end {
                return Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd);
            }
            if !fragment_is(source, semicolon, ";") {
                return Err(ParserGoldValidationError::WrongFixedDelimiter);
            }
        }
        ParserGoldTermination::OmittedBeforeRightCurly(right_curly) => {
            if !anchor_ok(source, right_curly) || right_curly.is_empty() {
                return Err(ParserGoldValidationError::InvalidRange);
            }
            if complete.end != expected_omitted_end || right_curly.start < complete.end {
                return Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd);
            }
            if !fragment_is(source, right_curly, "}") {
                return Err(ParserGoldValidationError::WrongFixedDelimiter);
            }
        }
        ParserGoldTermination::OmittedAtEndOfInput(eof) => {
            if !eof.is_empty() {
                return Err(ParserGoldValidationError::InvalidRange);
            }
            if complete.end != expected_omitted_end {
                return Err(ParserGoldValidationError::TerminationInconsistentWithCompleteEnd);
            }
            if eof.start != source.as_str().len() {
                return Err(ParserGoldValidationError::OmittedEofTerminalNotAtSourceEnd);
            }
        }
    }

    let _ = declaration.termination.evidence_range();
    let _ = terminal;
    Ok(())
}

fn ranges_overlap(left: GoldRange, right: GoldRange) -> bool {
    left.start < right.end && right.start < left.end
}
