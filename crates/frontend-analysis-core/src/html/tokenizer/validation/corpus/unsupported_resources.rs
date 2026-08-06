use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

#[rustfmt::skip]
pub(super) fn add_unsupported(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(unsupported_input(
        "UNSUP-001",
        "character reference in Data",
        "&x",
        0,
        Capability::CharacterReference(CharacterReferenceContext::Data),
        Availability::Deferred,
        ByteSpan::new(0, 1),
    ));
    fixtures.push(unsupported_input(
        "UNSUP-002",
        "character reference in an attribute value",
        "<a x=&x>",
        5,
        Capability::CharacterReference(CharacterReferenceContext::AttributeValue),
        Availability::Deferred,
        ByteSpan::new(5, 6),
    ));
    fixtures.push(unsupported_input(
        "UNSUP-003",
        "markup declaration boundary",
        "<!x>",
        0,
        Capability::MarkupDeclaration,
        Availability::Deferred,
        ByteSpan::new(0, 2),
    ));
    fixtures.push(unsupported_input(
        "UNSUP-004",
        "processing instruction boundary",
        "<?x>",
        0,
        Capability::ProcessingInstruction,
        Availability::Deferred,
        ByteSpan::new(0, 2),
    ));

    for (id, purpose, source, name_end, name, mode) in [
        ("UNSUP-005", "title requires RCDATA", "<title>x", 6, "title", TokenizerMode::Rcdata),
        ("UNSUP-006", "textarea requires RCDATA", "<textarea>x", 9, "textarea", TokenizerMode::Rcdata),
        ("UNSUP-007", "style requires RAWTEXT", "<style>x", 6, "style", TokenizerMode::RawText),
        ("UNSUP-008", "xmp requires RAWTEXT", "<xmp>x", 4, "xmp", TokenizerMode::RawText),
        ("UNSUP-009", "iframe requires RAWTEXT", "<iframe>x", 7, "iframe", TokenizerMode::RawText),
        ("UNSUP-010", "noembed requires RAWTEXT", "<noembed>x", 8, "noembed", TokenizerMode::RawText),
        ("UNSUP-011", "noframes requires RAWTEXT", "<noframes>x", 9, "noframes", TokenizerMode::RawText),
        ("UNSUP-012", "script requires Script data", "<script>x", 7, "script", TokenizerMode::ScriptData),
        ("UNSUP-013", "noscript requires tree-controlled mode", "<noscript>x", 9, "noscript", TokenizerMode::Noscript),
        ("UNSUP-014", "plaintext requires PLAINTEXT", "<plaintext>x", 10, "plaintext", TokenizerMode::Plaintext),
    ] {
        let tag_end = name_end + 1;
        let token = tag(
            source,
            TokenKind::StartTag,
            0,
            tag_end,
            0,
            1,
            1,
            name_end,
            name,
            Vec::new(),
            None,
            name_end,
            tag_end,
        );
        fixtures.push(incomplete(
            id,
            FixtureCategory::Unsupported,
            purpose,
            source,
            tag_end,
            vec![token],
            Vec::new(),
            Completion::Unsupported {
                capability: Capability::ContextDependentTokenizerMode(mode),
                availability: Availability::Deferred,
                trigger: UnsupportedTrigger::EmittedToken {
                    token_index: 0,
                    boundary: ByteSpan::new(tag_end, tag_end),
                },
            },
            Limits::generous(),
            usage(source, tag_end + 1, 1, 0, 0, name.len(), 0),
        ));
    }
}

#[rustfmt::skip]
pub(super) fn add_resources(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(resource_fixture(
        "RES-001",
        "source byte limit before preprocessing",
        "ab",
        0,
        Vec::new(),
        Vec::new(),
        Resource::SourceBytes,
        1,
        2,
        ByteSpan::new(0, 0),
        usage("ab", 0, 0, 0, 0, 0, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-002",
        "transition step limit preserves processed evidence",
        "a",
        1,
        vec![character("a", 0, 1, "a")],
        Vec::new(),
        Resource::TransitionSteps,
        1,
        2,
        ByteSpan::new(1, 1),
        usage("a", 1, 1, 0, 0, 1, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-003",
        "emitted-token limit rejects EOF after character output",
        "a",
        1,
        vec![character("a", 0, 1, "a")],
        Vec::new(),
        Resource::EmittedTokens,
        1,
        2,
        ByteSpan::new(1, 1),
        usage("a", 2, 1, 0, 0, 1, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-004",
        "diagnostic limit terminates before recovery mutation",
        "\0",
        0,
        Vec::new(),
        Vec::new(),
        Resource::Diagnostics,
        0,
        1,
        ByteSpan::new(0, 1),
        usage("\0", 1, 0, 0, 0, 0, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-005",
        "attributes-per-tag limit",
        "<a x y>",
        5,
        Vec::new(),
        Vec::new(),
        Resource::AttributesPerTag,
        1,
        2,
        ByteSpan::new(5, 5),
        usage("<a x y>", 6, 0, 0, 1, 2, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-006",
        "retained interpreted byte limit",
        "ab",
        1,
        vec![character("ab", 0, 1, "a")],
        Vec::new(),
        Resource::RetainedInterpretedBytes,
        1,
        2,
        ByteSpan::new(1, 1),
        usage("ab", 2, 1, 0, 0, 1, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-007",
        "temporary buffer byte limit",
        "<a",
        1,
        Vec::new(),
        Vec::new(),
        Resource::TemporaryBufferBytes,
        0,
        1,
        ByteSpan::new(1, 1),
        usage("<a", 1, 0, 0, 0, 0, 0),
    ));

    let mut zero_steps = Limits::generous();
    zero_steps.transition_steps = 0;
    fixtures.push(incomplete(
        "RES-008",
        FixtureCategory::Resource,
        "zero transition-step configuration fails before processing",
        "",
        0,
        Vec::new(),
        Vec::new(),
        Completion::InvalidConfiguration(ConfigurationFailure::ZeroTransitionStepLimit),
        zero_steps,
        usage("", 0, 0, 0, 0, 0, 0),
    ));
    let mut zero_tokens = Limits::generous();
    zero_tokens.emitted_tokens = 0;
    fixtures.push(incomplete(
        "RES-009",
        FixtureCategory::Resource,
        "zero emitted-token configuration fails before processing",
        "",
        0,
        Vec::new(),
        Vec::new(),
        Completion::InvalidConfiguration(ConfigurationFailure::ZeroEmittedTokenLimit),
        zero_tokens,
        usage("", 0, 0, 0, 0, 0, 0),
    ));
}

