//! The private TC-S1 tree-construction session.
//!
//! The session exclusively owns every piece of mutable tree-construction
//! state for exactly one run: the insertion mode, the open shell elements,
//! the head pointer, the private document mode and `frameset-ok` flag, the
//! constructed identity counter, the temporary node storage, and the
//! committed diagnostics, actions, and coverage.
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
//! # Refuse before mutate
//!
//! Rule selection and mutation are separated on purpose. [`classify`] is a
//! free function of the insertion mode and the admitted token only: it takes
//! no `&self`, performs no mutation, and returns either the step to apply or
//! the [`HtmlTreeCapability`] that TC-S1 does not prove. An unsupported cell
//! therefore cannot mutate anything, and the session is a valid semantic
//! construction checkpoint at every instant — no rollback, snapshot, or
//! generic checkpoint machinery is needed to freeze the last valid state.
//!
//! # Termination without a work limit
//!
//! TC-S1 introduces no tree resource dimension, limit, or work constant.
//! Per-token termination is structural: reprocessing is only ever expressed
//! as a transition to a *strictly later* insertion mode in the total order of
//! [`InsertionMode`], and [`HtmlTreeSession::switch_mode`] rejects any
//! transition that is not strictly forward. A finite strictly increasing walk
//! over a finite ordered enum cannot loop, and TC-S1 contains no recursion.

use crate::SourceAnchor;

use super::super::token::{HtmlTagKind, HtmlToken};
use super::result::{
    HtmlConstructedIdentityCounter, HtmlConstructedNodeId, HtmlDocumentShellParts,
    HtmlShellClosure, HtmlShellElement, HtmlShellElementName, HtmlShellElementOrigin,
    HtmlSynthesisCause, HtmlTextContribution, HtmlTextNode, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode,
    HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery, HtmlTreeTokenTrigger,
};

/// The TC-S1 insertion modes, in the total order document construction walks.
///
/// The order is load-bearing: every TC-S1 mode transition, whether it consumes
/// the token or reprocesses it, moves strictly forward, which is what bounds
/// per-token work without any numeric limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// A validated emitted token, normalized to the shapes TC-S1 admits.
///
/// Admission is a property of the token alone, so it is decided by [`admit`]
/// before the token reaches any insertion mode. A tag outside the proved
/// `html`/`head`/`body` shell, a tag carrying attributes, and a self-closing
/// tag are all refused here, before any mutation can happen.
#[derive(Debug, Clone, Copy)]
pub(super) enum AdmittedToken<'run> {
    Characters {
        source: &'run SourceAnchor,
        interpreted: &'run str,
    },
    StartTag {
        name: HtmlShellElementName,
        complete: &'run SourceAnchor,
        raw_name: &'run SourceAnchor,
    },
    EndTag {
        name: HtmlShellElementName,
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

/// Normalizes one validated emitted token into the TC-S1 admitted shapes.
///
/// Pure and mutation-free. Refuses, with exact typed capability evidence,
/// every token shape TC-S1 does not prove.
pub(super) fn admit(token: &HtmlToken) -> Result<AdmittedToken<'_>, HtmlTreeCapability> {
    match token {
        HtmlToken::Character(character) => Ok(AdmittedToken::Characters {
            source: character.source(),
            interpreted: character.interpreted(),
        }),
        HtmlToken::Tag(tag) => {
            let Some(name) = shell_element_name(tag.name().interpreted()) else {
                return Err(HtmlTreeCapability::NonShellElementTag);
            };
            if !tag.attributes().is_empty() {
                return Err(HtmlTreeCapability::ShellTagAttribute);
            }
            if tag.self_closing_solidus().is_some() {
                return Err(HtmlTreeCapability::SelfClosingShellTag);
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

/// The interpreted tag name of a proved shell element, if it is one.
fn shell_element_name(interpreted: &str) -> Option<HtmlShellElementName> {
    match interpreted {
        "html" => Some(HtmlShellElementName::Html),
        "head" => Some(HtmlShellElementName::Head),
        "body" => Some(HtmlShellElementName::Body),
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

/// One committed effect of a TC-S1 insertion-mode rule.
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
    /// Stop document parsing at this token.
    Stop,
}

/// Selects the TC-S1 rule for one (insertion mode, admitted token) cell.
///
/// This function is the complete, exhaustive statement of the TC-S1 action
/// set. Every cell it returns `Ok` for is a cell the accepted candidate-
/// independent GOLD proves; every other cell returns typed unsupported
/// evidence. Notably, no cell is admitted merely because a tag happens to be
/// named `html`, `head`, or `body`: shell names reached in document positions
/// the GOLD does not prove are refused like any other.
///
/// It takes no session state and mutates nothing, so an unsupported result is
/// structurally guaranteed to stop before mutation.
fn classify(
    mode: InsertionMode,
    token: &AdmittedToken<'_>,
) -> Result<ModeStep, HtmlTreeCapability> {
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
                    name: HtmlShellElementName::Html,
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
                    name: HtmlShellElementName::Head,
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
                    name: HtmlShellElementName::Html,
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
                    name: HtmlShellElementName::Head,
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::CloseHeadElement(HtmlShellClosure::AuthoredEndTag)),
                    next: Some(InsertionMode::AfterHead),
                }),
                AdmittedToken::StartTag {
                    name: HtmlShellElementName::Head,
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(Effect::RecordDuplicateHeadStartTag),
                    next: None,
                }),
                AdmittedToken::StartTag {
                    name: HtmlShellElementName::Html,
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
                    name: HtmlShellElementName::Body,
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
                    name: HtmlShellElementName::Head | HtmlShellElementName::Html,
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellStartTagPosition),
                AdmittedToken::EndTag {
                    name: HtmlShellElementName::Head,
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
        // and needs no whitespace refusal.
        InsertionMode::InBody => match token {
            AdmittedToken::Characters { .. } => Ok(ModeStep::Consume {
                effect: Some(Effect::InsertCharacters),
                next: None,
            }),
            AdmittedToken::StartTag {
                name: HtmlShellElementName::Body,
                ..
            } => Ok(ModeStep::Consume {
                effect: Some(Effect::RecordDuplicateBodyStartTag),
                next: None,
            }),
            AdmittedToken::StartTag {
                name: HtmlShellElementName::Head | HtmlShellElementName::Html,
                ..
            } => Err(HtmlTreeCapability::UnprovedShellStartTagPosition),
            AdmittedToken::EndTag {
                name: HtmlShellElementName::Body,
                ..
            } => Ok(ModeStep::Consume {
                effect: Some(Effect::AcknowledgeShellEndTag(HtmlShellElementName::Body)),
                next: Some(InsertionMode::AfterBody),
            }),
            AdmittedToken::EndTag {
                name: HtmlShellElementName::Head | HtmlShellElementName::Html,
                ..
            } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
            AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop),
        },
        // Reached by the proved `</body>` cell. End of file here stops the
        // bounded document parse, which is what the accepted G4, G6, and G7
        // cases require; the other `after body` rules the HTML Standard defines
        // (whitespace, comments, reprocessing anything else back in `in body`)
        // are not proved by TC-S1 and stay refused.
        InsertionMode::AfterBody => match token {
            AdmittedToken::EndTag {
                name: HtmlShellElementName::Html,
                ..
            } => Ok(ModeStep::Consume {
                effect: Some(Effect::AcknowledgeShellEndTag(HtmlShellElementName::Html)),
                next: Some(InsertionMode::AfterAfterBody),
            }),
            AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop),
            AdmittedToken::Characters { .. } => {
                Err(HtmlTreeCapability::UnprovedCharacterDataPosition)
            }
            AdmittedToken::StartTag { .. } => {
                Err(HtmlTreeCapability::UnprovedShellStartTagPosition)
            }
            AdmittedToken::EndTag { .. } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
        },
        InsertionMode::AfterAfterBody => match token {
            AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop),
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

/// What processing one admitted token concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenOutcome {
    /// The token was completely processed by supported actions.
    Consumed,
    /// The token stopped document parsing normally.
    StoppedParsing,
    /// TC-S1 does not prove this cell. Nothing was mutated for it.
    Unsupported(HtmlTreeCapability),
}

/// A TC-S1 construction-session invariant failure.
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
    /// An insertion-mode transition did not move strictly forward through the
    /// TC-S1 order.
    NonMonotonicInsertionModeTransition,
    /// Committed tree coverage moved backwards.
    NonMonotonicCommittedCoverage,
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

    /// Processes one admitted token to completion.
    ///
    /// Terminates because every iteration of the reprocessing loop moves the
    /// insertion mode strictly forward through the finite [`InsertionMode`]
    /// order, and [`Self::switch_mode`] refuses anything else. There is no
    /// recursion and no independent tree loop.
    pub(super) fn process(
        &mut self,
        token: &AdmittedToken<'_>,
        trigger: HtmlTreeTokenTrigger,
    ) -> Result<TokenOutcome, HtmlTreeSessionError> {
        loop {
            let step = match classify(self.mode, token) {
                Ok(step) => step,
                Err(capability) => return Ok(TokenOutcome::Unsupported(capability)),
            };
            match step {
                ModeStep::Stop => {
                    self.record_action(HtmlTreeActionKind::StoppedParsing, &trigger);
                    self.commit_token(token)?;
                    return Ok(TokenOutcome::StoppedParsing);
                }
                ModeStep::Consume { effect, next } => {
                    if let Some(effect) = effect {
                        self.apply(effect, &trigger, token)?;
                    }
                    if let Some(next) = next {
                        self.switch_mode(next)?;
                    }
                    self.commit_token(token)?;
                    return Ok(TokenOutcome::Consumed);
                }
                ModeStep::Reprocess { effect, next } => {
                    if let Some(effect) = effect {
                        self.apply(effect, &trigger, token)?;
                    }
                    self.switch_mode(next)?;
                    self.record_action(HtmlTreeActionKind::ReprocessedToken, &trigger);
                }
            }
        }
    }

    /// Consumes the session into the freeze boundary's input.
    ///
    /// The session itself never escapes, and nothing mutable travels in the
    /// returned parts.
    pub(super) fn finish(self, completion: HtmlTreeCompletion) -> HtmlDocumentShellParts {
        HtmlDocumentShellParts {
            nodes: self.nodes,
            root: self.root,
            admitted_creation_events: self.identities.admitted(),
            diagnostics: self.diagnostics,
            actions: self.actions,
            processed_tokens: self.processed_tokens,
            committed_prefix_end: self.committed_prefix_end,
            completion,
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
            HtmlTreeNodeKind::Element(HtmlShellElement::new(name, origin)),
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

    /// Moves the insertion mode strictly forward.
    ///
    /// Refusing any non-forward transition is what makes per-token
    /// termination structural instead of a work budget.
    fn switch_mode(&mut self, next: InsertionMode) -> Result<(), HtmlTreeSessionError> {
        if next <= self.mode {
            return Err(HtmlTreeSessionError::NonMonotonicInsertionModeTransition);
        }
        self.mode = next;
        Ok(())
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
    /// Together with the mode machine this bounds the open-element state to
    /// the admitted shell without any depth limit: a name can be pushed only
    /// while it is not already open.
    fn is_open(&self, name: HtmlShellElementName) -> bool {
        self.open_elements.iter().any(|id| {
            matches!(
                self.node(*id).map(HtmlTreeNode::kind),
                Some(HtmlTreeNodeKind::Element(element)) if element.name() == name
            )
        })
    }

    #[cfg(test)]
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
