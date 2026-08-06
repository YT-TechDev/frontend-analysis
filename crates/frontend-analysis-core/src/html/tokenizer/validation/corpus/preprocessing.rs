use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

pub(super) fn add_preprocessing(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(complete(
        "PRE-001",
        FixtureCategory::Preprocessing,
        "empty input and exact EOF",
        "",
        vec![eof(0)],
        Vec::new(),
        None,
        0,
    ));
    fixtures.push(complete_text(
        "PRE-002",
        FixtureCategory::Preprocessing,
        "ASCII character input",
        "a",
        "a",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_text(
        "PRE-003",
        FixtureCategory::Preprocessing,
        "multi-byte UTF-8 scalar input",
        "é界",
        "é界",
        Vec::new(),
        None,
    ));
    fixtures.push(complete(
        "PRE-004",
        FixtureCategory::Preprocessing,
        "leading UTF-8 BOM skipped with retained evidence",
        "\u{feff}a",
        vec![character("\u{feff}a", 3, 4, "a"), eof(4)],
        Vec::new(),
        Some(ByteSpan::new(0, 3)),
        0,
    ));
    fixtures.push(complete_text(
        "PRE-005",
        FixtureCategory::Preprocessing,
        "non-leading U+FEFF retained as character data",
        "a\u{feff}",
        "a\u{feff}",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_text(
        "PRE-006",
        FixtureCategory::Preprocessing,
        "LF preserves its authored byte range",
        "\n",
        "\n",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_text(
        "PRE-007",
        FixtureCategory::Preprocessing,
        "lone CR is interpreted as LF while retaining raw CR",
        "\r",
        "\n",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_text(
        "PRE-008",
        FixtureCategory::Preprocessing,
        "CRLF is one interpreted LF over a two-byte raw span",
        "\r\n",
        "\n",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_text(
        "PRE-009",
        FixtureCategory::Preprocessing,
        "mixed CR LF and CRLF normalization remains lossless",
        "\r\n\n\r",
        "\n\n\n",
        Vec::new(),
        None,
    ));
    let pre10 = "\u{000c}\0\u{fdd0}";
    fixtures.push(complete_text(
        "PRE-010",
        FixtureCategory::Preprocessing,
        "FF NUL control and noncharacter locations are explicit",
        pre10,
        "\u{000c}\u{fffd}\u{fdd0}",
        vec![
            diagnostic(
                DiagnosticCode::ControlCharacterInInputStream,
                0,
                1,
                DiagnosticContext::InputPreprocessing,
                DiagnosticHandling::Continued,
                DiagnosticSubject::InputLocation,
            ),
            diagnostic(
                DiagnosticCode::UnexpectedNullCharacter,
                1,
                2,
                DiagnosticContext::Data,
                DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter),
                DiagnosticSubject::InputLocation,
            ),
            diagnostic(
                DiagnosticCode::NoncharacterInInputStream,
                2,
                5,
                DiagnosticContext::InputPreprocessing,
                DiagnosticHandling::Continued,
                DiagnosticSubject::InputLocation,
            ),
        ],
        None,
    ));
}
