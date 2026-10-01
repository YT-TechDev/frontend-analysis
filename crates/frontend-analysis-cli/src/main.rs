//! `fa`: the Frontend Analysis command-line Product (Issue #857, ADR 0011).
//!
//! Phase 1 supports exactly two commands, each reading one source from stdin:
//!
//! ```text
//! fa css-selectors < style.css
//! fa es-binding-refs < source.js
//! ```
//!
//! The Product owns argv, bounded stdin acquisition, strict UTF-8 decoding,
//! invocation-local source identity, human-readable presentation, and the
//! process exit status. Analysis meaning is owned by Core and is reported
//! here without reinterpretation.
//!
//! Exit status: `0` when a report was produced (whatever its analysis
//! outcomes), `1` for a Product command or input acquisition failure, and `2`
//! for a returned Core boundary failure.

mod es_binding_refs;

use std::ffi::OsString;
use std::fmt::{Display, Write as _};
use std::io::{self, Read, Write};
use std::process::ExitCode;

use frontend_analysis_core::css::selectors::{
    CssParserCoverage, CssParserDiscardKind, CssParserRecoveryKind, CssParserRecoveryTermination,
    CssParserResourceKind, CssParserTermination, CssParserUnsupportedRegionKind, CssResourceKind,
    CssResourceRefusal, CssSelectorGrammarContext, CssSelectorIndeterminateReason,
    CssSelectorInvalidReason, CssSelectorOutcome, CssSelectorProfile, CssSelectorReport,
    CssSelectorResourceKind, CssSelectorTermination, CssSelectorUnsupportedFeature,
    CssStageCompletion, CssTokenizerResourceKind, CssTokenizerTermination, analyze_core_v1,
};
use frontend_analysis_core::ecmascript::binding_refs::analyze_selected_flat_lexical_binding_refs;
use frontend_analysis_core::{SourceAnchor, SourceId, SourceText};

/// Shared per-invocation Product stdin acquisition ceiling for every `fa`
/// command (approved in Issues #857 and #862). It is Product acquisition
/// policy, not a Core limit: it is deliberately larger than the Core CSS
/// tokenizer `SourceBytes` policy so Core source-size refusal stays observable
/// as an analysis result.
const CLI_STDIN_MAX_BYTES: usize = 49_152;

const CSS_SELECTORS_COMMAND: &str = "css-selectors";
const ES_BINDING_REFS_COMMAND: &str = "es-binding-refs";
const USAGE: &str = "usage: fa css-selectors < style.css\n       fa es-binding-refs < source.js";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    CssSelectors,
    EsBindingRefs,
}

const EXIT_REPORT: u8 = 0;
const EXIT_PRODUCT_FAILURE: u8 = 1;
const EXIT_CORE_FAILURE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let status = run(
        &args,
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    ExitCode::from(status)
}

fn run(
    args: &[OsString],
    stdin: &mut impl Read,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> u8 {
    let command = match check_command(args) {
        Ok(command) => command,
        Err(message) => {
            return product_failure(stderr, &format_args!("{message}\n{USAGE}"));
        }
    };

    let bytes = match read_bounded(stdin) {
        Ok(bytes) => bytes,
        Err(AcquisitionError::Read(kind)) => {
            return product_failure(stderr, &format_args!("failed to read stdin: {kind}"));
        }
        Err(AcquisitionError::TooLarge) => {
            return product_failure(
                stderr,
                &format_args!("stdin exceeds {CLI_STDIN_MAX_BYTES} bytes"),
            );
        }
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return product_failure(
                stderr,
                &format_args!(
                    "stdin is not valid UTF-8 (invalid sequence at byte {})",
                    error.utf8_error().valid_up_to()
                ),
            );
        }
    };

    let source = SourceText::new(SourceId::new(0), text);
    let rendered = match command {
        Command::CssSelectors => analyze_core_v1(&source)
            .map(|report| render_report(source.as_str().len(), &report))
            .map_err(|failure| failure.to_string()),
        Command::EsBindingRefs => analyze_selected_flat_lexical_binding_refs(&source)
            .map(|report| es_binding_refs::render_report(source.as_str().len(), &report))
            .map_err(|failure| failure.to_string()),
    };
    match rendered {
        Ok(rendered) => emit_report(stdout, stderr, &rendered),
        Err(failure) => core_failure(stderr, &failure),
    }
}

fn emit_report(stdout: &mut impl Write, stderr: &mut impl Write, rendered: &str) -> u8 {
    match stdout
        .write_all(rendered.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Ok(()) => EXIT_REPORT,
        Err(error) => product_failure(
            stderr,
            &format_args!("failed to write stdout: {}", error.kind()),
        ),
    }
}

fn check_command(args: &[OsString]) -> Result<Command, &'static str> {
    let command = match args.first() {
        None => return Err("missing command"),
        Some(command) if command.as_os_str() == CSS_SELECTORS_COMMAND => Command::CssSelectors,
        Some(command) if command.as_os_str() == ES_BINDING_REFS_COMMAND => Command::EsBindingRefs,
        Some(_) => return Err("unknown command"),
    };
    if args.len() > 1 {
        return Err("unexpected argument");
    }
    Ok(command)
}

#[derive(Debug, PartialEq, Eq)]
enum AcquisitionError {
    Read(io::ErrorKind),
    TooLarge,
}

/// Reads at most `CLI_STDIN_MAX_BYTES + 1` bytes; the extra byte only proves
/// that the Product acquisition ceiling was exceeded.
fn read_bounded(stdin: &mut impl Read) -> Result<Vec<u8>, AcquisitionError> {
    let mut bytes = Vec::new();
    stdin
        .take(CLI_STDIN_MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| AcquisitionError::Read(error.kind()))?;
    if bytes.len() > CLI_STDIN_MAX_BYTES {
        return Err(AcquisitionError::TooLarge);
    }
    Ok(bytes)
}

fn product_failure(stderr: &mut impl Write, message: &dyn Display) -> u8 {
    // Diagnostics are best effort; the exit status remains authoritative.
    let _ = writeln!(stderr, "fa: {message}");
    EXIT_PRODUCT_FAILURE
}

fn core_failure(stderr: &mut impl Write, failure: &dyn Display) -> u8 {
    let _ = writeln!(stderr, "fa: {failure}");
    EXIT_CORE_FAILURE
}

fn render_report(source_bytes: usize, report: &CssSelectorReport) -> String {
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

/// Renders retained source evidence as its byte range, one-based start line
/// and byte column, and the exact retained fragment escaped onto one line.
pub(crate) fn render_evidence(anchor: &SourceAnchor) -> String {
    let range = anchor.range();
    let start = anchor.start_coordinate();
    format!(
        "bytes {}..{}, line {}, byte column {}: \"{}\"",
        range.start(),
        range.end(),
        start.line_index() + 1,
        start.byte_column() + 1,
        escape_fragment(anchor.fragment())
    )
}

/// Deterministic terminal-safe escaping: printable ASCII is kept, `"` and
/// `\` are backslash-escaped, and every other scalar value becomes
/// `\u{hex}`.
fn escape_fragment(fragment: &str) -> String {
    let mut escaped = String::with_capacity(fragment.len());
    for character in fragment.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            ' '..='~' => escaped.push(character),
            _ => {
                let _ = write!(escaped, "\\u{{{:x}}}", u32::from(character));
            }
        }
    }
    escaped
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn only_the_exact_commands_are_accepted() {
        assert_eq!(
            check_command(&args(&["css-selectors"])),
            Ok(Command::CssSelectors)
        );
        assert_eq!(
            check_command(&args(&["es-binding-refs"])),
            Ok(Command::EsBindingRefs)
        );
        assert_eq!(
            check_command(&args(&["es-binding-refs", "a.js"])),
            Err("unexpected argument")
        );
        assert_eq!(check_command(&args(&[])), Err("missing command"));
        assert_eq!(check_command(&args(&["css"])), Err("unknown command"));
        assert_eq!(check_command(&args(&["--help"])), Err("unknown command"));
        assert_eq!(
            check_command(&args(&["css-selectors", "style.css"])),
            Err("unexpected argument")
        );
    }

    #[test]
    fn bounded_read_accepts_the_cap_and_rejects_one_more_byte() {
        let at_cap = vec![b'a'; CLI_STDIN_MAX_BYTES];
        assert_eq!(read_bounded(&mut at_cap.as_slice()).unwrap().len(), 49_152);

        let over_cap = vec![b'a'; CLI_STDIN_MAX_BYTES + 1];
        assert_eq!(
            read_bounded(&mut over_cap.as_slice()),
            Err(AcquisitionError::TooLarge)
        );
    }

    #[test]
    fn bounded_read_never_consumes_beyond_the_cap_plus_one() {
        let input = vec![b'a'; CLI_STDIN_MAX_BYTES + 100];
        let mut remaining = input.as_slice();
        assert_eq!(
            read_bounded(&mut remaining),
            Err(AcquisitionError::TooLarge)
        );
        assert_eq!(remaining.len(), 99);
    }

    #[test]
    fn read_failure_is_a_product_failure() {
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            }
        }
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let status = run(
            &args(&["css-selectors"]),
            &mut Failing,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(status, 1);
        assert!(stdout.is_empty());
        assert_eq!(
            String::from_utf8(stderr).unwrap(),
            "fa: failed to read stdin: permission denied\n"
        );
    }

    #[test]
    fn returned_core_failure_maps_to_exit_two_on_stderr() {
        let mut stderr = Vec::new();
        let status = core_failure(&mut stderr, &"example Core failure");

        assert_eq!(status, 2);
        assert_eq!(
            String::from_utf8(stderr).unwrap(),
            "fa: example Core failure\n"
        );
    }

    #[test]
    fn fragment_escaping_is_one_line_and_terminal_safe() {
        assert_eq!(escape_fragment("a.b > c"), "a.b > c");
        assert_eq!(escape_fragment("[x=\"y\"]"), "[x=\\\"y\\\"]");
        assert_eq!(escape_fragment(r".\66 oo"), r".\\66 oo");
        assert_eq!(escape_fragment("a\r\n\t"), "a\\u{d}\\u{a}\\u{9}");
        assert_eq!(
            escape_fragment("\u{feff}.é\u{1b}"),
            "\\u{feff}.\\u{e9}\\u{1b}"
        );
        assert_eq!(escape_fragment("\u{7f}\u{202e}"), "\\u{7f}\\u{202e}");
    }
}
