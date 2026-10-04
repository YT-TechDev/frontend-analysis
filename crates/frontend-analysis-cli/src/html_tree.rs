//! Human-readable presentation of the `fa html-tree` report (#864).
//!
//! Every classification, relationship, order, and source anchor is read from
//! the owned Core report; this module only chooses wording. Source evidence is
//! rendered from retained anchors and nothing is searched or reconstructed.
//! Constructed identities are report-local creation ordinals, not source
//! positions and not browser DOM identity.

use std::fmt::Write as _;

use frontend_analysis_core::SourceAnchor;
use frontend_analysis_core::html::tree::{
    HtmlCharacterReferenceContext, HtmlTokenizerCapabilityAvailability,
    HtmlTokenizerDiagnosticCode, HtmlTokenizerMode, HtmlTokenizerUnsupportedCapability,
    HtmlTreeCompletion, HtmlTreeDiagnosticCode, HtmlTreeElementName, HtmlTreeIncompleteCause,
    HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeNodeProvenance, HtmlTreeRecovery, HtmlTreeReport,
    HtmlTreeResourceKind, HtmlTreeSynthesisCause, HtmlTreeUnsupportedCapability,
};

use crate::{escape_fragment, render_evidence};

pub(crate) fn render_report(source_bytes: usize, report: &HtmlTreeReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "capability: html-tree");
    let _ = writeln!(
        out,
        "source: id {}; {source_bytes} bytes",
        report.source_id().value()
    );
    let _ = writeln!(
        out,
        "profile: selected document construction; scripting disabled"
    );
    render_completion(&mut out, report.completion());
    let coverage = report.coverage();
    // Range only: the committed prefix can span the whole source, and its
    // fragment would only repeat the input on one very long line.
    let prefix = coverage.committed_prefix().range();
    let _ = writeln!(
        out,
        "coverage: committed authored prefix bytes {}..{}; processed tokens {}",
        prefix.start(),
        prefix.end(),
        coverage.processed_tokens()
    );
    let _ = writeln!(
        out,
        "tokenizer diagnostics: {}",
        report.tokenizer_diagnostics().len()
    );
    let _ = writeln!(out, "tree diagnostics: {}", report.tree_diagnostics().len());
    let _ = writeln!(out, "nodes: {}", report.nodes().len());

    let _ = writeln!(out);
    render_tree(&mut out, report);

    if !report.tokenizer_diagnostics().is_empty() {
        let _ = writeln!(out);
        for (index, diagnostic) in report.tokenizer_diagnostics().iter().enumerate() {
            let _ = writeln!(
                out,
                "tokenizer diagnostic {}: {}",
                index + 1,
                tokenizer_diagnostic_code(diagnostic.code())
            );
            let _ = writeln!(
                out,
                "  location: {}",
                render_evidence(diagnostic.location())
            );
        }
    }
    if !report.tree_diagnostics().is_empty() {
        let _ = writeln!(out);
        for (index, diagnostic) in report.tree_diagnostics().iter().enumerate() {
            let _ = writeln!(
                out,
                "tree diagnostic {}: {}",
                index + 1,
                tree_diagnostic_code(diagnostic.code())
            );
            let _ = writeln!(out, "  recovery: {}", recovery(diagnostic.recovery()));
            let _ = writeln!(out, "  trigger: {}", render_optional(diagnostic.trigger()));
        }
    }
    out
}

fn render_optional(anchor: Option<&SourceAnchor>) -> String {
    match anchor {
        Some(anchor) => render_evidence(anchor),
        None => "none (no authored boundary)".to_owned(),
    }
}

fn render_completion(out: &mut String, completion: &HtmlTreeCompletion) {
    match completion {
        HtmlTreeCompletion::Complete => {
            let _ = writeln!(out, "completion: complete");
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(unsupported)) => {
            let _ = writeln!(
                out,
                "completion: incomplete; tree unsupported: {}",
                tree_capability(unsupported.capability())
            );
            let _ = writeln!(out, "  trigger: {}", render_optional(unsupported.trigger()));
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TokenizerUnsupported(
            unsupported,
        )) => {
            let availability = match unsupported.availability() {
                HtmlTokenizerCapabilityAvailability::Deferred => "deferred",
                HtmlTokenizerCapabilityAvailability::Unsupported => "unsupported",
            };
            let _ = writeln!(
                out,
                "completion: incomplete; tokenizer unsupported: {} ({availability})",
                tokenizer_capability(unsupported.capability())
            );
            let _ = writeln!(out, "  trigger: {}", render_evidence(unsupported.trigger()));
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) => {
            let _ = writeln!(
                out,
                "completion: incomplete; resource limited: {} (limit {}, attempted {})",
                resource_kind(limit.kind()),
                limit.limit(),
                limit.attempted()
            );
            let _ = writeln!(out, "  at: {}", render_evidence(limit.at()));
        }
    }
}

/// Renders the final parent/child structure iteratively so rendering depth
/// is not bounded by the process stack.
fn render_tree(out: &mut String, report: &HtmlTreeReport) {
    let mut pending = vec![(report.root(), 0usize)];
    while let Some((id, depth)) = pending.pop() {
        let Some(node) = report.node(id) else {
            let _ = writeln!(out, "{}<missing node {}>", "  ".repeat(depth), id.value());
            continue;
        };
        render_node(out, node, depth);
        for child in node.children().iter().rev() {
            pending.push((*child, depth + 1));
        }
    }
}

fn render_node(out: &mut String, node: &HtmlTreeNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let id = node.id().value();
    match node.kind() {
        HtmlTreeNodeKind::Document => {
            let _ = writeln!(out, "{indent}#{id} document");
            let _ = writeln!(out, "{indent}    authored evidence: none");
        }
        HtmlTreeNodeKind::Element(element) => {
            let _ = writeln!(
                out,
                "{indent}#{id} element {}",
                element_name(element.name())
            );
            match element.provenance() {
                HtmlTreeNodeProvenance::None => {
                    let _ = writeln!(out, "{indent}    authored evidence: none");
                }
                HtmlTreeNodeProvenance::AuthoredStartTag { complete, raw_name } => {
                    let _ = writeln!(
                        out,
                        "{indent}    authored start tag: {}",
                        render_evidence(complete)
                    );
                    let _ = writeln!(
                        out,
                        "{indent}    authored raw name: {}",
                        render_evidence(raw_name)
                    );
                }
                HtmlTreeNodeProvenance::Synthesized(cause) => {
                    let _ = writeln!(out, "{indent}    authored evidence: none");
                    let _ = writeln!(out, "{indent}    synthesized: {}", synthesis_cause(*cause));
                }
            }
        }
        HtmlTreeNodeKind::Text(text) => {
            let _ = writeln!(
                out,
                "{indent}#{id} text \"{}\"",
                escape_fragment(text.interpreted())
            );
            for (index, contribution) in text.contributions().iter().enumerate() {
                let _ = writeln!(
                    out,
                    "{indent}    contribution {}: source {}; interpreted \"{}\"",
                    index + 1,
                    render_evidence(contribution.source()),
                    escape_fragment(contribution.interpreted())
                );
            }
        }
    }
}

fn element_name(name: HtmlTreeElementName) -> &'static str {
    match name {
        HtmlTreeElementName::Html => "html",
        HtmlTreeElementName::Head => "head",
        HtmlTreeElementName::Body => "body",
        HtmlTreeElementName::Div => "div",
        HtmlTreeElementName::Section => "section",
        HtmlTreeElementName::Paragraph => "p",
        HtmlTreeElementName::Style => "style",
        HtmlTreeElementName::Title => "title",
    }
}

fn synthesis_cause(cause: HtmlTreeSynthesisCause) -> &'static str {
    match cause {
        HtmlTreeSynthesisCause::ImpliedByDocumentStructure => "implied by document structure",
        HtmlTreeSynthesisCause::UnmatchedParagraphEndTag => "unmatched paragraph end tag",
    }
}

fn resource_kind(kind: HtmlTreeResourceKind) -> &'static str {
    match kind {
        HtmlTreeResourceKind::SourceBytes => "source bytes",
        HtmlTreeResourceKind::TransitionSteps => "transition steps",
        HtmlTreeResourceKind::EmittedTokens => "emitted tokens",
        HtmlTreeResourceKind::TokenizerDiagnostics => "tokenizer diagnostics",
        HtmlTreeResourceKind::AttributesPerTag => "attributes per tag",
        HtmlTreeResourceKind::RetainedInterpretedBytes => "retained interpreted bytes",
        HtmlTreeResourceKind::TemporaryBufferBytes => "temporary buffer bytes",
    }
}

fn tree_capability(capability: HtmlTreeUnsupportedCapability) -> &'static str {
    match capability {
        HtmlTreeUnsupportedCapability::NonShellElementTag => "non-shell element tag",
        HtmlTreeUnsupportedCapability::ShellTagAttribute => "shell tag attribute",
        HtmlTreeUnsupportedCapability::SelfClosingShellTag => "self-closing shell tag",
        HtmlTreeUnsupportedCapability::WhitespaceSensitiveCharacterData => {
            "whitespace-sensitive character data"
        }
        HtmlTreeUnsupportedCapability::UnprovedCharacterDataPosition => {
            "unproved character data position"
        }
        HtmlTreeUnsupportedCapability::UnprovedShellStartTagPosition => {
            "unproved shell start tag position"
        }
        HtmlTreeUnsupportedCapability::UnprovedShellEndTagPosition => {
            "unproved shell end tag position"
        }
        HtmlTreeUnsupportedCapability::SelectedOrdinaryTagOutsideInBody => {
            "selected ordinary tag outside in-body"
        }
        HtmlTreeUnsupportedCapability::ShellTagWithOpenSelectedOrdinaryElement => {
            "shell tag with open selected ordinary element"
        }
        HtmlTreeUnsupportedCapability::SelectedOrdinaryTagAttribute => {
            "selected ordinary tag attribute"
        }
        HtmlTreeUnsupportedCapability::SelfClosingSelectedOrdinaryTag => {
            "self-closing selected ordinary tag"
        }
        HtmlTreeUnsupportedCapability::ParagraphTagOutsideInBody => "paragraph tag outside in-body",
        HtmlTreeUnsupportedCapability::ParagraphTagAttribute => "paragraph tag attribute",
        HtmlTreeUnsupportedCapability::SelfClosingParagraphTag => "self-closing paragraph tag",
        HtmlTreeUnsupportedCapability::ShellTagWithOpenParagraphElement => {
            "shell tag with open paragraph element"
        }
        HtmlTreeUnsupportedCapability::StyleTagAttribute => "style tag attribute",
        HtmlTreeUnsupportedCapability::SelfClosingStyleTag => "self-closing style tag",
        HtmlTreeUnsupportedCapability::StyleTagOutsideSelectedLifecycle => {
            "style tag outside selected lifecycle"
        }
        HtmlTreeUnsupportedCapability::TitleTagAttribute => "title tag attribute",
        HtmlTreeUnsupportedCapability::SelfClosingTitleTag => "self-closing title tag",
        HtmlTreeUnsupportedCapability::TitleTagOutsideSelectedLifecycle => {
            "title tag outside selected lifecycle"
        }
    }
}

fn tokenizer_capability(capability: HtmlTokenizerUnsupportedCapability) -> String {
    match capability {
        HtmlTokenizerUnsupportedCapability::CharacterReference { context } => {
            let context = match context {
                HtmlCharacterReferenceContext::Data => "data",
                HtmlCharacterReferenceContext::AttributeValue => "attribute value",
            };
            format!("character reference in {context}")
        }
        HtmlTokenizerUnsupportedCapability::MarkupDeclaration => "markup declaration".to_owned(),
        HtmlTokenizerUnsupportedCapability::ProcessingInstruction => {
            "processing instruction".to_owned()
        }
        HtmlTokenizerUnsupportedCapability::BogusCommentRecovery => {
            "bogus comment recovery".to_owned()
        }
        HtmlTokenizerUnsupportedCapability::ContextDependentTokenizerMode { mode } => {
            let mode = match mode {
                HtmlTokenizerMode::Rcdata => "RCDATA",
                HtmlTokenizerMode::RawText => "RAWTEXT",
                HtmlTokenizerMode::ScriptData => "script data",
                HtmlTokenizerMode::Plaintext => "PLAINTEXT",
                HtmlTokenizerMode::Noscript => "noscript",
            };
            format!("context-dependent tokenizer mode {mode}")
        }
        HtmlTokenizerUnsupportedCapability::TreeConstructionControlledState => {
            "tree-construction-controlled state".to_owned()
        }
        HtmlTokenizerUnsupportedCapability::ForeignContentControl => {
            "foreign content control".to_owned()
        }
        HtmlTokenizerUnsupportedCapability::NumericCharacterReferenceInData => {
            "numeric character reference in data".to_owned()
        }
        HtmlTokenizerUnsupportedCapability::NumericCharacterReferenceInRcdata => {
            "numeric character reference in RCDATA".to_owned()
        }
        HtmlTokenizerUnsupportedCapability::RcdataNullRecovery => "RCDATA NUL recovery".to_owned(),
    }
}

fn tokenizer_diagnostic_code(code: HtmlTokenizerDiagnosticCode) -> &'static str {
    match code {
        HtmlTokenizerDiagnosticCode::NoncharacterInInputStream => "noncharacter in input stream",
        HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream => {
            "control character in input stream"
        }
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter => "unexpected null character",
        HtmlTokenizerDiagnosticCode::EofBeforeTagName => "EOF before tag name",
        HtmlTokenizerDiagnosticCode::InvalidFirstCharacterOfTagName => {
            "invalid first character of tag name"
        }
        HtmlTokenizerDiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName => {
            "unexpected question mark instead of tag name"
        }
        HtmlTokenizerDiagnosticCode::MissingEndTagName => "missing end tag name",
        HtmlTokenizerDiagnosticCode::EofInTag => "EOF in tag",
        HtmlTokenizerDiagnosticCode::UnexpectedEqualsSignBeforeAttributeName => {
            "unexpected equals sign before attribute name"
        }
        HtmlTokenizerDiagnosticCode::UnexpectedCharacterInAttributeName => {
            "unexpected character in attribute name"
        }
        HtmlTokenizerDiagnosticCode::MissingAttributeValue => "missing attribute value",
        HtmlTokenizerDiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue => {
            "unexpected character in unquoted attribute value"
        }
        HtmlTokenizerDiagnosticCode::MissingWhitespaceBetweenAttributes => {
            "missing whitespace between attributes"
        }
        HtmlTokenizerDiagnosticCode::UnexpectedSolidusInTag => "unexpected solidus in tag",
        HtmlTokenizerDiagnosticCode::DuplicateAttribute => "duplicate attribute",
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes => "end tag with attributes",
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus => "end tag with trailing solidus",
        HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference => {
            "missing semicolon after character reference"
        }
        HtmlTokenizerDiagnosticCode::UnknownNamedCharacterReference => {
            "unknown named character reference"
        }
        HtmlTokenizerDiagnosticCode::AbsenceOfDigitsInNumericCharacterReference => {
            "absence of digits in numeric character reference"
        }
        HtmlTokenizerDiagnosticCode::NullCharacterReference => "null character reference",
        HtmlTokenizerDiagnosticCode::CharacterReferenceOutsideUnicodeRange => {
            "character reference outside Unicode range"
        }
        HtmlTokenizerDiagnosticCode::SurrogateCharacterReference => "surrogate character reference",
        HtmlTokenizerDiagnosticCode::NoncharacterCharacterReference => {
            "noncharacter character reference"
        }
        HtmlTokenizerDiagnosticCode::ControlCharacterReference => "control character reference",
    }
}

fn tree_diagnostic_code(code: HtmlTreeDiagnosticCode) -> &'static str {
    match code {
        HtmlTreeDiagnosticCode::MissingDoctype => "missing doctype",
        HtmlTreeDiagnosticCode::DuplicateHeadStartTag => "duplicate head start tag",
        HtmlTreeDiagnosticCode::DuplicateBodyStartTag => "duplicate body start tag",
        HtmlTreeDiagnosticCode::AfterBodyCharacterData => "after-body character data",
        HtmlTreeDiagnosticCode::NullCharacterInBody => "null character in in-body",
        HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements => {
            "body end tag with open selected ordinary elements"
        }
        HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements => {
            "html end tag with open selected ordinary elements"
        }
        HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag => {
            "unmatched selected ordinary end tag"
        }
        HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag => {
            "misnested selected ordinary end tag"
        }
        HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile => {
            "open selected ordinary element at end of file"
        }
        HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag => "unmatched paragraph end tag",
        HtmlTreeDiagnosticCode::StyleEndOfFileInText => "style end of file in text",
        HtmlTreeDiagnosticCode::TitleEndOfFileInText => "title end of file in text",
    }
}

fn recovery(recovery: HtmlTreeRecovery) -> &'static str {
    match recovery {
        HtmlTreeRecovery::ContinuedInQuirksDocumentMode => "continued in quirks document mode",
        HtmlTreeRecovery::DuplicateShellStartTagProducedNoNode => {
            "duplicate shell start tag produced no node"
        }
        HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken => {
            "switched to in-body and reprocessed same token"
        }
        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements => {
            "switched to after-body preserving open elements"
        }
        HtmlTreeRecovery::IgnoredToken => "ignored token",
        HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget => {
            "popped intervening selected ordinary elements and closed target"
        }
        HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements => {
            "stopped parsing with open selected ordinary elements"
        }
        HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt => {
            "synthesized paragraph element and closed it"
        }
        HtmlTreeRecovery::PoppedStyleAtEndOfFileAndRestoredInHead => {
            "popped style at end of file and restored in head"
        }
        HtmlTreeRecovery::PoppedTitleAtEndOfFileAndRestoredInHead => {
            "popped title at end of file and restored in head"
        }
    }
}
