//! Supplemental candidate-independent `REG-113-*` regression fixtures.
//!
//! These fixtures are separate from the immutable 72-fixture initial corpus
//! (`PRE-`, `TOK-`, `ERR-`, `UNSUP-`, `RES-`, `ADV-`). They capture the
//! cross-product of emission-conditioned and observation-conditioned
//! diagnostics with top-level `EmittedTokens` resource refusal, per the
//! merged #111 emission-conditioned diagnostic contract described in
//! `docs/development/HTML_TOKENIZER_VALIDATION.md`.
//!
//! Every expected observation here is derived directly from the pinned
//! WHATWG algorithm and the approved #109/#110/#111 contracts, cross-checked
//! against `ERR-011`, `ERR-016`, and `ERR-017`. No production tokenizer
//! candidate output was used to author these values.

use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

fn emission_refusal_limits() -> Limits {
    let mut limits = Limits::generous();
    limits.emitted_tokens = 1;
    limits
}

/// `REG-113-end-tag-attributes-emission-refusal`
///
/// Source: `z</a x>`. The leading `z` Character token consumes the single
/// permitted `EmittedTokens` slot, so the end-tag token attempted by the
/// `AfterAttributeName` dispatch on the reconsumed `>` is refused as a whole
/// atomic sub-effect: no partial commit of the close delimiter occurs, so
/// the authored `>` byte at [6, 7) stays in the unprocessed suffix (compare
/// `RES-005`/`RES-007`, where an equivalent all-or-nothing refusal excludes
/// the triggering byte from the processed prefix).
///
/// `EndTagWithAttributes` is emission-conditioned (#111): its parse-error
/// fact exists only once the end-tag token emission commits. Because
/// emission never commits here, the diagnostic must not appear at all, even
/// though the abandoned tag builder held authored attribute evidence (`x`).
fn end_tag_attributes_emission_refusal() -> HtmlTokenizerFixture {
    let source = "z</a x>";
    // Independently derived state-dispatch trace (#109), one transition per
    // attempted dispatch including reconsume, per the #111 counting rule:
    //   1. Data('z')                                   -- emits Character 'z'
    //   2. Data('<')                                   -- switches TagOpen
    //   3. TagOpen('/')                                 -- switches EndTagOpen
    //   4. EndTagOpen('a', reconsume)                   -- creates end tag, reconsume TagName
    //   5. TagName(reconsumed 'a')                       -- appends name "a"
    //   6. TagName(' ')                                  -- switches BeforeAttributeName
    //   7. BeforeAttributeName('x', create, reconsume)    -- creates attribute, reconsume AttributeName
    //   8. AttributeName(reconsumed 'x')                  -- appends attribute name "x"
    //   9. AttributeName('>', reconsume)                  -- switches AfterAttributeName, reconsume
    //  10. AfterAttributeName(reconsumed '>', attempt emit) -- commits the dispatch; the
    //      atomic close-delimiter-commit + token-emission sub-effect is refused
    //      (EmittedTokens limit=1, attempted=2). No further dispatch (no EOF)
    //      follows a refused run.
    let transition_steps = 10;
    let processed_end = 6; // excludes the refused '>' at [6, 7)

    incomplete(
        "REG-113-end-tag-attributes-emission-refusal",
        FixtureCategory::Regression,
        "EndTagWithAttributes must not appear when the corresponding end-tag \
         token emission is refused by the EmittedTokens resource limit",
        source,
        processed_end,
        vec![character(source, 0, 1, "z")],
        Vec::new(),
        Completion::ResourceLimit {
            resource: Resource::EmittedTokens,
            limit: 1,
            attempted: 2,
            at: ByteSpan::new(processed_end, processed_end),
        },
        emission_refusal_limits(),
        // retained_interpreted_bytes = 'z' (1) + abandoned tag name "a" (1)
        // + abandoned attribute name "x" (1) = 3. Attribute value is Missing
        // (0 interpreted bytes). Builder-owned evidence counts toward the
        // RetainedInterpretedBytes resource while the builder is active,
        // independent of whether the tag later emits (docs: Resource
        // Ownership).
        usage(source, transition_steps, 1, 0, 1, 3, 0),
    )
}

/// `REG-113-end-tag-trailing-solidus-emission-refusal`
///
/// Source: `z</a/>`. Same emission-refusal shape as above, but the abandoned
/// end-tag builder carries lexical trailing-solidus evidence instead of an
/// attribute. `EndTagWithTrailingSolidus` is emission-conditioned (#111) and
/// must not appear when the end-tag token never emits.
fn end_tag_trailing_solidus_emission_refusal() -> HtmlTokenizerFixture {
    let source = "z</a/>";
    // Independently derived state-dispatch trace (#109):
    //   1. Data('z')                                -- emits Character 'z'
    //   2. Data('<')                                 -- switches TagOpen
    //   3. TagOpen('/')                               -- switches EndTagOpen
    //   4. EndTagOpen('a', reconsume)                 -- creates end tag, reconsume TagName
    //   5. TagName(reconsumed 'a')                     -- appends name "a"
    //   6. TagName('/')                                -- switches SelfClosingStartTag
    //   7. SelfClosingStartTag('>', attempt emit)       -- commits the dispatch; the
    //      atomic close-delimiter-commit + token-emission sub-effect is refused
    //      (EmittedTokens limit=1, attempted=2). No further dispatch follows.
    let transition_steps = 7;
    let processed_end = 5; // excludes the refused '>' at [5, 6)

    incomplete(
        "REG-113-end-tag-trailing-solidus-emission-refusal",
        FixtureCategory::Regression,
        "EndTagWithTrailingSolidus must not appear when the corresponding \
         end-tag token emission is refused by the EmittedTokens resource limit",
        source,
        processed_end,
        vec![character(source, 0, 1, "z")],
        Vec::new(),
        Completion::ResourceLimit {
            resource: Resource::EmittedTokens,
            limit: 1,
            attempted: 2,
            at: ByteSpan::new(processed_end, processed_end),
        },
        emission_refusal_limits(),
        // retained_interpreted_bytes = 'z' (1) + abandoned tag name "a" (1)
        // = 2. No attribute is created for a trailing solidus.
        usage(source, transition_steps, 1, 0, 0, 2, 0),
    )
}

/// `REG-113-missing-attribute-value-emission-refusal`
///
/// Source: `z<a b=>` (the candidate-independent `MissingAttributeValue`
/// source already proven by `ERR-011`, prefixed the same way). Unlike the two
/// end-tag fixtures above, `MissingAttributeValue` is observation-conditioned
/// (#111): its underlying fact becomes true once the `BeforeAttributeValue`
/// dispatch commits `Recovered(CompletedTagWithMissingAttributeValue)`, which
/// the merged #111 contract defines as builder/token-construction completion
/// -- NOT emitted-vector insertion. That recovery sub-effect commits
/// independently of the separate, later-refused top-level token-emission
/// sub-effect, so the diagnostic remains valid with an `AbandonedInput`
/// subject even though the start tag itself never emits.
fn missing_attribute_value_emission_refusal() -> HtmlTokenizerFixture {
    let source = "z<a b=>";
    // Independently derived state-dispatch trace (#109), cross-checked
    // against ERR-011 ("<a x=>"):
    //   1. Data('z')                                -- emits Character 'z'
    //   2. Data('<')                                 -- switches TagOpen
    //   3. TagOpen('a', reconsume)                    -- creates start tag, reconsume TagName
    //   4. TagName(reconsumed 'a')                     -- appends name "a"
    //   5. TagName(' ')                                -- switches BeforeAttributeName
    //   6. BeforeAttributeName('b', create, reconsume)  -- creates attribute, reconsume AttributeName
    //   7. AttributeName(reconsumed 'b')                -- appends attribute name "b"
    //   8. AttributeName('=')                           -- switches BeforeAttributeValue
    //   9. BeforeAttributeValue('>')                    -- commits
    //      Recovered(CompletedTagWithMissingAttributeValue): the tag builder
    //      completes (close delimiter '>' at [6, 7) committed as abandoned
    //      evidence) and the MissingAttributeValue diagnostic is recorded.
    //      This sub-effect is independent of, and commits before, the
    //      separate EmittedTokens refusal of the resulting token's insertion
    //      into the emitted-token vector (limit=1, attempted=2). No further
    //      dispatch (no EOF) follows a refused run.
    let transition_steps = 9;
    // The tag builder completed (including the close delimiter), so the
    // whole authored tag span [1, 7) is processed evidence even though the
    // resulting token never emits; processed_end reaches the full source.
    let processed_end = source.len();
    let abandoned_region = ByteSpan::new(1, 7);
    let diagnostic_location = ByteSpan::new(6, 7);

    incomplete(
        "REG-113-missing-attribute-value-emission-refusal",
        FixtureCategory::Regression,
        "MissingAttributeValue remains valid with an AbandonedInput subject \
         after its builder-level recovery already committed, even when the \
         later independent top-level token emission is refused",
        source,
        processed_end,
        vec![character(source, 0, 1, "z")],
        vec![diagnostic(
            DiagnosticCode::MissingAttributeValue,
            diagnostic_location.start,
            diagnostic_location.end,
            DiagnosticContext::BeforeAttributeValue,
            DiagnosticHandling::Recovered(RecoveryKind::CompletedTagWithMissingAttributeValue),
            DiagnosticSubject::AbandonedInput(abandoned_region),
        )],
        Completion::ResourceLimit {
            resource: Resource::EmittedTokens,
            limit: 1,
            attempted: 2,
            at: ByteSpan::new(processed_end, processed_end),
        },
        emission_refusal_limits(),
        // retained_interpreted_bytes = 'z' (1) + abandoned tag name "a" (1)
        // + abandoned attribute name "b" (1) = 3. The attribute value is
        // MissingAfterEquals (0 interpreted bytes).
        usage(source, transition_steps, 1, 1, 1, 3, 0),
    )
}

/// `REG-113-end-tag-atomic-diagnostics-limit-refusal`
///
/// Source: `</a x/>`. The end-tag builder for `a` collects both an
/// attribute (`x`) and a trailing solidus, so successful emission would
/// require committing both emission-conditioned diagnostic facts,
/// `EndTagWithAttributes` and `EndTagWithTrailingSolidus` (#111), together
/// with the end-tag token itself. With `max_diagnostics = 1` and zero
/// diagnostics committed so far, the compound operation requires 2 pending
/// diagnostics against a limit of 1, so the whole atomic operation --
/// both diagnostics and the token emission alike -- is refused (merged #111
/// atomic multi-diagnostic resource accounting, PR #128). No end-tag token,
/// no diagnostic, and no EOF token results.
fn end_tag_atomic_diagnostics_limit_refusal() -> HtmlTokenizerFixture {
    let source = "</a x/>";
    // Independently derived state-dispatch trace (#109), cross-checked
    // against ERR-016 ("</a x>") and ERR-017 ("</a/>"):
    //   1. Data('<')                                     -- switches TagOpen
    //   2. TagOpen('/')                                    -- switches EndTagOpen
    //   3. EndTagOpen('a', reconsume)                      -- creates end tag, reconsume TagName
    //   4. TagName(reconsumed 'a')                         -- appends name "a"
    //   5. TagName(' ')                                    -- switches BeforeAttributeName
    //   6. BeforeAttributeName('x', create, reconsume)      -- creates attribute, reconsume AttributeName
    //   7. AttributeName(reconsumed 'x')                    -- appends attribute name "x"
    //   8. AttributeName('/', reconsume)                    -- switches AfterAttributeName, reconsume
    //   9. AfterAttributeName(reconsumed '/')                -- switches SelfClosingStartTag
    //  10. SelfClosingStartTag('>', compound emission attempt refused) -- commits
    //      the dispatch; the atomic compound operation (both
    //      emission-conditioned diagnostics plus token emission) requires 2
    //      pending diagnostics against Diagnostics limit=1 with 0 committed,
    //      so the whole operation is refused. No further dispatch (no EOF)
    //      follows a refused run.
    let transition_steps = 10;
    let processed_end = 6; // excludes the refused '>' at [6, 7)

    let mut limits = Limits::generous();
    limits.diagnostics = 1;

    incomplete(
        "REG-113-end-tag-atomic-diagnostics-limit-refusal",
        FixtureCategory::Regression,
        "the atomic multi-diagnostic end-tag emission attempt is refused by \
         the Diagnostics resource limit when the required pending \
         diagnostics (EndTagWithAttributes and EndTagWithTrailingSolidus) \
         exceed the configured limit, per merged #111 atomic multi-diagnostic \
         resource accounting",
        source,
        processed_end,
        Vec::new(),
        Vec::new(),
        Completion::ResourceLimit {
            resource: Resource::Diagnostics,
            limit: 1,
            attempted: 2,
            at: ByteSpan::new(processed_end, processed_end),
        },
        limits,
        // retained_interpreted_bytes = abandoned tag name "a" (1) +
        // abandoned attribute name "x" (1) = 2. The trailing solidus
        // contributes no interpreted bytes.
        usage(source, transition_steps, 0, 0, 1, 2, 0),
    )
}

pub(super) fn add_regressions(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(end_tag_attributes_emission_refusal());
    fixtures.push(end_tag_trailing_solidus_emission_refusal());
    fixtures.push(missing_attribute_value_emission_refusal());
    fixtures.push(end_tag_atomic_diagnostics_limit_refusal());
}
