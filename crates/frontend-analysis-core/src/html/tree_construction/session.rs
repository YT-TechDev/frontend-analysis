//! The private TC-S1 tree-construction session, with its accepted TC-S2,
//! TC-S3, and TC-S4 successors.
//!
//! The session exclusively owns every piece of mutable tree-construction
//! state for exactly one run: the insertion mode, the open elements — the
//! shell plus TC-S3's selected ordinary elements — the head pointer, the
//! private document mode and `frameset-ok` flag, the constructed identity
//! counter, the temporary node storage, and the committed diagnostics,
//! actions, and coverage.
//!
//! Ownership boundaries this module keeps:
//!
//! - the session never calls the tokenizer and holds no tokenizer state; it
//!   is driven token by token by [`super::driver`];
//! - the session never escapes to a consumer — it is consumed by
//!   [`Self::finish`] into [`HtmlDocumentShellParts`], which the driver hands
//!   to [`super::result::freeze`];
//! - no mutable session state travels into the frozen result, and the private
//!   document mode and `frameset-ok` flag are never exposed there at all.
//!
//! # Two gates, both before mutation
//!
//! Rule selection and mutation are separated on purpose, and the separation is
//! now two distinct gates:
//!
//! 1. [`admit`] is *pure lexical admission*. It is a function of the validated
//!    token alone, reads no construction state, and recognizes exactly the two
//!    closed categories [`AdmittedElementName::Shell`] and
//!    [`AdmittedElementName::SelectedOrdinary`] while propagating the token's
//!    exact evidence. Unsupported tag syntax — an unproved name, attributes, a
//!    self-closing solidus — is refused here.
//! 2. [`classify`] is *read-only selection*. It is a free function of the
//!    actual insertion mode, the already-read ordered names of the open
//!    selected ordinary elements, and the admitted token: it takes no `&self`,
//!    performs no mutation, and returns either the step to apply or the
//!    [`HtmlTreeCapability`] this subsystem does not prove.
//!
//! TC-S4 adds a third separation inside the mutating half. A supported
//! selected ordinary end tag first resolves a complete
//! [`SelectedOrdinaryEndPlan`] from read-only state — nearest same-name
//! target, the complete intervening suffix, and the bounded implied-end no-op
//! invariant — and only then commits, in one stack mutation followed by the
//! ordered evidence. There is no partial pop, no rollback, and no state to
//! reconstruct after a refusal.
//!
//! An unsupported cell therefore cannot mutate anything, and the session is a
//! valid semantic construction checkpoint at every instant — no rollback,
//! snapshot, or generic checkpoint machinery is needed to freeze the last
//! valid state.
//!
//! # Termination without a work limit
//!
//! This subsystem introduces no tree resource dimension, limit, or work
//! constant. The session performs exactly one insertion-mode rule evaluation
//! per [`HtmlTreeSession::dispatch`] call and never loops internally; per-token
//! termination is instead a structural property [`super::driver`] proves by
//! tracking, for one emitted token, which insertion modes have already been
//! evaluated and refusing to evaluate any of them twice. That is what allows
//! TC-S2's validated `AfterBody -> InBody` recovery back-edge to coexist with
//! finite per-token work: the bound comes from the finite [`InsertionMode`]
//! domain, not from a strictly-forward order or a numeric budget.
//!
//! TC-S4's recovery adds no work dimension either, and no budget stands behind
//! it. Termination stays structural: the nearest-target lookup and the suffix
//! plan each traverse the finite open-element stack once, a committed selected
//! end removes at least its own target from that stack, and the number of
//! recovery relations is exactly the number of elements removed above it. No
//! same-token redispatch, tokenizer feedback, or retry is introduced.

use crate::SourceAnchor;

use super::super::token::{HtmlTagKind, HtmlToken};
use super::result::{
    HtmlConstructedIdentityCounter, HtmlConstructedNodeId, HtmlDocumentShellParts, HtmlElement,
    HtmlSelectedOrdinaryElement, HtmlSelectedOrdinaryElementName, HtmlShellClosure,
    HtmlShellElement, HtmlShellElementName, HtmlShellElementOrigin, HtmlSynthesisCause,
    HtmlTextContribution, HtmlTextNode, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability,
    HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode, HtmlTreeNode, HtmlTreeNodeKind,
    HtmlTreeRecovery, HtmlTreeTokenTrigger,
};

/// The document-construction insertion modes.
///
/// TC-S2 introduces a validated `AfterBody -> InBody` recovery back-edge, so
/// this type deliberately derives no ordering: per-token termination is
/// proved structurally by [`super::driver`] rather than by a strictly forward
/// walk through a total order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

/// The private document mode. TC-S1 never exposes it in the frozen result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HtmlDocumentMode {
    NoQuirks,
    Quirks,
}

/// The closed element-name categories the pure lexical admission gate
/// recognizes.
///
/// The gate stays token-only: it decides which closed domain a tag name
/// belongs to and nothing else. It reads no construction state, so it can
/// never be the place a state-dependent selection decision hides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdmittedElementName {
    Shell(HtmlShellElementName),
    SelectedOrdinary(HtmlSelectedOrdinaryElementName),
}

/// A validated emitted token, normalized to the shapes this subsystem admits.
///
/// Admission is a property of the token alone, so it is decided by [`admit`]
/// before the token reaches any insertion mode. A tag outside the proved
/// element set, a tag carrying attributes, and a self-closing tag are all
/// refused here, before any mutation can happen.
#[derive(Debug, Clone, Copy)]
pub(super) enum AdmittedToken<'run> {
    Characters {
        source: &'run SourceAnchor,
        interpreted: &'run str,
    },
    StartTag {
        name: AdmittedElementName,
        complete: &'run SourceAnchor,
        raw_name: &'run SourceAnchor,
    },
    EndTag {
        name: AdmittedElementName,
        complete: &'run SourceAnchor,
    },
    EndOfFile {
        at: usize,
    },
}

/// Builds one emitted token's trigger evidence.
///
/// Trigger evidence is never an authored origin: it records which token caused
/// an action, including actions that create structure the token did not
/// author. End of file has no authored extent and receives no empty or dummy
/// anchor. Defined for every validated token, admitted or not, so an
/// unsupported stop can name its exact trigger.
pub(super) fn token_trigger(token: &HtmlToken, token_index: usize) -> HtmlTreeTokenTrigger {
    match token {
        HtmlToken::Character(character) => {
            HtmlTreeTokenTrigger::authored(token_index, character.source().clone())
        }
        HtmlToken::Tag(tag) => HtmlTreeTokenTrigger::authored(token_index, tag.complete().clone()),
        HtmlToken::EndOfFile(_) => HtmlTreeTokenTrigger::end_of_file(token_index),
    }
}

impl AdmittedToken<'_> {
    /// Whether this token is a selected ordinary tag, start or end.
    ///
    /// Token-shape evidence only: it reads no construction state and decides
    /// nothing about the tree.
    fn is_selected_ordinary_tag(&self) -> bool {
        matches!(
            self,
            Self::StartTag {
                name: AdmittedElementName::SelectedOrdinary(_),
                ..
            } | Self::EndTag {
                name: AdmittedElementName::SelectedOrdinary(_),
                ..
            }
        )
    }

    /// Whether this token is a shell tag, start or end.
    fn is_shell_tag(&self) -> bool {
        matches!(
            self,
            Self::StartTag {
                name: AdmittedElementName::Shell(_),
                ..
            } | Self::EndTag {
                name: AdmittedElementName::Shell(_),
                ..
            }
        )
    }

    /// The exclusive source offset this token's committed processing covers.
    fn committed_end(&self) -> usize {
        match self {
            Self::Characters { source, .. } => source.range().end(),
            Self::StartTag { complete, .. } | Self::EndTag { complete, .. } => {
                complete.range().end()
            }
            Self::EndOfFile { at } => *at,
        }
    }
}

/// Normalizes one validated emitted token into the admitted shapes.
///
/// Pure and mutation-free. Refuses, with exact typed capability evidence,
/// every token shape this subsystem does not prove.
pub(super) fn admit(token: &HtmlToken) -> Result<AdmittedToken<'_>, HtmlTreeCapability> {
    match token {
        HtmlToken::Character(character) => Ok(AdmittedToken::Characters {
            source: character.source(),
            interpreted: character.interpreted(),
        }),
        HtmlToken::Tag(tag) => {
            // Resolve the closed name domain first, so each syntax refusal can
            // report the capability that belongs to that domain. The frozen
            // predecessor variants keep exactly their old meaning, and a
            // selected ordinary tag never reports a shell-specific one.
            let Some(name) = admitted_element_name(tag.name().interpreted()) else {
                return Err(HtmlTreeCapability::NonShellElementTag);
            };
            if !tag.attributes().is_empty() {
                return Err(match name {
                    AdmittedElementName::Shell(_) => HtmlTreeCapability::ShellTagAttribute,
                    AdmittedElementName::SelectedOrdinary(_) => {
                        HtmlTreeCapability::SelectedOrdinaryTagAttribute
                    }
                });
            }
            if tag.self_closing_solidus().is_some() {
                return Err(match name {
                    AdmittedElementName::Shell(_) => HtmlTreeCapability::SelfClosingShellTag,
                    AdmittedElementName::SelectedOrdinary(_) => {
                        HtmlTreeCapability::SelfClosingSelectedOrdinaryTag
                    }
                });
            }
            match tag.kind() {
                HtmlTagKind::Start => Ok(AdmittedToken::StartTag {
                    name,
                    complete: tag.complete(),
                    raw_name: tag.name().source(),
                }),
                HtmlTagKind::End => Ok(AdmittedToken::EndTag {
                    name,
                    complete: tag.complete(),
                }),
            }
        }
        HtmlToken::EndOfFile(end_of_file) => Ok(AdmittedToken::EndOfFile {
            at: end_of_file.source().range().start(),
        }),
    }
}

/// The closed domain an interpreted tag name belongs to, if it is proved.
///
/// The two categories stay separate closed sets. Nothing is admitted merely
/// because a name looks like an element name, and no arbitrary-name or
/// namespace-switching path exists here.
fn admitted_element_name(interpreted: &str) -> Option<AdmittedElementName> {
    match interpreted {
        "html" => Some(AdmittedElementName::Shell(HtmlShellElementName::Html)),
        "head" => Some(AdmittedElementName::Shell(HtmlShellElementName::Head)),
        "body" => Some(AdmittedElementName::Shell(HtmlShellElementName::Body)),
        "div" => Some(AdmittedElementName::SelectedOrdinary(
            HtmlSelectedOrdinaryElementName::Div,
        )),
        "section" => Some(AdmittedElementName::SelectedOrdinary(
            HtmlSelectedOrdinaryElementName::Section,
        )),
        _ => None,
    }
}

/// Whether the interpreted characters contain any HTML whitespace character.
///
/// Outside the `in body` insertion mode, the HTML Standard treats whitespace
/// characters differently from other characters, and the project-owned
/// tokenizer emits contiguous Data runs rather than one token per character.
/// TC-S1 proves no whitespace-sensitive character handling and no
/// character-run splitting, so a run whose handling would depend on that
/// distinction is refused rather than guessed at.
fn contains_html_whitespace(interpreted: &str) -> bool {
    interpreted
        .chars()
        .any(|character| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
}

/// The TC-S2 partition of one tokenizer-emitted aggregate interpreted
/// character run.
///
/// Classification inspects the existing interpreted string as one scan; it
/// never accesses `SourceText`, never calls the tokenizer, and never creates a
/// substring or fabricated sub-range. The run stays exactly one observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterRunClass {
    AllHtmlWhitespace,
    AllNonHtmlWhitespace,
    Mixed,
}

/// Classifies an interpreted character run by the same HTML whitespace set
/// [`contains_html_whitespace`] uses.
fn classify_character_run(interpreted: &str) -> CharacterRunClass {
    let mut any_whitespace = false;
    let mut any_non_whitespace = false;
    for character in interpreted.chars() {
        if matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ') {
            any_whitespace = true;
        } else {
            any_non_whitespace = true;
        }
    }
    match (any_whitespace, any_non_whitespace) {
        (true, true) => CharacterRunClass::Mixed,
        (false, true) => CharacterRunClass::AllNonHtmlWhitespace,
        // An admitted character token is never empty, so `(false, false)`
        // cannot occur; treating it as whitespace here fabricates nothing
        // because no such run reaches this function.
        (true, false) | (false, false) => CharacterRunClass::AllHtmlWhitespace,
    }
}

/// Where a shell element node's existence comes from, as selected by the
/// insertion-mode rule that inserts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElementProvenance {
    /// The trigger token's own authored start tag is this node's origin.
    AuthoredByTriggerToken,
    /// The node has no authored source. The trigger token made it necessary
    /// but is not its origin.
    Synthesized,
}

/// One committed effect of an insertion-mode rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Effect {
    RecordMissingDoctype,
    InsertHtmlElement(ElementProvenance),
    InsertHeadElement(ElementProvenance),
    InsertBodyElement(ElementProvenance),
    InsertCharacters,
    RecordDuplicateHeadStartTag,
    RecordDuplicateBodyStartTag,
    CloseHeadElement(HtmlShellClosure),
    AcknowledgeShellEndTag(HtmlShellElementName),
    RecordAfterBodyCharacterData,
    InsertSelectedOrdinaryElement(HtmlSelectedOrdinaryElementName),
    CloseSelectedOrdinaryElement(HtmlSelectedOrdinaryElementName),
    RecoverInterveningSelectedOrdinaryElementsAndCloseTarget(HtmlSelectedOrdinaryElementName),
    RecordUnmatchedSelectedOrdinaryEndTag(HtmlSelectedOrdinaryElementName),
    RecordOpenSelectedOrdinaryElementAtEndOfFile,
}

/// What an insertion-mode rule does with the current token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModeStep {
    /// Apply the effect if present, optionally move to `next`, then consume
    /// the token.
    Consume {
        effect: Option<Effect>,
        next: Option<InsertionMode>,
    },
    /// Apply the effect if present, move to `next`, then hand the same token
    /// to that mode. One token stays one observation.
    Reprocess {
        effect: Option<Effect>,
        next: InsertionMode,
    },
    /// Apply the effect if present, then stop document parsing at this token.
    ///
    /// The optional effect is what lets the accepted end-of-file branch record
    /// its open-selected-element diagnostic while still reusing the ordinary
    /// stop behaviour: nothing is popped, closed, or synthesized here.
    Stop { effect: Option<Effect> },
}

/// The selected `in body` character step: insert the token's characters and
/// leave the actual insertion mode unchanged.
///
/// Shared between ordinary `InBody` character handling and TC-S2's
/// `AfterBody + AllHtmlWhitespace` delegation, so the two never duplicate
/// [`HtmlTreeSession::insert_characters`]. `next: None` is what keeps the
/// delegating call's actual mode exactly where it was.
fn selected_in_body_character_step() -> ModeStep {
    ModeStep::Consume {
        effect: Some(Effect::InsertCharacters),
        next: None,
    }
}

/// Selects the rule for one (insertion mode, selected-element state, admitted
/// token) cell.
///
/// This function is the complete, exhaustive statement of the proved action
/// set. Every cell it returns `Ok` for is a cell the accepted candidate-
/// independent GOLD proves; every other cell returns typed unsupported
/// evidence. Notably, no cell is admitted merely because a tag happens to be
/// named `html`, `head`, `body`, or `div`: an admitted name reached in a
/// document position the GOLD does not prove is refused like any other.
///
/// This is the second of the two gates. It is deliberately read-only: it
/// borrows nothing mutable, reads the caller's already-taken ordered
/// selected-name projection, and mutates nothing, so an unsupported result is
/// structurally guaranteed to stop before mutation.
///
/// TC-S3 needed only the selected depth here, because its represented state
/// was `[html, body] ++ [div]^k` and every selected end tag therefore matched
/// the current node or nothing. TC-S4's represented state is `[html, body] ++
/// W` with `W ∈ {Div, Section}*`, so selecting between the matching, the
/// heterogeneous recovery, and the ignored cells needs the ordered names —
/// and nothing more. The projection carries no constructed identity, so no
/// relationship meaning can leak into rule selection.
fn classify(
    mode: InsertionMode,
    open_selected_ordinary: &[HtmlSelectedOrdinaryElementName],
    token: &AdmittedToken<'_>,
) -> Result<ModeStep, HtmlTreeCapability> {
    // A selected ordinary tag is proved only in the actual `in body` mode.
    // Refusing here, before the mode match, is what keeps a selected `div`
    // outside `in body` from reaching the shell walk, the missing-DOCTYPE
    // recovery, a mode change, an action, coverage, or an identity.
    if !matches!(mode, InsertionMode::InBody) && token.is_selected_ordinary_tag() {
        return Err(HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody);
    }
    match mode {
        // Every admitted token is content, and no DOCTYPE can reach TC-S1:
        // the project-owned tokenizer reports markup declarations as its own
        // unsupported capability, so a DOCTYPE token never exists here.
        InsertionMode::Initial => {
            reject_whitespace_sensitive_characters(token)?;
            Ok(ModeStep::Reprocess {
                effect: Some(Effect::RecordMissingDoctype),
                next: InsertionMode::BeforeHtml,
            })
        }
        InsertionMode::BeforeHtml => {
            reject_whitespace_sensitive_characters(token)?;
            match token {
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Html),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::InsertHtmlElement(
                        ElementProvenance::AuthoredByTriggerToken,
                    )),
                    next: Some(InsertionMode::BeforeHead),
                }),
                // The HTML Standard routes `head`, `body`, `html`, and `br`
                // end tags here through the same "anything else" entry as
                // characters, other start tags, and end of file, so the
                // uniform implied-`html` step covers every admitted token.
                _ => Ok(ModeStep::Reprocess {
                    effect: Some(Effect::InsertHtmlElement(ElementProvenance::Synthesized)),
                    next: InsertionMode::BeforeHead,
                }),
            }
        }
        InsertionMode::BeforeHead => {
            reject_whitespace_sensitive_characters(token)?;
            match token {
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Head),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::InsertHeadElement(
                        ElementProvenance::AuthoredByTriggerToken,
                    )),
                    next: Some(InsertionMode::InHead),
                }),
                // An `html` start tag here is processed by the `in body`
                // rules, which TC-S1 does not prove.
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Html),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellStartTagPosition),
                _ => Ok(ModeStep::Reprocess {
                    effect: Some(Effect::InsertHeadElement(ElementProvenance::Synthesized)),
                    next: InsertionMode::InHead,
                }),
            }
        }
        InsertionMode::InHead => {
            reject_whitespace_sensitive_characters(token)?;
            match token {
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Head),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::CloseHeadElement(HtmlShellClosure::AuthoredEndTag)),
                    next: Some(InsertionMode::AfterHead),
                }),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Head),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::RecordDuplicateHeadStartTag),
                    next: None,
                }),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Html),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellStartTagPosition),
                // `body` and `html` end tags are routed to the same "anything
                // else" entry as other start tags, characters, and end of
                // file.
                _ => Ok(ModeStep::Reprocess {
                    effect: Some(Effect::CloseHeadElement(HtmlShellClosure::ImpliedByToken)),
                    next: InsertionMode::AfterHead,
                }),
            }
        }
        InsertionMode::AfterHead => {
            reject_whitespace_sensitive_characters(token)?;
            match token {
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Body),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::InsertBodyElement(
                        ElementProvenance::AuthoredByTriggerToken,
                    )),
                    next: Some(InsertionMode::InBody),
                }),
                // A `head` start tag is a parse error that ignores the token,
                // an `html` start tag is processed by the `in body` rules,
                // and a `head` end tag is an "any other end tag" parse error
                // that ignores the token. None of the three is proved.
                AdmittedToken::StartTag {
                    name:
                        AdmittedElementName::Shell(
                            HtmlShellElementName::Head | HtmlShellElementName::Html,
                        ),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellStartTagPosition),
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Head),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
                _ => Ok(ModeStep::Reprocess {
                    effect: Some(Effect::InsertBodyElement(ElementProvenance::Synthesized)),
                    next: InsertionMode::InBody,
                }),
            }
        }
        // Inside `in body`, whitespace and non-whitespace characters are
        // inserted identically, so a contiguous Data run needs no splitting
        // and needs no whitespace refusal. TC-S3 adds the selected `div`
        // start, matching end, stray end, and end-of-file branches here, and
        // nothing else: `S(k) = [html, body] ++ [div]^k` is the whole
        // represented state.
        InsertionMode::InBody => {
            // No shell interaction over an open selected ordinary element is
            // proved. Refusing before the token match keeps `</body>` with an
            // open `div` from committing any part of the body close.
            if !open_selected_ordinary.is_empty() && token.is_shell_tag() {
                return Err(HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement);
            }
            match token {
                AdmittedToken::Characters { .. } => Ok(selected_in_body_character_step()),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::SelectedOrdinary(name),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::InsertSelectedOrdinaryElement(*name)),
                    next: None,
                }),
                // A supported selected ordinary end tag selects its nearest
                // same-name open target, and which of the three cells applies
                // is decided entirely by where that target is.
                AdmittedToken::EndTag {
                    name: AdmittedElementName::SelectedOrdinary(name),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(match selected_end_target(open_selected_ordinary, *name) {
                        // No same-name target is open. A stray selected
                        // ordinary end tag is a parse error that ignores the
                        // token: one diagnostic, one ignored disposition, and
                        // committed progress, with the tree, the open
                        // elements, the mode, identity, closure, and recovery
                        // evidence all unchanged.
                        SelectedEndTarget::Absent => {
                            Effect::RecordUnmatchedSelectedOrdinaryEndTag(*name)
                        }
                        // The nearest same-name target is the current node.
                        // This is the accepted TC-S3 cell, unchanged: the end
                        // tag closes exactly that element, nothing is
                        // recovered, and no misnested diagnostic is recorded.
                        SelectedEndTarget::Current => Effect::CloseSelectedOrdinaryElement(*name),
                        // The nearest same-name target is open below one or
                        // more differently-nested selected ordinary elements.
                        // Those are recovery-popped, current-first, and the
                        // target is then closed by this same authored end tag
                        // — one misnested diagnostic, however many pops.
                        SelectedEndTarget::NonCurrent => {
                            Effect::RecoverInterveningSelectedOrdinaryElementsAndCloseTarget(*name)
                        }
                    }),
                    next: None,
                }),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Body),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::RecordDuplicateBodyStartTag),
                    next: None,
                }),
                AdmittedToken::StartTag {
                    name:
                        AdmittedElementName::Shell(
                            HtmlShellElementName::Head | HtmlShellElementName::Html,
                        ),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellStartTagPosition),
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Body),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::AcknowledgeShellEndTag(HtmlShellElementName::Body)),
                    next: Some(InsertionMode::AfterBody),
                }),
                AdmittedToken::EndTag {
                    name:
                        AdmittedElementName::Shell(
                            HtmlShellElementName::Head | HtmlShellElementName::Html,
                        ),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
                // End of file with a selected ordinary element still open is
                // one diagnostic plus the ordinary stop: nothing is popped, no
                // close is synthesized, and no end-tag anchor or closure
                // evidence is fabricated for a token that has no authored
                // extent.
                AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop {
                    effect: (!open_selected_ordinary.is_empty())
                        .then_some(Effect::RecordOpenSelectedOrdinaryElementAtEndOfFile),
                }),
            }
        }
        // Reached by the proved `</body>` cell. End of file here stops the
        // bounded document parse, which is what the accepted G4, G6, and G7
        // cases require. TC-S2 additionally proves the uniform aggregate
        // character run: an all-whitespace run delegates to the selected
        // `in body` text step without changing the actual mode; an
        // all-non-whitespace run records one diagnostic and reprocesses into
        // `InBody`; a mixed run is refused whole, before any mutation. The
        // other `after body` rules the HTML Standard defines (comments,
        // reprocessing other token shapes back in `in body`) remain
        // unproved and stay refused.
        InsertionMode::AfterBody => match token {
            AdmittedToken::EndTag {
                name: AdmittedElementName::Shell(HtmlShellElementName::Html),
                ..
            } => Ok(ModeStep::Consume {
                effect: Some(Effect::AcknowledgeShellEndTag(HtmlShellElementName::Html)),
                next: Some(InsertionMode::AfterAfterBody),
            }),
            AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop { effect: None }),
            AdmittedToken::Characters { interpreted, .. } => {
                match classify_character_run(interpreted) {
                    CharacterRunClass::AllHtmlWhitespace => Ok(selected_in_body_character_step()),
                    CharacterRunClass::AllNonHtmlWhitespace => Ok(ModeStep::Reprocess {
                        effect: Some(Effect::RecordAfterBodyCharacterData),
                        next: InsertionMode::InBody,
                    }),
                    CharacterRunClass::Mixed => {
                        Err(HtmlTreeCapability::WhitespaceSensitiveCharacterData)
                    }
                }
            }
            AdmittedToken::StartTag { .. } => {
                Err(HtmlTreeCapability::UnprovedShellStartTagPosition)
            }
            AdmittedToken::EndTag { .. } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
        },
        InsertionMode::AfterAfterBody => match token {
            AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop { effect: None }),
            AdmittedToken::Characters { .. } => {
                Err(HtmlTreeCapability::UnprovedCharacterDataPosition)
            }
            AdmittedToken::StartTag { .. } => {
                Err(HtmlTreeCapability::UnprovedShellStartTagPosition)
            }
            AdmittedToken::EndTag { .. } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
        },
    }
}

/// Where a supported selected ordinary end tag's nearest same-name target
/// lies, relative to the current node.
///
/// This is a read-only classification of the ordered selected-name projection
/// alone. It carries no constructed identity: which identities the resolved
/// cell then acts on is the session's own transaction to resolve, before it
/// mutates anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedEndTarget {
    /// No selected ordinary element of that name is open.
    Absent,
    /// The nearest same-name selected ordinary element is the current node.
    Current,
    /// The nearest same-name selected ordinary element is open below at least
    /// one other selected ordinary element.
    NonCurrent,
}

/// Classifies a supported selected ordinary end tag against the ordered
/// selected-name projection, innermost last.
///
/// The bounded represented state is `[html, body] ++ W` with
/// `W ∈ {Div, Section}*`, so the whole selected slice sits above `body` and a
/// reverse scan of `W` is exactly the bounded selected-element scope walk.
/// This is deliberately not general WHATWG scope coverage and claims none.
fn selected_end_target(
    open_selected_ordinary: &[HtmlSelectedOrdinaryElementName],
    name: HtmlSelectedOrdinaryElementName,
) -> SelectedEndTarget {
    match open_selected_ordinary
        .iter()
        .rposition(|open| *open == name)
    {
        None => SelectedEndTarget::Absent,
        Some(position) if position + 1 == open_selected_ordinary.len() => {
            SelectedEndTarget::Current
        }
        Some(_) => SelectedEndTarget::NonCurrent,
    }
}

/// Whether an element in the represented state would be popped by the HTML
/// Standard's "generate implied end tags" step.
///
/// Written as a total function over the closed represented element domain
/// rather than as a generalized implied-end generator. Neither the shell
/// elements nor the selected ordinary `div` and `section` is an implied-end
/// element, so the step is a no-op over every state this subsystem can reach —
/// which is exactly the bounded invariant the accepted end-tag branches rely
/// on. The selected arm matches each closed name rather than the domain as a
/// whole, so growing that domain again forces this question to be answered for
/// the new member instead of being silently inherited. `p`, which *is* an
/// implied-end element, is outside both closed domains and stays unsupported.
fn is_implied_end_element(element: &HtmlElement) -> bool {
    match element {
        HtmlElement::Shell(_) => false,
        HtmlElement::SelectedOrdinary(selected) => match selected.name() {
            HtmlSelectedOrdinaryElementName::Div | HtmlSelectedOrdinaryElementName::Section => {
                false
            }
        },
    }
}

/// Refuses a character run whose handling in the current mode would depend on
/// the whitespace/non-whitespace distinction TC-S1 does not prove.
fn reject_whitespace_sensitive_characters(
    token: &AdmittedToken<'_>,
) -> Result<(), HtmlTreeCapability> {
    match token {
        AdmittedToken::Characters { interpreted, .. } if contains_html_whitespace(interpreted) => {
            Err(HtmlTreeCapability::WhitespaceSensitiveCharacterData)
        }
        _ => Ok(()),
    }
}

/// What processing one admitted token concluded, once [`super::driver`] has
/// driven it to a terminal disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenOutcome {
    /// The token was completely processed by supported actions.
    Consumed,
    /// The token stopped document parsing normally.
    StoppedParsing,
    /// This subsystem does not prove this cell. Nothing was mutated for it.
    Unsupported(HtmlTreeCapability),
}

/// What one [`HtmlTreeSession::dispatch`] call — exactly one insertion-mode
/// rule evaluation — concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DispatchOutcome {
    /// The token was completely processed by this rule.
    Consumed,
    /// The rule reprocessed the same token into a new actual insertion mode.
    /// The session committed nothing for the token: no coverage, no
    /// processed-token count. [`super::driver`] redispatches the same
    /// admitted token and trigger without re-admitting or reconstructing
    /// either.
    ReprocessSameToken,
    /// The token stopped document parsing normally.
    StoppedParsing,
    /// This subsystem does not prove this cell. Nothing was mutated for it.
    Unsupported(HtmlTreeCapability),
}

/// A construction-session invariant failure.
///
/// This is an operation/boundary error, never an HTML parse diagnostic and
/// never unsupported input. Every variant carries only structural evidence;
/// `Debug` and `Display` never expose authored source content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeSessionError {
    /// The committed semantic creation-event counter is exhausted.
    ConstructedIdentityExhausted,
    /// A shell element insertion found no recorded insertion parent.
    MissingInsertionParent,
    /// An authored insertion was requested for a token that is not a start
    /// tag.
    AuthoredInsertionWithoutStartTag,
    /// A shell element name was opened while already open.
    DuplicateOpenShellElement(HtmlShellElementName),
    /// A shell element was closed while it was not the open element.
    ClosedShellElementIsNotOpen(HtmlShellElementName),
    /// A relationship named a node the session does not hold.
    UnknownConstructedNode(HtmlConstructedNodeId),
    /// Character insertion found no open insertion target.
    MissingCharacterInsertionTarget,
    /// Character insertion found a coalescing target that is not a text node.
    InvalidCharacterCoalescingTarget(HtmlConstructedNodeId),
    /// While processing one emitted token, the driver observed the same
    /// insertion mode evaluated twice. This is the structural per-token
    /// termination proof: it is never expected to fire for a proved cell and
    /// carries no numeric budget.
    RepeatedInsertionModeEvaluation,
    /// Committed tree coverage moved backwards.
    NonMonotonicCommittedCoverage,
    /// A selected ordinary close was requested while no element of that name
    /// was in the bounded selected-element scope.
    SelectedOrdinaryElementIsNotInScope(HtmlSelectedOrdinaryElementName),
    /// A selected ordinary close was requested while the current node was not
    /// that element.
    SelectedOrdinaryElementIsNotCurrent(HtmlSelectedOrdinaryElementName),
    /// Implied-end-tag generation would not have been a no-op over the
    /// represented state. This is the bounded invariant that stands in for a
    /// generalized implied-end generator; it is never expected to fire while
    /// the represented state is `[html, body] ++ W` with
    /// `W ∈ {Div, Section}*`.
    ImpliedEndGenerationIsNotABoundedNoOp,
    /// A heterogeneous selected ordinary recovery was requested while the
    /// nearest same-name target was in fact the current node, which is the
    /// matching-closure cell rather than the recovery cell.
    SelectedOrdinaryRecoveryTargetIsCurrent(HtmlSelectedOrdinaryElementName),
    /// The open elements above a resolved selected ordinary target are not all
    /// selected ordinary elements, so the bounded suffix the recovery cell
    /// requires does not exist. It is never expected to fire while the
    /// represented state is `[html, body] ++ W`.
    SelectedOrdinaryRecoverySuffixIsNotSelected(HtmlConstructedNodeId),
    /// An unmatched selected ordinary end tag was recorded while an element of
    /// that name was in fact in the bounded selected-element scope.
    UnmatchedSelectedOrdinaryEndTagWithElementInScope(HtmlSelectedOrdinaryElementName),
    /// The open-selected-element end-of-file diagnostic was recorded while no
    /// selected ordinary element was open.
    NoOpenSelectedOrdinaryElementAtEndOfFile,
}

impl std::fmt::Display for HtmlTreeSessionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "HTML document shell session invariant violation: {self:?}"
        )
    }
}

impl std::error::Error for HtmlTreeSessionError {}

/// One supported selected ordinary end tag's complete pre-resolved decision.
///
/// Built entirely from read-only session state before the transaction
/// mutates anything, and dropped when the transaction ends. It is never
/// stored on the session, never reaches the freeze boundary, and never
/// escapes to a consumer.
///
/// `target_open_element` is a private, ephemeral open-element stack
/// coordinate that exists only so the commit can be one `truncate`. It is
/// deliberately *not* semantic identity, is never recorded in any action,
/// diagnostic, or node, and is meaningless the moment the stack changes.
/// Only `target` and `intervening_current_first` carry durable meaning, and
/// both are [`HtmlConstructedNodeId`] semantic creation-event identities.
struct SelectedOrdinaryEndPlan {
    /// The nearest same-name open selected ordinary element the end tag
    /// selects. This one is closed by its own matching end tag.
    target: HtmlConstructedNodeId,
    /// Where the target sits on the private open-element stack.
    target_open_element: usize,
    /// The complete open selected ordinary suffix above the target, ordered
    /// current node first and target-ward last. Empty exactly when the target
    /// is already the current node.
    intervening_current_first: Vec<HtmlConstructedNodeId>,
}

/// The exclusive owner of one run's mutable tree-construction state.
pub(super) struct HtmlTreeSession {
    identities: HtmlConstructedIdentityCounter,
    nodes: Vec<HtmlTreeNode>,
    root: HtmlConstructedNodeId,
    open_elements: Vec<HtmlConstructedNodeId>,
    head_element: Option<HtmlConstructedNodeId>,
    mode: InsertionMode,
    document_mode: HtmlDocumentMode,
    frameset_ok: bool,
    diagnostics: Vec<HtmlTreeDiagnostic>,
    actions: Vec<HtmlTreeAction>,
    processed_tokens: usize,
    committed_prefix_end: usize,
}

impl HtmlTreeSession {
    /// Starts a run by committing the Document root creation event.
    ///
    /// The root has no authored source and no synthesis cause: it is the
    /// parse result's container, not implied markup.
    pub(super) fn new() -> Result<Self, HtmlTreeSessionError> {
        let mut identities = HtmlConstructedIdentityCounter::new();
        let root = identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        let document = HtmlTreeNode::new(root, None, Vec::new(), HtmlTreeNodeKind::Document);
        identities.commit(root);
        Ok(Self {
            identities,
            nodes: vec![document],
            root,
            open_elements: Vec::new(),
            head_element: None,
            mode: InsertionMode::Initial,
            document_mode: HtmlDocumentMode::NoQuirks,
            frameset_ok: true,
            diagnostics: Vec::new(),
            actions: Vec::new(),
            processed_tokens: 0,
            committed_prefix_end: 0,
        })
    }

    /// Performs exactly one insertion-mode rule evaluation for the given
    /// admitted token.
    ///
    /// This is the session's complete unit of work per call: it never loops
    /// and never itself redispatches. [`super::driver`] owns same-token
    /// redispatch orchestration and calls this once per dispatch, retaining
    /// the same admitted token and trigger across a
    /// [`DispatchOutcome::ReprocessSameToken`] result.
    pub(super) fn dispatch(
        &mut self,
        token: &AdmittedToken<'_>,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<DispatchOutcome, HtmlTreeSessionError> {
        // Both gates run before anything can change: the token was already
        // admitted lexically, and this reads the private selected-element
        // state without borrowing it mutably.
        let open_selected_ordinary = self.open_selected_ordinary_names();
        let step = match classify(self.mode, &open_selected_ordinary, token) {
            Ok(step) => step,
            Err(capability) => return Ok(DispatchOutcome::Unsupported(capability)),
        };
        match step {
            ModeStep::Stop { effect } => {
                if let Some(effect) = effect {
                    self.apply(effect, trigger, token)?;
                }
                self.record_action(HtmlTreeActionKind::StoppedParsing, trigger);
                self.commit_token(token)?;
                Ok(DispatchOutcome::StoppedParsing)
            }
            ModeStep::Consume { effect, next } => {
                if let Some(effect) = effect {
                    self.apply(effect, trigger, token)?;
                }
                if let Some(next) = next {
                    self.switch_mode(next);
                }
                self.commit_token(token)?;
                Ok(DispatchOutcome::Consumed)
            }
            ModeStep::Reprocess { effect, next } => {
                if let Some(effect) = effect {
                    self.apply(effect, trigger, token)?;
                }
                self.switch_mode(next);
                self.record_action(HtmlTreeActionKind::ReprocessedToken, trigger);
                Ok(DispatchOutcome::ReprocessSameToken)
            }
        }
    }

    /// Consumes the session into the freeze boundary's input.
    ///
    /// The session itself never escapes, and nothing mutable travels in the
    /// returned parts.
    pub(super) fn finish(self, completion: HtmlTreeCompletion) -> HtmlDocumentShellParts {
        // Snapshot the actual final open selected state before the mutable
        // stack is dropped, so the freeze boundary can validate the committed
        // action stream against it instead of trusting this session. Only the
        // semantic identities travel; the stack itself does not.
        let final_open_selected_ordinary = self.open_selected_ordinary_ids();
        HtmlDocumentShellParts {
            nodes: self.nodes,
            root: self.root,
            admitted_creation_events: self.identities.admitted(),
            diagnostics: self.diagnostics,
            actions: self.actions,
            processed_tokens: self.processed_tokens,
            committed_prefix_end: self.committed_prefix_end,
            completion,
            final_open_selected_ordinary,
        }
    }

    fn apply(
        &mut self,
        effect: Effect,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        match effect {
            Effect::RecordMissingDoctype => {
                self.document_mode = HtmlDocumentMode::Quirks;
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::MissingDoctype,
                    trigger,
                    HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
                );
                Ok(())
            }
            Effect::InsertHtmlElement(provenance) => {
                self.insert_shell_element(HtmlShellElementName::Html, provenance, trigger, token)?;
                Ok(())
            }
            Effect::InsertHeadElement(provenance) => {
                let head = self.insert_shell_element(
                    HtmlShellElementName::Head,
                    provenance,
                    trigger,
                    token,
                )?;
                self.head_element = Some(head);
                Ok(())
            }
            Effect::InsertBodyElement(provenance) => {
                self.insert_shell_element(HtmlShellElementName::Body, provenance, trigger, token)?;
                self.frameset_ok = false;
                Ok(())
            }
            Effect::InsertCharacters => self.insert_characters(trigger, token),
            Effect::RecordDuplicateHeadStartTag => {
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::DuplicateHeadStartTag,
                    trigger,
                    HtmlTreeRecovery::DuplicateShellStartTagProducedNoNode,
                );
                self.record_action(
                    HtmlTreeActionKind::DuplicateShellStartTagCreatedNoNode {
                        name: HtmlShellElementName::Head,
                    },
                    trigger,
                );
                Ok(())
            }
            Effect::RecordDuplicateBodyStartTag => {
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::DuplicateBodyStartTag,
                    trigger,
                    HtmlTreeRecovery::DuplicateShellStartTagProducedNoNode,
                );
                self.record_action(
                    HtmlTreeActionKind::DuplicateShellStartTagCreatedNoNode {
                        name: HtmlShellElementName::Body,
                    },
                    trigger,
                );
                self.frameset_ok = false;
                Ok(())
            }
            Effect::CloseHeadElement(closure) => self.close_head_element(closure, trigger),
            Effect::AcknowledgeShellEndTag(name) => {
                self.record_action(
                    HtmlTreeActionKind::AcknowledgedShellEndTag { name },
                    trigger,
                );
                Ok(())
            }
            Effect::RecordAfterBodyCharacterData => {
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::AfterBodyCharacterData,
                    trigger,
                    HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken,
                );
                Ok(())
            }
            Effect::InsertSelectedOrdinaryElement(name) => {
                self.insert_selected_ordinary_element(name, trigger, token)
            }
            Effect::CloseSelectedOrdinaryElement(name) => {
                self.close_selected_ordinary_element(name, trigger)
            }
            Effect::RecoverInterveningSelectedOrdinaryElementsAndCloseTarget(name) => {
                self.recover_selected_ordinary_suffix_and_close_target(name, trigger)
            }
            Effect::RecordUnmatchedSelectedOrdinaryEndTag(name) => {
                // The bounded not-in-scope condition is what makes this the
                // ignored-end cell rather than the closing cell, so it is
                // proved before either the diagnostic or the disposition is
                // recorded.
                if self.nearest_open_selected_ordinary(name).is_some() {
                    return Err(
                        HtmlTreeSessionError::UnmatchedSelectedOrdinaryEndTagWithElementInScope(
                            name,
                        ),
                    );
                }
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag,
                    trigger,
                    HtmlTreeRecovery::IgnoredToken,
                );
                self.record_action(
                    HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name },
                    trigger,
                );
                Ok(())
            }
            Effect::RecordOpenSelectedOrdinaryElementAtEndOfFile => {
                if self.open_selected_ordinary_ids().is_empty() {
                    return Err(HtmlTreeSessionError::NoOpenSelectedOrdinaryElementAtEndOfFile);
                }
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile,
                    trigger,
                    HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements,
                );
                Ok(())
            }
        }
    }

    /// Creates one shell element node.
    ///
    /// Every fallible step — insertion parent resolution, open-element
    /// uniqueness, authored evidence availability, and identity headroom — is
    /// resolved before the first mutation, and the creation counter advances
    /// only after the whole action has committed.
    fn insert_shell_element(
        &mut self,
        name: HtmlShellElementName,
        provenance: ElementProvenance,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<HtmlConstructedNodeId, HtmlTreeSessionError> {
        let parent =
            match name {
                HtmlShellElementName::Html => self.root,
                HtmlShellElementName::Head | HtmlShellElementName::Body => *self
                    .open_elements
                    .last()
                    .ok_or(HtmlTreeSessionError::MissingInsertionParent)?,
            };
        if self.node(parent).is_none() {
            return Err(HtmlTreeSessionError::UnknownConstructedNode(parent));
        }
        if self.is_open(name) {
            return Err(HtmlTreeSessionError::DuplicateOpenShellElement(name));
        }
        let origin = match provenance {
            ElementProvenance::AuthoredByTriggerToken => {
                let AdmittedToken::StartTag {
                    complete, raw_name, ..
                } = token
                else {
                    return Err(HtmlTreeSessionError::AuthoredInsertionWithoutStartTag);
                };
                HtmlShellElementOrigin::Authored {
                    complete: (*complete).clone(),
                    raw_name: (*raw_name).clone(),
                }
            }
            ElementProvenance::Synthesized => {
                HtmlShellElementOrigin::Synthesized(HtmlSynthesisCause::ImpliedByDocumentStructure)
            }
        };
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;

        let node = HtmlTreeNode::new(
            reserved,
            Some(parent),
            Vec::new(),
            HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(name, origin))),
        );
        self.node_mut(parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?
            .push_child(reserved);
        self.nodes.push(node);
        self.open_elements.push(reserved);
        self.identities.commit(reserved);

        let kind = match provenance {
            ElementProvenance::AuthoredByTriggerToken => {
                HtmlTreeActionKind::InsertedAuthoredShellElement {
                    node: reserved,
                    name,
                }
            }
            ElementProvenance::Synthesized => HtmlTreeActionKind::InsertedSynthesizedShellElement {
                node: reserved,
                name,
                cause: HtmlSynthesisCause::ImpliedByDocumentStructure,
            },
        };
        self.record_action(kind, trigger);
        Ok(reserved)
    }

    /// Creates one selected ordinary element node from the trigger token's
    /// own authored start tag.
    ///
    /// Every fallible step — start-tag evidence availability, insertion parent
    /// resolution, and identity headroom — is resolved before the first
    /// mutation, and the creation counter advances only after the whole action
    /// has committed. Nesting is deliberately not restricted here: `S(k)`
    /// admits any `k`, and no numeric nesting cap is introduced.
    fn insert_selected_ordinary_element(
        &mut self,
        name: HtmlSelectedOrdinaryElementName,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        let AdmittedToken::StartTag {
            complete, raw_name, ..
        } = token
        else {
            return Err(HtmlTreeSessionError::AuthoredInsertionWithoutStartTag);
        };
        let parent = *self
            .open_elements
            .last()
            .ok_or(HtmlTreeSessionError::MissingInsertionParent)?;
        if self.node(parent).is_none() {
            return Err(HtmlTreeSessionError::UnknownConstructedNode(parent));
        }
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;

        let element =
            HtmlSelectedOrdinaryElement::new(name, (*complete).clone(), (*raw_name).clone());
        let node = HtmlTreeNode::new(
            reserved,
            Some(parent),
            Vec::new(),
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(element)),
        );
        self.node_mut(parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?
            .push_child(reserved);
        self.nodes.push(node);
        self.open_elements.push(reserved);
        self.identities.commit(reserved);

        self.record_action(
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                node: reserved,
                name,
            },
            trigger,
        );
        Ok(())
    }

    /// Closes the nearest same-name open selected ordinary element for its own
    /// exact authored end tag, when that element is already the current node.
    ///
    /// The accepted TC-S3 branch is proved in order and entirely before the
    /// pop, and stays exactly what it was: the bounded
    /// selected-element-in-scope condition and the bounded implied-end no-op
    /// invariant are resolved by [`Self::plan_selected_ordinary_end`], and
    /// this cell additionally requires the resolved target to be current — a
    /// non-empty intervening suffix here is the heterogeneous recovery cell,
    /// not this one. Only then is exactly that identity popped and the closure
    /// recorded, relating the semantic node identity to the trigger's exact
    /// authored end tag. No identity is admitted, no recovery relation is
    /// recorded, no misnested diagnostic is emitted, and the end tag never
    /// becomes the node's origin.
    fn close_selected_ordinary_element(
        &mut self,
        name: HtmlSelectedOrdinaryElementName,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let plan = self.plan_selected_ordinary_end(name)?;
        if !plan.intervening_current_first.is_empty() {
            return Err(HtmlTreeSessionError::SelectedOrdinaryElementIsNotCurrent(
                name,
            ));
        }
        self.open_elements.truncate(plan.target_open_element);
        self.record_action(
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
                node: plan.target,
                name,
            },
            trigger,
        );
        Ok(())
    }

    /// Recovers a heterogeneous open selected ordinary suffix and closes the
    /// nearest same-name target, all for one exact authored end tag.
    ///
    /// The complete decision — nearest same-name target, the complete
    /// intervening suffix in current-first order, the bounded implied-end
    /// no-op invariant, and the requirement that the target is not already
    /// current — is resolved by [`Self::plan_selected_ordinary_end`] before
    /// anything mutates. The commit is then one coherent stack mutation
    /// followed by the ordered evidence, so there is no partial pop, no
    /// rollback, and no state to reconstruct after a refusal.
    ///
    /// The two relations stay distinct on purpose. Each intervening element
    /// gets one recovery-pop relation naming the target that caused it, and
    /// never a fabricated matching closure: no matching end tag caused its
    /// removal. A later authored end tag of that element's own name may still
    /// appear in the source, but the element has already left the open state,
    /// so that end tag is unmatched and closes nothing. The target alone gets
    /// the matching closure. All of them, and the single misnested diagnostic,
    /// carry the same exact authored end tag as trigger evidence, and it is
    /// the authored origin of none of them. No identity is admitted.
    fn recover_selected_ordinary_suffix_and_close_target(
        &mut self,
        name: HtmlSelectedOrdinaryElementName,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let plan = self.plan_selected_ordinary_end(name)?;
        if plan.intervening_current_first.is_empty() {
            return Err(HtmlTreeSessionError::SelectedOrdinaryRecoveryTargetIsCurrent(name));
        }

        self.open_elements.truncate(plan.target_open_element);
        self.record_diagnostic(
            HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag,
            trigger,
            HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
        );
        for node in plan.intervening_current_first {
            self.record_action(
                HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag {
                    node,
                    target: plan.target,
                },
                trigger,
            );
        }
        self.record_action(
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
                node: plan.target,
                name,
            },
            trigger,
        );
        Ok(())
    }

    /// Resolves the complete meaning of a supported selected ordinary end tag
    /// before any mutation.
    ///
    /// Read-only by construction: it borrows `&self`, so nothing it inspects
    /// can already have been changed by the decision it is making. Every
    /// fallible and precondition-sensitive part of the transaction lives here
    /// — nearest same-name target lookup, the bounded implied-end no-op
    /// invariant, and proof that the complete suffix above the target really
    /// is the bounded selected suffix the accepted cells describe. The caller
    /// then classifies the plan and commits in one step.
    fn plan_selected_ordinary_end(
        &self,
        name: HtmlSelectedOrdinaryElementName,
    ) -> Result<SelectedOrdinaryEndPlan, HtmlTreeSessionError> {
        let target_open_element = self.nearest_open_selected_ordinary(name).ok_or(
            HtmlTreeSessionError::SelectedOrdinaryElementIsNotInScope(name),
        )?;
        if !self.implied_end_generation_is_a_no_op() {
            return Err(HtmlTreeSessionError::ImpliedEndGenerationIsNotABoundedNoOp);
        }
        let mut intervening_current_first = Vec::new();
        for id in self.open_elements[target_open_element + 1..].iter().rev() {
            if !self.is_open_selected_ordinary(*id) {
                return Err(HtmlTreeSessionError::SelectedOrdinaryRecoverySuffixIsNotSelected(*id));
            }
            intervening_current_first.push(*id);
        }
        Ok(SelectedOrdinaryEndPlan {
            target: self.open_elements[target_open_element],
            target_open_element,
            intervening_current_first,
        })
    }

    /// Inserts the trigger token's characters at the current insertion
    /// position, coalescing into the adjacent text node when one is already
    /// the last child there.
    fn insert_characters(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        let AdmittedToken::Characters {
            source,
            interpreted,
        } = token
        else {
            return Err(HtmlTreeSessionError::MissingCharacterInsertionTarget);
        };
        let parent = *self
            .open_elements
            .last()
            .ok_or(HtmlTreeSessionError::MissingCharacterInsertionTarget)?;
        let parent_node = self
            .node(parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?;
        let adjacent_text = parent_node
            .children()
            .last()
            .copied()
            .filter(|child| self.is_text(*child));
        let contribution = HtmlTextContribution::new((*source).clone(), (*interpreted).to_owned());

        if let Some(text_id) = adjacent_text {
            self.node_mut(text_id)
                .ok_or(HtmlTreeSessionError::UnknownConstructedNode(text_id))?
                .text_mut()
                .ok_or(HtmlTreeSessionError::InvalidCharacterCoalescingTarget(
                    text_id,
                ))?
                .append(contribution);
            self.record_action(
                HtmlTreeActionKind::AppendedToTextNode { node: text_id },
                trigger,
            );
            return Ok(());
        }

        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        let text = HtmlTextNode::new((*interpreted).to_owned(), vec![contribution]);
        let node = HtmlTreeNode::new(
            reserved,
            Some(parent),
            Vec::new(),
            HtmlTreeNodeKind::Text(text),
        );
        self.node_mut(parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?
            .push_child(reserved);
        self.nodes.push(node);
        self.identities.commit(reserved);
        self.record_action(
            HtmlTreeActionKind::InsertedTextNode { node: reserved },
            trigger,
        );
        Ok(())
    }

    fn close_head_element(
        &mut self,
        closure: HtmlShellClosure,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let head = self
            .head_element
            .ok_or(HtmlTreeSessionError::ClosedShellElementIsNotOpen(
                HtmlShellElementName::Head,
            ))?;
        if self.open_elements.last() != Some(&head) {
            return Err(HtmlTreeSessionError::ClosedShellElementIsNotOpen(
                HtmlShellElementName::Head,
            ));
        }
        self.open_elements.pop();
        self.record_action(
            HtmlTreeActionKind::ClosedShellElement {
                node: head,
                name: HtmlShellElementName::Head,
                closure,
            },
            trigger,
        );
        Ok(())
    }

    /// Moves the actual insertion mode to `next`.
    ///
    /// This subsystem's per-token termination proof lives in
    /// [`super::driver`], which tracks already-evaluated modes across
    /// [`Self::dispatch`] calls for one token; the session itself no longer
    /// restricts which mode a proved rule may select next, which is what
    /// admits TC-S2's validated `AfterBody -> InBody` recovery back-edge.
    fn switch_mode(&mut self, next: InsertionMode) {
        self.mode = next;
    }

    fn commit_token(&mut self, token: &AdmittedToken<'_>) -> Result<(), HtmlTreeSessionError> {
        let end = token.committed_end();
        if end < self.committed_prefix_end {
            return Err(HtmlTreeSessionError::NonMonotonicCommittedCoverage);
        }
        self.committed_prefix_end = end;
        self.processed_tokens += 1;
        Ok(())
    }

    fn record_action(&mut self, kind: HtmlTreeActionKind, trigger: &HtmlTreeTokenTrigger) {
        self.actions
            .push(HtmlTreeAction::new(kind, trigger.clone()));
    }

    fn record_diagnostic(
        &mut self,
        code: HtmlTreeDiagnosticCode,
        trigger: &HtmlTreeTokenTrigger,
        recovery: HtmlTreeRecovery,
    ) {
        self.diagnostics
            .push(HtmlTreeDiagnostic::new(code, trigger.clone(), recovery));
    }

    /// Resolves a constructed identity by searching, never by indexing
    /// storage.
    fn node(&self, id: HtmlConstructedNodeId) -> Option<&HtmlTreeNode> {
        self.nodes.iter().find(|node| node.id() == id)
    }

    fn node_mut(&mut self, id: HtmlConstructedNodeId) -> Option<&mut HtmlTreeNode> {
        self.nodes.iter_mut().find(|node| node.id() == id)
    }

    fn is_text(&self, id: HtmlConstructedNodeId) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Text(_))
        )
    }

    /// Whether a shell element name is currently open.
    ///
    /// Together with the mode machine this bounds the shell part of the
    /// open-element state without any depth limit: a shell name can be pushed
    /// only while it is not already open. Selected ordinary elements are a
    /// separate domain and are deliberately not counted here.
    fn is_open(&self, name: HtmlShellElementName) -> bool {
        self.open_elements.iter().any(|id| {
            matches!(
                self.node(*id).map(HtmlTreeNode::kind),
                Some(HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)))
                    if shell.name() == name
            )
        })
    }

    /// Whether an open-element entry is the selected ordinary element of this
    /// name, resolved by semantic constructed identity.
    fn is_selected_ordinary(
        &self,
        id: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    ) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)))
                if selected.name() == name
        )
    }

    /// The semantic identities of the currently open selected ordinary
    /// elements, in open-element stack order with the innermost last.
    ///
    /// Semantic identities only: no storage position, and no borrow of the
    /// private stack itself, escapes through this.
    fn open_selected_ordinary_ids(&self) -> Vec<HtmlConstructedNodeId> {
        self.open_elements
            .iter()
            .copied()
            .filter(|id| {
                matches!(
                    self.node(*id).map(HtmlTreeNode::kind),
                    Some(HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(_)))
                )
            })
            .collect()
    }

    /// Whether an open-element entry is a selected ordinary element at all.
    fn is_open_selected_ordinary(&self, id: HtmlConstructedNodeId) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(_)))
        )
    }

    /// The closed selected names of the currently open selected ordinary
    /// elements, in open-element stack order with the innermost last.
    ///
    /// This is the `W` of the represented state `[html, body] ++ W`, and it is
    /// the only construction state the read-only [`classify`] gate needs. It
    /// carries no constructed identity, so rule selection cannot come to
    /// depend on a relationship the transaction has not resolved yet. It is
    /// read-only construction state, not a budget: nothing compares its length
    /// against a maximum.
    fn open_selected_ordinary_names(&self) -> Vec<HtmlSelectedOrdinaryElementName> {
        self.open_elements
            .iter()
            .filter_map(|id| match self.node(*id).map(HtmlTreeNode::kind) {
                Some(HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected))) => {
                    Some(selected.name())
                }
                _ => None,
            })
            .collect()
    }

    /// The nearest same-name open selected ordinary element, as a private
    /// open-element stack position.
    ///
    /// Walks the private open-element stack by semantic constructed identity,
    /// from the current node outwards, and stops at the represented HTML
    /// boundary. This is deliberately **not** general WHATWG scope coverage
    /// and claims none: the represented state contains only `html`, `body`,
    /// and selected ordinary elements, so `html` is the only boundary element
    /// that can occur in it.
    ///
    /// The returned position is ephemeral and private. It is a coordinate for
    /// the commit that immediately follows, never semantic identity, and it is
    /// never recorded anywhere durable.
    fn nearest_open_selected_ordinary(
        &self,
        name: HtmlSelectedOrdinaryElementName,
    ) -> Option<usize> {
        for (position, id) in self.open_elements.iter().enumerate().rev() {
            match self.node(*id).map(HtmlTreeNode::kind) {
                Some(HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)))
                    if selected.name() == name =>
                {
                    return Some(position);
                }
                Some(HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)))
                    if shell.name() == HtmlShellElementName::Html =>
                {
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    /// Whether the HTML Standard's implied-end-tag generation would pop
    /// nothing over the current represented state.
    ///
    /// No generalized implied-end generator is introduced. Instead
    /// [`is_implied_end_element`] is a total function over the closed
    /// represented element domain, so this invariant is validated rather than
    /// assumed — and growing that domain forces the question to be answered
    /// again rather than silently skipped.
    fn implied_end_generation_is_a_no_op(&self) -> bool {
        self.open_elements.iter().all(|id| {
            !matches!(
                self.node(*id).map(HtmlTreeNode::kind),
                Some(HtmlTreeNodeKind::Element(element)) if is_implied_end_element(element)
            )
        })
    }

    /// The actual insertion mode.
    ///
    /// [`super::driver`] reads this to build and check its per-token
    /// evaluated-mode history; it is also used by tests.
    pub(super) fn insertion_mode(&self) -> InsertionMode {
        self.mode
    }

    #[cfg(test)]
    pub(super) fn document_mode(&self) -> HtmlDocumentMode {
        self.document_mode
    }

    #[cfg(test)]
    pub(super) fn frameset_ok(&self) -> bool {
        self.frameset_ok
    }

    #[cfg(test)]
    pub(super) fn open_element_count(&self) -> usize {
        self.open_elements.len()
    }

    #[cfg(test)]
    pub(super) fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
