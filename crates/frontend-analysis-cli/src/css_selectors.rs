//! Human-readable presentation of the `fa css-selectors` report (#857).
//!
//! Analysis meaning remains owned by the CSS CoreV1 facade. This module only
//! renders the retained report and source evidence deterministically.

use std::fmt::Write as _;

use frontend_analysis_core::css::selectors::{
    CssParserCoverage, CssParserDiscardKind, CssParserRecoveryKind, CssParserRecoveryTermination,
    CssParserResourceKind, CssParserTermination, CssParserUnsupportedRegionKind, CssResourceKind,
    CssResourceRefusal, CssSelectorGrammarContext, CssSelectorIndeterminateReason,
    CssSelectorInvalidReason, CssSelectorOutcome, CssSelectorProfile, CssSelectorReport,
    CssSelectorResourceKind, CssSelectorTermination, CssSelectorUnsupportedFeature,
    CssStageCompletion, CssTokenizerResourceKind, CssTokenizerTermination,
};

use crate::render_evidence;

pub(crate) fn render_report(source_bytes: usize, report: &CssSelectorReport) -> String {
    let mut out = String::new();
    let profile = match report.profile() {
        CssSelectorProfile::CoreV1 => "CoreV1",
    };
    let _ = writeln!(out, "capability: css-selectors");
    let _ = writeln!(out, "profile: {profile}");
    let _ = writeln!(
        out,
        "source: id {}; {source_bytes} bytes",
        report.source_id().value()
    );

    let tokenizer = report.tokenizer();
    let termination = match tokenizer.termination() {
        CssTokenizerTermination::EndOfInput => "end of input".to_owned(),
        CssTokenizerTermination::ResourceLimit(refusal) => render_refusal(refusal),
    };
    let _ = writeln!(
        out,
        "tokenizer: {}; {termination}; diagnostics {}",
        completion(tokenizer.completion()),
        tokenizer.diagnostics()
    );

    let parser = report.parser();
    let termination = match parser.termination() {
        CssParserTermination::EndOfTokenizerInput => "end of tokenizer input".to_owned(),
        CssParserTermination::UpstreamTokenizerIncomplete => {
            "upstream tokenizer incomplete".to_owned()
        }
        CssParserTermination::ResourceLimit(refusal) => render_refusal(refusal),
    };
    let coverage = match parser.coverage() {
        CssParserCoverage::SupportedForSelectedQuestion => "supported for selected question",
        CssParserCoverage::ContainsUnsupportedContexts => "contains unsupported contexts",
    };
    let _ = writeln!(
        out,
        "parser: {}; {termination}; coverage {coverage}; diagnostics {}; recovery records {}; \
         unsupported regions {}; discard records {}",
        completion(parser.completion()),
        parser.diagnostics(),
        parser.recovery_records(),
        parser.unsupported_regions(),
        parser.discard_records()
    );
    for (index, record) in parser.recovery().iter().enumerate() {
        let kind = match record.kind() {
            CssParserRecoveryKind::MalformedBlockItem => "malformed block item",
        };
        let termination = match record.termination() {
            CssParserRecoveryTermination::AuthoredSemicolon => "authored semicolon",
            CssParserRecoveryTermination::EnclosingBlockEnd => "enclosing block end",
            CssParserRecoveryTermination::EndOfInput => "end of input",
        };
        let _ = writeln!(out, "recovery {}: {kind}; {termination}", index + 1);
        let _ = writeln!(out, "  source: {}", render_evidence(record.region()));
    }
    for (index, record) in parser.unsupported().iter().enumerate() {
        let kind = match record.kind() {
            CssParserUnsupportedRegionKind::TopLevelAtRule => "top-level at-rule",
            CssParserUnsupportedRegionKind::NestedContentRemainder => "nested content remainder",
            CssParserUnsupportedRegionKind::NestedAtRule => "nested at-rule",
            CssParserUnsupportedRegionKind::UnqualifiedKeyframeBlock => {
                "unqualified keyframe block"
            }
        };
        let _ = writeln!(out, "unsupported {}: {kind}", index + 1);
        let _ = writeln!(out, "  source: {}", render_evidence(record.region()));
    }
    for (index, record) in parser.discard().iter().enumerate() {
        let kind = match record.kind() {
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule => {
                "top-level custom-property-like qualified rule"
            }
        };
        let _ = writeln!(out, "discard {}: {kind}", index + 1);
        let _ = writeln!(out, "  source: {}", render_evidence(record.region()));
    }

    let selector = report.selector();
    let termination = match selector.termination() {
        CssSelectorTermination::AllRetainedQualifiedContextsProcessed => {
            "all retained qualified contexts processed".to_owned()
        }
        CssSelectorTermination::ResourceLimit(refusal) => render_refusal(refusal),
    };
    let _ = writeln!(
        out,
        "selector: {}; {termination}",
        completion(selector.completion())
    );

    let _ = writeln!(out, "observations: {}", report.observations().len());
    for (index, observation) in report.observations().iter().enumerate() {
        let (outcome, subject) = match observation.outcome() {
            CssSelectorOutcome::QualifiedBySelectedGrammar => {
                ("qualified by selected grammar".to_owned(), None)
            }
            CssSelectorOutcome::InvalidForSelectedGrammar { reason, subject } => (
                format!("invalid for selected grammar ({})", invalid_reason(*reason)),
                Some(Some(subject)),
            ),
            CssSelectorOutcome::UnsupportedBySelectedGrammarProfile { feature, subject } => (
                format!(
                    "unsupported by selected grammar profile ({})",
                    unsupported_feature(*feature)
                ),
                Some(Some(subject)),
            ),
            CssSelectorOutcome::Indeterminate { reason, subject } => (
                format!("indeterminate ({})", indeterminate_reason(*reason)),
                Some(subject.as_ref()),
            ),
        };
        let _ = writeln!(out, "observation {}: {outcome}", index + 1);
        let _ = writeln!(out, "  context: {}", render_evidence(observation.context()));
        match subject {
            None => {}
            Some(Some(subject)) => {
                let _ = writeln!(out, "  subject: {}", render_evidence(subject));
            }
            Some(None) => {
                let _ = writeln!(out, "  subject: none");
            }
        }
        let grammar = match observation.grammar_context() {
            CssSelectorGrammarContext::NormalSelectorList => "normal selector list",
            CssSelectorGrammarContext::NestedRelativeSelectorList => {
                "nested relative selector list"
            }
            CssSelectorGrammarContext::ScopedRelativeSelectorList => {
                "scoped relative selector list"
            }
        };
        let _ = writeln!(out, "  grammar: {grammar}");
    }
    out
}

fn completion(completion: CssStageCompletion) -> &'static str {
    match completion {
        CssStageCompletion::Complete => "complete",
        CssStageCompletion::Incomplete => "incomplete",
    }
}

fn render_refusal(refusal: &CssResourceRefusal) -> String {
    let kind = match refusal.kind() {
        CssResourceKind::Tokenizer(kind) => match kind {
            CssTokenizerResourceKind::SourceBytes => "source bytes",
            CssTokenizerResourceKind::AlgorithmSteps => "algorithm steps",
            CssTokenizerResourceKind::LexicalItems => "lexical items",
            CssTokenizerResourceKind::Diagnostics => "diagnostics",
            CssTokenizerResourceKind::RetainedInterpretedBytes => "retained interpreted bytes",
            CssTokenizerResourceKind::TemporaryBufferBytes => "temporary buffer bytes",
        },
        CssResourceKind::Parser(kind) => match kind {
            CssParserResourceKind::AlgorithmSteps => "algorithm steps",
            CssParserResourceKind::PeakComponentDepth => "peak component depth",
            CssParserResourceKind::PeakContextDepth => "peak context depth",
            CssParserResourceKind::DeclarationOccurrences => "declaration occurrences",
            CssParserResourceKind::ParserDiagnostics => "parser diagnostics",
            CssParserResourceKind::RecoveryRecords => "recovery records",
            CssParserResourceKind::UnsupportedRegions => "unsupported regions",
            CssParserResourceKind::DiscardRecords => "discard records",
            CssParserResourceKind::ContextRecords => "context records",
        },
        CssResourceKind::Selector(kind) => match kind {
            CssSelectorResourceKind::AlgorithmSteps => "algorithm steps",
            CssSelectorResourceKind::PeakSelectorDepth => "peak selector depth",
            CssSelectorResourceKind::Observations => "observations",
        },
    };
    let location = refusal.location().start_coordinate();
    format!(
        "resource limit {kind} (limit {}, attempted {}) at byte {}, line {}, byte column {}",
        refusal.limit(),
        refusal.attempted(),
        location.byte_offset(),
        location.line_index() + 1,
        location.byte_column() + 1
    )
}

fn invalid_reason(reason: CssSelectorInvalidReason) -> &'static str {
    match reason {
        CssSelectorInvalidReason::EmptySelectorList => "empty selector list",
        CssSelectorInvalidReason::UnexpectedComma => "unexpected comma",
        CssSelectorInvalidReason::UnexpectedCombinator => "unexpected combinator",
        CssSelectorInvalidReason::InvalidCompoundOrder => "invalid compound order",
        CssSelectorInvalidReason::InvalidAttributeSelector => "invalid attribute selector",
        CssSelectorInvalidReason::InvalidPseudoSyntax => "invalid pseudo syntax",
        CssSelectorInvalidReason::InvalidNestingSelectorPlacement => {
            "invalid nesting selector placement"
        }
        CssSelectorInvalidReason::InvalidFunctionalPseudoArgument => {
            "invalid functional pseudo argument"
        }
        CssSelectorInvalidReason::NestedHasNotAllowed => "nested :has() not allowed",
        CssSelectorInvalidReason::UnexpectedToken => "unexpected token",
    }
}

fn unsupported_feature(feature: CssSelectorUnsupportedFeature) -> &'static str {
    match feature {
        CssSelectorUnsupportedFeature::IdentifierPseudoClass => "identifier pseudo-class",
        CssSelectorUnsupportedFeature::FunctionalPseudoClass => "functional pseudo-class",
        CssSelectorUnsupportedFeature::PseudoElement => "pseudo-element",
        CssSelectorUnsupportedFeature::FunctionalPseudoElement => "functional pseudo-element",
        CssSelectorUnsupportedFeature::OtherSelectorFeature => "other selector feature",
    }
}

fn indeterminate_reason(reason: CssSelectorIndeterminateReason) -> &'static str {
    match reason {
        CssSelectorIndeterminateReason::MissingNamespaceEnvironment => {
            "missing namespace environment"
        }
        CssSelectorIndeterminateReason::UnavailableStructuralContext => {
            "unavailable structural context"
        }
    }
}
