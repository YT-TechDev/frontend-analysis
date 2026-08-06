use std::fmt;

use super::expected::{Attribute, CanonicalRun, ExpectedRun, ObservedRun, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ObservationMismatch {
    fixture_id: &'static str,
    path: String,
}

impl ObservationMismatch {
    fn new(fixture_id: &'static str, path: impl Into<String>) -> Self {
        Self {
            fixture_id,
            path: path.into(),
        }
    }

    pub(super) fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for ObservationMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML tokenizer observation mismatch: {}.{}",
            self.fixture_id, self.path
        )
    }
}

impl std::error::Error for ObservationMismatch {}

pub(super) fn compare(
    fixture_id: &'static str,
    expected: &ExpectedRun,
    observed: &ObservedRun,
) -> Result<(), ObservationMismatch> {
    compare_run(fixture_id, &expected.0, &observed.0)
}

fn compare_run(
    fixture_id: &'static str,
    expected: &CanonicalRun,
    observed: &CanonicalRun,
) -> Result<(), ObservationMismatch> {
    if expected.skipped_leading_bom != observed.skipped_leading_bom {
        return Err(ObservationMismatch::new(
            fixture_id,
            "expected.preprocessing.skipped_leading_bom",
        ));
    }
    if expected.tokens.len() != observed.tokens.len() {
        return Err(ObservationMismatch::new(
            fixture_id,
            "expected.tokens.length",
        ));
    }
    for (index, (expected_token, observed_token)) in
        expected.tokens.iter().zip(&observed.tokens).enumerate()
    {
        compare_token(fixture_id, index, expected_token, observed_token)?;
    }
    if expected.diagnostics.len() != observed.diagnostics.len() {
        return Err(ObservationMismatch::new(
            fixture_id,
            "expected.diagnostics.length",
        ));
    }
    for (index, (expected_diagnostic, observed_diagnostic)) in expected
        .diagnostics
        .iter()
        .zip(&observed.diagnostics)
        .enumerate()
    {
        if expected_diagnostic != observed_diagnostic {
            return Err(ObservationMismatch::new(
                fixture_id,
                format!("expected.diagnostics[{index}]"),
            ));
        }
    }
    if expected.coverage != observed.coverage {
        return Err(ObservationMismatch::new(fixture_id, "expected.coverage"));
    }
    if expected.completion != observed.completion {
        return Err(ObservationMismatch::new(fixture_id, "expected.completion"));
    }
    if expected.limits != observed.limits {
        return Err(ObservationMismatch::new(fixture_id, "expected.limits"));
    }
    if expected.usage != observed.usage {
        return Err(ObservationMismatch::new(fixture_id, "expected.usage"));
    }
    Ok(())
}

fn compare_token(
    fixture_id: &'static str,
    token_index: usize,
    expected: &Token,
    observed: &Token,
) -> Result<(), ObservationMismatch> {
    let prefix = format!("expected.tokens[{token_index}]");
    match (expected, observed) {
        (
            Token::Character {
                source: expected_source,
                interpreted: expected_interpreted,
            },
            Token::Character {
                source: observed_source,
                interpreted: observed_interpreted,
            },
        ) => {
            if expected_source != observed_source {
                return Err(ObservationMismatch::new(
                    fixture_id,
                    format!("{prefix}.source"),
                ));
            }
            if expected_interpreted != observed_interpreted {
                return Err(ObservationMismatch::new(
                    fixture_id,
                    format!("{prefix}.interpreted"),
                ));
            }
        }
        (
            Token::Tag {
                kind: expected_kind,
                complete: expected_complete,
                open_delimiter: expected_open,
                name: expected_name,
                interpreted_name: expected_interpreted_name,
                attributes: expected_attributes,
                self_closing_solidus: expected_solidus,
                close_delimiter: expected_close,
            },
            Token::Tag {
                kind: observed_kind,
                complete: observed_complete,
                open_delimiter: observed_open,
                name: observed_name,
                interpreted_name: observed_interpreted_name,
                attributes: observed_attributes,
                self_closing_solidus: observed_solidus,
                close_delimiter: observed_close,
            },
        ) => {
            for (suffix, equal) in [
                ("kind", expected_kind == observed_kind),
                ("complete", expected_complete == observed_complete),
                ("open_delimiter", expected_open == observed_open),
                ("name", expected_name == observed_name),
                (
                    "interpreted_name",
                    expected_interpreted_name == observed_interpreted_name,
                ),
                ("self_closing_solidus", expected_solidus == observed_solidus),
                ("close_delimiter", expected_close == observed_close),
            ] {
                if !equal {
                    return Err(ObservationMismatch::new(
                        fixture_id,
                        format!("{prefix}.{suffix}"),
                    ));
                }
            }
            if expected_attributes.len() != observed_attributes.len() {
                return Err(ObservationMismatch::new(
                    fixture_id,
                    format!("{prefix}.attributes.length"),
                ));
            }
            for (attribute_index, (expected_attribute, observed_attribute)) in expected_attributes
                .iter()
                .zip(observed_attributes)
                .enumerate()
            {
                compare_attribute(
                    fixture_id,
                    &format!("{prefix}.attributes[{attribute_index}]"),
                    expected_attribute,
                    observed_attribute,
                )?;
            }
        }
        (Token::EndOfFile { at: expected_at }, Token::EndOfFile { at: observed_at }) => {
            if expected_at != observed_at {
                return Err(ObservationMismatch::new(fixture_id, format!("{prefix}.at")));
            }
        }
        _ => {
            return Err(ObservationMismatch::new(
                fixture_id,
                format!("{prefix}.kind"),
            ));
        }
    }
    Ok(())
}

fn compare_attribute(
    fixture_id: &'static str,
    prefix: &str,
    expected: &Attribute,
    observed: &Attribute,
) -> Result<(), ObservationMismatch> {
    for (suffix, equal) in [
        ("complete", expected.complete == observed.complete),
        ("name", expected.name == observed.name),
        (
            "interpreted_name",
            expected.interpreted_name == observed.interpreted_name,
        ),
        ("value", expected.value == observed.value),
        (
            "interpreted_value",
            expected.interpreted_value == observed.interpreted_value,
        ),
        ("disposition", expected.disposition == observed.disposition),
    ] {
        if !equal {
            return Err(ObservationMismatch::new(
                fixture_id,
                format!("{prefix}.{suffix}"),
            ));
        }
    }
    Ok(())
}
