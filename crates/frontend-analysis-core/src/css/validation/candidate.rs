//! Test-only production candidate observation adapter.
//!
//! Projects the production `CssTokenizerRunResult` into a shape comparable
//! against the independent #135 gold model. Gold is never generated from or
//! rewritten by production output; this module only reads production
//! results to compare them against fixtures authored in `fixtures.rs`.

use crate::css::token::{
    CssCommentTermination, CssExponentSign, CssHashType, CssLexicalItem, CssNumberSign,
    CssNumberType, CssNumericValue, CssTokenKind,
};
use crate::css::tokenizer::diagnostic::{
    CssTokenizerDiagnostic, CssTokenizerDiagnosticCode, CssTokenizerDiagnosticContext,
    CssTokenizerDiagnosticSubject, CssTokenizerRecovery,
};
use crate::css::tokenizer::input::CssInputCursor;
use crate::css::tokenizer::producer::run;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use crate::{SourceId, SourceText};

use super::gold::{
    GoldCommentTermination, GoldCompletion, GoldDiagnostic, GoldDiagnosticCode,
    GoldDiagnosticContext, GoldDiagnosticSubject, GoldFixture, GoldHashType, GoldLexicalItem,
    GoldNumber, GoldNumberType, GoldRecovery, GoldSign, GoldTermination, GoldTokenKind,
};

pub(super) fn run_fixture(
    fixture: &GoldFixture,
    limits: CssTokenizerLimits,
) -> CssTokenizerRunResult {
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    run(&source, limits)
        .unwrap_or_else(|error| panic!("{}: production tokenizer error: {error:?}", fixture.id))
}

pub(super) fn assert_matches_gold(fixture: &GoldFixture, result: &CssTokenizerRunResult) {
    let id = fixture.id;
    assert_eq!(
        result.source_id(),
        SourceId::new(fixture.source_id),
        "{id}: source id"
    );

    match fixture.leading_bom {
        Some(range) => {
            let bom = result
                .leading_bom()
                .unwrap_or_else(|| panic!("{id}: missing leading bom evidence"));
            assert_eq!(
                (bom.range().start(), bom.range().end()),
                (range.start, range.end),
                "{id}: leading bom range"
            );
        }
        None => assert!(
            result.leading_bom().is_none(),
            "{id}: unexpected leading bom"
        ),
    }

    assert_eq!(
        result.lexical_items().len(),
        fixture.items.len(),
        "{id}: lexical item count"
    );
    for (index, (actual, expected)) in result
        .lexical_items()
        .iter()
        .zip(&fixture.items)
        .enumerate()
    {
        assert_item_matches(id, index, actual, expected);
    }

    assert_eq!(
        result.diagnostics().len(),
        fixture.diagnostics.len(),
        "{id}: diagnostic count"
    );
    for (index, (actual, expected)) in result
        .diagnostics()
        .iter()
        .zip(&fixture.diagnostics)
        .enumerate()
    {
        assert_diagnostic_matches(id, index, actual, expected);
    }

    match fixture.completion {
        GoldCompletion::Complete => assert_eq!(
            result.completion(),
            CssTokenizerCompletion::Complete,
            "{id}: completion"
        ),
        GoldCompletion::Incomplete => assert_eq!(
            result.completion(),
            CssTokenizerCompletion::Incomplete,
            "{id}: completion"
        ),
    }

    match fixture.termination {
        GoldTermination::EndOfInput => {
            assert!(
                matches!(result.termination(), CssTokenizerTermination::EndOfInput),
                "{id}: termination"
            );
        }
        GoldTermination::ResourceLimit { .. } => {
            panic!("{id}: candidate wiring only compares EndOfInput normative fixtures")
        }
    }

    assert_eq!(
        (
            result.terminal().range().start(),
            result.terminal().range().end()
        ),
        (fixture.terminal.start, fixture.terminal.end),
        "{id}: terminal"
    );
}

fn assert_item_matches(
    id: &str,
    index: usize,
    actual: &CssLexicalItem,
    expected: &GoldLexicalItem,
) {
    let range = expected.range();
    assert_eq!(
        (
            actual.source().range().start(),
            actual.source().range().end()
        ),
        (range.start, range.end),
        "{id}: item {index} range"
    );

    match (actual, expected) {
        (CssLexicalItem::SemanticToken(token), GoldLexicalItem::Token { kind, .. }) => {
            assert_token_kind_matches(id, index, token.kind(), kind);
        }
        (
            CssLexicalItem::Comment(comment),
            GoldLexicalItem::Comment {
                open, termination, ..
            },
        ) => {
            assert_eq!(
                (
                    comment.open_delimiter().range().start(),
                    comment.open_delimiter().range().end()
                ),
                (open.start, open.end),
                "{id}: item {index} comment open delimiter"
            );
            match (comment.termination(), termination) {
                (
                    CssCommentTermination::Closed { close_delimiter },
                    GoldCommentTermination::Closed { close },
                ) => {
                    assert_eq!(
                        (
                            close_delimiter.range().start(),
                            close_delimiter.range().end()
                        ),
                        (close.start, close.end),
                        "{id}: item {index} comment close delimiter"
                    );
                }
                (CssCommentTermination::EndOfInput, GoldCommentTermination::EndOfInput) => {}
                _ => panic!("{id}: item {index} comment termination mismatch"),
            }
        }
        _ => panic!("{id}: item {index} lexical item class mismatch"),
    }
}

fn assert_token_kind_matches(
    id: &str,
    index: usize,
    actual: &CssTokenKind,
    expected: &GoldTokenKind,
) {
    match (actual, expected) {
        (CssTokenKind::Ident(value), GoldTokenKind::Ident(expected_value)) => {
            assert_eq!(value, expected_value, "{id}: item {index} ident value");
        }
        (CssTokenKind::Function(value), GoldTokenKind::Function(expected_value)) => {
            assert_eq!(value, expected_value, "{id}: item {index} function value");
        }
        (CssTokenKind::AtKeyword(value), GoldTokenKind::AtKeyword(expected_value)) => {
            assert_eq!(value, expected_value, "{id}: item {index} at-keyword value");
        }
        (
            CssTokenKind::Hash { value, hash_type },
            GoldTokenKind::Hash {
                value: expected_value,
                hash_type: expected_type,
            },
        ) => {
            assert_eq!(value, expected_value, "{id}: item {index} hash value");
            let matches_type = matches!(
                (hash_type, expected_type),
                (CssHashType::Id, GoldHashType::Id)
                    | (CssHashType::Unrestricted, GoldHashType::Unrestricted)
            );
            assert!(matches_type, "{id}: item {index} hash type");
        }
        (CssTokenKind::String(value), GoldTokenKind::String(expected_value)) => {
            assert_eq!(value, expected_value, "{id}: item {index} string value");
        }
        (CssTokenKind::BadString, GoldTokenKind::BadString) => {}
        (CssTokenKind::Url(value), GoldTokenKind::Url(expected_value)) => {
            assert_eq!(value, expected_value, "{id}: item {index} url value");
        }
        (CssTokenKind::BadUrl, GoldTokenKind::BadUrl) => {}
        (CssTokenKind::Delim(value), GoldTokenKind::Delim(expected_value)) => {
            assert_eq!(value, expected_value, "{id}: item {index} delim value");
        }
        (CssTokenKind::Number { value, number_type }, GoldTokenKind::Number(expected_number)) => {
            assert_numeric_matches(id, index, value, *number_type, expected_number);
        }
        (CssTokenKind::Percentage { value }, GoldTokenKind::Percentage(expected_number)) => {
            assert_numeric_matches(
                id,
                index,
                value,
                value.required_number_type(),
                expected_number,
            );
        }
        (
            CssTokenKind::Dimension {
                value,
                number_type,
                unit,
            },
            GoldTokenKind::Dimension {
                number,
                unit: expected_unit,
            },
        ) => {
            assert_numeric_matches(id, index, value, *number_type, number);
            assert_eq!(unit, expected_unit, "{id}: item {index} dimension unit");
        }
        (CssTokenKind::Whitespace, GoldTokenKind::Whitespace) => {}
        (CssTokenKind::Cdo, GoldTokenKind::Cdo) => {}
        (CssTokenKind::Cdc, GoldTokenKind::Cdc) => {}
        (CssTokenKind::Colon, GoldTokenKind::Colon) => {}
        (CssTokenKind::Semicolon, GoldTokenKind::Semicolon) => {}
        (CssTokenKind::Comma, GoldTokenKind::Comma) => {}
        (CssTokenKind::LeftSquareBracket, GoldTokenKind::LeftSquareBracket) => {}
        (CssTokenKind::RightSquareBracket, GoldTokenKind::RightSquareBracket) => {}
        (CssTokenKind::LeftParenthesis, GoldTokenKind::LeftParenthesis) => {}
        (CssTokenKind::RightParenthesis, GoldTokenKind::RightParenthesis) => {}
        (CssTokenKind::LeftCurlyBracket, GoldTokenKind::LeftCurlyBracket) => {}
        (CssTokenKind::RightCurlyBracket, GoldTokenKind::RightCurlyBracket) => {}
        _ => panic!("{id}: item {index} token kind mismatch: {actual:?} vs {expected:?}"),
    }
}

fn assert_numeric_matches(
    id: &str,
    index: usize,
    value: &CssNumericValue,
    number_type: CssNumberType,
    expected: &GoldNumber,
) {
    let expected_type = match expected.number_type {
        GoldNumberType::Integer => CssNumberType::Integer,
        GoldNumberType::Number => CssNumberType::Number,
    };
    assert_eq!(number_type, expected_type, "{id}: item {index} number type");

    let sign_matches = matches!(
        (value.sign(), expected.sign),
        (None, None)
            | (Some(CssNumberSign::Plus), Some(GoldSign::Plus))
            | (Some(CssNumberSign::Minus), Some(GoldSign::Minus))
    );
    assert!(sign_matches, "{id}: item {index} number sign");

    let decimal = value.decimal();
    assert_eq!(
        decimal.integer_digits(),
        expected.integer_digits,
        "{id}: item {index} integer digits"
    );
    assert_eq!(
        decimal.fraction_digits(),
        expected.fraction_digits,
        "{id}: item {index} fraction digits"
    );

    match (decimal.exponent(), expected.exponent_digits) {
        (None, None) => {}
        (Some(exponent), Some(expected_digits)) => {
            assert_eq!(
                exponent.digits(),
                expected_digits,
                "{id}: item {index} exponent digits"
            );
            let exponent_sign_matches = matches!(
                (exponent.sign(), expected.exponent_sign),
                (None, None)
                    | (Some(CssExponentSign::Plus), Some(GoldSign::Plus))
                    | (Some(CssExponentSign::Minus), Some(GoldSign::Minus))
            );
            assert!(exponent_sign_matches, "{id}: item {index} exponent sign");
        }
        _ => panic!("{id}: item {index} exponent presence mismatch"),
    }
}

fn assert_diagnostic_matches(
    id: &str,
    index: usize,
    actual: &CssTokenizerDiagnostic,
    expected: &GoldDiagnostic,
) {
    assert_eq!(
        actual.code(),
        map_diagnostic_code(expected.code),
        "{id}: diagnostic {index} code"
    );
    assert_eq!(
        (
            actual.location().range().start(),
            actual.location().range().end()
        ),
        (expected.location.start, expected.location.end),
        "{id}: diagnostic {index} location"
    );
    assert_eq!(
        actual.context(),
        map_diagnostic_context(expected.context),
        "{id}: diagnostic {index} context"
    );
    assert_eq!(
        actual.recovery(),
        map_recovery(expected.recovery),
        "{id}: diagnostic {index} recovery"
    );

    match expected.subject {
        GoldDiagnosticSubject::InputLocation => assert!(
            matches!(
                actual.subject(),
                CssTokenizerDiagnosticSubject::InputLocation
            ),
            "{id}: diagnostic {index} subject"
        ),
        GoldDiagnosticSubject::LexicalItem(expected_index) => assert!(
            matches!(
                actual.subject(),
                CssTokenizerDiagnosticSubject::LexicalItem { index } if *index == expected_index
            ),
            "{id}: diagnostic {index} subject"
        ),
        GoldDiagnosticSubject::InputRegion(range) => match actual.subject() {
            CssTokenizerDiagnosticSubject::InputRegion { region } => assert_eq!(
                (region.range().start(), region.range().end()),
                (range.start, range.end),
                "{id}: diagnostic {index} subject region"
            ),
            _ => panic!("{id}: diagnostic {index} expected input-region subject"),
        },
    }
}

fn map_diagnostic_code(code: GoldDiagnosticCode) -> CssTokenizerDiagnosticCode {
    match code {
        GoldDiagnosticCode::InvalidEscape => CssTokenizerDiagnosticCode::InvalidEscape,
        GoldDiagnosticCode::EofInComment => CssTokenizerDiagnosticCode::EofInComment,
        GoldDiagnosticCode::EofInString => CssTokenizerDiagnosticCode::EofInString,
        GoldDiagnosticCode::NewlineInString => CssTokenizerDiagnosticCode::NewlineInString,
        GoldDiagnosticCode::EofInUrl => CssTokenizerDiagnosticCode::EofInUrl,
        GoldDiagnosticCode::InvalidUrlCodePoint => CssTokenizerDiagnosticCode::InvalidUrlCodePoint,
        GoldDiagnosticCode::InvalidUrlEscape => CssTokenizerDiagnosticCode::InvalidUrlEscape,
        GoldDiagnosticCode::EofInEscape => CssTokenizerDiagnosticCode::EofInEscape,
    }
}

fn map_diagnostic_context(context: GoldDiagnosticContext) -> CssTokenizerDiagnosticContext {
    match context {
        GoldDiagnosticContext::TokenStart => CssTokenizerDiagnosticContext::TokenStart,
        GoldDiagnosticContext::Comment => CssTokenizerDiagnosticContext::Comment,
        GoldDiagnosticContext::String => CssTokenizerDiagnosticContext::String,
        GoldDiagnosticContext::Url => CssTokenizerDiagnosticContext::Url,
        GoldDiagnosticContext::Escape => CssTokenizerDiagnosticContext::Escape,
    }
}

fn map_recovery(recovery: GoldRecovery) -> CssTokenizerRecovery {
    match recovery {
        GoldRecovery::EmitDelimiter => CssTokenizerRecovery::EmitDelimiter,
        GoldRecovery::PreserveUnterminatedComment => {
            CssTokenizerRecovery::PreserveUnterminatedComment
        }
        GoldRecovery::EmitStringAtEndOfInput => CssTokenizerRecovery::EmitStringAtEndOfInput,
        GoldRecovery::EmitBadStringAndReconsumeNewline => {
            CssTokenizerRecovery::EmitBadStringAndReconsumeNewline
        }
        GoldRecovery::EmitUrlAtEndOfInput => CssTokenizerRecovery::EmitUrlAtEndOfInput,
        GoldRecovery::EmitBadUrl => CssTokenizerRecovery::EmitBadUrl,
        GoldRecovery::SubstituteReplacementCharacterForEofEscape => {
            CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape
        }
    }
}

/// Directly observes the production lazy preprocessing cursor's semantic
/// unit + raw-range behavior, independent of full tokenization, for
/// `InputPreprocessing` gold fixtures. Preprocessing units are never
/// exposed through production `CssTokenizerRunResult`.
pub(super) fn collect_preprocessed_units(source: &str, start: usize) -> Vec<(char, usize, usize)> {
    let mut cursor = CssInputCursor::new(source, start);
    let mut units = Vec::new();
    while let Some(unit) = cursor.consume() {
        units.push((unit.semantic(), unit.raw_start(), unit.raw_end()));
    }
    units
}

/// A canonical, order-preserving textual signature of a run result, used
/// only to compare two independent production runs for determinism.
pub(super) fn canonical_signature(result: &CssTokenizerRunResult) -> String {
    use std::fmt::Write;

    let mut signature = String::new();
    for item in result.lexical_items() {
        let range = item.source().range();
        let _ = write!(signature, "[{},{})", range.start(), range.end());
        match item {
            CssLexicalItem::SemanticToken(token) => {
                let _ = write!(signature, "{:?}", token.kind());
            }
            CssLexicalItem::Comment(comment) => {
                let _ = write!(signature, "Comment{:?}", comment.termination());
            }
        }
    }
    for diagnostic in result.diagnostics() {
        let location = diagnostic.location().range();
        let _ = write!(
            signature,
            "|{:?}@[{},{})",
            diagnostic.code(),
            location.start(),
            location.end()
        );
    }
    let terminal = result.terminal().range();
    let _ = write!(
        signature,
        "|{:?}|{:?}|term[{},{})",
        result.completion(),
        result.termination(),
        terminal.start(),
        terminal.end()
    );
    signature
}
