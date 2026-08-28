use crate::html::token::{
    HtmlAttributeDisposition, HtmlAttributeValueSyntax, HtmlTagKind, HtmlToken,
};
use crate::SourceText;

use super::super::diagnostic::{
    HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext, HtmlTokenizerDiagnosticHandling,
    HtmlTokenizerDiagnosticSubject, HtmlTokenizerRecoveryKind,
};
use super::super::resource::{
    HtmlTokenizerConfigurationFailure, HtmlTokenizerInvariantFailure, HtmlTokenizerResource,
};
use super::super::result::{
    HtmlCharacterReferenceContext, HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability,
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerMode,
    HtmlTokenizerRunResult, HtmlTokenizerUnsupportedTrigger,
};
use super::expected::{
    Attribute, AttributeDisposition, AttributeValue, Availability, ByteSpan, CanonicalRun,
    Capability, CharacterReferenceContext, Completion, ConfigurationFailure, Coverage, Diagnostic,
    DiagnosticCode, DiagnosticContext, DiagnosticHandling, DiagnosticSubject, InvariantFailure,
    Lexeme, Limits, ObservedRun, RecoveryKind, Resource, Token, TokenKind, TokenizerMode,
    UnsupportedTrigger, Usage,
};

pub(super) fn observe(source: &SourceText, run: &HtmlTokenizerRunResult) -> ObservedRun {
    ObservedRun(CanonicalRun {
        skipped_leading_bom: run
            .preprocessing()
            .skipped_leading_bom()
            .map(anchor_span),
        tokens: run
            .tokens()
            .iter()
            .map(|token| observe_token(source, token))
            .collect(),
        diagnostics: run.diagnostics().iter().map(observe_diagnostic).collect(),
        coverage: Coverage {
            processed_prefix: anchor_span(run.coverage().processed_prefix()),
            unprocessed_suffix: anchor_span(run.coverage().unprocessed_suffix()),
        },
        completion: observe_completion(run.completion()),
        limits: Limits {
            source_bytes: run.limits().max_source_bytes(),
            transition_steps: run.limits().max_transition_steps(),
            emitted_tokens: run.limits().max_emitted_tokens(),
            diagnostics: run.limits().max_diagnostics(),
            attributes_per_tag: run.limits().max_attributes_per_tag(),
            retained_interpreted_bytes: run.limits().max_retained_interpreted_bytes(),
            temporary_buffer_bytes: run.limits().max_temporary_buffer_bytes(),
        },
        usage: Usage {
            source_bytes: run.usage().source_bytes(),
            transition_steps: run.usage().transition_steps(),
            emitted_tokens: run.usage().emitted_tokens(),
            diagnostics: run.usage().diagnostics(),
            peak_attributes_per_tag: run.usage().peak_attributes_per_tag(),
            retained_interpreted_bytes: run.usage().retained_interpreted_bytes(),
            peak_temporary_buffer_bytes: run.usage().peak_temporary_buffer_bytes(),
        },
    })
}

fn observe_token(source: &SourceText, token: &HtmlToken) -> Token {
    match token {
        HtmlToken::Character(character) => Token::Character {
            source: lexeme(source, character.source()),
            interpreted: character.interpreted().to_owned(),
        },
        HtmlToken::Tag(tag) => Token::Tag {
            kind: match tag.kind() {
                HtmlTagKind::Start => TokenKind::StartTag,
                HtmlTagKind::End => TokenKind::EndTag,
            },
            complete: lexeme(source, tag.complete()),
            open_delimiter: lexeme(source, tag.open_delimiter()),
            name: lexeme(source, tag.name().source()),
            interpreted_name: tag.name().interpreted().to_owned(),
            attributes: tag
                .attributes()
                .iter()
                .map(|attribute| Attribute {
                    complete: lexeme(source, attribute.complete()),
                    name: lexeme(source, attribute.name().source()),
                    interpreted_name: attribute.name().interpreted().to_owned(),
                    value: observe_attribute_value(source, attribute.value_syntax()),
                    interpreted_value: attribute.interpreted_value().to_owned(),
                    disposition: match attribute.disposition() {
                        HtmlAttributeDisposition::Effective => AttributeDisposition::Effective,
                        HtmlAttributeDisposition::DuplicateOf { first_index } => {
                            AttributeDisposition::DuplicateOf(first_index)
                        }
                    },
                })
                .collect(),
            self_closing_solidus: tag
                .self_closing_solidus()
                .map(|anchor| lexeme(source, anchor)),
            close_delimiter: lexeme(source, tag.close_delimiter()),
        },
        HtmlToken::EndOfFile(eof) => Token::EndOfFile {
            at: anchor_span(eof.source()),
        },
    }
}

fn observe_attribute_value(source: &SourceText, value: &HtmlAttributeValueSyntax) -> AttributeValue {
    match value {
        HtmlAttributeValueSyntax::Missing => AttributeValue::Missing,
        HtmlAttributeValueSyntax::MissingAfterEquals {
            equals,
            value_boundary,
        } => AttributeValue::MissingAfterEquals {
            equals: lexeme(source, equals),
            value_boundary: anchor_span(value_boundary),
        },
        HtmlAttributeValueSyntax::Unquoted { equals, value } => AttributeValue::Unquoted {
            equals: lexeme(source, equals),
            value: lexeme(source, value),
        },
        HtmlAttributeValueSyntax::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => AttributeValue::DoubleQuoted {
            equals: lexeme(source, equals),
            open_quote: lexeme(source, open_quote),
            value: lexeme(source, value),
            close_quote: lexeme(source, close_quote),
        },
        HtmlAttributeValueSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => AttributeValue::SingleQuoted {
            equals: lexeme(source, equals),
            open_quote: lexeme(source, open_quote),
            value: lexeme(source, value),
            close_quote: lexeme(source, close_quote),
        },
    }
}

fn observe_diagnostic(diagnostic: &super::super::diagnostic::HtmlTokenizerDiagnostic) -> Diagnostic {
    Diagnostic {
        code: match diagnostic.code() {
            HtmlTokenizerDiagnosticCode::NoncharacterInInputStream => {
                DiagnosticCode::NoncharacterInInputStream
            }
            HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream => {
                DiagnosticCode::ControlCharacterInInputStream
            }
            HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter => {
                DiagnosticCode::UnexpectedNullCharacter
            }
            HtmlTokenizerDiagnosticCode::EofBeforeTagName => DiagnosticCode::EofBeforeTagName,
            HtmlTokenizerDiagnosticCode::InvalidFirstCharacterOfTagName => {
                DiagnosticCode::InvalidFirstCharacterOfTagName
            }
            HtmlTokenizerDiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName => {
                DiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName
            }
            HtmlTokenizerDiagnosticCode::MissingEndTagName => DiagnosticCode::MissingEndTagName,
            HtmlTokenizerDiagnosticCode::EofInTag => DiagnosticCode::EofInTag,
            HtmlTokenizerDiagnosticCode::UnexpectedEqualsSignBeforeAttributeName => {
                DiagnosticCode::UnexpectedEqualsSignBeforeAttributeName
            }
            HtmlTokenizerDiagnosticCode::UnexpectedCharacterInAttributeName => {
                DiagnosticCode::UnexpectedCharacterInAttributeName
            }
            HtmlTokenizerDiagnosticCode::MissingAttributeValue => {
                DiagnosticCode::MissingAttributeValue
            }
            HtmlTokenizerDiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue => {
                DiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue
            }
            HtmlTokenizerDiagnosticCode::MissingWhitespaceBetweenAttributes => {
                DiagnosticCode::MissingWhitespaceBetweenAttributes
            }
            HtmlTokenizerDiagnosticCode::UnexpectedSolidusInTag => {
                DiagnosticCode::UnexpectedSolidusInTag
            }
            HtmlTokenizerDiagnosticCode::DuplicateAttribute => DiagnosticCode::DuplicateAttribute,
            HtmlTokenizerDiagnosticCode::EndTagWithAttributes => {
                DiagnosticCode::EndTagWithAttributes
            }
            HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference => {
                DiagnosticCode::MissingSemicolonAfterCharacterReference
            }
            HtmlTokenizerDiagnosticCode::UnknownNamedCharacterReference => {
                DiagnosticCode::UnknownNamedCharacterReference
            }
            HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus => {
                DiagnosticCode::EndTagWithTrailingSolidus
            }
        },
        location: anchor_span(diagnostic.location()),
        context: match diagnostic.context() {
            HtmlTokenizerDiagnosticContext::InputPreprocessing => {
                DiagnosticContext::InputPreprocessing
            }
            HtmlTokenizerDiagnosticContext::Data => DiagnosticContext::Data,
            HtmlTokenizerDiagnosticContext::TagOpen => DiagnosticContext::TagOpen,
            HtmlTokenizerDiagnosticContext::EndTagOpen => DiagnosticContext::EndTagOpen,
            HtmlTokenizerDiagnosticContext::TagName => DiagnosticContext::TagName,
            HtmlTokenizerDiagnosticContext::BeforeAttributeName => {
                DiagnosticContext::BeforeAttributeName
            }
            HtmlTokenizerDiagnosticContext::AttributeName => DiagnosticContext::AttributeName,
            HtmlTokenizerDiagnosticContext::AfterAttributeName => {
                DiagnosticContext::AfterAttributeName
            }
            HtmlTokenizerDiagnosticContext::BeforeAttributeValue => {
                DiagnosticContext::BeforeAttributeValue
            }
            HtmlTokenizerDiagnosticContext::AttributeValueDoubleQuoted => {
                DiagnosticContext::AttributeValueDoubleQuoted
            }
            HtmlTokenizerDiagnosticContext::AttributeValueSingleQuoted => {
                DiagnosticContext::AttributeValueSingleQuoted
            }
            HtmlTokenizerDiagnosticContext::AttributeValueUnquoted => {
                DiagnosticContext::AttributeValueUnquoted
            }
            HtmlTokenizerDiagnosticContext::AfterAttributeValueQuoted => {
                DiagnosticContext::AfterAttributeValueQuoted
            }
            HtmlTokenizerDiagnosticContext::NamedCharacterReference => {
                DiagnosticContext::NamedCharacterReference
            }
            HtmlTokenizerDiagnosticContext::AmbiguousAmpersand => {
                DiagnosticContext::AmbiguousAmpersand
            }
            HtmlTokenizerDiagnosticContext::SelfClosingStartTag => {
                DiagnosticContext::SelfClosingStartTag
            }
        },
        handling: match diagnostic.handling() {
            HtmlTokenizerDiagnosticHandling::Continued => DiagnosticHandling::Continued,
            HtmlTokenizerDiagnosticHandling::Recovered(kind) => {
                DiagnosticHandling::Recovered(observe_recovery(kind))
            }
            HtmlTokenizerDiagnosticHandling::Stopped => DiagnosticHandling::Stopped,
        },
        subject: match diagnostic.subject() {
            HtmlTokenizerDiagnosticSubject::InputLocation => DiagnosticSubject::InputLocation,
            HtmlTokenizerDiagnosticSubject::EmittedToken { token_index } => {
                DiagnosticSubject::EmittedToken(*token_index)
            }
            HtmlTokenizerDiagnosticSubject::AbandonedInput { region } => {
                DiagnosticSubject::AbandonedInput(anchor_span(region))
            }
        },
    }
}

fn observe_recovery(kind: HtmlTokenizerRecoveryKind) -> RecoveryKind {
    match kind {
        HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter => {
            RecoveryKind::ReplacedNullWithReplacementCharacter
        }
        HtmlTokenizerRecoveryKind::EmittedLiteralMarkupPrefix => {
            RecoveryKind::EmittedLiteralMarkupPrefix
        }
        HtmlTokenizerRecoveryKind::IgnoredUnexpectedInput => {
            RecoveryKind::IgnoredUnexpectedInput
        }
        HtmlTokenizerRecoveryKind::StartedAttributeAtUnexpectedEqualsSign => {
            RecoveryKind::StartedAttributeAtUnexpectedEqualsSign
        }
        HtmlTokenizerRecoveryKind::ReconsumedInData => RecoveryKind::ReconsumedInData,
        HtmlTokenizerRecoveryKind::ReconsumedBeforeAttributeName => {
            RecoveryKind::ReconsumedBeforeAttributeName
        }
        HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue => {
            RecoveryKind::CompletedTagWithMissingAttributeValue
        }
        HtmlTokenizerRecoveryKind::PreservedDuplicateAttributeOccurrence => {
            RecoveryKind::PreservedDuplicateAttributeOccurrence
        }
        HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence => {
            RecoveryKind::PreservedEndTagLexicalEvidence
        }
        HtmlTokenizerRecoveryKind::AbandonedIncompleteTagAtEof => {
            RecoveryKind::AbandonedIncompleteTagAtEof
        }
    }
}

fn observe_completion(completion: &HtmlTokenizerCompletion) -> Completion {
    match completion {
        HtmlTokenizerCompletion::Complete => Completion::Complete,
        HtmlTokenizerCompletion::Incomplete(cause) => match cause {
            HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported) => {
                Completion::Unsupported {
                    capability: match unsupported.capability() {
                        HtmlTokenizerCapability::CharacterReference { context } => {
                            Capability::CharacterReference(match context {
                                HtmlCharacterReferenceContext::Data => {
                                    CharacterReferenceContext::Data
                                }
                                HtmlCharacterReferenceContext::AttributeValue => {
                                    CharacterReferenceContext::AttributeValue
                                }
                            })
                        }
                        HtmlTokenizerCapability::MarkupDeclaration => Capability::MarkupDeclaration,
                        HtmlTokenizerCapability::ProcessingInstruction => {
                            Capability::ProcessingInstruction
                        }
                        HtmlTokenizerCapability::BogusCommentRecovery => {
                            Capability::BogusCommentRecovery
                        }
                        HtmlTokenizerCapability::ContextDependentTokenizerMode { mode } => {
                            Capability::ContextDependentTokenizerMode(match mode {
                                HtmlTokenizerMode::Rcdata => TokenizerMode::Rcdata,
                                HtmlTokenizerMode::RawText => TokenizerMode::RawText,
                                HtmlTokenizerMode::ScriptData => TokenizerMode::ScriptData,
                                HtmlTokenizerMode::Plaintext => TokenizerMode::Plaintext,
                                HtmlTokenizerMode::Noscript => TokenizerMode::Noscript,
                            })
                        }
                        HtmlTokenizerCapability::TreeConstructionControlledState => {
                            Capability::TreeConstructionControlledState
                        }
                        HtmlTokenizerCapability::NumericCharacterReferenceInRcdata => {
                            Capability::NumericCharacterReferenceInRcdata
                        }
                        HtmlTokenizerCapability::RcdataNullRecovery => {
                            Capability::RcdataNullRecovery
                        }
                        HtmlTokenizerCapability::ForeignContentControl => {
                            Capability::ForeignContentControl
                        }
                    },
                    availability: match unsupported.availability() {
                        HtmlTokenizerCapabilityAvailability::Deferred => Availability::Deferred,
                        HtmlTokenizerCapabilityAvailability::Unsupported => {
                            Availability::Unsupported
                        }
                    },
                    trigger: match unsupported.trigger() {
                        HtmlTokenizerUnsupportedTrigger::Input(anchor) => {
                            UnsupportedTrigger::Input(anchor_span(anchor))
                        }
                        HtmlTokenizerUnsupportedTrigger::EmittedToken {
                            token_index,
                            boundary,
                        } => UnsupportedTrigger::EmittedToken {
                            token_index: *token_index,
                            boundary: anchor_span(boundary),
                        },
                    },
                }
            }
            HtmlTokenizerIncompleteCause::ResourceLimit(limit) => Completion::ResourceLimit {
                resource: match limit.resource() {
                    HtmlTokenizerResource::SourceBytes => Resource::SourceBytes,
                    HtmlTokenizerResource::TransitionSteps => Resource::TransitionSteps,
                    HtmlTokenizerResource::EmittedTokens => Resource::EmittedTokens,
                    HtmlTokenizerResource::Diagnostics => Resource::Diagnostics,
                    HtmlTokenizerResource::AttributesPerTag => Resource::AttributesPerTag,
                    HtmlTokenizerResource::RetainedInterpretedBytes => {
                        Resource::RetainedInterpretedBytes
                    }
                    HtmlTokenizerResource::TemporaryBufferBytes => {
                        Resource::TemporaryBufferBytes
                    }
                },
                limit: limit.limit(),
                attempted: limit.attempted(),
                at: anchor_span(limit.at()),
            },
            HtmlTokenizerIncompleteCause::InvalidConfiguration(failure) => {
                Completion::InvalidConfiguration(match failure {
                    HtmlTokenizerConfigurationFailure::ZeroTransitionStepLimit => {
                        ConfigurationFailure::ZeroTransitionStepLimit
                    }
                    HtmlTokenizerConfigurationFailure::ZeroEmittedTokenLimit => {
                        ConfigurationFailure::ZeroEmittedTokenLimit
                    }
                })
            }
            HtmlTokenizerIncompleteCause::InternalInvariantFailure(failure) => {
                Completion::InternalInvariantFailure(match failure {
                    HtmlTokenizerInvariantFailure::CursorState => InvariantFailure::CursorState,
                    HtmlTokenizerInvariantFailure::ReconsumeState => {
                        InvariantFailure::ReconsumeState
                    }
                    HtmlTokenizerInvariantFailure::BuilderState => InvariantFailure::BuilderState,
                    HtmlTokenizerInvariantFailure::EmissionOrder => InvariantFailure::EmissionOrder,
                    HtmlTokenizerInvariantFailure::SourceEvidenceConstruction => {
                        InvariantFailure::SourceEvidenceConstruction
                    }
                    HtmlTokenizerInvariantFailure::CounterOverflow => {
                        InvariantFailure::CounterOverflow
                    }
                    HtmlTokenizerInvariantFailure::RunAssembly => InvariantFailure::RunAssembly,
                })
            }
        },
    }
}

fn lexeme(source: &SourceText, anchor: &crate::SourceAnchor) -> Lexeme {
    let value = anchor_span(anchor);
    Lexeme {
        span: value,
        authored: source.as_str().as_bytes()[value.start..value.end].to_vec(),
    }
}

fn anchor_span(anchor: &crate::SourceAnchor) -> ByteSpan {
    ByteSpan::new(anchor.range().start(), anchor.range().end())
}
