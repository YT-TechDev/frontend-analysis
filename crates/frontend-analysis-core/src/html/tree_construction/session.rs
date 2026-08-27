//! The private HTML tree-construction session through TC-S9.
//!
//! The session exclusively owns every piece of mutable tree-construction
//! state for exactly one run: insertion mode, retained original insertion mode
//! while the selected Style lifecycle is in Text, open elements, the head
//! pointer, the private document mode and `frameset-ok` flag, constructed
//! identity, temporary node storage, committed diagnostics/actions/coverage,
//! and the one private tokenizer-feedback acknowledgement that may be pending.
//!
//! Ownership boundaries this module keeps:
//!
//! - the session never calls the tokenizer and holds no tokenizer cursor or
//!   lexical `State`; it is driven token by token by [`super::driver`];
//! - TC-S9 emits only the semantic [`HtmlTreeTokenizerFeedback::EnterRawText`]
//!   request. The Core coordinator maps that request to tokenizer control and
//!   acknowledges successful application before later source may be produced;
//! - the session never escapes to a consumer — it is consumed by
//!   [`Self::finish`] into [`HtmlDocumentShellParts`], which the driver hands
//!   to [`super::result::freeze`];
//! - no mutable session state travels into the frozen result.
//!
//! Rule selection and mutation remain separated. [`admit`] is pure lexical
//! admission over the validated token. [`classify`] is read-only selection
//! over actual insertion mode and bounded semantic open-state projections.
//! Unsupported cells therefore still refuse before mutation.
//!
//! TC-S9 adds one deliberate two-phase exception to ordinary token commit:
//! inserting the selected authored `<style>` first records a private pending
//! feedback obligation and returns it to the coordinator without committing
//! the token's processed coverage. Only successful tokenizer application and
//! [`Self::acknowledge_tokenizer_feedback`] commit that token, retain original
//! `InHead`, enter `Text`, clear the obligation, and permit later source
//! production. A half-coordinated Style start therefore cannot freeze as a
//! durable successful result.

use crate::SourceAnchor;

use super::super::token::{HtmlTagKind, HtmlToken};
use super::result::{
    HtmlConstructedIdentityCounter, HtmlConstructedNodeId, HtmlDocumentShellParts, HtmlElement,
    HtmlParagraphClosure, HtmlParagraphElement, HtmlParagraphElementOrigin,
    HtmlParagraphSynthesisCause, HtmlSelectedOrdinaryElement, HtmlSelectedOrdinaryElementName,
    HtmlShellClosure, HtmlShellElement, HtmlShellElementName, HtmlShellElementOrigin,
    HtmlStyleElement, HtmlSynthesisCause, HtmlTextContribution, HtmlTextNode, HtmlTreeAction,
    HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic,
    HtmlTreeDiagnosticCode, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery, HtmlTreeTokenTrigger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    Text,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HtmlDocumentMode {
    NoQuirks,
    Quirks,
}

/// The only tree-to-tokenizer semantic request authorized by TC-S9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HtmlTreeTokenizerFeedback {
    EnterRawText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdmittedElementName {
    Shell(HtmlShellElementName),
    SelectedOrdinary(HtmlSelectedOrdinaryElementName),
    Paragraph,
    Style,
}

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

    fn is_style_tag(&self) -> bool {
        matches!(
            self,
            Self::StartTag {
                name: AdmittedElementName::Style,
                ..
            } | Self::EndTag {
                name: AdmittedElementName::Style,
                ..
            }
        )
    }

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

pub(super) fn admit(token: &HtmlToken) -> Result<AdmittedToken<'_>, HtmlTreeCapability> {
    match token {
        HtmlToken::Character(character) => Ok(AdmittedToken::Characters {
            source: character.source(),
            interpreted: character.interpreted(),
        }),
        HtmlToken::Tag(tag) => {
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
                    AdmittedElementName::Style => HtmlTreeCapability::StyleTagAttribute,
                });
            }
            if tag.self_closing_solidus().is_some() {
                return Err(match name {
                    AdmittedElementName::Shell(_) => HtmlTreeCapability::SelfClosingShellTag,
                    AdmittedElementName::SelectedOrdinary(_) => {
                        HtmlTreeCapability::SelfClosingSelectedOrdinaryTag
                    }
                    AdmittedElementName::Paragraph => HtmlTreeCapability::SelfClosingParagraphTag,
                    AdmittedElementName::Style => HtmlTreeCapability::SelfClosingStyleTag,
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
        "style" => Some(AdmittedElementName::Style),
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
    AcknowledgeBodyEndTagWithOpenSelectedOrdinaryElements,
    RecordHtmlEndTagWithOpenSelectedOrdinaryElements,
    RecordAfterBodyCharacterData,
    InsertSelectedOrdinaryElement(HtmlSelectedOrdinaryElementName),
    CloseSelectedOrdinaryElement(HtmlSelectedOrdinaryElementName),
    RecoverInterveningSelectedOrdinaryElementsAndCloseTarget(HtmlSelectedOrdinaryElementName),
    PopParagraphThenResolveSelectedOrdinaryEnd(HtmlSelectedOrdinaryElementName),
    RecordUnmatchedSelectedOrdinaryEndTag(HtmlSelectedOrdinaryElementName),
    RecordOpenSelectedOrdinaryElementAtEndOfFile,
    InsertParagraphElement,
    CloseParagraphElement,
    CloseParagraphThenInsertParagraph,
    CloseParagraphThenInsertSelectedOrdinaryElement(HtmlSelectedOrdinaryElementName),
    SynthesizeAndCloseParagraphForUnmatchedEnd,
    InsertStyleElement,
    CloseStyleElement,
    PopStyleElementAtEndOfFile,
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
    RequestTokenizerFeedback {
        effect: Effect,
        feedback: HtmlTreeTokenizerFeedback,
    },
    ConsumeRestoringOriginal {
        effect: Effect,
    },
    ReprocessRestoringOriginal {
        effect: Effect,
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

fn classify(
    mode: InsertionMode,
    open_selected_ordinary: &[HtmlSelectedOrdinaryElementName],
    paragraph_is_current: bool,
    token: &AdmittedToken<'_>,
) -> Result<ModeStep, HtmlTreeCapability> {
    if token.is_style_tag() {
        match (mode, token) {
            (
                InsertionMode::InHead,
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Style,
                    ..
                },
            )
            | (
                InsertionMode::Text,
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Style,
                    ..
                },
            ) => {}
            _ => return Err(HtmlTreeCapability::StyleTagOutsideSelectedLifecycle),
        }
    }
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
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Style,
                    ..
                } => Ok(ModeStep::RequestTokenizerFeedback {
                    effect: Effect::InsertStyleElement,
                    feedback: HtmlTreeTokenizerFeedback::EnterRawText,
                }),
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
        InsertionMode::Text => match token {
            AdmittedToken::Characters { .. } => Ok(selected_in_body_character_step()),
            AdmittedToken::EndTag {
                name: AdmittedElementName::Style,
                ..
            } => Ok(ModeStep::ConsumeRestoringOriginal {
                effect: Effect::CloseStyleElement,
            }),
            AdmittedToken::EndOfFile { .. } => Ok(ModeStep::ReprocessRestoringOriginal {
                effect: Effect::PopStyleElementAtEndOfFile,
            }),
            AdmittedToken::StartTag { .. } => {
                Err(HtmlTreeCapability::UnprovedShellStartTagPosition)
            }
            AdmittedToken::EndTag { .. } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
        },
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
            if matches!(
                token,
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Body),
                    ..
                }
            ) {
                return Ok(ModeStep::Consume {
                    effect: Some(if open_selected_ordinary.is_empty() {
                        Effect::AcknowledgeShellEndTag(HtmlShellElementName::Body)
                    } else {
                        Effect::AcknowledgeBodyEndTagWithOpenSelectedOrdinaryElements
                    }),
                    next: Some(InsertionMode::AfterBody),
                });
            }
            if matches!(
                token,
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Html),
                    ..
                }
            ) {
                return Ok(ModeStep::Reprocess {
                    effect: (!open_selected_ordinary.is_empty())
                        .then_some(Effect::RecordHtmlEndTagWithOpenSelectedOrdinaryElements),
                    next: InsertionMode::AfterBody,
                });
            }
            if paragraph_is_current && token.is_shell_tag() {
                return Err(HtmlTreeCapability::ShellTagWithOpenParagraphElement);
            }
            if !open_selected_ordinary.is_empty() && token.is_shell_tag() {
                return Err(HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement);
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
                        SelectedEndTarget::Current | SelectedEndTarget::NonCurrent
                            if paragraph_is_current =>
                        {
                            Effect::PopParagraphThenResolveSelectedOrdinaryEnd(*name)
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
                    name:
                        AdmittedElementName::Shell(
                            HtmlShellElementName::Body | HtmlShellElementName::Html,
                        ),
                    ..
                } => unreachable!("handled by the TC-S7/TC-S8 pre-firewall branches"),
                AdmittedToken::EndTag {
                    name: AdmittedElementName::Shell(HtmlShellElementName::Head),
                    ..
                } => Err(HtmlTreeCapability::UnprovedShellEndTagPosition),
                AdmittedToken::StartTag {
                    name: AdmittedElementName::Style,
                    ..
                }
                | AdmittedToken::EndTag {
                    name: AdmittedElementName::Style,
                    ..
                } => unreachable!("Style firewall handled before the mode match"),
                AdmittedToken::EndOfFile { .. } => Ok(ModeStep::Stop {
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

fn is_implied_end_element(element: &HtmlElement) -> bool {
    match element {
        HtmlElement::Shell(_) | HtmlElement::Style(_) => false,
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
    TokenizerFeedbackRequested(HtmlTreeTokenizerFeedback),
    StoppedParsing,
    Unsupported(HtmlTreeCapability),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DispatchOutcome {
    Consumed,
    TokenizerFeedbackRequested(HtmlTreeTokenizerFeedback),
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
    NoOpenSelectedOrdinaryElementAtBodyEnd,
    NoOpenSelectedOrdinaryElementAtHtmlEnd,
    ParagraphIsNotCurrent(HtmlConstructedNodeId),
    MultipleOpenParagraphElements,
    ParagraphElementIsNotCurrent,
    ParagraphElementAlreadyOpen,
    TokenizerFeedbackAlreadyPending,
    TokenizerFeedbackAcknowledgedWithoutPending,
    TokenizerFeedbackAcknowledgementMismatch,
    TokenizerFeedbackRequestedOutsideInHead,
    OriginalInsertionModeAlreadyRetained,
    MissingOriginalInsertionMode,
    OriginalInsertionModeWasNotInHead,
    StyleElementAlreadyOpen,
    StyleElementIsNotCurrent,
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

struct SelectedOrdinaryEndOverParagraphPlan {
    paragraph: HtmlConstructedNodeId,
    target: HtmlConstructedNodeId,
    target_open_element: usize,
    intervening_current_first: Vec<HtmlConstructedNodeId>,
}

struct PreparedInsertion {
    parent_storage_index: usize,
    reserved: HtmlConstructedNodeId,
    node: HtmlTreeNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingTokenizerFeedback {
    feedback: HtmlTreeTokenizerFeedback,
    committed_end: usize,
}

pub(super) struct HtmlTreeSession {
    identities: HtmlConstructedIdentityCounter,
    nodes: Vec<HtmlTreeNode>,
    root: HtmlConstructedNodeId,
    open_elements: Vec<HtmlConstructedNodeId>,
    head_element: Option<HtmlConstructedNodeId>,
    mode: InsertionMode,
    original_insertion_mode: Option<InsertionMode>,
    pending_tokenizer_feedback: Option<PendingTokenizerFeedback>,
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
            original_insertion_mode: None,
            pending_tokenizer_feedback: None,
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
        if self.pending_tokenizer_feedback.is_some() {
            return Err(HtmlTreeSessionError::TokenizerFeedbackAlreadyPending);
        }
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
            ModeStep::RequestTokenizerFeedback { effect, feedback } => {
                if self.mode != InsertionMode::InHead {
                    return Err(HtmlTreeSessionError::TokenizerFeedbackRequestedOutsideInHead);
                }
                self.apply(effect, trigger, token)?;
                self.pending_tokenizer_feedback = Some(PendingTokenizerFeedback {
                    feedback,
                    committed_end: token.committed_end(),
                });
                Ok(DispatchOutcome::TokenizerFeedbackRequested(feedback))
            }
            ModeStep::ConsumeRestoringOriginal { effect } => {
                self.apply(effect, trigger, token)?;
                self.restore_original_insertion_mode()?;
                self.commit_token(token)?;
                Ok(DispatchOutcome::Consumed)
            }
            ModeStep::ReprocessRestoringOriginal { effect } => {
                self.apply(effect, trigger, token)?;
                self.restore_original_insertion_mode()?;
                self.record_action(HtmlTreeActionKind::ReprocessedToken, trigger);
                Ok(DispatchOutcome::ReprocessSameToken)
            }
        }
    }

    /// Completes the tree half of the TC-S9 two-phase Style start only after
    /// the coordinator has successfully applied the requested tokenizer
    /// control. No later source can have been requested while this obligation
    /// was pending.
    pub(super) fn acknowledge_tokenizer_feedback(
        &mut self,
        feedback: HtmlTreeTokenizerFeedback,
    ) -> Result<(), HtmlTreeSessionError> {
        let pending = self
            .pending_tokenizer_feedback
            .take()
            .ok_or(HtmlTreeSessionError::TokenizerFeedbackAcknowledgedWithoutPending)?;
        if pending.feedback != feedback {
            self.pending_tokenizer_feedback = Some(pending);
            return Err(HtmlTreeSessionError::TokenizerFeedbackAcknowledgementMismatch);
        }
        if self.mode != InsertionMode::InHead {
            return Err(HtmlTreeSessionError::TokenizerFeedbackRequestedOutsideInHead);
        }
        if self.original_insertion_mode.is_some() {
            return Err(HtmlTreeSessionError::OriginalInsertionModeAlreadyRetained);
        }
        self.original_insertion_mode = Some(self.mode);
        self.switch_mode(InsertionMode::Text);
        self.commit_end(pending.committed_end)
    }

    pub(super) fn finish(self, completion: HtmlTreeCompletion) -> HtmlDocumentShellParts {
        let final_open_selected_ordinary = self.open_selected_ordinary_ids();
        let final_open_paragraph = self
            .open_elements
            .iter()
            .copied()
            .find(|id| self.is_paragraph(*id));
        let final_open_style = self
            .open_elements
            .iter()
            .copied()
            .find(|id| self.is_style(*id));
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
            final_open_style,
            final_style_text_mode_active: self.mode == InsertionMode::Text,
            final_style_original_in_head_retained: self.original_insertion_mode
                == Some(InsertionMode::InHead),
            pending_tokenizer_feedback: self.pending_tokenizer_feedback.is_some(),
            coordinated_raw_text_entry_tokens: Vec::new(),
            coordinated_raw_text_close_tokens: Vec::new(),
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
            Effect::AcknowledgeBodyEndTagWithOpenSelectedOrdinaryElements => {
                if self.open_selected_ordinary_ids().is_empty() {
                    return Err(HtmlTreeSessionError::NoOpenSelectedOrdinaryElementAtBodyEnd);
                }
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
                    trigger,
                    HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
                );
                self.record_action(
                    HtmlTreeActionKind::AcknowledgedShellEndTag {
                        name: HtmlShellElementName::Body,
                    },
                    trigger,
                );
                Ok(())
            }
            Effect::RecordHtmlEndTagWithOpenSelectedOrdinaryElements => {
                if self.open_selected_ordinary_ids().is_empty() {
                    return Err(HtmlTreeSessionError::NoOpenSelectedOrdinaryElementAtHtmlEnd);
                }
                self.record_diagnostic(
                    HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
                    trigger,
                    HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
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
            Effect::PopParagraphThenResolveSelectedOrdinaryEnd(name) => {
                self.pop_paragraph_then_resolve_selected_ordinary_end(name, trigger)
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
            Effect::InsertStyleElement => self.insert_style_element(trigger, token),
            Effect::CloseStyleElement => self.close_style_element(trigger),
            Effect::PopStyleElementAtEndOfFile => self.pop_style_element_at_eof(trigger),
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

    fn insert_style_element(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
        token: &AdmittedToken<'_>,
    ) -> Result<(), HtmlTreeSessionError> {
        if self
            .open_elements
            .iter()
            .copied()
            .any(|id| self.is_style(id))
        {
            return Err(HtmlTreeSessionError::StyleElementAlreadyOpen);
        }
        let AdmittedToken::StartTag {
            name: AdmittedElementName::Style,
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
        if Some(parent) != self.head_element {
            return Err(HtmlTreeSessionError::MissingInsertionParent);
        }
        let parent_storage_index = self
            .nodes
            .iter()
            .position(|node| node.id() == parent)
            .ok_or(HtmlTreeSessionError::UnknownConstructedNode(parent))?;
        let reserved = self
            .identities
            .reserve()
            .ok_or(HtmlTreeSessionError::ConstructedIdentityExhausted)?;
        let element = HtmlStyleElement::new((*complete).clone(), (*raw_name).clone());
        let node = HtmlTreeNode::new(
            reserved,
            Some(parent),
            Vec::new(),
            HtmlTreeNodeKind::Element(HtmlElement::Style(element)),
        );
        self.nodes[parent_storage_index].push_child(reserved);
        self.nodes.push(node);
        self.open_elements.push(reserved);
        self.identities.commit(reserved);
        self.record_action(
            HtmlTreeActionKind::InsertedAuthoredStyleElement { node: reserved },
            trigger,
        );
        Ok(())
    }

    fn close_style_element(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let style = *self
            .open_elements
            .last()
            .ok_or(HtmlTreeSessionError::StyleElementIsNotCurrent)?;
        if !self.is_style(style) {
            return Err(HtmlTreeSessionError::StyleElementIsNotCurrent);
        }
        self.open_elements.pop();
        self.record_action(
            HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { node: style },
            trigger,
        );
        Ok(())
    }

    fn pop_style_element_at_eof(
        &mut self,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let style = *self
            .open_elements
            .last()
            .ok_or(HtmlTreeSessionError::StyleElementIsNotCurrent)?;
        if !self.is_style(style) {
            return Err(HtmlTreeSessionError::StyleElementIsNotCurrent);
        }
        self.record_diagnostic(
            HtmlTreeDiagnosticCode::StyleEndOfFileInText,
            trigger,
            HtmlTreeRecovery::PoppedStyleAtEndOfFileAndRestoredInHead,
        );
        self.open_elements.pop();
        self.record_action(
            HtmlTreeActionKind::PoppedStyleElementAtEndOfFile { node: style },
            trigger,
        );
        Ok(())
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

    fn pop_paragraph_then_resolve_selected_ordinary_end(
        &mut self,
        name: HtmlSelectedOrdinaryElementName,
        trigger: &HtmlTreeTokenTrigger,
    ) -> Result<(), HtmlTreeSessionError> {
        let plan = self.plan_selected_ordinary_end_over_paragraph(name)?;
        self.open_elements.truncate(plan.target_open_element);
        self.record_action(
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
                node: plan.paragraph,
                target: plan.target,
            },
            trigger,
        );
        if !plan.intervening_current_first.is_empty() {
            self.record_diagnostic(
                HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag,
                trigger,
                HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
            );
        }
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

    fn plan_selected_ordinary_end_over_paragraph(
        &self,
        name: HtmlSelectedOrdinaryElementName,
    ) -> Result<SelectedOrdinaryEndOverParagraphPlan, HtmlTreeSessionError> {
        let paragraph = self
            .open_paragraph()?
            .ok_or(HtmlTreeSessionError::ParagraphElementIsNotCurrent)?;
        if self.open_elements.last() != Some(&paragraph) {
            return Err(HtmlTreeSessionError::ParagraphIsNotCurrent(paragraph));
        }
        let target_open_element = self.nearest_open_selected_ordinary(name).ok_or(
            HtmlTreeSessionError::SelectedOrdinaryElementIsNotInScope(name),
        )?;
        let paragraph_open_element = self.open_elements.len().saturating_sub(1);
        if target_open_element >= paragraph_open_element {
            return Err(HtmlTreeSessionError::SelectedOrdinaryElementIsNotInScope(
                name,
            ));
        }
        let mut intervening_current_first = Vec::new();
        for id in self.open_elements[target_open_element + 1..paragraph_open_element]
            .iter()
            .rev()
        {
            if !self.is_open_selected_ordinary(*id) {
                return Err(HtmlTreeSessionError::SelectedOrdinaryRecoverySuffixIsNotSelected(*id));
            }
            intervening_current_first.push(*id);
        }
        Ok(SelectedOrdinaryEndOverParagraphPlan {
            paragraph,
            target: self.open_elements[target_open_element],
            target_open_element,
            intervening_current_first,
        })
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

    fn restore_original_insertion_mode(&mut self) -> Result<(), HtmlTreeSessionError> {
        let original = self
            .original_insertion_mode
            .take()
            .ok_or(HtmlTreeSessionError::MissingOriginalInsertionMode)?;
        if original != InsertionMode::InHead {
            return Err(HtmlTreeSessionError::OriginalInsertionModeWasNotInHead);
        }
        self.switch_mode(original);
        Ok(())
    }

    fn switch_mode(&mut self, next: InsertionMode) {
        self.mode = next;
    }

    fn commit_token(&mut self, token: &AdmittedToken<'_>) -> Result<(), HtmlTreeSessionError> {
        self.commit_end(token.committed_end())
    }

    fn commit_end(&mut self, end: usize) -> Result<(), HtmlTreeSessionError> {
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

    fn is_style(&self, id: HtmlConstructedNodeId) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Element(HtmlElement::Style(_)))
        )
    }

    fn is_paragraph(&self, id: HtmlConstructedNodeId) -> bool {
        matches!(
            self.node(id).map(HtmlTreeNode::kind),
            Some(HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)))
        )
    }

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
