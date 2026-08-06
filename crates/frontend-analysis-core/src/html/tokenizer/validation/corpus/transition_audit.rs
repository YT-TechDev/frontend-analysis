//! Candidate-independent transition-step derivation audit for the initial
//! 72-fixture corpus.
//!
//! # Authority
//!
//! This audit table is authored directly against the pinned WHATWG states
//! approved in #109 (Data, Tag open, End tag open, Tag name, Before
//! attribute name, Attribute name, After attribute name, Before attribute
//! value, Attribute value double/single/unquoted, After attribute value
//! (quoted), Self-closing start tag) and the #111 counting rule: "one
//! transition step per attempted specification-state transition, including
//! reconsume." It does not consult, generate from, or compare against any
//! production tokenizer implementation; #113 has not implemented one.
//!
//! # Adopted derivation model
//!
//! 1. Each preprocessed input unit (already CRLF/CR-normalized to a single
//!    interpreted LF unit where applicable) or the conceptual EOF unit,
//!    examined under a specification state -- whether by fresh consumption
//!    or by reconsume -- is exactly one committed transition step.
//! 2. A leading UTF-8 BOM skip is input preprocessing, not a transition
//!    step (#109 §3).
//! 3. A diagnostic detected purely during preprocessing input-unit creation
//!    (`DiagnosticContext::InputPreprocessing`) adds no extra step; the
//!    flagged unit still costs exactly one step when a state examines it.
//! 4. A step whose dispatch discovers that the required destination is an
//!    unimplemented capability still counts as one attempted transition (the
//!    discovery itself); no further steps are attempted. Coverage
//!    (`processed_prefix`/`unprocessed_suffix`) is governed separately by
//!    the already-approved #111 rule that trigger bytes remain unprocessed
//!    unless fully consumed by an approved transition -- coverage and
//!    transition-step accounting are orthogonal and are not required to
//!    advance together.
//! 5. A step whose sub-effect (token emission, diagnostic append, attribute
//!    creation, buffer growth) would exceed a resource limit *other than*
//!    `transition_steps` still counts as one attempted transition; only the
//!    specific sub-effect is refused and nothing after it is attempted.
//! 6. A step that would itself exceed the `transition_steps` limit does not
//!    commit (self-referential: the thing being limited is the count of
//!    attempted transitions).
//!
//! Rules 5 and 6 are cross-checked by [`RES-002`] (transition_steps is the
//! limited resource: the next step does not commit) versus [`RES-003`],
//! [`RES-004`], [`RES-006`] (a different resource is limited: the examining
//! step commits, only its specific sub-effect is refused).
//!
//! # Reading this table
//!
//! Each entry is `(fixture_id, committed transition_steps)`. The full
//! ordered per-fixture state trace (attempted transitions, reconsume points,
//! and the terminal transition or refusal) is recorded as a comment directly
//! above that fixture's construction in `preprocessing.rs`, `supported.rs`,
//! `diagnostics.rs`, `unsupported_resources.rs`, and `adversarial.rs`. This
//! table exists to make the full 72-fixture inventory reviewable in one
//! place and to be mechanically cross-checked against the corpus by
//! [`transition_audit_covers_every_fixture_and_matches_corpus`].
#[rustfmt::skip]
pub(super) const TRANSITION_STEP_AUDIT: &[(&str, usize)] = &[
    // PRE: input preprocessing and UTF-8/CRLF/BOM evidence.
    ("PRE-001", 1),  // Data(EOF)
    ("PRE-002", 2),  // Data('a')+Data(EOF)
    ("PRE-003", 3),  // Data('é')+Data('界')+Data(EOF) -- two scalar units, not 5 UTF-8 bytes
    ("PRE-004", 2),  // BOM skip is not a step; Data('a')+Data(EOF)
    ("PRE-005", 3),  // Data('a')+Data(U+FEFF)+Data(EOF)
    ("PRE-006", 2),  // Data(LF)+Data(EOF)
    ("PRE-007", 2),  // CR normalizes to one LF unit; Data(LF)+Data(EOF)
    ("PRE-008", 2),  // CRLF normalizes to one LF unit; Data(LF)+Data(EOF) -- not 2 raw bytes
    ("PRE-009", 4),  // three normalized LF units (CRLF, LF, CR) + Data(EOF)
    ("PRE-010", 5),  // 4 Data-context scalar units (FF, U+0001, NUL, U+FDD0) + Data(EOF)

    // TOK: clean supported token observations.
    ("TOK-001", 4),  // Data('a')+Data('b')+Data('c')+Data(EOF)
    ("TOK-002", 4),  // 3 scalar Data transitions + Data(EOF)
    ("TOK-003", 5),  // <a>
    ("TOK-004", 7),  // <DiV>
    ("TOK-005", 6),  // </a>
    ("TOK-006", 8),  // </DIV>
    ("TOK-007", 7),  // <a >
    ("TOK-008", 9),  // <a x>
    ("TOK-009", 11), // <a x=y>
    ("TOK-010", 18), // <a x="" y="z">
    ("TOK-011", 18), // <a x='' y='z'>
    ("TOK-012", 6),  // <a/>

    // ERR: one primary case per approved diagnostic code.
    ("ERR-001", 2),  // Data(U+FDD0)+Data(EOF)
    ("ERR-002", 2),  // Data(U+0001)+Data(EOF)
    ("ERR-003", 2),  // Data(NUL)+Data(EOF)
    ("ERR-004", 3),  // Data(<)+TagOpen(EOF, literal '<')+Data(EOF)
    ("ERR-005", 4),  // Data(<)+TagOpen('1',reconsume Data)+Data(reconsume '1')+Data(EOF)
    ("ERR-006", 2),  // Data(<)+TagOpen('?', deferred PI) -- already hand-authored correctly
    ("ERR-007", 4),  // </>
    ("ERR-008", 4),  // <a  (EOF in tag name)
    ("ERR-009", 9),  // <a =x>
    ("ERR-010", 11), // <a x"y>
    ("ERR-011", 9),  // <a x=>
    ("ERR-012", 13), // <a x=a"b>
    ("ERR-013", 16), // <a x="y"z>
    ("ERR-014", 12), // <a /x>
    ("ERR-015", 13), // <a x x>
    ("ERR-016", 10), // </a x>
    ("ERR-017", 7),  // </a/>

    // UNSUP: explicit unsupported/deferred capability boundaries.
    ("UNSUP-001", 1),  // Data('&', discovers CharacterReference)
    ("UNSUP-002", 9),  // <a x=&x> -- 7 committed + BeforeAttrValue('&') + Unquoted(reconsume '&', discovers)
    ("UNSUP-003", 2),  // Data(<, always-approved)+TagOpen('!', discovers MarkupDeclaration)
    ("UNSUP-004", 2),  // Data(<)+TagOpen('?', discovers ProcessingInstruction)
    ("UNSUP-005", 8),  // <title>x   (N+3 formula, N=5)
    ("UNSUP-006", 11), // <textarea>x (N=8)
    ("UNSUP-007", 8),  // <style>x (N=5)
    ("UNSUP-008", 6),  // <xmp>x (N=3)
    ("UNSUP-009", 9),  // <iframe>x (N=6)
    ("UNSUP-010", 10), // <noembed>x (N=7)
    ("UNSUP-011", 11), // <noframes>x (N=8)
    ("UNSUP-012", 9),  // <script>x (N=6)
    ("UNSUP-013", 11), // <noscript>x (N=8)
    ("UNSUP-014", 12), // <plaintext>x (N=9)

    // RES: resource and invalid-configuration states.
    ("RES-001", 0), // SourceBytes checked before preprocessing: no steps attempted
    ("RES-002", 1), // transition_steps is the limited resource itself (rule 6)
    ("RES-003", 2), // EmittedTokens limited; Data(EOF) step still commits (rule 5)
    ("RES-004", 1), // Diagnostics limited; Data(NUL) step still commits (rule 5)
    ("RES-005", 9), // AttributesPerTag limited; AfterAttributeName('y') step still commits (rule 5)
    ("RES-006", 2), // RetainedInterpretedBytes limited; Data('b') step still commits (rule 5)
    ("RES-007", 3), // TemporaryBufferBytes limited; TagName(reconsume 'a') step still commits (rule 5)
    ("RES-008", 0), // InvalidConfiguration before any processing
    ("RES-009", 0), // InvalidConfiguration before any processing

    // ADV: adversarial and cross-cutting invariants.
    ("ADV-001", 4),  // <1
    ("ADV-002", 4),  // <>
    ("ADV-003", 25), // <a x=1 y='2' z="3">
    ("ADV-004", 13), // <a X x>
    ("ADV-005", 6),  // <a\0>
    ("ADV-006", 13), // <a x\0=y\0>
    ("ADV-007", 13), // é<a x=界>ß
    ("ADV-008", 8),  // <a =>
    ("ADV-009", 9),  // <a x='  (EOF in single-quoted value)
    ("ADV-010", 65), // 64 'a' characters + Data(EOF)
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::super::initial_corpus;
    use super::TRANSITION_STEP_AUDIT;

    #[test]
    fn transition_audit_covers_every_fixture_and_matches_corpus() {
        let fixtures = initial_corpus();
        assert_eq!(
            fixtures.len(),
            TRANSITION_STEP_AUDIT.len(),
            "audit table must cover exactly the initial 72-fixture inventory"
        );

        let audit_ids: BTreeSet<&str> =
            TRANSITION_STEP_AUDIT.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            audit_ids.len(),
            TRANSITION_STEP_AUDIT.len(),
            "audit table must not contain duplicate fixture IDs"
        );

        let corpus_ids: BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
        assert_eq!(
            audit_ids, corpus_ids,
            "audit table fixture IDs must exactly match the corpus inventory"
        );

        for fixture in &fixtures {
            let (_, expected_steps) = TRANSITION_STEP_AUDIT
                .iter()
                .find(|(id, _)| *id == fixture.id)
                .unwrap_or_else(|| panic!("missing transition audit entry for {}", fixture.id));
            assert_eq!(
                fixture.expected.0.usage.transition_steps,
                *expected_steps,
                "{}: fixture usage.transition_steps must match the independently \
                 derived audit value",
                fixture.id
            );
        }
    }
}
