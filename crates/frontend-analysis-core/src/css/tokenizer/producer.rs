//! Default-mode CSS Syntax Module Level 3 tokenizer producer.
//!
//! Implements the bounded, lossless tokenizer selected by #132/#133/#134:
//! `&SourceText + CssTokenizerLimits -> Result<CssTokenizerRunResult,
//! CssTokenizerRunError>`. `unicode ranges allowed` is fixed to `false`;
//! `unicode-range-token` is intentionally absent.

use crate::{SourceAnchor, SourceRangeError, SourceText};

use super::super::token::{
    CssCommentEvidence, CssCommentTermination, CssDecimalExponent, CssDecimalValue,
    CssExponentSign, CssHashType, CssLexicalItem, CssNumberSign, CssNumericValue, CssToken,
    CssTokenContractError, CssTokenKind,
};
use super::diagnostic::{
    CssTokenizerDiagnostic, CssTokenizerDiagnosticCode, CssTokenizerDiagnosticContext,
    CssTokenizerDiagnosticSubject, CssTokenizerRecovery,
};
use super::input::CssInputCursor;
use super::resource::{
    CssTokenizerLimits, CssTokenizerResourceContractError, CssTokenizerResourceKind,
    CssTokenizerResourceLimitEvidence, CssTokenizerResourceUsage,
};
use super::result::{
    CssTokenizerCompletion, CssTokenizerRunError, CssTokenizerRunResult, CssTokenizerTermination,
};

/// Tokenizes `source_text` under `limits`, producing the complete default
/// ordinary-stylesheet lossless lexical evidence sequence.
pub(crate) fn run(
    source_text: &SourceText,
    limits: CssTokenizerLimits,
) -> Result<CssTokenizerRunResult, CssTokenizerRunError> {
    let mut producer = Producer::new(source_text, limits);
    match producer.execute() {
        Ok(()) => producer.finish_complete(),
        Err(Flow::Terminate(terminate)) => producer.finish_resource_limited(terminate),
        Err(Flow::Invariant(error)) => Err(error),
    }
}

/// A resource-limit signal that unwinds the current item attempt.
struct Terminate {
    kind: CssTokenizerResourceKind,
    limit: usize,
    attempted: usize,
}

/// Internal producer control flow: either a resource limit (recoverable
/// into an `Incomplete` result) or a typed internal invariant failure.
enum Flow {
    Terminate(Terminate),
    Invariant(CssTokenizerRunError),
}

impl From<SourceRangeError> for Flow {
    fn from(error: SourceRangeError) -> Self {
        Self::Invariant(error.into())
    }
}

impl From<CssTokenContractError> for Flow {
    fn from(error: CssTokenContractError) -> Self {
        Self::Invariant(error.into())
    }
}

impl From<CssTokenizerResourceContractError> for Flow {
    fn from(error: CssTokenizerResourceContractError) -> Self {
        Self::Invariant(error.into())
    }
}

struct PendingDiagnostic {
    code: CssTokenizerDiagnosticCode,
    location: SourceAnchor,
    context: CssTokenizerDiagnosticContext,
    recovery: CssTokenizerRecovery,
}

struct RecognizedItem {
    item: CssLexicalItem,
    retained_bytes: usize,
}

struct Producer<'a> {
    source_text: &'a SourceText,
    limits: CssTokenizerLimits,
    committed_pos: usize,
    leading_bom: Option<SourceAnchor>,
    lexical_items: Vec<CssLexicalItem>,
    diagnostics: Vec<CssTokenizerDiagnostic>,
    algorithm_steps: usize,
    retained_interpreted_bytes: usize,
    peak_temporary_buffer_bytes: usize,
    pending_diagnostics: Vec<PendingDiagnostic>,
    scratch_live: usize,
    scratch_peak: usize,
}

impl<'a> Producer<'a> {
    fn new(source_text: &'a SourceText, limits: CssTokenizerLimits) -> Self {
        Self {
            source_text,
            limits,
            committed_pos: 0,
            leading_bom: None,
            lexical_items: Vec::new(),
            diagnostics: Vec::new(),
            algorithm_steps: 0,
            retained_interpreted_bytes: 0,
            peak_temporary_buffer_bytes: 0,
            pending_diagnostics: Vec::new(),
            scratch_live: 0,
            scratch_peak: 0,
        }
    }

    fn execute(&mut self) -> Result<(), Flow> {
        let source = self.source_text.as_str();
        let source_len = source.len();
        let has_bom = source.starts_with('\u{feff}');
        let bom_end = if has_bom { '\u{feff}'.len_utf8() } else { 0 };

        if has_bom {
            self.leading_bom = Some(self.source_text.anchor(0, bom_end)?);
        }
        self.committed_pos = bom_end;

        let source_bytes_limit = self.limits.limit(CssTokenizerResourceKind::SourceBytes);
        if source_len > source_bytes_limit {
            return Err(Flow::Terminate(Terminate {
                kind: CssTokenizerResourceKind::SourceBytes,
                limit: source_bytes_limit,
                attempted: source_len,
            }));
        }

        let mut cursor = CssInputCursor::new(source, bom_end);

        loop {
            if cursor.peek_at(0).is_none() {
                break;
            }

            let prospective_items = self.lexical_items.len() + 1;
            let items_limit = self.limits.limit(CssTokenizerResourceKind::LexicalItems);
            if prospective_items > items_limit {
                return Err(Flow::Terminate(Terminate {
                    kind: CssTokenizerResourceKind::LexicalItems,
                    limit: items_limit,
                    attempted: prospective_items,
                }));
            }

            self.step()?;
            self.begin_item();

            let recognized = if cursor.peek_at(0) == Some('/') && cursor.peek_at(1) == Some('*') {
                self.consume_comment(&mut cursor)?
            } else {
                self.consume_token(&mut cursor)?
            };

            self.commit(recognized)?;
        }

        Ok(())
    }

    fn begin_item(&mut self) {
        self.pending_diagnostics.clear();
        self.scratch_live = 0;
        self.scratch_peak = 0;
    }

    fn step(&mut self) -> Result<(), Flow> {
        self.algorithm_steps += 1;
        let limit = self.limits.limit(CssTokenizerResourceKind::AlgorithmSteps);
        if self.algorithm_steps > limit {
            return Err(Flow::Terminate(Terminate {
                kind: CssTokenizerResourceKind::AlgorithmSteps,
                limit,
                attempted: self.algorithm_steps,
            }));
        }
        Ok(())
    }

    fn grow_scratch(&mut self, additional: usize) -> Result<(), Flow> {
        let prospective = self.scratch_live + additional;
        let limit = self
            .limits
            .limit(CssTokenizerResourceKind::TemporaryBufferBytes);
        if prospective > limit {
            return Err(Flow::Terminate(Terminate {
                kind: CssTokenizerResourceKind::TemporaryBufferBytes,
                limit,
                attempted: prospective,
            }));
        }
        self.scratch_live = prospective;
        self.scratch_peak = self.scratch_peak.max(self.scratch_live);
        Ok(())
    }

    fn shrink_scratch(&mut self, amount: usize) {
        self.scratch_live = self.scratch_live.saturating_sub(amount);
    }

    fn push_diagnostic(
        &mut self,
        code: CssTokenizerDiagnosticCode,
        start: usize,
        end: usize,
    ) -> Result<(), Flow> {
        let location = self.source_text.anchor(start, end)?;
        let (context, recovery) = diagnostic_handling(code);
        self.pending_diagnostics.push(PendingDiagnostic {
            code,
            location,
            context,
            recovery,
        });
        Ok(())
    }

    fn commit(&mut self, recognized: RecognizedItem) -> Result<(), Flow> {
        let prospective_diagnostics = self.diagnostics.len() + self.pending_diagnostics.len();
        let diagnostics_limit = self.limits.limit(CssTokenizerResourceKind::Diagnostics);
        if prospective_diagnostics > diagnostics_limit {
            return Err(Flow::Terminate(Terminate {
                kind: CssTokenizerResourceKind::Diagnostics,
                limit: diagnostics_limit,
                attempted: prospective_diagnostics,
            }));
        }

        let prospective_retained = self.retained_interpreted_bytes + recognized.retained_bytes;
        let retained_limit = self
            .limits
            .limit(CssTokenizerResourceKind::RetainedInterpretedBytes);
        if prospective_retained > retained_limit {
            return Err(Flow::Terminate(Terminate {
                kind: CssTokenizerResourceKind::RetainedInterpretedBytes,
                limit: retained_limit,
                attempted: prospective_retained,
            }));
        }

        let index = self.lexical_items.len();
        let end = recognized.item.source().range().end();
        self.lexical_items.push(recognized.item);

        for pending in std::mem::take(&mut self.pending_diagnostics) {
            let diagnostic = CssTokenizerDiagnostic::new(
                self.source_text,
                pending.code,
                pending.location,
                pending.context,
                pending.recovery,
                CssTokenizerDiagnosticSubject::LexicalItem { index },
            )
            .map_err(|error| {
                Flow::Invariant(CssTokenizerRunError::diagnostic_contract_violation(
                    index, error,
                ))
            })?;
            self.diagnostics.push(diagnostic);
        }

        self.committed_pos = end;
        self.retained_interpreted_bytes = prospective_retained;
        self.peak_temporary_buffer_bytes = self.peak_temporary_buffer_bytes.max(self.scratch_peak);
        Ok(())
    }

    fn finish_complete(self) -> Result<CssTokenizerRunResult, CssTokenizerRunError> {
        let source_len = self.source_text.as_str().len();
        let terminal = self
            .source_text
            .anchor(self.committed_pos, self.committed_pos)?;
        let processed_prefix = self.source_text.anchor(0, self.committed_pos)?;
        let unprocessed_remainder = self.source_text.anchor(self.committed_pos, source_len)?;
        let resources = CssTokenizerResourceUsage::new(
            source_len,
            self.algorithm_steps,
            self.lexical_items.len(),
            self.diagnostics.len(),
            self.retained_interpreted_bytes,
            self.peak_temporary_buffer_bytes,
        );
        CssTokenizerRunResult::new(
            self.source_text,
            self.leading_bom,
            self.lexical_items,
            self.diagnostics,
            processed_prefix,
            unprocessed_remainder,
            terminal,
            CssTokenizerCompletion::Complete,
            CssTokenizerTermination::EndOfInput,
            resources,
        )
    }

    fn finish_resource_limited(
        self,
        terminate: Terminate,
    ) -> Result<CssTokenizerRunResult, CssTokenizerRunError> {
        let source_len = self.source_text.as_str().len();
        let terminal = self
            .source_text
            .anchor(self.committed_pos, self.committed_pos)?;
        let processed_prefix = self.source_text.anchor(0, self.committed_pos)?;
        let unprocessed_remainder = self.source_text.anchor(self.committed_pos, source_len)?;
        let evidence = CssTokenizerResourceLimitEvidence::new(
            self.source_text,
            terminate.kind,
            terminate.limit,
            terminate.attempted,
            terminal.clone(),
        )?;
        let resources = CssTokenizerResourceUsage::new(
            source_len,
            self.algorithm_steps,
            self.lexical_items.len(),
            self.diagnostics.len(),
            self.retained_interpreted_bytes,
            self.peak_temporary_buffer_bytes,
        );
        CssTokenizerRunResult::new(
            self.source_text,
            self.leading_bom,
            self.lexical_items,
            self.diagnostics,
            processed_prefix,
            unprocessed_remainder,
            terminal,
            CssTokenizerCompletion::Incomplete,
            CssTokenizerTermination::ResourceLimit(evidence),
            resources,
        )
    }

    fn finish_token(
        &mut self,
        kind: CssTokenKind,
        start: usize,
        end: usize,
    ) -> Result<RecognizedItem, Flow> {
        let anchor = self.source_text.anchor(start, end)?;
        let retained_bytes = retained_bytes_for_kind(&kind);
        let token = CssToken::new(self.source_text, anchor, kind)?;
        Ok(RecognizedItem {
            item: CssLexicalItem::SemanticToken(token),
            retained_bytes,
        })
    }

    fn skip_whitespace(&mut self, cursor: &mut CssInputCursor<'_>) -> Result<(), Flow> {
        loop {
            self.step()?;
            match cursor.peek_at(0) {
                Some(character) if is_css_whitespace(character) => {
                    cursor.consume();
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn consume_comment(&mut self, cursor: &mut CssInputCursor<'_>) -> Result<RecognizedItem, Flow> {
        let open_start = cursor.raw_position();
        cursor.consume();
        cursor.consume();
        let open_end = cursor.raw_position();

        loop {
            self.step()?;
            match cursor.consume() {
                None => {
                    let complete_end = cursor.raw_position();
                    self.push_diagnostic(
                        CssTokenizerDiagnosticCode::EofInComment,
                        complete_end,
                        complete_end,
                    )?;
                    let complete = self.source_text.anchor(open_start, complete_end)?;
                    let open = self.source_text.anchor(open_start, open_end)?;
                    let evidence = CssCommentEvidence::new(
                        self.source_text,
                        complete,
                        open,
                        CssCommentTermination::EndOfInput,
                    )?;
                    return Ok(RecognizedItem {
                        item: CssLexicalItem::Comment(evidence),
                        retained_bytes: 0,
                    });
                }
                Some(unit) if unit.semantic() == '*' && cursor.peek_at(0) == Some('/') => {
                    cursor.consume();
                    let complete_end = cursor.raw_position();
                    let close = self.source_text.anchor(unit.raw_start(), complete_end)?;
                    let complete = self.source_text.anchor(open_start, complete_end)?;
                    let open = self.source_text.anchor(open_start, open_end)?;
                    let evidence = CssCommentEvidence::new(
                        self.source_text,
                        complete,
                        open,
                        CssCommentTermination::Closed {
                            close_delimiter: close,
                        },
                    )?;
                    return Ok(RecognizedItem {
                        item: CssLexicalItem::Comment(evidence),
                        retained_bytes: 0,
                    });
                }
                Some(_) => {}
            }
        }
    }

    fn consume_token(&mut self, cursor: &mut CssInputCursor<'_>) -> Result<RecognizedItem, Flow> {
        let Some(unit) = cursor.consume() else {
            unreachable!("caller guarantees a non-EOF next unit");
        };
        let start = unit.raw_start();
        let semantic = unit.semantic();

        if is_css_whitespace(semantic) {
            self.skip_whitespace(cursor)?;
            return self.finish_token(CssTokenKind::Whitespace, start, cursor.raw_position());
        }

        match semantic {
            '"' => self.consume_string_token(cursor, '"', start),
            '\'' => self.consume_string_token(cursor, '\'', start),
            '#' => self.consume_hash_or_delim(cursor, start),
            '(' => self.finish_token(CssTokenKind::LeftParenthesis, start, unit.raw_end()),
            ')' => self.finish_token(CssTokenKind::RightParenthesis, start, unit.raw_end()),
            '+' => {
                if would_start_number(Some('+'), cursor.peek_at(0), cursor.peek_at(1)) {
                    cursor.reconsume_unit(unit);
                    self.consume_numeric_token(cursor, start)
                } else {
                    self.finish_token(CssTokenKind::Delim('+'), start, unit.raw_end())
                }
            }
            ',' => self.finish_token(CssTokenKind::Comma, start, unit.raw_end()),
            '-' => {
                if would_start_number(Some('-'), cursor.peek_at(0), cursor.peek_at(1)) {
                    cursor.reconsume_unit(unit);
                    self.consume_numeric_token(cursor, start)
                } else if cursor.peek_at(0) == Some('-') && cursor.peek_at(1) == Some('>') {
                    cursor.consume();
                    cursor.consume();
                    self.finish_token(CssTokenKind::Cdc, start, cursor.raw_position())
                } else if would_start_ident_sequence(
                    Some('-'),
                    cursor.peek_at(0),
                    cursor.peek_at(1),
                ) {
                    cursor.reconsume_unit(unit);
                    self.consume_ident_like_token(cursor, start)
                } else {
                    self.finish_token(CssTokenKind::Delim('-'), start, unit.raw_end())
                }
            }
            '.' => {
                if would_start_number(Some('.'), cursor.peek_at(0), cursor.peek_at(1)) {
                    cursor.reconsume_unit(unit);
                    self.consume_numeric_token(cursor, start)
                } else {
                    self.finish_token(CssTokenKind::Delim('.'), start, unit.raw_end())
                }
            }
            ':' => self.finish_token(CssTokenKind::Colon, start, unit.raw_end()),
            ';' => self.finish_token(CssTokenKind::Semicolon, start, unit.raw_end()),
            '<' => {
                if cursor.peek_at(0) == Some('!')
                    && cursor.peek_at(1) == Some('-')
                    && cursor.peek_at(2) == Some('-')
                {
                    cursor.consume();
                    cursor.consume();
                    cursor.consume();
                    self.finish_token(CssTokenKind::Cdo, start, cursor.raw_position())
                } else {
                    self.finish_token(CssTokenKind::Delim('<'), start, unit.raw_end())
                }
            }
            '@' => {
                if would_start_ident_sequence(
                    cursor.peek_at(0),
                    cursor.peek_at(1),
                    cursor.peek_at(2),
                ) {
                    let name = self.consume_ident_sequence(cursor)?;
                    let end = cursor.raw_position();
                    self.finish_token(CssTokenKind::AtKeyword(name), start, end)
                } else {
                    self.finish_token(CssTokenKind::Delim('@'), start, unit.raw_end())
                }
            }
            '[' => self.finish_token(CssTokenKind::LeftSquareBracket, start, unit.raw_end()),
            '\\' => {
                if cursor.peek_at(0) != Some('\n') {
                    cursor.reconsume_unit(unit);
                    self.consume_ident_like_token(cursor, start)
                } else {
                    self.push_diagnostic(
                        CssTokenizerDiagnosticCode::InvalidEscape,
                        start,
                        unit.raw_end(),
                    )?;
                    self.finish_token(CssTokenKind::Delim('\\'), start, unit.raw_end())
                }
            }
            ']' => self.finish_token(CssTokenKind::RightSquareBracket, start, unit.raw_end()),
            '{' => self.finish_token(CssTokenKind::LeftCurlyBracket, start, unit.raw_end()),
            '}' => self.finish_token(CssTokenKind::RightCurlyBracket, start, unit.raw_end()),
            character if character.is_ascii_digit() => {
                cursor.reconsume_unit(unit);
                self.consume_numeric_token(cursor, start)
            }
            character if is_ident_start(character) => {
                cursor.reconsume_unit(unit);
                self.consume_ident_like_token(cursor, start)
            }
            character => self.finish_token(CssTokenKind::Delim(character), start, unit.raw_end()),
        }
    }

    fn consume_hash_or_delim(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
        start: usize,
    ) -> Result<RecognizedItem, Flow> {
        let is_ident_next =
            matches!(cursor.peek_at(0), Some(character) if is_ident_code_point(character));
        let is_escape_next = is_valid_escape(cursor.peek_at(0), cursor.peek_at(1));
        if !is_ident_next && !is_escape_next {
            return self.finish_token(CssTokenKind::Delim('#'), start, start + '#'.len_utf8());
        }

        let hash_type = if would_start_ident_sequence(
            cursor.peek_at(0),
            cursor.peek_at(1),
            cursor.peek_at(2),
        ) {
            CssHashType::Id
        } else {
            CssHashType::Unrestricted
        };
        let value = self.consume_ident_sequence(cursor)?;
        let end = cursor.raw_position();
        self.finish_token(CssTokenKind::Hash { value, hash_type }, start, end)
    }

    fn consume_ident_sequence(&mut self, cursor: &mut CssInputCursor<'_>) -> Result<String, Flow> {
        let mut value = String::new();
        loop {
            self.step()?;
            match cursor.peek_at(0) {
                Some(character) if is_ident_code_point(character) => {
                    cursor.consume();
                    self.grow_scratch(character.len_utf8())?;
                    value.push(character);
                }
                Some('\\') if is_valid_escape(cursor.peek_at(0), cursor.peek_at(1)) => {
                    cursor.consume();
                    let decoded = self.consume_escaped_code_point(cursor)?;
                    self.grow_scratch(decoded.len_utf8())?;
                    value.push(decoded);
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn consume_escaped_code_point(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
    ) -> Result<char, Flow> {
        self.step()?;
        match cursor.consume() {
            None => {
                let position = cursor.raw_position();
                self.push_diagnostic(CssTokenizerDiagnosticCode::EofInEscape, position, position)?;
                Ok('\u{FFFD}')
            }
            Some(unit) if unit.semantic().is_ascii_hexdigit() => {
                let mut hex = String::new();
                self.grow_scratch(1)?;
                hex.push(unit.semantic());

                for _ in 0..5 {
                    self.step()?;
                    match cursor.peek_at(0) {
                        Some(character) if character.is_ascii_hexdigit() => {
                            cursor.consume();
                            self.grow_scratch(1)?;
                            hex.push(character);
                        }
                        _ => break,
                    }
                }

                self.step()?;
                if matches!(cursor.peek_at(0), Some('\n' | '\t' | ' ')) {
                    cursor.consume();
                }

                let code_point = hex.chars().fold(0u32, |accumulator, character| {
                    accumulator * 16 + character.to_digit(16).unwrap_or(0)
                });
                self.shrink_scratch(hex.len());

                let decoded = if code_point == 0
                    || (0xD800..=0xDFFF).contains(&code_point)
                    || code_point > 0x10FFFF
                {
                    '\u{FFFD}'
                } else {
                    char::from_u32(code_point).unwrap_or('\u{FFFD}')
                };
                Ok(decoded)
            }
            Some(unit) => Ok(unit.semantic()),
        }
    }

    fn consume_string_token(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
        quote: char,
        start: usize,
    ) -> Result<RecognizedItem, Flow> {
        let mut value = String::new();
        loop {
            self.step()?;
            match cursor.consume() {
                None => {
                    let position = cursor.raw_position();
                    self.push_diagnostic(
                        CssTokenizerDiagnosticCode::EofInString,
                        position,
                        position,
                    )?;
                    return self.finish_token(CssTokenKind::String(value), start, position);
                }
                Some(unit) if unit.semantic() == quote => {
                    return self.finish_token(CssTokenKind::String(value), start, unit.raw_end());
                }
                Some(unit) if unit.semantic() == '\n' => {
                    self.push_diagnostic(
                        CssTokenizerDiagnosticCode::NewlineInString,
                        unit.raw_start(),
                        unit.raw_start(),
                    )?;
                    cursor.reconsume_unit(unit);
                    return self.finish_token(CssTokenKind::BadString, start, unit.raw_start());
                }
                Some(unit) if unit.semantic() == '\\' => match cursor.peek_at(0) {
                    None => {}
                    Some('\n') => {
                        cursor.consume();
                    }
                    Some(_) => {
                        let decoded = self.consume_escaped_code_point(cursor)?;
                        self.grow_scratch(decoded.len_utf8())?;
                        value.push(decoded);
                    }
                },
                Some(unit) => {
                    self.grow_scratch(unit.semantic().len_utf8())?;
                    value.push(unit.semantic());
                }
            }
        }
    }

    fn consume_numeric_token(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
        start: usize,
    ) -> Result<RecognizedItem, Flow> {
        let (sign, integer_digits, fraction_digits, exponent) = self.consume_number(cursor)?;
        let decimal = CssDecimalValue::new(integer_digits, fraction_digits, exponent)?;
        let numeric_value = CssNumericValue::new(decimal, sign);
        let number_type = numeric_value.required_number_type();

        if would_start_ident_sequence(cursor.peek_at(0), cursor.peek_at(1), cursor.peek_at(2)) {
            let unit_value = self.consume_ident_sequence(cursor)?;
            let end = cursor.raw_position();
            self.finish_token(
                CssTokenKind::Dimension {
                    value: numeric_value,
                    number_type,
                    unit: unit_value,
                },
                start,
                end,
            )
        } else if cursor.peek_at(0) == Some('%') {
            cursor.consume();
            let end = cursor.raw_position();
            self.finish_token(
                CssTokenKind::Percentage {
                    value: numeric_value,
                },
                start,
                end,
            )
        } else {
            let end = cursor.raw_position();
            self.finish_token(
                CssTokenKind::Number {
                    value: numeric_value,
                    number_type,
                },
                start,
                end,
            )
        }
    }

    #[allow(clippy::type_complexity)]
    fn consume_number(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
    ) -> Result<
        (
            Option<CssNumberSign>,
            String,
            String,
            Option<CssDecimalExponent>,
        ),
        Flow,
    > {
        let sign = match cursor.peek_at(0) {
            Some('+') => {
                cursor.consume();
                Some(CssNumberSign::Plus)
            }
            Some('-') => {
                cursor.consume();
                Some(CssNumberSign::Minus)
            }
            _ => None,
        };

        let mut integer_digits = String::new();
        loop {
            self.step()?;
            match cursor.peek_at(0) {
                Some(character) if character.is_ascii_digit() => {
                    cursor.consume();
                    self.grow_scratch(1)?;
                    integer_digits.push(character);
                }
                _ => break,
            }
        }

        let mut fraction_digits = String::new();
        if cursor.peek_at(0) == Some('.')
            && matches!(cursor.peek_at(1), Some(character) if character.is_ascii_digit())
        {
            cursor.consume();
            loop {
                self.step()?;
                match cursor.peek_at(0) {
                    Some(character) if character.is_ascii_digit() => {
                        cursor.consume();
                        self.grow_scratch(1)?;
                        fraction_digits.push(character);
                    }
                    _ => break,
                }
            }
        }

        let mut exponent = None;
        if matches!(cursor.peek_at(0), Some('e' | 'E')) {
            let (exponent_sign, digit_probe_index) = match cursor.peek_at(1) {
                Some('+') => (Some(CssExponentSign::Plus), 2),
                Some('-') => (Some(CssExponentSign::Minus), 2),
                _ => (None, 1),
            };
            if matches!(cursor.peek_at(digit_probe_index), Some(character) if character.is_ascii_digit())
            {
                cursor.consume();
                if exponent_sign.is_some() {
                    cursor.consume();
                }
                let mut exponent_digits = String::new();
                loop {
                    self.step()?;
                    match cursor.peek_at(0) {
                        Some(character) if character.is_ascii_digit() => {
                            cursor.consume();
                            self.grow_scratch(1)?;
                            exponent_digits.push(character);
                        }
                        _ => break,
                    }
                }
                exponent = Some(CssDecimalExponent::new(exponent_sign, exponent_digits)?);
            }
        }

        if integer_digits.is_empty() {
            self.grow_scratch(1)?;
            integer_digits.push('0');
        }

        Ok((sign, integer_digits, fraction_digits, exponent))
    }

    fn consume_ident_like_token(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
        start: usize,
    ) -> Result<RecognizedItem, Flow> {
        let name = self.consume_ident_sequence(cursor)?;

        if name.eq_ignore_ascii_case("url") && cursor.peek_at(0) == Some('(') {
            cursor.consume();
            let paren_end = cursor.raw_position();

            // CSS Syntax §4.3.4 can consume part of a whitespace run while
            // disambiguating a quoted `url(` from an unquoted one. Those
            // code points are only ever peeked here (never committed via
            // `cursor.consume`), so recognition of the following item
            // naturally resumes at `paren_end` and reclaims the complete
            // authored whitespace run for the next semantic token. See the
            // #136 "quoted url( disambiguation whitespace provenance"
            // qualification.
            let mut probe = *cursor;
            loop {
                self.step()?;
                match probe.peek_at(0) {
                    Some(character) if is_css_whitespace(character) => {
                        probe.consume();
                    }
                    _ => break,
                }
            }

            if matches!(probe.peek_at(0), Some('"') | Some('\'')) {
                return self.finish_token(CssTokenKind::Function(name), start, paren_end);
            }

            return self.consume_url_token(cursor, start);
        }

        if cursor.peek_at(0) == Some('(') {
            cursor.consume();
            let end = cursor.raw_position();
            return self.finish_token(CssTokenKind::Function(name), start, end);
        }

        let end = cursor.raw_position();
        self.finish_token(CssTokenKind::Ident(name), start, end)
    }

    fn consume_url_token(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
        start: usize,
    ) -> Result<RecognizedItem, Flow> {
        self.skip_whitespace(cursor)?;

        let mut value = String::new();
        loop {
            self.step()?;
            match cursor.consume() {
                None => {
                    let position = cursor.raw_position();
                    self.push_diagnostic(CssTokenizerDiagnosticCode::EofInUrl, position, position)?;
                    return self.finish_token(CssTokenKind::Url(value), start, position);
                }
                Some(unit) if unit.semantic() == ')' => {
                    return self.finish_token(CssTokenKind::Url(value), start, unit.raw_end());
                }
                Some(unit) if is_css_whitespace(unit.semantic()) => {
                    self.skip_whitespace(cursor)?;
                    match cursor.consume() {
                        None => {
                            let position = cursor.raw_position();
                            self.push_diagnostic(
                                CssTokenizerDiagnosticCode::EofInUrl,
                                position,
                                position,
                            )?;
                            return self.finish_token(CssTokenKind::Url(value), start, position);
                        }
                        Some(closing) if closing.semantic() == ')' => {
                            return self.finish_token(
                                CssTokenKind::Url(value),
                                start,
                                closing.raw_end(),
                            );
                        }
                        Some(other) => {
                            cursor.reconsume_unit(other);
                            return self.consume_bad_url_remnants(cursor, start);
                        }
                    }
                }
                Some(unit)
                    if matches!(unit.semantic(), '"' | '\'' | '(')
                        || is_non_printable(unit.semantic()) =>
                {
                    self.push_diagnostic(
                        CssTokenizerDiagnosticCode::InvalidUrlCodePoint,
                        unit.raw_start(),
                        unit.raw_end(),
                    )?;
                    return self.consume_bad_url_remnants(cursor, start);
                }
                Some(unit) if unit.semantic() == '\\' => {
                    if is_valid_escape(Some('\\'), cursor.peek_at(0)) {
                        let decoded = self.consume_escaped_code_point(cursor)?;
                        self.grow_scratch(decoded.len_utf8())?;
                        value.push(decoded);
                    } else {
                        self.push_diagnostic(
                            CssTokenizerDiagnosticCode::InvalidUrlEscape,
                            unit.raw_start(),
                            unit.raw_end(),
                        )?;
                        return self.consume_bad_url_remnants(cursor, start);
                    }
                }
                Some(unit) => {
                    self.grow_scratch(unit.semantic().len_utf8())?;
                    value.push(unit.semantic());
                }
            }
        }
    }

    fn consume_bad_url_remnants(
        &mut self,
        cursor: &mut CssInputCursor<'_>,
        start: usize,
    ) -> Result<RecognizedItem, Flow> {
        loop {
            self.step()?;
            match cursor.consume() {
                None => break,
                Some(unit) if unit.semantic() == ')' => break,
                Some(unit) if unit.semantic() == '\\' => {
                    if is_valid_escape(Some('\\'), cursor.peek_at(0)) {
                        let _ = self.consume_escaped_code_point(cursor)?;
                    }
                }
                Some(_) => {}
            }
        }
        let end = cursor.raw_position();
        self.finish_token(CssTokenKind::BadUrl, start, end)
    }
}

fn retained_bytes_for_kind(kind: &CssTokenKind) -> usize {
    match kind {
        CssTokenKind::Ident(value)
        | CssTokenKind::Function(value)
        | CssTokenKind::AtKeyword(value)
        | CssTokenKind::String(value)
        | CssTokenKind::Url(value) => value.len(),
        CssTokenKind::Hash { value, .. } => value.len(),
        CssTokenKind::Number { value, .. } => numeric_retained_bytes(value),
        CssTokenKind::Percentage { value } => numeric_retained_bytes(value),
        CssTokenKind::Dimension { value, unit, .. } => numeric_retained_bytes(value) + unit.len(),
        CssTokenKind::BadString
        | CssTokenKind::BadUrl
        | CssTokenKind::Delim(_)
        | CssTokenKind::Whitespace
        | CssTokenKind::Cdo
        | CssTokenKind::Cdc
        | CssTokenKind::Colon
        | CssTokenKind::Semicolon
        | CssTokenKind::Comma
        | CssTokenKind::LeftSquareBracket
        | CssTokenKind::RightSquareBracket
        | CssTokenKind::LeftParenthesis
        | CssTokenKind::RightParenthesis
        | CssTokenKind::LeftCurlyBracket
        | CssTokenKind::RightCurlyBracket => 0,
    }
}

fn numeric_retained_bytes(value: &CssNumericValue) -> usize {
    let decimal = value.decimal();
    decimal.integer_digits().len()
        + decimal.fraction_digits().len()
        + decimal
            .exponent()
            .map_or(0, |exponent| exponent.digits().len())
}

fn diagnostic_handling(
    code: CssTokenizerDiagnosticCode,
) -> (CssTokenizerDiagnosticContext, CssTokenizerRecovery) {
    match code {
        CssTokenizerDiagnosticCode::InvalidEscape => (
            CssTokenizerDiagnosticContext::TokenStart,
            CssTokenizerRecovery::EmitDelimiter,
        ),
        CssTokenizerDiagnosticCode::EofInComment => (
            CssTokenizerDiagnosticContext::Comment,
            CssTokenizerRecovery::PreserveUnterminatedComment,
        ),
        CssTokenizerDiagnosticCode::EofInString => (
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitStringAtEndOfInput,
        ),
        CssTokenizerDiagnosticCode::NewlineInString => (
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitBadStringAndReconsumeNewline,
        ),
        CssTokenizerDiagnosticCode::EofInUrl => (
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitUrlAtEndOfInput,
        ),
        CssTokenizerDiagnosticCode::InvalidUrlCodePoint
        | CssTokenizerDiagnosticCode::InvalidUrlEscape => (
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitBadUrl,
        ),
        CssTokenizerDiagnosticCode::EofInEscape => (
            CssTokenizerDiagnosticContext::Escape,
            CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        ),
    }
}

fn is_css_whitespace(character: char) -> bool {
    matches!(character, '\n' | '\t' | ' ')
}

fn is_non_printable(character: char) -> bool {
    matches!(character, '\u{0}'..='\u{8}' | '\u{B}' | '\u{E}'..='\u{1F}' | '\u{7F}')
}

fn is_non_ascii_ident_code_point(character: char) -> bool {
    matches!(character,
        '\u{B7}'
        | '\u{C0}'..='\u{D6}'
        | '\u{D8}'..='\u{F6}'
        | '\u{F8}'..='\u{37D}'
        | '\u{37F}'..='\u{1FFF}'
        | '\u{200C}'
        | '\u{200D}'
        | '\u{203F}'
        | '\u{2040}'
        | '\u{2070}'..='\u{218F}'
        | '\u{2C00}'..='\u{2FEF}'
        | '\u{3001}'..='\u{D7FF}'
        | '\u{F900}'..='\u{FDCF}'
        | '\u{FDF0}'..='\u{FFFD}'
    ) || character as u32 >= 0x1_0000
}

fn is_ident_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_' || is_non_ascii_ident_code_point(character)
}

fn is_ident_code_point(character: char) -> bool {
    is_ident_start(character) || character.is_ascii_digit() || character == '-'
}

fn is_valid_escape(first: Option<char>, second: Option<char>) -> bool {
    first == Some('\\') && second != Some('\n')
}

fn would_start_ident_sequence(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> bool {
    match first {
        Some('-') => {
            matches!(second, Some(character) if is_ident_start(character))
                || second == Some('-')
                || is_valid_escape(second, third)
        }
        Some('\\') => is_valid_escape(first, second),
        Some(character) => is_ident_start(character),
        None => false,
    }
}

fn would_start_number(first: Option<char>, second: Option<char>, third: Option<char>) -> bool {
    match first {
        Some('+' | '-') => {
            matches!(second, Some(character) if character.is_ascii_digit())
                || (second == Some('.')
                    && matches!(third, Some(character) if character.is_ascii_digit()))
        }
        Some('.') => matches!(second, Some(character) if character.is_ascii_digit()),
        Some(character) => character.is_ascii_digit(),
        None => false,
    }
}
