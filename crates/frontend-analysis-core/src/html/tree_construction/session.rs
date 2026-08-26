//! The private TC-S1 tree-construction session, with its accepted TC-S2,
//! TC-S3, TC-S4, and TC-S5 successors.
//!
//! The session exclusively owns every piece of mutable tree-construction
//! state for exactly one run: the insertion mode, the open elements — the
//! shell, TC-S3/TC-S4's selected ordinary elements, and TC-S5's bounded
//! Paragraph — the head pointer, the private document mode and `frameset-ok`
//! flag, the constructed identity counter, the temporary node storage, and the
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
//! # Two gates, both before mutation
//!
//! Rule selection and mutation are separated on purpose, and the separation is
//! now two distinct gates:
//!
//! 1. [`admit`] is *pure lexical admission*. It is a function of the validated
//!    token alone, reads no construction state, and recognizes exactly the
//!    three closed categories [`AdmittedElementName::Shell`],
//!    [`AdmittedElementName::SelectedOrdinary`], and
//!    [`AdmittedElementName::Paragraph`] while propagating the token's exact
//!    evidence. Unsupported tag syntax — an unproved name, attributes, a
//!    self-closing solidus — is refused here.
//! 2. [`classify`] is *read-only selection*. It is a free function of the
//!    actual insertion mode, the already-read ordered names of the open
//!    selected ordinary elements, the bounded fact that a Paragraph is
//!    current, and the admitted token: it takes no `&self`, performs no
//!    mutation, and returns either the step to apply or the
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
//! TC-S5 keeps that pattern. Cells with more than one semantic effect use one
//! focused compound effect, but every fallible part of the node insertion is
//! resolved into a private [`PreparedInsertion`] before the current P is
//! popped or an unmatched-P diagnostic is recorded. Once the first semantic
//! mutation commits, the remainder is infallible: no rollback or generalized
//! effect-list machinery is introduced.
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
//! TC-S4's recovery and TC-S5's Paragraph transactions add no work dimension
//! either. All scans traverse the finite open-element stack; TC-S5 adds no
//! redispatch, tokenizer feedback, retry loop, generalized scope engine, or
//! generalized implied-end generator.

use crate::SourceAnchor;

use super::super::token::{HtmlTagKind, HtmlToken};
use super::result::{
    HtmlConstructedIdentityCounter, HtmlConstructedNodeId, HtmlDocumentShellParts, HtmlElement,
    HtmlParagraphClosure, HtmlParagraphElement, HtmlParagraphElementOrigin,
    HtmlParagraphSynthesisCause, HtmlSelectedOrdinaryElement, HtmlSelectedOrdinaryElementName,
    HtmlShellClosure, HtmlShellElement, HtmlShellElementName, HtmlShellElementOrigin,
    HtmlSynthesisCause, HtmlTextContribution, HtmlTextNode, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode,
    HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery, HtmlTreeTokenTrigger,
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
    Paragraph,
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

    /// Whether this token is a Paragraph tag, start or end.
    fn is_paragraph_tag(&self) -> bool {
        matches!(
            self,
            Self::StartTag {
                name: AdmittedElementName::Paragraph,
                ..
            } | Self::EndTag {
                name: AdmittedElementName::Paragraph,
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
            // predecessor variants keep exactly their old meaning.
            let Some(name) = admitted_element_name(tag.name().interpreted()) else {
                return Err(HtmlTreeCapability::NonShellElementTag);
            };
            if !tag.attributes().is_empty() {
                return Err(match name {
                    AdmittedElementName::Shell(_) => HtmlTreeCapability::ShellTagAttribute,
                    AdmittedElementName::SelectedOrdinary(_) => {
                        HtmlTreeCapability::SelectedOrdinaryTagAttribute
                    }
                    AdmittedElementName::Paragraph => HtmlTreeCapability::ParagraphTagAttribute,
                });
            }
            if tag.self_closing_solidus().is_some() {
                return Err(match name {
                    AdmittedElementName::Shell(_) => HtmlTreeCapability::SelfClosingShellTag,
                    AdmittedElementName::SelectedOrdinary(_) => {
                        HtmlTreeCapability::SelfClosingSelectedOrdinaryTag
                    }
                    AdmittedElementName::Paragraph => HtmlTreeCapability::SelfClosingParagraphTag,
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
        "p" => Some(AdmittedElementName::Paragraph),
        _ => None,
    }
}

fn contains_html_whitespace(interpreted: &str) -> bool {
    interpreted
        .chars()
        .any(|character| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterRunClass {
    AllHtmlWhitespace,
    AllNonHtmlWhitespace,
    Mixed,
}

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
        (true, false) | (false, false) => CharacterRunClass::AllHtmlWhitespace,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElementProvenance {
    AuthoredByTriggerToken,
    Synthesized,
}

/// One committed effect of an insertion-mode rule.
///
/// TC-S5 keeps [`ModeStep`] single-effect. Multi-effect P cells are represented
/// by focused compound effects whose complete insertion plan is resolved before
/// the first mutation.
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
    InsertParagraphElement,
    CloseParagraphElement,
    CloseParagraphThenInsertParagraph,
    CloseParagraphThenInsertSelectedOrdinaryElement(HtmlSelectedOrdinaryElementName),
    SynthesizeAndCloseParagraphForUnmatchedEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModeStep {
    Consume {
        effect: Option<Effect>,
        next: Option<InsertionMode>,
    },
    Reprocess {
        effect: Option<Effect>,
        next: InsertionMode,
    },
    Stop {
        effect: Option<Effect>,
    },
}

fn selected_in_body_character_step() -> ModeStep {
    ModeStep::Consume {
        effect: Some(Effect::InsertCharacters),
        next: None,
    }
}

/// Read-only rule selection over the accepted bounded state.
fn classify(
    mode: InsertionMode,
    open_selected_ordinary: &[HtmlSelectedOrdinaryElementName],
    paragraph_is_current: bool,
    token: &AdmittedToken<'_>,
) -> Result<ModeStep, HtmlTreeCapability> {
    if !matches!(mode, InsertionMode::InBody) && token.is_selected_ordinary_tag() {
        return Err(HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody);
    }
    if !matches!(mode, InsertionMode::InBody) && token.is_paragraph_tag() {
        return Err(HtmlTreeCapability::ParagraphTagOutsideInBody);
    }
    match mode {
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
        InsertionMode::InBody => {
            if paragraph_is_current && token.is_shell_tag() {
                return Err(HtmlTreeCapability::ShellTagWithOpenParagraphElement);
            }
            if !open_selected_ordinary.is_empty() && token.is_shell_tag() {
                return Err(HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement);
            }
            if paragraph_is_current
                && matches!(
                    token,
                    AdmittedToken::EndTag {
                        name: AdmittedElementName::SelectedOrdinary(_),
                        ..
                    }
                )
            {
                return Err(HtmlTreeCapability::SelectedOrdinaryEndTagWithOpenParagraphElement);
            }
            match token {
                AdmittedToken::Characters { .. } => Ok(selected_in_body_character_step()),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Paragraph,
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(if paragraph_is_current {
                        Effect::CloseParagraphThenInsertParagraph
                    } else {
                        Effect::InsertParagraphElement
                    }),
                    next: None,
                }),
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Paragraph,
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(if paragraph_is_current {
                        Effect::CloseParagraphElement
                    } else {
                        Effect::SynthesizeAndCloseParagraphForUnmatchedEnd
                    }),
                    next: None,
                }),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::SelectedOrdinary(name),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(if paragraph_is_current {
                        Effect::CloseParagraphThenInsertSelectedOrdinaryElement(*name)
                    } else {
                        Effect::InsertSelectedOrdinaryElement(*name)
                    }),
                    next: None,
                }),
                AdmittedToken::EndTag {
                    name: AdmittedElementName::SelectedOrdinary(name),
                    ..
                } => Ok(ModeStep::Consume {
                    effect: Some(match selected_end_target(open_selected_ordinary, *name) {
                        SelectedEndTarget::Absent => {
                            Effect::RecordUnmatchedSelectedOrdinaryEndTag(*name)
                        }
                        SelectedEndTarget::Current => Effect::CloseSelectedOrdinaryElement(*name),
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
                AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop {
                    // P itself is an allowed stack member at EOF. A selected
                    // Div/Section ancestor retains the predecessor diagnostic.
                    effect: (!open_selected_ordinary.is_empty())
                        .then_some(Effect::RecordOpenSelectedOrdinaryElementAtEndOfFile),
                }),
            }
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedEndTarget {
    Absent,
    Current,
    NonCurrent,
}

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

/// Total over the expanded closed represented element domain. P is the one
/// member that *is* an implied-end element; selected-end cells over an open P
/// are therefore refused before this predecessor safety check can be called.
fn is_implied_end_element(element: &HtmlElement) -> bool {
    match element {
        HtmlElement::Shell(_) => false,
        HtmlElement::SelectedOrdinary(selected) => match selected.name() {
            HtmlSelectedOrdinaryElementName::Div | HtmlSelectedOrdinaryElementName::Section => {
                false
            }
        },
        HtmlElement::Paragraph(_) => true,
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenOutcome {
    Consumed,
    StoppedParsing,
    Unsupported(HtmlTreeCapability),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DispatchOutcome {
    Consumed,
    ReprocessSameToken,
    StoppedParsing,
    Unsupported(HtmlTreeCapability),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeSessionError {
    ConstructedIdentityExhausted,
    MissingInsertionParent,
    AuthoredInsertionWithoutStartTag,
    DuplicateOpenShellElement(HtmlShellElementName),
    ClosedShellElementIsNotOpen(HtmlShellElementName),
    UnknownConstructedNode(HtmlConstructedNodeId),
    MissingCharacterInsertionTarget,
    InvalidCharacterCoalescingTarget(HtmlConstructedNodeId),
    RepeatedInsertionModeEvaluation,
    NonMonotonicCommittedCoverage,
    SelectedOrdinaryElementIsNotInScope(HtmlSelectedOrdinaryElementName),
    SelectedOrdinaryElementIsNotCurrent(HtmlSelectedOrdinaryElementName),
    ImpliedEndGenerationIsNotABoundedNoOp,
    SelectedOrdinaryRecoveryTargetIsCurrent(HtmlSelectedOrdinaryElementName),
    SelectedOrdinaryRecoverySuffixIsNotSelected(HtmlConstructedNodeId),
    UnmatchedSelectedOrdinaryEndTagWithElementInScope(HtmlSelectedOrdinaryElementName),
    NoOpenSelectedOrdinaryElementAtEndOfFile,
    ParagraphIsNotCurrent(HtmlConstructedNodeId),
    MultipleOpenParagraphElements,
    ParagraphElementIsNotCurrent,
    ParagraphElementAlreadyOpen,
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

struct SelectedOrdinaryEndPlan {
    target: HtmlConstructedNodeId,
    target_open_element: usize,
    intervening_current_first: Vec<HtmlConstructedNodeId>,
}

/// A completely pre-resolved node insertion. The private storage coordinate is
/// ephemeral and exists only to make the commit infallible after another
/// semantic effect has already happened.
struct PreparedInsertion {
    parent_storage_index: usize,
    reserved: HtmlConstructedNodeId,
    node: HtmlTreeNode,
}

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

    pub(super) fn dispatch(
        &mut self,
        token: &AdmittedToken<'_>,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<DispatchOutcome, HtmlTreeSessionError> {
        let open_selected_ordinary = self.open_selected_ordinary_names();
        let paragraph_is_current = self.open_paragraph()?.is_some();
        let step = match classify(
            self.mode,
            &open_selected_ordinary,
            paragraph_is_current,
            token,
        ) {
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

    pub(super) fn finish(self, completion: HtmlTreeCompletion) -> HtmlDocumentShellParts {
        let final_open_selected_ordinary = self.open_selected_ordinary_ids();
        let final_open_paragraph = self
            .open_elements
            .iter()
            .copied()
            .find(|id| self.is_paragraph(*id));
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
            final_open_paragraph,
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
            Effect::InsertParagraphElement => self.insert_paragraph(trigger, token),
            Effect::CloseParagraphElement => {
                self.close_paragraph(HtmlParagraphClosure::MatchingEndTag, trigger)
            }
            Effect::CloseParagraphThenInsertParagraph => {
                self.close_paragraph_then_insert_paragraph(trigger, token)
            }
            Effect::CloseParagraphThenInsertSelectedOrdinaryElement(name) => {
                self.close_paragraph_then_insert_selected(name, trigger, token)
            }
            Effect::SynthesizeAndCloseParagraphForUnmatchedEnd => {
                self.synthesize_and_close_paragraph(trigger)
            }
        }
    }

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

    fn insert_selected_ordinary_element(
        &mut self,
        name: HtmlSelectedOrdinaryElementName,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        let plan = self.prepare_selected_insertion(name, token)?;
        let node = plan.reserved;
        self.commit_prepared_insertion(plan);
        self.record_action(
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, name },
            trigger,
        );
        Ok(())
    }

    fn prepare_selected_insertion(
        &self,
        name: HtmlSelectedOrdinaryElementName,
        token: &AdmittedToken<'_>,
    ) -> Result<PreparedInsertion, HtmlTreeSessionError> {
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
        let parent_storage_index = self
            .nodes
            .iter()
            .position(|node| node.id() == parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?;
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        let element =
            HtmlSelectedOrdinaryElement::new(name, (*complete).clone(), (*raw_name).clone());
        Ok(PreparedInsertion {
            parent_storage_index,
            reserved,
            node: HtmlTreeNode::new(
                reserved,
                Some(parent),
                Vec::new(),
                HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(element)),
            ),
        })
    }

    fn prepare_authored_paragraph_insertion(
        &self,
        token: &AdmittedToken<'_>,
    ) -> Result<PreparedInsertion, HtmlTreeSessionError> {
        let AdmittedToken::StartTag {
            name: AdmittedElementName::Paragraph,
            complete,
            raw_name,
        } = token
        else {
            return Err(HtmlTreeSessionError::AuthoredInsertionWithoutStartTag);
        };
        let parent = *self
            .open_elements
            .last()
            .ok_or(HtmlTreeSessionError::MissingInsertionParent)?;
        let parent_storage_index = self
            .nodes
            .iter()
            .position(|node| node.id() == parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?;
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        let paragraph = HtmlParagraphElement::new(HtmlParagraphElementOrigin::Authored {
            complete: (*complete).clone(),
            raw_name: (*raw_name).clone(),
        });
        Ok(PreparedInsertion {
            parent_storage_index,
            reserved,
            node: HtmlTreeNode::new(
                reserved,
                Some(parent),
                Vec::new(),
                HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)),
            ),
        })
    }

    fn prepare_synthesized_paragraph_insertion(
        &self,
    ) -> Result<PreparedInsertion, HtmlTreeSessionError> {
        let parent = *self
            .open_elements
            .last()
            .ok_or(HtmlTreeSessionError::MissingInsertionParent)?;
        let parent_storage_index = self
            .nodes
            .iter()
            .position(|node| node.id() == parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?;
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        let paragraph = HtmlParagraphElement::new(HtmlParagraphElementOrigin::Synthesized(
            HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
        ));
        Ok(PreparedInsertion {
            parent_storage_index,
            reserved,
            node: HtmlTreeNode::new(
                reserved,
                Some(parent),
                Vec::new(),
                HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)),
            ),
        })
    }

    fn commit_prepared_insertion(&mut self, plan: PreparedInsertion) {
        self.nodes[plan.parent_storage_index].push_child(plan.reserved);
        self.nodes.push(plan.node);
        self.open_elements.push(plan.reserved);
        self.identities.commit(plan.reserved);
    }

    fn insert_paragraph(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        if self.open_paragraph()?.is_some() {
            return Err(HtmlTreeSessionError::ParagraphElementAlreadyOpen);
        }
        let plan = self.prepare_authored_paragraph_insertion(token)?;
        let node = plan.reserved;
        self.commit_prepared_insertion(plan);
        self.record_action(
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node },
            trigger,
        );
        Ok(())
    }

    fn close_paragraph(
        &mut self,
        closure: HtmlParagraphClosure,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let paragraph = self
            .open_paragraph()?
            .ok_or(HtmlTreeSessionError::ParagraphElementIsNotCurrent)?;
        if self.open_elements.last() != Some(&paragraph) {
            return Err(HtmlTreeSessionError::ParagraphIsNotCurrent(paragraph));
        }
        // Under TC-S5 P is current whenever present, so generate-implied-end-
        // tags-except-P has nothing above P to pop. This proves only the
        // bounded no-op and does not call the generic predecessor helper.
        self.open_elements.pop();
        self.record_action(
            HtmlTreeActionKind::ClosedParagraphElement {
                node: paragraph,
                closure,
            },
            trigger,
        );
        Ok(())
    }

    fn close_paragraph_then_insert_paragraph(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        let paragraph = self
            .open_paragraph()?
            .ok_or(HtmlTreeSessionError::ParagraphElementIsNotCurrent)?;
        let plan = self.prepare_insertion_after_current_paragraph(
            |parent, parent_storage_index, reserved| {
                let AdmittedToken::StartTag {
                    name: AdmittedElementName::Paragraph,
                    complete,
                    raw_name,
                } = token
                else {
                    return Err(HtmlTreeSessionError::AuthoredInsertionWithoutStartTag);
                };
                let element = HtmlParagraphElement::new(HtmlParagraphElementOrigin::Authored {
                    complete: (*complete).clone(),
                    raw_name: (*raw_name).clone(),
                });
                Ok(PreparedInsertion {
                    parent_storage_index,
                    reserved,
                    node: HtmlTreeNode::new(
                        reserved,
                        Some(parent),
                        Vec::new(),
                        HtmlTreeNodeKind::Element(HtmlElement::Paragraph(element)),
                    ),
                })
            },
        )?;
        let new_node = plan.reserved;
        self.open_elements.pop();
        self.record_action(
            HtmlTreeActionKind::ClosedParagraphElement {
                node: paragraph,
                closure: HtmlParagraphClosure::StartTriggered,
            },
            trigger,
        );
        self.commit_prepared_insertion(plan);
        self.record_action(
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node: new_node },
            trigger,
        );
        Ok(())
    }

    fn close_paragraph_then_insert_selected(
        &mut self,
        name: HtmlSelectedOrdinaryElementName,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        let paragraph = self
            .open_paragraph()?
            .ok_or(HtmlTreeSessionError::ParagraphElementIsNotCurrent)?;
        let plan = self.prepare_insertion_after_current_paragraph(
            |parent, parent_storage_index, reserved| {
                let AdmittedToken::StartTag {
                    name: AdmittedElementName::SelectedOrdinary(token_name),
                    complete,
                    raw_name,
                } = token
                else {
                    return Err(HtmlTreeSessionError::AuthoredInsertionWithoutStartTag);
                };
                if *token_name != name {
                    return Err(HtmlTreeSessionError::AuthoredInsertionWithoutStartTag);
                }
                let element = HtmlSelectedOrdinaryElement::new(
                    name,
                    (*complete).clone(),
                    (*raw_name).clone(),
                );
                Ok(PreparedInsertion {
                    parent_storage_index,
                    reserved,
                    node: HtmlTreeNode::new(
                        reserved,
                        Some(parent),
                        Vec::new(),
                        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(element)),
                    ),
                })
            },
        )?;
        let new_node = plan.reserved;
        self.open_elements.pop();
        self.record_action(
            HtmlTreeActionKind::ClosedParagraphElement {
                node: paragraph,
                closure: HtmlParagraphClosure::StartTriggered,
            },
            trigger,
        );
        self.commit_prepared_insertion(plan);
        self.record_action(
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                node: new_node,
                name,
            },
            trigger,
        );
        Ok(())
    }

    fn prepare_insertion_after_current_paragraph<F>(
        &self,
        build: F,
    ) -> Result<PreparedInsertion, HtmlTreeSessionError>
    where
        F: FnOnce(
            HtmlConstructedNodeId,
            usize,
            HtmlConstructedNodeId,
        ) -> Result<PreparedInsertion, HtmlTreeSessionError>,
    {
        let paragraph = self
            .open_paragraph()?
            .ok_or(HtmlTreeSessionError::ParagraphElementIsNotCurrent)?;
        if self.open_elements.last() != Some(&paragraph) {
            return Err(HtmlTreeSessionError::ParagraphIsNotCurrent(paragraph));
        }
        let parent = *self
            .open_elements
            .get(self.open_elements.len().saturating_sub(2))
            .ok_or(HtmlTreeSessionError::MissingInsertionParent)?;
        let parent_storage_index = self
            .nodes
            .iter()
            .position(|node| node.id() == parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?;
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        build(parent, parent_storage_index, reserved)
    }

    fn synthesize_and_close_paragraph(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        if self.open_paragraph()?.is_some() {
            return Err(HtmlTreeSessionError::ParagraphElementAlreadyOpen);
        }
        let plan = self.prepare_synthesized_paragraph_insertion()?;
        let node = plan.reserved;

        // All fallible work is complete. From the diagnostic onward the three
        // validated semantic effects commit without any operation that can
        // return failure.
        self.record_diagnostic(
            HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag,
            trigger,
            HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt,
        );
        self.commit_prepared_insertion(plan);
        self.record_action(
            HtmlTreeActionKind::InsertedSynthesizedParagraphElement {
                node,
                cause: HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
            },
            trigger,
        );
        let popped = self.open_elements.pop();
        debug_assert_eq!(popped, Some(node));
        self.record_action(
            HtmlTreeActionKind::ClosedParagraphElement {
                node,
                closure: HtmlParagraphClosure::UnmatchedEndTagSynthesized,
            },
            trigger,
        );
        Ok(())
    }

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

    fn is_open(&self, name: HtmlShellElementName) -> bool {
        self.open_elements.iter().any(|id| {
            matches!(
                self.node(*id).map(HtmlTreeNode::kind),
                Some(HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)))
                    if shell.name() == name
            )
        })
    }

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

    fn is_paragraph(&self, id: HtmlConstructedNodeId) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)))
        )
    }

    /// Returns the one open P exactly when it is current. Any other P shape is
    /// an internal invariant failure, never unsupported input.
    fn open_paragraph(&self) -> Result<Option<HtmlConstructedNodeId>, HtmlTreeSessionError> {
        let mut found = None;
        for id in &self.open_elements {
            if self.is_paragraph(*id) {
                if found.is_some() {
                    return Err(HtmlTreeSessionError::MultipleOpenParagraphElements);
                }
                found = Some(*id);
            }
        }
        if let Some(paragraph) = found
            && self.open_elements.last() != Some(&paragraph)
        {
            return Err(HtmlTreeSessionError::ParagraphIsNotCurrent(paragraph));
        }
        Ok(found)
    }

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

    fn is_open_selected_ordinary(&self, id: HtmlConstructedNodeId) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(_)))
        )
    }

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

    fn implied_end_generation_is_a_no_op(&self) -> bool {
        self.open_elements.iter().all(|id| {
            !matches!(
                self.node(*id).map(HtmlTreeNode::kind),
                Some(HtmlTreeNodeKind::Element(element)) if is_implied_end_element(element)
            )
        })
    }

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
