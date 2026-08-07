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
        // Data('&', discovers CharacterReference required) = 1.
        // Coverage stays empty: the trigger byte is not fully consumed by
        // an approved transition; coverage and step accounting are separate.
        1,
    ));
    fixtures.push(unsupported_input(
        "UNSUP-002",
        "character reference in an attribute value",
        "<a x=&x>",
        5,
        Capability::CharacterReference(CharacterReferenceContext::AttributeValue),
        Availability::Deferred,
        ByteSpan::new(5, 6),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(=) [7 steps, processed_end=5]
        // +BeforeAttrValue('&',reconsume Unquoted)+Unquoted(reconsume '&', discovers capability) = 9
        9,
    ));
    fixtures.push(unsupported_input(
        "UNSUP-003",
        "markup declaration boundary",
        "<!x>",
        0,
        Capability::MarkupDeclaration,
        Availability::Deferred,
        ByteSpan::new(0, 2),
        // Data(<)+TagOpen('!', discovers MarkupDeclaration required) = 2.
        2,
    ));
    // Not built with unsupported_input(): that helper always authors zero
    // diagnostics, but TagOpen('?') is also the primary dispatch for
    // UnexpectedQuestionMarkInsteadOfTagName (see ERR-006, source "<?").
    // The pinned WHATWG Tag open state requires that parse error
    // unconditionally on '?', regardless of what follows, so this fixture
    // must record it too; the trailing authored "x>" remains unprocessed.
    fixtures.push(incomplete(
        "UNSUP-004",
        FixtureCategory::Unsupported,
        "processing instruction boundary",
        "<?x>",
        2,
        Vec::new(),
        vec![diagnostic(
            DiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName,
            1,
            2,
            DiagnosticContext::TagOpen,
            DiagnosticHandling::Stopped,
            DiagnosticSubject::InputLocation,
        )],
        Completion::Unsupported {
            capability: Capability::ProcessingInstruction,
            availability: Availability::Deferred,
            trigger: UnsupportedTrigger::Input(ByteSpan::new(2, 2)),
        },
        Limits::generous(),
        // Data(<)+TagOpen('?', UnexpectedQuestionMarkInsteadOfTagName,
        // deferred PI) = 2 — the same TagOpen('?') dispatch as ERR-006.
        usage("<?x>", 2, 0, 1, 0, 0, 0),
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
        // For a context-changing name of N normalized ASCII units:
        // Data('<') + TagOpen(first, reconsume) + N TagName examinations
        // (including the reconsumed first unit) + TagName('>') = N + 3.
        // The tag is emitted and processing stops at the empty boundary after
        // '>'; the following authored data unit remains unprocessed.
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
    // No normalized unit or state transition is attempted: source-size
    // preflight rejects attempted=2 at the empty boundary before processing.
    fixtures.push(resource_fixture(
        "RES-001", "source byte limit before preprocessing", "ab", 0,
        Vec::new(), Vec::new(), Resource::SourceBytes, 1, 2,
        ByteSpan::new(0, 0), usage("ab", 0, 0, 0, 0, 0, 0),
    ));
    // Data('a') commits. Data(EOF) is attempted=2 but cannot commit because
    // TransitionSteps itself is limited to 1; processed_end remains 1.
    fixtures.push(resource_fixture(
        "RES-002", "transition step limit preserves processed evidence", "a", 1,
        vec![character("a", 0, 1, "a")], Vec::new(),
        Resource::TransitionSteps, 1, 2, ByteSpan::new(1, 1),
        usage("a", 1, 1, 0, 0, 1, 0),
    ));
    // Data('a') and Data(EOF) both commit. EOF emission attempts token 2 and
    // is refused; token emission is not an additional transition.
    fixtures.push(resource_fixture(
        "RES-003", "emitted-token limit rejects EOF after character output", "a", 1,
        vec![character("a", 0, 1, "a")], Vec::new(),
        Resource::EmittedTokens, 1, 2, ByteSpan::new(1, 1),
        usage("a", 2, 1, 0, 0, 1, 0),
    ));
    // Data(NUL) commits, but diagnostic append attempted=1 is refused before
    // replacement/recovery mutation; processed_end remains 0.
    fixtures.push(resource_fixture(
        "RES-004", "diagnostic limit terminates before recovery mutation", "\0", 0,
        Vec::new(), Vec::new(), Resource::Diagnostics, 0, 1,
        ByteSpan::new(0, 1), usage("\0", 1, 0, 0, 0, 0, 0),
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
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(sp,reconsume AfterAttrName)+AfterAttrName(reconsume sp)
        // [8 committed, processed_end=5] +AfterAttrName(y, attempts second attribute creation)
        // commits as transition 9, while attempted attribute count 2 is refused.
        usage("<a x y>", 9, 0, 0, 1, 2, 0),
    ));
    // Data('a') commits one retained byte. Data('b') commits transition 2,
    // but retained interpreted byte attempted=2 is refused at boundary 1.
    fixtures.push(resource_fixture(
        "RES-006", "retained interpreted byte limit", "ab", 1,
        vec![character("ab", 0, 1, "a")], Vec::new(),
        Resource::RetainedInterpretedBytes, 1, 2, ByteSpan::new(1, 1),
        usage("ab", 2, 1, 0, 0, 1, 0),
    ));
    fixtures.push(resource_fixture(
        "RES-007",
        "retained interpreted byte limit in active tag-name builder",
        "<a",
        1,
        Vec::new(),
        Vec::new(),
        Resource::RetainedInterpretedBytes,
        0,
        1,
        ByteSpan::new(1, 1),
        // Data(<)+TagOpen(a,create tag,reconsume) [2 committed, processed_end=1]
        // +TagName(reconsumed a) commits transition 3, but the active builder's
        // retained interpreted name append attempted=1 is refused. This is not
        // TemporaryBufferBytes: the tag-name builder owns retained output evidence.
        usage("<a", 3, 0, 0, 0, 0, 0),
    ));

    let mut zero_steps = Limits::generous();
    zero_steps.transition_steps = 0;
    fixtures.push(incomplete(
        "RES-008", FixtureCategory::Resource,
        "zero transition-step configuration fails before processing", "", 0,
        Vec::new(), Vec::new(),
        Completion::InvalidConfiguration(ConfigurationFailure::ZeroTransitionStepLimit),
        zero_steps, usage("", 0, 0, 0, 0, 0, 0),
    ));
    let mut zero_tokens = Limits::generous();
    zero_tokens.emitted_tokens = 0;
    fixtures.push(incomplete(
        "RES-009", FixtureCategory::Resource,
        "zero emitted-token configuration fails before processing", "", 0,
        Vec::new(), Vec::new(),
        Completion::InvalidConfiguration(ConfigurationFailure::ZeroEmittedTokenLimit),
        zero_tokens, usage("", 0, 0, 0, 0, 0, 0),
    ));
}
