//! Candidate-independent validation of the public `html::tree` facade.
//!
//! Expected values are authored by hand from the accepted tree/tokenizer
//! contracts and golds (for example the accepted `<title>a&amp;b</title>`
//! contribution gold and the accepted tokenizer resource corpus) and from
//! hand-counted source byte offsets. None is captured from the facade.

use super::*;
use crate::html::tokenizer::resource::HtmlTokenizerResourceLimit;
use crate::{SourceId, SourceText};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(0), text.to_owned())
}

fn analyze_text(text: &str) -> HtmlTreeReport {
    analyze_selected_document_tree(&source(text)).expect("analysis returns a report")
}

fn span(anchor: &SourceAnchor) -> (usize, usize, String) {
    let range = anchor.range();
    (range.start(), range.end(), anchor.fragment().to_owned())
}

fn at(start: usize, end: usize, fragment: &str) -> (usize, usize, String) {
    (start, end, fragment.to_owned())
}

/// Final tree order as `depth:id:label` lines, walked only through explicit
/// parent/child identities.
fn outline(report: &HtmlTreeReport) -> Vec<String> {
    fn walk(report: &HtmlTreeReport, id: HtmlTreeNodeId, depth: usize, out: &mut Vec<String>) {
        let node = report.node(id).expect("child identity resolves");
        let label = match node.kind() {
            HtmlTreeNodeKind::Document => "document".to_owned(),
            HtmlTreeNodeKind::Element(element) => format!("{:?}", element.name()).to_lowercase(),
            HtmlTreeNodeKind::DocumentType(_) => "doctype".to_owned(),
            HtmlTreeNodeKind::Text(text) => format!("text {:?}", text.interpreted()),
        };
        out.push(format!("{depth}:{}:{label}", id.value()));
        for child in node.children() {
            walk(report, *child, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    walk(report, report.root(), 0, &mut out);
    out
}

fn element(report: &HtmlTreeReport, id: u32) -> &HtmlTreeElement {
    let node = report
        .nodes()
        .iter()
        .find(|node| node.id().value() == id)
        .expect("node exists");
    match node.kind() {
        HtmlTreeNodeKind::Element(element) => element,
        other => panic!("node {id} is not an element: {other:?}"),
    }
}

fn text(report: &HtmlTreeReport, id: u32) -> &HtmlTreeText {
    let node = report
        .nodes()
        .iter()
        .find(|node| node.id().value() == id)
        .expect("node exists");
    match node.kind() {
        HtmlTreeNodeKind::Text(text) => text,
        other => panic!("node {id} is not text: {other:?}"),
    }
}

fn authored_spans(element: &HtmlTreeElement) -> ((usize, usize, String), (usize, usize, String)) {
    match element.provenance() {
        HtmlTreeNodeProvenance::AuthoredStartTag { complete, raw_name } => {
            (span(complete), span(raw_name))
        }
        other => panic!("expected authored provenance, got {other:?}"),
    }
}

fn assert_synthesized(element: &HtmlTreeElement, cause: HtmlTreeSynthesisCause) {
    match element.provenance() {
        HtmlTreeNodeProvenance::Synthesized(actual) => assert_eq!(*actual, cause),
        other => panic!("expected synthesized provenance, got {other:?}"),
    }
}

fn assert_complete(report: &HtmlTreeReport) {
    assert!(
        matches!(report.completion(), HtmlTreeCompletion::Complete),
        "{:?}",
        report.completion()
    );
}

#[test]
fn empty_source_is_a_complete_document_with_synthesized_shell() {
    let report = analyze_text("");

    assert_complete(&report);
    assert_eq!(report.source_id(), SourceId::new(0));
    assert_eq!(
        outline(&report),
        ["0:0:document", "1:1:html", "2:2:head", "2:3:body"]
    );
    assert_eq!(report.root().value(), 0);
    assert!(matches!(
        report.node(report.root()).unwrap().kind(),
        HtmlTreeNodeKind::Document
    ));
    for (id, name) in [
        (1, HtmlTreeElementName::Html),
        (2, HtmlTreeElementName::Head),
        (3, HtmlTreeElementName::Body),
    ] {
        let shell = element(&report, id);
        assert_eq!(shell.name(), name);
        assert_synthesized(shell, HtmlTreeSynthesisCause::ImpliedByDocumentStructure);
    }
    // The EOF token is the only token processed over an empty source.
    assert_eq!(span(report.coverage().committed_prefix()), at(0, 0, ""));
    assert_eq!(report.coverage().processed_tokens(), 1);
    assert!(report.tokenizer_diagnostics().is_empty());
    let [diagnostic] = report.tree_diagnostics() else {
        panic!("expected exactly the missing-doctype diagnostic");
    };
    assert_eq!(diagnostic.code(), HtmlTreeDiagnosticCode::MissingDoctype);
    assert_eq!(
        diagnostic.recovery(),
        HtmlTreeRecovery::ContinuedInQuirksDocumentMode
    );
    // An end-of-file trigger has no authored boundary and none is invented.
    assert!(diagnostic.trigger().is_none());
}

#[test]
fn document_root_provenance_is_semantic_absence() {
    let report = analyze_text("");
    let root = report.node(report.root()).unwrap();
    assert!(root.parent().is_none());
    assert!(matches!(root.kind(), HtmlTreeNodeKind::Document));
}

#[test]
fn explicit_shell_tags_retain_authored_complete_and_raw_name_anchors() {
    // <html>0..6 <head>6..12 </head>12..19 <body>19..25 </body>25..32
    // </html>32..39
    let report = analyze_text("<html><head></head><body></body></html>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        ["0:0:document", "1:1:html", "2:2:head", "2:3:body"]
    );
    assert_eq!(
        authored_spans(element(&report, 1)),
        (at(0, 6, "<html>"), at(1, 5, "html"))
    );
    assert_eq!(
        authored_spans(element(&report, 2)),
        (at(6, 12, "<head>"), at(7, 11, "head"))
    );
    assert_eq!(
        authored_spans(element(&report, 3)),
        (at(19, 25, "<body>"), at(20, 24, "body"))
    );
    assert_eq!(report.coverage().processed_tokens(), 7);
    assert_eq!(span(report.coverage().committed_prefix()).1, 39);
    let [diagnostic] = report.tree_diagnostics() else {
        panic!("expected exactly the missing-doctype diagnostic");
    };
    assert_eq!(diagnostic.code(), HtmlTreeDiagnosticCode::MissingDoctype);
    assert_eq!(span(diagnostic.trigger().unwrap()), at(0, 6, "<html>"));
}

#[test]
fn omitted_shell_keeps_synthesis_distinct_from_authored_child() {
    // <body>0..6 <div>6..11 </div>11..17
    let report = analyze_text("<body><div></div>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:html",
            "2:2:head",
            "2:3:body",
            "3:4:div"
        ]
    );
    assert_synthesized(
        element(&report, 1),
        HtmlTreeSynthesisCause::ImpliedByDocumentStructure,
    );
    assert_synthesized(
        element(&report, 2),
        HtmlTreeSynthesisCause::ImpliedByDocumentStructure,
    );
    assert_eq!(
        authored_spans(element(&report, 3)),
        (at(0, 6, "<body>"), at(1, 5, "body"))
    );
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Div);
    assert_eq!(
        authored_spans(element(&report, 4)),
        (at(6, 11, "<div>"), at(7, 10, "div"))
    );
    assert_eq!(report.coverage().processed_tokens(), 4);
}

#[test]
fn authored_raw_name_spelling_is_distinct_from_the_interpreted_element_name() {
    let report = analyze_text("<body><DIV></DIV>");

    assert_complete(&report);
    let div = element(&report, 4);
    assert_eq!(div.name(), HtmlTreeElementName::Div);
    assert_eq!(authored_spans(div), (at(6, 11, "<DIV>"), at(7, 10, "DIV")));
}

#[test]
fn nested_selected_elements_follow_final_parent_child_order() {
    // <body>0..6 <div>6..11 <section>11..20 <p>20..23 t23..24 </p>24..28
    // </section>28..38 </div>38..44 </body>44..51
    let report = analyze_text("<body><div><section><p>t</p></section></div></body>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:html",
            "2:2:head",
            "2:3:body",
            "3:4:div",
            "4:5:section",
            "5:6:paragraph",
            "6:7:text \"t\"",
        ]
    );
    let node = |id: u32| {
        report
            .nodes()
            .iter()
            .find(|node| node.id().value() == id)
            .unwrap()
    };
    assert_eq!(node(7).parent().map(|id| id.value()), Some(6));
    assert_eq!(node(6).parent().map(|id| id.value()), Some(5));
    assert_eq!(node(1).parent().map(|id| id.value()), Some(0));
    assert_eq!(
        authored_spans(element(&report, 5)),
        (at(11, 20, "<section>"), at(12, 19, "section"))
    );
    assert_eq!(element(&report, 6).name(), HtmlTreeElementName::Paragraph);
    assert_eq!(
        authored_spans(element(&report, 6)),
        (at(20, 23, "<p>"), at(21, 22, "p"))
    );
    let t = text(&report, 7);
    assert_eq!(t.interpreted(), "t");
    let [contribution] = t.contributions() else {
        panic!("expected one contribution");
    };
    assert_eq!(span(contribution.source()), at(23, 24, "t"));
    assert_eq!(contribution.interpreted(), "t");
}

#[test]
fn unmatched_paragraph_end_tag_synthesizes_a_paragraph_without_authored_anchors() {
    // <body>0..6 </p>6..10
    let report = analyze_text("<body></p>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:html",
            "2:2:head",
            "2:3:body",
            "3:4:paragraph"
        ]
    );
    assert_synthesized(
        element(&report, 4),
        HtmlTreeSynthesisCause::UnmatchedParagraphEndTag,
    );
    let diagnostic = report
        .tree_diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code() == HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag)
        .expect("unmatched paragraph end tag diagnostic");
    assert_eq!(
        diagnostic.recovery(),
        HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt
    );
    // The trigger is the authored end tag; it is evidence for the diagnostic,
    // never an authored origin of the synthesized element.
    assert_eq!(span(diagnostic.trigger().unwrap()), at(6, 10, "</p>"));
}

#[test]
fn style_rawtext_keeps_markup_like_text_as_one_text_node() {
    // <head>0..6 <style>6..13 a<b>13..17 </style>17..25 </head>25..32
    let report = analyze_text("<head><style>a<b></style></head>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:html",
            "2:2:head",
            "3:3:style",
            "4:4:text \"a<b>\"",
            // End of file completes the implied shell with a body.
            "2:5:body",
        ]
    );
    assert_eq!(
        authored_spans(element(&report, 3)),
        (at(6, 13, "<style>"), at(7, 12, "style"))
    );
    let rawtext = text(&report, 4);
    assert_eq!(rawtext.interpreted(), "a<b>");
    // Ordered, contiguous authored contributions cover exactly 13..17.
    let mut cursor = 13;
    let mut rebuilt = String::new();
    for contribution in rawtext.contributions() {
        let (start, end, fragment) = span(contribution.source());
        assert_eq!(start, cursor);
        assert_eq!(fragment, contribution.interpreted());
        rebuilt.push_str(contribution.interpreted());
        cursor = end;
    }
    assert_eq!(cursor, 17);
    assert_eq!(rebuilt, "a<b>");
}

#[test]
fn title_rcdata_named_reference_keeps_interpreted_text_and_ordered_contributions() {
    // Accepted TC-S10 gold for `<title>a&amp;b</title>`: one coalesced text
    // "a&b" with authored contributions 7..8 "a", 8..13 "&amp;" -> "&",
    // 13..14 "b".
    let report = analyze_text("<title>a&amp;b</title>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:html",
            "2:2:head",
            "3:3:title",
            "4:4:text \"a&b\"",
            // End of file completes the implied shell with a body.
            "2:5:body",
        ]
    );
    assert_eq!(
        authored_spans(element(&report, 3)),
        (at(0, 7, "<title>"), at(1, 6, "title"))
    );
    let title_text = text(&report, 4);
    assert_eq!(title_text.interpreted(), "a&b");
    let contributions: Vec<_> = title_text
        .contributions()
        .iter()
        .map(|contribution| (span(contribution.source()), contribution.interpreted()))
        .collect();
    assert_eq!(
        contributions,
        [
            (at(7, 8, "a"), "a"),
            (at(8, 13, "&amp;"), "&"),
            (at(13, 14, "b"), "b"),
        ]
    );
    // Authored syntax and interpreted value are different things.
    assert_ne!(title_text.contributions()[1].source().fragment(), "&");
}

#[test]
fn complete_report_with_tokenizer_diagnostic_remains_complete() {
    // Input-preprocessing diagnostics keep the offending source scalar as
    // their exact location: `<title>` is 0..7, `a` is 7..8, and the
    // U+0001 control character is 8..9.
    let report = analyze_text("<title>a\u{1}b</title>");

    assert_complete(&report);
    let [diagnostic] = report.tokenizer_diagnostics() else {
        panic!("expected exactly one tokenizer diagnostic");
    };
    assert_eq!(
        diagnostic.code(),
        HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream
    );
    assert_eq!(span(diagnostic.location()), at(8, 9, "\u{1}"));
    // Tokenizer and tree diagnostics stay in separate sequences.
    assert!(
        report
            .tree_diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.code() == HtmlTreeDiagnosticCode::MissingDoctype)
    );
}

#[test]
fn tree_unsupported_capability_retains_its_authored_trigger() {
    // <body>0..6 <span>6..12
    let report = analyze_text("<body><span>");

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(unsupported)) =
        report.completion()
    else {
        panic!("expected tree unsupported, got {:?}", report.completion());
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTreeUnsupportedCapability::NonShellElementTag
    );
    assert_eq!(span(unsupported.trigger().unwrap()), at(6, 12, "<span>"));
    // The unsupported trigger is not an authored origin of any node.
    assert_eq!(
        outline(&report),
        ["0:0:document", "1:1:html", "2:2:head", "2:3:body"]
    );
    assert_eq!(
        span(report.coverage().committed_prefix()),
        at(0, 6, "<body>")
    );
    assert_eq!(report.coverage().processed_tokens(), 1);
}

#[test]
fn tokenizer_unsupported_capability_retains_availability_and_trigger() {
    // UNSUP-003 (`<!x>`: markup declaration at 0..2, Deferred), shifted by
    // the 6-byte `<body>` prefix. UNSUP-001 is a retired historical ID: Numeric
    // character references are supported (TOK-013).
    let report = analyze_text("<body><!x>");
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TokenizerUnsupported(unsupported)) =
        report.completion()
    else {
        panic!(
            "expected tokenizer unsupported, got {:?}",
            report.completion()
        );
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTokenizerUnsupportedCapability::MarkupDeclaration
    );
    assert_eq!(
        unsupported.availability(),
        HtmlTokenizerCapabilityAvailability::Deferred
    );
    assert_eq!(span(unsupported.trigger()), at(6, 8, "<!"));
}

#[test]
fn data_named_reference_keeps_interpreted_text_and_ordered_contributions() {
    let report = analyze_text("<body>a&amp;b</body>");

    assert_complete(&report);
    let body_text = text(&report, 4);
    assert_eq!(body_text.interpreted(), "a&b");
    let contributions: Vec<_> = body_text
        .contributions()
        .iter()
        .map(|contribution| (span(contribution.source()), contribution.interpreted()))
        .collect();
    assert_eq!(
        contributions,
        [
            (at(6, 7, "a"), "a"),
            (at(7, 12, "&amp;"), "&"),
            (at(12, 13, "b"), "b"),
        ]
    );
}

#[test]
fn data_numeric_reference_keeps_interpreted_text_and_authored_source() {
    let report = analyze_text("<body>a&#38;b&#65;</body>");

    assert_complete(&report);
    let body_text = text(&report, 4);
    assert_eq!(body_text.interpreted(), "a&bA");
    let contributions: Vec<_> = body_text
        .contributions()
        .iter()
        .map(|contribution| (span(contribution.source()), contribution.interpreted()))
        .collect();
    assert_eq!(
        contributions,
        [
            (at(6, 7, "a"), "a"),
            (at(7, 12, "&#38;"), "&"),
            (at(12, 13, "b"), "b"),
            (at(13, 18, "&#65;"), "A"),
        ]
    );
}

#[test]
fn numeric_reference_is_no_longer_the_unsupported_capability() {
    // The unsupported variants remain public API, but the selected paths stop
    // emitting them; a lower-layer sentinel is `<!xx>`.
    let report = analyze_text("<body>&#65;");
    assert_complete(&report);
    let report = analyze_text("<body><!xx>");
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TokenizerUnsupported(unsupported)) =
        report.completion()
    else {
        panic!("expected tokenizer unsupported");
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTokenizerUnsupportedCapability::MarkupDeclaration
    );
}

#[test]
fn source_bytes_boundary_is_36864_admitted_and_36865_refused() {
    let admitted = analyze_text(&format!("<body>{}", "x".repeat(36_864 - 6)));
    assert_complete(&admitted);
    assert_eq!(span(admitted.coverage().committed_prefix()).1, 36_864);

    let refused = analyze_text(&format!("<body>{}", "x".repeat(36_865 - 6)));
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        refused.completion()
    else {
        panic!("expected resource limited, got {:?}", refused.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::SourceBytes);
    assert_eq!(limit.limit(), 36_864);
    assert_eq!(limit.attempted(), 36_865);
    // Over-limit source commits zero tokenizer work.
    assert_eq!(span(limit.at()), at(0, 0, ""));
    assert_eq!(span(refused.coverage().committed_prefix()), at(0, 0, ""));
    assert_eq!(refused.coverage().processed_tokens(), 0);
    assert_eq!(outline(&refused), ["0:0:document"]);
}

#[test]
fn emitted_token_limit_commits_6144_and_refuses_attempted_6145() {
    // `<body>` plus 3,071 `<div></div>` pairs plus one `<div>` is
    // 1 + 6,142 + 1 = 6,144 tokens; the EOF token is attempted token 6,145.
    // Source length: 6 + 11 * 3,071 + 5 = 33,792 bytes (within 36,864).
    let source_text = format!("<body>{}<div>", "<div></div>".repeat(3_071));
    assert_eq!(source_text.len(), 33_792);

    let report = analyze_text(&source_text);

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::EmittedTokens);
    assert_eq!(limit.limit(), 6_144);
    assert_eq!(limit.attempted(), 6_145);
    assert_eq!(span(limit.at()), at(33_792, 33_792, ""));
    assert_eq!(report.coverage().processed_tokens(), 6_144);
}

#[test]
fn attributes_per_tag_limit_admits_one_and_refuses_the_second() {
    // Accepted RES-005: `<a x y>` refuses attempted attribute 2 (limit 1) at
    // the `y` offset. Here `<body>` shifts `<div x y>` so `y` is at 13.
    let report = analyze_text("<body><div x y>");

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::AttributesPerTag);
    assert_eq!(limit.limit(), 1);
    assert_eq!(limit.attempted(), 2);
    assert_eq!(span(limit.at()), at(13, 13, ""));
}

#[test]
fn tokenizer_diagnostic_limit_refuses_the_257th_diagnostic() {
    // Accepted RES-004: the refused diagnostic's location is the offending
    // character itself. 256 NULs are admitted; the 257th NUL is at
    // 6 + 256 = 262..263.
    let report = analyze_text(&format!("<body>{}", "\0".repeat(257)));

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::TokenizerDiagnostics);
    assert_eq!(limit.limit(), 256);
    assert_eq!(limit.attempted(), 257);
    assert_eq!(span(limit.at()), at(262, 263, "\0"));
    assert_eq!(report.tokenizer_diagnostics().len(), 256);
    // The 256 admitted NULs are each ignored by the tree: one projected
    // diagnostic per NUL, no text node, and lower-layer incompleteness is not
    // upgraded.
    let ignored = report
        .tree_diagnostics()
        .iter()
        .filter(|d| d.code() == HtmlTreeDiagnosticCode::NullCharacterInBody)
        .count();
    assert_eq!(ignored, 256);
    assert!(
        report
            .nodes()
            .iter()
            .all(|node| !matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
    );
}

#[test]
fn authored_data_nul_projects_an_ignored_token_tree_diagnostic() {
    // <body>0..6 NUL 6..7 : the NUL is exact tokenizer evidence, the tree
    // ignores it, and no text node or U+FFFD exists.
    let report = analyze_text("<body>\0");
    assert!(matches!(report.completion(), HtmlTreeCompletion::Complete));
    let [tokenizer_diagnostic] = report.tokenizer_diagnostics() else {
        panic!("expected exactly the tokenizer NUL diagnostic");
    };
    assert_eq!(
        tokenizer_diagnostic.code(),
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter
    );
    assert_eq!(span(tokenizer_diagnostic.location()), at(6, 7, "\0"));
    let null = report
        .tree_diagnostics()
        .iter()
        .find(|d| d.code() == HtmlTreeDiagnosticCode::NullCharacterInBody)
        .expect("tree NUL diagnostic");
    assert_eq!(null.recovery(), HtmlTreeRecovery::IgnoredToken);
    assert_eq!(
        span(null.trigger().expect("authored trigger")),
        at(6, 7, "\0")
    );
    assert!(
        report
            .nodes()
            .iter()
            .all(|node| !matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
    );
}

#[test]
fn after_after_body_character_recovery_projects_its_own_diagnostic() {
    // <body>0..6 </body>6..13 </html>13..20 x20..21. The recovery diagnostic
    // is the distinct after-after-body code, never the after-body one.
    let report = analyze_text("<body></body></html>x");
    assert!(matches!(report.completion(), HtmlTreeCompletion::Complete));
    assert!(
        report
            .tree_diagnostics()
            .iter()
            .all(|d| d.code() != HtmlTreeDiagnosticCode::AfterBodyCharacterData)
    );
    let recoveries: Vec<_> = report
        .tree_diagnostics()
        .iter()
        .filter(|d| d.code() == HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData)
        .collect();
    let [recovery] = recoveries.as_slice() else {
        panic!("exactly one after-after-body recovery diagnostic");
    };
    assert_eq!(
        recovery.recovery(),
        HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
    );
    assert_eq!(
        span(recovery.trigger().expect("authored trigger")),
        at(20, 21, "x")
    );
    let texts: Vec<&HtmlTreeText> = report
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) => Some(text),
            _ => None,
        })
        .collect();
    let [text] = texts.as_slice() else {
        panic!("one text node");
    };
    assert_eq!(text.interpreted(), "x");
}

#[test]
fn after_after_body_nul_projects_distinct_recovery_and_ignore_diagnostics() {
    // </html> ends at 20; the authored U+0000 is 20..21.
    let report = analyze_text("<body></body></html>\0");
    assert!(matches!(report.completion(), HtmlTreeCompletion::Complete));
    let [tokenizer_diagnostic] = report.tokenizer_diagnostics() else {
        panic!("expected exactly the tokenizer NUL diagnostic");
    };
    assert_eq!(
        tokenizer_diagnostic.code(),
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter
    );
    assert_eq!(span(tokenizer_diagnostic.location()), at(20, 21, "\0"));

    let codes: Vec<_> = report
        .tree_diagnostics()
        .iter()
        .map(|d| (d.code(), d.recovery()))
        .collect();
    assert_eq!(
        codes,
        vec![
            (
                HtmlTreeDiagnosticCode::MissingDoctype,
                HtmlTreeRecovery::ContinuedInQuirksDocumentMode
            ),
            (
                HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData,
                HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
            ),
            (
                HtmlTreeDiagnosticCode::NullCharacterInBody,
                HtmlTreeRecovery::IgnoredToken
            ),
        ]
    );
    for diagnostic in &report.tree_diagnostics()[1..] {
        assert_eq!(
            span(diagnostic.trigger().expect("authored trigger")),
            at(20, 21, "\0")
        );
    }
    assert!(
        report
            .nodes()
            .iter()
            .all(|node| !matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
    );
}

#[test]
fn text_around_an_ignored_data_nul_keeps_exact_contributions() {
    // <body>0..6 a6..7 NUL7..8 b8..9
    let report = analyze_text("<body>a\0b</body>");
    let texts: Vec<&HtmlTreeText> = report
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) => Some(text),
            _ => None,
        })
        .collect();
    let [text] = texts.as_slice() else {
        panic!("one text node");
    };
    assert_eq!(text.interpreted(), "ab");
    let spans: Vec<_> = text
        .contributions()
        .iter()
        .map(|c| (span(c.source()), c.interpreted().to_owned()))
        .collect();
    assert_eq!(
        spans,
        vec![
            (at(6, 7, "a"), "a".to_owned()),
            (at(8, 9, "b"), "b".to_owned()),
        ]
    );
}

#[test]
fn every_tokenizer_resource_kind_projects_exact_kind_limit_attempted_and_anchor() {
    let probe = source("abcdef");
    for (resource, expected_kind) in [
        (
            HtmlTokenizerResource::SourceBytes,
            HtmlTreeResourceKind::SourceBytes,
        ),
        (
            HtmlTokenizerResource::TransitionSteps,
            HtmlTreeResourceKind::TransitionSteps,
        ),
        (
            HtmlTokenizerResource::EmittedTokens,
            HtmlTreeResourceKind::EmittedTokens,
        ),
        (
            HtmlTokenizerResource::Diagnostics,
            HtmlTreeResourceKind::TokenizerDiagnostics,
        ),
        (
            HtmlTokenizerResource::AttributesPerTag,
            HtmlTreeResourceKind::AttributesPerTag,
        ),
        (
            HtmlTokenizerResource::RetainedInterpretedBytes,
            HtmlTreeResourceKind::RetainedInterpretedBytes,
        ),
        (
            HtmlTokenizerResource::TemporaryBufferBytes,
            HtmlTreeResourceKind::TemporaryBufferBytes,
        ),
    ] {
        // Distinct hand-chosen values per kind so a field swap is visible.
        let (limit, attempted, start, end) = (
            3 + expected_kind as usize,
            11 + expected_kind as usize,
            1,
            4,
        );
        let internal = HtmlTokenizerResourceLimit::new(
            &probe,
            resource,
            limit,
            attempted,
            probe.anchor(start, end).unwrap(),
        )
        .unwrap();

        let projected = project_resource_limit(&internal);

        assert_eq!(projected.kind(), expected_kind);
        assert_eq!(projected.limit(), limit);
        assert_eq!(projected.attempted(), attempted);
        assert_eq!(span(projected.at()), at(1, 4, "bcd"));
    }
    // The seven public kinds cover exactly the seven internal resources.
    assert_eq!(HtmlTokenizerResource::ALL.len(), 7);
}

#[test]
fn fixed_vector_values_are_the_frozen_issue_864_vector() {
    let limits = fixed_limits();
    assert_eq!(limits.max_source_bytes(), 36_864);
    assert_eq!(limits.max_transition_steps(), 79_872);
    assert_eq!(limits.max_emitted_tokens(), 6_144);
    assert_eq!(limits.max_diagnostics(), 256);
    assert_eq!(limits.max_attributes_per_tag(), 1);
    assert_eq!(limits.max_retained_interpreted_bytes(), 49_152);
    assert_eq!(limits.max_temporary_buffer_bytes(), 0);
}

#[test]
fn mixed_exact_source_workload_stays_within_the_fixed_vector() {
    // 256 `&amp` without semicolons in a title is exactly 256 tokenizer
    // diagnostics; a body text pad brings the source to exactly 36,864 bytes.
    let head = format!("<title>{}</title><body>", "&amp".repeat(256));
    let pad = 36_864 - head.len();
    let text_source = format!("{head}{}", "x".repeat(pad));
    assert_eq!(text_source.len(), 36_864);

    let src = source(&text_source);
    let analysis = construct_html_document_shell(&src, fixed_limits()).unwrap();
    let usage = analysis.tokenizer_run().usage();

    assert!(analysis.is_complete());
    assert_eq!(usage.source_bytes(), 36_864);
    assert_eq!(usage.diagnostics(), 256);
    assert_eq!(usage.peak_temporary_buffer_bytes(), 0);
    assert!(usage.retained_interpreted_bytes() <= 49_152);
    assert!(usage.transition_steps() <= 79_872);
    assert!(usage.emitted_tokens() <= 6_144);

    // One more diagnostic is the first refusal.
    let over = format!("<title>{}</title><body>x", "&amp".repeat(257));
    let report = analyze_text(&over);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::TokenizerDiagnostics);
    assert_eq!(limit.attempted(), 257);
}

#[test]
fn temporary_buffer_stays_zero_on_named_reference_and_rawtext_paths() {
    for text_source in [
        "<title>a&amp;b</title>",
        "<title>&notit;&bogus;&amp</title>",
        "<head><style>a<b>&amp;</style></head>",
    ] {
        let src = source(text_source);
        let analysis = construct_html_document_shell(&src, fixed_limits()).unwrap();
        assert!(analysis.is_complete(), "{text_source}");
        assert_eq!(
            analysis
                .tokenizer_run()
                .usage()
                .peak_temporary_buffer_bytes(),
            0,
            "{text_source}"
        );
    }
}

#[test]
fn node_count_stays_within_emitted_tokens_plus_four() {
    for text_source in [
        "",
        "<body>",
        "<!DOCTYPE html>",
        "<body><div></div>",
        "<body><div><section><p>t</p></section></div></body>",
        "<body></p></p></p>",
        "<title>a&amp;b</title>",
        "<head><style>a<b></style></head>",
        "<html><head></head><body><div><div><div></html>",
    ] {
        let src = source(text_source);
        let analysis = construct_html_document_shell(&src, fixed_limits()).unwrap();
        let tokens = analysis.tokenizer_run().usage().emitted_tokens();
        assert!(
            analysis.node_count() <= tokens + 4,
            "{text_source}: {} nodes for {tokens} tokens",
            analysis.node_count()
        );
    }
}

#[test]
fn report_remains_usable_after_the_caller_source_is_dropped() {
    let report = {
        let caller = source("<body><DIV>t</DIV>");
        analyze_text_from(&caller)
    };

    let div = element(&report, 4);
    assert_eq!(authored_spans(div).1, at(7, 10, "DIV"));
    assert_eq!(text(&report, 5).interpreted(), "t");
    assert_eq!(
        report.coverage().committed_prefix().fragment(),
        "<body><DIV>t</DIV>"
    );
}

fn analyze_text_from(source: &SourceText) -> HtmlTreeReport {
    analyze_selected_document_tree(source).expect("report")
}

#[test]
fn reversed_internal_storage_cannot_change_identity_or_final_placement() {
    for text_source in [
        "",
        "<body><div><section><p>t</p></section></div></body>",
        "<title>a&amp;b</title>",
        "<body></p>",
    ] {
        let src = source(text_source);
        let normal = construct_html_document_shell(&src, fixed_limits()).unwrap();
        let reversed = construct_html_document_shell(&src, fixed_limits())
            .unwrap()
            .with_reversed_storage();

        let a = project(src.id(), &normal).unwrap();
        let b = project(src.id(), &reversed).unwrap();

        assert_eq!(outline(&a), outline(&b), "{text_source}");
        let ids = |report: &HtmlTreeReport| {
            report
                .nodes()
                .iter()
                .map(|node| (node.id().value(), node.parent().map(|p| p.value())))
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&a), ids(&b), "{text_source}");
        // Public order is ascending creation identity, never storage order.
        let values: Vec<u32> = b.nodes().iter().map(|node| node.id().value()).collect();
        let mut sorted = values.clone();
        sorted.sort_unstable();
        assert_eq!(values, sorted);
    }
}

#[test]
fn identity_is_creation_order_not_final_tree_position() {
    // `</p>` synthesizes the Paragraph after body exists, so the Paragraph's
    // identity is later than body's although it is a child of body.
    let report = analyze_text("<body></p>");
    let body = element_node(&report, 3);
    assert_eq!(body.children(), [HtmlTreeNodeId(4)]);
    assert!(report.node(HtmlTreeNodeId(4)).is_some());
    assert!(report.node(HtmlTreeNodeId(99)).is_none());
}

fn element_node(report: &HtmlTreeReport, id: u32) -> &HtmlTreeNode {
    report.node(HtmlTreeNodeId(id)).unwrap()
}

#[test]
fn invalid_fixed_configuration_fails_closed_to_internal_failure() {
    // A zero transition-step limit is an invalid tokenizer configuration. The
    // fixed vector can never produce it, so reaching it is an internal
    // contract failure, never an incomplete report.
    let limits = HtmlTokenizerLimits::new(36_864, 0, 6_144, 256, 1, 49_152, 0);
    assert_eq!(
        analyze(&source("<body>"), limits).unwrap_err(),
        HtmlTreeCoreFailure::InternalFailure
    );
    let limits = HtmlTokenizerLimits::new(36_864, 79_872, 0, 256, 1, 49_152, 0);
    assert_eq!(
        analyze(&source(""), limits).unwrap_err(),
        HtmlTreeCoreFailure::InternalFailure
    );
}

#[test]
fn projection_reservation_failure_is_not_a_tokenizer_resource_refusal() {
    // A reservation no allocator can satisfy exercises the same fallible
    // helpers the projection uses, without any production raw-limit knob.
    assert_eq!(
        vec_with_capacity::<u64>(usize::MAX).unwrap_err(),
        HtmlTreeCoreFailure::ProjectionResourceExhausted
    );
    assert_eq!(
        vec_with_capacity::<u8>(isize::MAX as usize + 1).unwrap_err(),
        HtmlTreeCoreFailure::ProjectionResourceExhausted
    );
    assert_ne!(
        HtmlTreeCoreFailure::ProjectionResourceExhausted,
        HtmlTreeCoreFailure::InternalFailure
    );
    assert_eq!(owned_str("abc").unwrap(), "abc");
    assert!(vec_with_capacity::<u8>(16).unwrap().capacity() >= 16);
}

#[test]
fn core_failure_display_names_no_source_content() {
    assert_eq!(
        HtmlTreeCoreFailure::InternalFailure.to_string(),
        "HTML tree analysis returned a Core internal failure"
    );
    assert_eq!(
        HtmlTreeCoreFailure::ProjectionResourceExhausted.to_string(),
        "HTML tree analysis returned a Core failure: report projection resources exhausted"
    );
}

// ---------------------------------------------------------------------------
// Selected canonical DocumentType (Issue #892)
// ---------------------------------------------------------------------------

fn document_type(report: &HtmlTreeReport, id: u32) -> &HtmlTreeDocumentType {
    let node = element_node(report, id);
    match node.kind() {
        HtmlTreeNodeKind::DocumentType(doctype) => doctype,
        other => panic!("node {id} is not a DocumentType: {other:?}"),
    }
}

#[test]
fn canonical_doctype_is_a_distinct_node_kind_under_the_document() {
    // `<!DOCTYPE html>` is 0..15; `html` is 10..14.
    let report = analyze_text("<!DOCTYPE html>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:doctype",
            "1:2:html",
            "2:3:head",
            "2:4:body"
        ]
    );
    let node = element_node(&report, 1);
    // Distinct from Document, Element and Text.
    assert!(matches!(node.kind(), HtmlTreeNodeKind::DocumentType(_)));
    assert!(node.children().is_empty());
    assert_eq!(node.parent(), Some(report.root()));
    // Normal identity lookup finds it.
    assert!(std::ptr::eq(report.node(HtmlTreeNodeId(1)).unwrap(), node));
    // The Document's explicit child order places it before `html`.
    assert_eq!(
        element_node(&report, 0).children(),
        [HtmlTreeNodeId(1), HtmlTreeNodeId(2)]
    );
    let doctype = document_type(&report, 1);
    assert_eq!(span(doctype.complete()), at(0, 15, "<!DOCTYPE html>"));
    assert_eq!(span(doctype.authored_name()), at(10, 14, "html"));
    // MissingDoctype is absent and nothing else is diagnosed.
    assert!(report.tree_diagnostics().is_empty());
    assert!(report.tokenizer_diagnostics().is_empty());
    assert_eq!(report.coverage().processed_tokens(), 2);
    assert_eq!(span(report.coverage().committed_prefix()).1, 15);
}

#[test]
fn canonical_doctype_before_an_explicit_shell_keeps_its_place_and_the_shell_evidence() {
    // `<!DOCTYPE html>` 0..15 `<html>` 15..21 `<head>` 21..27 `</head>` 27..34
    // `<body>` 34..40 `</body>` 40..47 `</html>` 47..54.
    let report = analyze_text("<!DOCTYPE html><html><head></head><body></body></html>");

    assert_complete(&report);
    assert_eq!(
        outline(&report),
        [
            "0:0:document",
            "1:1:doctype",
            "1:2:html",
            "2:3:head",
            "2:4:body"
        ]
    );
    assert_eq!(authored_spans(element(&report, 2)).0, at(15, 21, "<html>"));
    assert!(report.tree_diagnostics().is_empty());
}

#[test]
fn canonical_doctype_authored_case_survives_in_the_exact_anchors() {
    let report = analyze_text("<!DoCtYpE HTML>");
    let doctype = document_type(&report, 1);
    assert_eq!(span(doctype.complete()), at(0, 15, "<!DoCtYpE HTML>"));
    assert_eq!(span(doctype.authored_name()), at(10, 14, "HTML"));

    let report = analyze_text("<!doctype html>");
    assert_eq!(
        span(document_type(&report, 1).complete()),
        at(0, 15, "<!doctype html>")
    );
}

#[test]
fn canonical_doctype_evidence_survives_dropping_the_caller_source() {
    let report = {
        let caller = source("<!DOCTYPE\r\n  HTML  >");
        analyze_text_from(&caller)
    };
    // `<!DOCTYPE` 0..9, CRLF 9..11, two spaces 11..13, `HTML` 13..17, two
    // spaces 17..19, `>` 19..20.
    let doctype = document_type(&report, 1);
    assert_eq!(
        span(doctype.complete()),
        at(0, 20, "<!DOCTYPE\r\n  HTML  >")
    );
    assert_eq!(span(doctype.authored_name()), at(13, 17, "HTML"));
}

#[test]
fn canonical_doctype_identity_and_placement_do_not_depend_on_storage_order() {
    for text_source in ["<!DOCTYPE html>", "<!DOCTYPE html><html></html>"] {
        let src = source(text_source);
        let normal = construct_html_document_shell(&src, fixed_limits()).unwrap();
        let reversed = construct_html_document_shell(&src, fixed_limits())
            .unwrap()
            .with_reversed_storage();
        let a = project(src.id(), &normal).unwrap();
        let b = project(src.id(), &reversed).unwrap();

        assert_eq!(outline(&a), outline(&b), "{text_source}");
        let summary = |report: &HtmlTreeReport| {
            report
                .nodes()
                .iter()
                .map(|node| (node.id().value(), node.parent().map(|p| p.value())))
                .collect::<Vec<_>>()
        };
        assert_eq!(summary(&a), summary(&b), "{text_source}");
        assert_eq!(
            span(document_type(&b, 1).complete()),
            at(0, 15, "<!DOCTYPE html>")
        );
        assert_eq!(
            element_node(&b, 0).children().first(),
            Some(&HtmlTreeNodeId(1))
        );
    }
}

#[test]
fn canonical_doctype_identity_is_independent_of_the_source_identity() {
    for id in [0_u64, 5, 4_000_000_000] {
        let src = SourceText::new(SourceId::new(id), "<!DOCTYPE html>".to_owned());
        let report = analyze_selected_document_tree(&src).expect("report");
        assert_eq!(outline(&report), outline(&analyze_text("<!DOCTYPE html>")));
        assert_eq!(report.source_id(), SourceId::new(id));
    }
}

#[test]
fn doctype_outside_initial_projects_its_own_unsupported_capability() {
    // The second DOCTYPE is 15..30.
    let report = analyze_text("<!DOCTYPE html><!DOCTYPE html>");

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(unsupported)) =
        report.completion()
    else {
        panic!("expected tree unsupported, got {:?}", report.completion());
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTreeUnsupportedCapability::DoctypeOutsideInitial
    );
    assert_eq!(
        span(unsupported.trigger().unwrap()),
        at(15, 30, "<!DOCTYPE html>")
    );
    // No second DocumentType was constructed and the refused token is not
    // committed.
    assert_eq!(
        report
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind(), HtmlTreeNodeKind::DocumentType(_)))
            .count(),
        1
    );
    assert_eq!(span(report.coverage().committed_prefix()).1, 15);
    assert_eq!(report.coverage().processed_tokens(), 1);
}

#[test]
fn non_selected_markup_declarations_remain_tokenizer_unsupported() {
    for (text_source, trigger) in [
        ("<!xx>", at(0, 2, "<!")),
        ("<!DOCTYPE svg>", at(0, 2, "<!")),
        ("<!DOCTYPEhtml>", at(0, 2, "<!")),
        ("<!DOCTYPE html PUBLIC \"x\">", at(0, 2, "<!")),
        ("<body><!DOCTYPE>", at(6, 8, "<!")),
    ] {
        let report = analyze_text(text_source);
        let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TokenizerUnsupported(
            unsupported,
        )) = report.completion()
        else {
            panic!("{text_source}: got {:?}", report.completion());
        };
        assert_eq!(
            unsupported.capability(),
            HtmlTokenizerUnsupportedCapability::MarkupDeclaration,
            "{text_source}"
        );
        assert_eq!(span(unsupported.trigger()), trigger, "{text_source}");
        assert!(
            !report
                .nodes()
                .iter()
                .any(|node| matches!(node.kind(), HtmlTreeNodeKind::DocumentType(_))),
            "{text_source}"
        );
    }
}

#[test]
fn canonical_doctype_resource_refusals_stay_resource_limited() {
    use crate::html::tokenizer::resource::HtmlTokenizerLimits;
    let analyze_with = |text_source: &str, limits: HtmlTokenizerLimits| {
        let src = source(text_source);
        let analysis = construct_html_document_shell(&src, limits).unwrap();
        project(src.id(), &analysis).unwrap()
    };

    // Retained interpreted bytes: the selected name needs four.
    let tight = HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 1, 3, 0);
    let report = analyze_with("<!DOCTYPE html>", tight);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::RetainedInterpretedBytes);
    assert_eq!(limit.limit(), 3);
    assert_eq!(limit.attempted(), 4);
    assert_eq!(outline(&report), ["0:0:document"]);

    // Emitted tokens: `x` fits, the DOCTYPE would be the second token.
    let one = HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 1, 4_096, 0);
    let report = analyze_with("x<!DOCTYPE html>", one);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::EmittedTokens);
    assert_eq!((limit.limit(), limit.attempted()), (1, 2));
    assert!(
        !report
            .nodes()
            .iter()
            .any(|node| matches!(node.kind(), HtmlTreeNodeKind::DocumentType(_)))
    );

    // Transition steps: one short of the seventeen the source needs.
    let short = HtmlTokenizerLimits::new(1_024, 16, 1_024, 1_024, 1, 4_096, 0);
    let report = analyze_with("<!DOCTYPE html>", short);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::TransitionSteps);
    assert_eq!((limit.limit(), limit.attempted()), (16, 17));
}

// ---- #898: selected ordinary close/recovery relation projection ----
//
// Every expectation below is hand-authored from the accepted #894/#895
// lifecycle theorem and hand-counted byte offsets, never captured from the
// projection. Identities follow creation order: with an authored `<body>`
// first, document 0, html 1, head 2 and body 3 precede the first selected
// element, which is #4.

/// `close #n @start..end "fragment"` or `pop #n -> #t @start..end "fragment"`,
/// in report order. Subject and target are printed as report-local ids only.
fn relations(report: &HtmlTreeReport) -> Vec<String> {
    report
        .selected_ordinary_relations()
        .iter()
        .map(|relation| match relation {
            HtmlTreeSelectedOrdinaryRelation::MatchingClose { node, trigger } => {
                let (start, end, fragment) = span(trigger);
                format!("close #{} @{start}..{end} {fragment:?}", node.value())
            }
            HtmlTreeSelectedOrdinaryRelation::RecoveryPopByAncestorEndTag {
                node,
                target,
                trigger,
            } => {
                let (start, end, fragment) = span(trigger);
                format!(
                    "pop #{} -> #{} @{start}..{end} {fragment:?}",
                    node.value(),
                    target.value()
                )
            }
        })
        .collect()
}

#[test]
fn simple_matching_close_projects_one_relation_without_a_target() {
    // <body>0..6 <article>6..15 </article>15..25
    let report = analyze_text("<body><article></article>");

    assert_complete(&report);
    assert_eq!(relations(&report), [r#"close #4 @15..25 "</article>""#]);
    let [HtmlTreeSelectedOrdinaryRelation::MatchingClose { node, trigger }] =
        report.selected_ordinary_relations()
    else {
        panic!("expected exactly one matching close");
    };
    assert_eq!(*node, HtmlTreeNodeId(4));
    assert_eq!(trigger.source_id(), SourceId::new(0));
    assert_eq!(
        element(&report, node.value()).name(),
        HtmlTreeElementName::Article
    );
}

#[test]
fn same_name_nesting_closes_inner_then_outer_with_distinct_identities() {
    // <body>0..6 <article>6..15 <article>15..24 </article>24..34 </article>34..44
    let report = analyze_text("<body><article><article></article></article>");

    assert_complete(&report);
    assert_eq!(
        relations(&report),
        [
            r#"close #5 @24..34 "</article>""#,
            r#"close #4 @34..44 "</article>""#,
        ]
    );
    assert!(report.tree_diagnostics().iter().all(|diagnostic| {
        diagnostic.code() != HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
    }));
}

#[test]
fn heterogeneous_recovery_pops_the_inner_element_then_closes_the_target() {
    // <body>0..6 <article>6..15 <nav>15..20 </article>20..30
    let report = analyze_text("<body><article><nav></article>");

    assert_complete(&report);
    assert_eq!(
        relations(&report),
        [
            r#"pop #5 -> #4 @20..30 "</article>""#,
            r#"close #4 @20..30 "</article>""#,
        ]
    );
    assert_eq!(element(&report, 5).name(), HtmlTreeElementName::Nav);
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Article);
}

#[test]
fn one_trigger_projects_every_pop_current_first_then_the_target_close() {
    // <body>0..6 <header>6..14 <main>14..20 <aside>20..27 </header>27..36
    let report = analyze_text("<body><header><main><aside></header>");

    assert_complete(&report);
    assert_eq!(
        relations(&report),
        [
            r#"pop #6 -> #4 @27..36 "</header>""#,
            r#"pop #5 -> #4 @27..36 "</header>""#,
            r#"close #4 @27..36 "</header>""#,
        ]
    );
    assert_eq!(element(&report, 6).name(), HtmlTreeElementName::Aside);
    assert_eq!(element(&report, 5).name(), HtmlTreeElementName::Main);
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Header);
}

#[test]
fn recovery_target_is_the_nearest_constructed_same_name_ancestor_not_the_name() {
    // <body>0..6 <div>6..11 <div>11..16 <nav>16..21 </div>21..27 </div>27..33
    // The first </div> targets the INNER div (#5); only the second closes #4.
    let report = analyze_text("<body><div><div><nav></div></div>");

    assert_complete(&report);
    assert_eq!(
        relations(&report),
        [
            r#"pop #6 -> #5 @21..27 "</div>""#,
            r#"close #5 @21..27 "</div>""#,
            r#"close #4 @27..33 "</div>""#,
        ]
    );
    // Both candidate targets are `div`; identity, not name, selects #5.
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Div);
    assert_eq!(element(&report, 5).name(), HtmlTreeElementName::Div);
}

#[test]
fn independent_triggers_keep_validated_processing_order() {
    // <body>0..6 <div>6..11 <nav>11..16 </nav>16..22 </div>22..28
    let report = analyze_text("<body><div><nav></nav></div>");

    assert_complete(&report);
    assert_eq!(
        relations(&report),
        [
            r#"close #5 @16..22 "</nav>""#,
            r#"close #4 @22..28 "</div>""#,
        ]
    );
}

#[test]
fn implied_paragraph_pop_is_not_projected_between_selected_relations() {
    // <body>0..6 <div>6..11 <nav>11..16 <p>16..19 </div>19..25
    // The Paragraph is outside the selected domain; only nav and div appear.
    let report = analyze_text("<body><div><nav><p></div>");

    assert_complete(&report);
    assert_eq!(
        relations(&report),
        [
            r#"pop #5 -> #4 @19..25 "</div>""#,
            r#"close #4 @19..25 "</div>""#,
        ]
    );
    assert_eq!(element(&report, 6).name(), HtmlTreeElementName::Paragraph);
}

#[test]
fn mixed_authored_casing_keeps_the_exact_complete_end_tag_range() {
    // <body>0..6 <ArTiClE>6..15 </aRtIcLe>15..25
    let report = analyze_text("<body><ArTiClE></aRtIcLe>");

    assert_complete(&report);
    assert_eq!(relations(&report), [r#"close #4 @15..25 "</aRtIcLe>""#]);
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Article);
}

#[test]
fn relation_trigger_carries_the_report_source_identity() {
    let src = SourceText::new(SourceId::new(7), "<body><nav></nav>".to_owned());
    let report = analyze_text_from(&src);

    assert_eq!(report.source_id(), SourceId::new(7));
    let [HtmlTreeSelectedOrdinaryRelation::MatchingClose { trigger, .. }] =
        report.selected_ordinary_relations()
    else {
        panic!("expected exactly one matching close");
    };
    assert_eq!(trigger.source_id(), SourceId::new(7));
    // <body>0..6 <nav>6..11 </nav>11..17
    assert_eq!(span(trigger), at(11, 17, "</nav>"));
}

#[test]
fn unmatched_end_tag_projects_no_relation_and_keeps_its_diagnostic() {
    let report = analyze_text("<body></article>");

    assert_complete(&report);
    assert!(report.selected_ordinary_relations().is_empty());
    assert!(report.tree_diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
    }));
}

#[test]
fn end_of_file_does_not_fabricate_a_close_for_an_open_element() {
    let report = analyze_text("<body><article>");

    assert!(report.selected_ordinary_relations().is_empty());
    assert!(report.tree_diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile
    }));
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Article);
}

#[test]
fn relation_committed_before_an_unsupported_stop_stays_visible() {
    // <body>0..6 <div>6..11 </div>11..17 <span>17..23 (unsupported)
    let report = analyze_text("<body><div></div><span>");

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(unsupported)) =
        report.completion()
    else {
        panic!("expected tree unsupported, got {:?}", report.completion());
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTreeUnsupportedCapability::NonShellElementTag
    );
    assert_eq!(span(unsupported.trigger().unwrap()), at(17, 23, "<span>"));
    assert_eq!(relations(&report), [r#"close #4 @11..17 "</div>""#]);
    // Coverage ends where the committed work ends; absence after it is not
    // closure evidence.
    assert_eq!(
        span(report.coverage().committed_prefix()),
        at(0, 17, "<body><div></div>")
    );
    assert_eq!(report.coverage().processed_tokens(), 3);
}

#[test]
fn open_element_before_an_unsupported_stop_gets_no_relation() {
    // <body>0..6 <div>6..11 <span>11..17 (unsupported)
    let report = analyze_text("<body><div><span>");

    assert!(matches!(
        report.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(_))
    ));
    assert!(report.selected_ordinary_relations().is_empty());
}

#[test]
fn every_relation_resolves_through_the_report_and_no_subject_repeats() {
    for text_source in [
        "<body><article></article>",
        "<body><article><article></article></article>",
        "<body><header><main><aside></header>",
        "<body><div><div><nav></div></div>",
        "<body><div><nav><p></div>",
        "<body><div><nav></nav></div><section><footer></section>",
    ] {
        let report = analyze_text(text_source);
        let selected_nodes = report
            .nodes()
            .iter()
            .filter(|node| {
                matches!(
                    node.kind(),
                    HtmlTreeNodeKind::Element(element) if !matches!(
                        element.name(),
                        HtmlTreeElementName::Html
                            | HtmlTreeElementName::Head
                            | HtmlTreeElementName::Body
                            | HtmlTreeElementName::Paragraph
                            | HtmlTreeElementName::Style
                            | HtmlTreeElementName::Title
                    )
                )
            })
            .count();

        let mut subjects = Vec::new();
        for relation in report.selected_ordinary_relations() {
            let (node, target) = match relation {
                HtmlTreeSelectedOrdinaryRelation::MatchingClose { node, .. } => (*node, None),
                HtmlTreeSelectedOrdinaryRelation::RecoveryPopByAncestorEndTag {
                    node,
                    target,
                    ..
                } => (*node, Some(*target)),
            };
            assert!(report.node(node).is_some(), "{text_source}");
            if let Some(target) = target {
                assert!(report.node(target).is_some(), "{text_source}");
                assert_ne!(node, target, "{text_source}");
            }
            assert!(!subjects.contains(&node), "{text_source}: repeated subject");
            subjects.push(node);
        }
        // Bounded by constructed selected nodes; every closed node here.
        assert!(subjects.len() <= selected_nodes, "{text_source}");
    }
}

#[test]
fn relation_projection_is_deterministic_and_independent_of_storage_order() {
    for text_source in [
        "<body><header><main><aside></header>",
        "<body><div><div><nav></div></div>",
        "<body><div></div><span>",
    ] {
        let src = source(text_source);
        let first = analyze_text_from(&src);
        let second = analyze_text_from(&src);
        let reversed = project(
            src.id(),
            &construct_html_document_shell(&src, fixed_limits())
                .unwrap()
                .with_reversed_storage(),
        )
        .unwrap();

        assert_eq!(relations(&first), relations(&second), "{text_source}");
        assert_eq!(relations(&first), relations(&reversed), "{text_source}");
    }
}

#[test]
fn relation_domain_is_closed_to_exactly_the_two_selected_meanings() {
    // An exhaustive match without a wildcard fails to compile if a variant is
    // added, widening the accepted public domain unnoticed.
    fn meaning(relation: &HtmlTreeSelectedOrdinaryRelation) -> &'static str {
        match relation {
            HtmlTreeSelectedOrdinaryRelation::MatchingClose { .. } => "close",
            HtmlTreeSelectedOrdinaryRelation::RecoveryPopByAncestorEndTag { .. } => "pop",
        }
    }
    let report = analyze_text("<body><article><nav></article>");
    let meanings: Vec<_> = report
        .selected_ordinary_relations()
        .iter()
        .map(meaning)
        .collect();
    assert_eq!(meanings, ["pop", "close"]);
}

// ---------------------------------------------------------------------------
// SelectedOrdinary authored attribute evidence (#902)
//
// Expected spans are hand-counted byte offsets of the literal source; the
// expected fragment is always the slice of that same literal, never a value
// read back from the report.
// ---------------------------------------------------------------------------

type Span = (usize, usize);

/// Hand-authored authored-syntax expectation, independent of the public enum.
enum Syntax {
    Missing,
    MissingAfterEquals {
        equals: Span,
        boundary: Span,
    },
    Unquoted {
        equals: Span,
        value: Span,
    },
    DoubleQuoted {
        equals: Span,
        open: Span,
        value: Span,
        close: Span,
    },
    SingleQuoted {
        equals: Span,
        open: Span,
        value: Span,
        close: Span,
    },
}

struct Row {
    node: u32,
    complete: Span,
    name: Span,
    syntax: Syntax,
    interpreted_name: &'static str,
    interpreted_value: &'static str,
}

fn sliced(text: &str, (start, end): Span) -> (usize, usize, String) {
    (start, end, text[start..end].to_owned())
}

fn assert_row(text: &str, actual: &HtmlTreeSelectedOrdinaryAttribute, expected: &Row) {
    let label = format!("{text:?} node {}", expected.node);
    assert_eq!(actual.node().value(), expected.node, "{label}");
    assert_eq!(
        span(actual.complete()),
        sliced(text, expected.complete),
        "{label}"
    );
    assert_eq!(
        span(actual.authored_name()),
        sliced(text, expected.name),
        "{label}"
    );
    assert_eq!(
        actual.interpreted_name(),
        expected.interpreted_name,
        "{label}"
    );
    assert_eq!(
        actual.interpreted_value(),
        expected.interpreted_value,
        "{label}"
    );
    for anchor in [actual.complete(), actual.authored_name()] {
        assert_eq!(anchor.source_id(), SourceId::new(0), "{label}");
    }
    let s = |anchor: &SourceAnchor, expected: Span| {
        assert_eq!(anchor.source_id(), SourceId::new(0), "{label}");
        assert_eq!(span(anchor), sliced(text, expected), "{label}");
    };
    match (actual.value_syntax(), &expected.syntax) {
        (HtmlTreeAttributeValueSyntax::Missing, Syntax::Missing) => {}
        (
            HtmlTreeAttributeValueSyntax::MissingAfterEquals {
                equals,
                value_boundary,
            },
            Syntax::MissingAfterEquals {
                equals: e,
                boundary: b,
            },
        ) => {
            s(equals, *e);
            s(value_boundary, *b);
        }
        (
            HtmlTreeAttributeValueSyntax::Unquoted { equals, value },
            Syntax::Unquoted {
                equals: e,
                value: v,
            },
        ) => {
            s(equals, *e);
            s(value, *v);
        }
        (
            HtmlTreeAttributeValueSyntax::DoubleQuoted {
                equals,
                open_quote,
                value,
                close_quote,
            },
            Syntax::DoubleQuoted {
                equals: e,
                open: o,
                value: v,
                close: c,
            },
        )
        | (
            HtmlTreeAttributeValueSyntax::SingleQuoted {
                equals,
                open_quote,
                value,
                close_quote,
            },
            Syntax::SingleQuoted {
                equals: e,
                open: o,
                value: v,
                close: c,
            },
        ) => {
            s(equals, *e);
            s(open_quote, *o);
            s(value, *v);
            s(close_quote, *c);
        }
        (actual, _) => panic!("{label}: wrong authored syntax {actual:?}"),
    }
}

fn assert_rows(text: &str, expected: &[Row]) {
    let report = analyze_text(text);
    let actual = report.selected_ordinary_attributes();
    assert_eq!(actual.len(), expected.len(), "{text:?}: row count");
    for (actual, expected) in actual.iter().zip(expected) {
        assert_row(text, actual, expected);
    }
}

#[test]
fn zero_attribute_selected_nodes_have_no_row() {
    for text in ["<body><div></div>", "<body><nav><main></main></nav>"] {
        let report = analyze_text(text);
        assert_complete(&report);
        assert!(report.selected_ordinary_attributes().is_empty(), "{text:?}");
    }
}

#[test]
fn double_quoted_attribute_is_projected_with_exact_anchors() {
    // <body>0..6 <div id="a">6..18; id="a" 11..17
    assert_rows(
        r#"<body><div id="a"></div>"#,
        &[Row {
            node: 4,
            complete: (11, 17),
            name: (11, 13),
            syntax: Syntax::DoubleQuoted {
                equals: (13, 14),
                open: (14, 15),
                value: (15, 16),
                close: (16, 17),
            },
            interpreted_name: "id",
            interpreted_value: "a",
        }],
    );
}

#[test]
fn single_quoted_attribute_is_projected_with_exact_anchors() {
    // <section class='x y'> : name 15..20 = 20..21 ' 21..22 x y 22..25 ' 25..26
    assert_rows(
        "<body><section class='x y'></section>",
        &[Row {
            node: 4,
            complete: (15, 26),
            name: (15, 20),
            syntax: Syntax::SingleQuoted {
                equals: (20, 21),
                open: (21, 22),
                value: (22, 25),
                close: (25, 26),
            },
            interpreted_name: "class",
            interpreted_value: "x y",
        }],
    );
}

#[test]
fn unquoted_attribute_is_projected_with_exact_anchors() {
    // <nav data=v> : data 11..15 = 15..16 v 16..17
    assert_rows(
        "<body><nav data=v></nav>",
        &[Row {
            node: 4,
            complete: (11, 17),
            name: (11, 15),
            syntax: Syntax::Unquoted {
                equals: (15, 16),
                value: (16, 17),
            },
            interpreted_name: "data",
            interpreted_value: "v",
        }],
    );
}

#[test]
fn empty_quoted_values_keep_their_quote_syntax() {
    // <main a=""> : a 12..13 = 13..14 " 14..15 (empty 15..15) " 15..16
    assert_rows(
        r#"<body><main a=""></main>"#,
        &[Row {
            node: 4,
            complete: (12, 16),
            name: (12, 13),
            syntax: Syntax::DoubleQuoted {
                equals: (13, 14),
                open: (14, 15),
                value: (15, 15),
                close: (15, 16),
            },
            interpreted_name: "a",
            interpreted_value: "",
        }],
    );
    // <aside a=''> : a 13..14 = 14..15 ' 15..16 (empty 16..16) ' 16..17
    assert_rows(
        "<body><aside a=''></aside>",
        &[Row {
            node: 4,
            complete: (13, 17),
            name: (13, 14),
            syntax: Syntax::SingleQuoted {
                equals: (14, 15),
                open: (15, 16),
                value: (16, 16),
                close: (16, 17),
            },
            interpreted_name: "a",
            interpreted_value: "",
        }],
    );
}

#[test]
fn missing_and_missing_after_equals_stay_distinct_from_each_other_and_from_empty_quotes() {
    // <header hidden> : hidden 14..20, no `=`.
    assert_rows(
        "<body><header hidden></header>",
        &[Row {
            node: 4,
            complete: (14, 20),
            name: (14, 20),
            syntax: Syntax::Missing,
            interpreted_name: "hidden",
            interpreted_value: "",
        }],
    );
    // <footer a=> : a 14..15 = 15..16, empty boundary at the attribute end.
    assert_rows(
        "<body><footer a=></footer>",
        &[Row {
            node: 4,
            complete: (14, 16),
            name: (14, 15),
            syntax: Syntax::MissingAfterEquals {
                equals: (15, 16),
                boundary: (16, 16),
            },
            interpreted_name: "a",
            interpreted_value: "",
        }],
    );
}

#[test]
fn mixed_authored_case_keeps_authored_spelling_and_normalized_interpretation() {
    // <DIV ID=A> : DIV 7..10, ID 11..13 = 13..14 A 14..15
    let text = "<body><DIV ID=A></DIV>";
    assert_rows(
        text,
        &[Row {
            node: 4,
            complete: (11, 15),
            name: (11, 13),
            syntax: Syntax::Unquoted {
                equals: (13, 14),
                value: (14, 15),
            },
            interpreted_name: "id",
            interpreted_value: "A",
        }],
    );
    let report = analyze_text(text);
    assert_eq!(
        report.selected_ordinary_attributes()[0]
            .authored_name()
            .fragment(),
        "ID"
    );
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Div);
}

#[test]
fn all_eight_selected_names_project_one_row_each() {
    let names = [
        ("div", HtmlTreeElementName::Div),
        ("section", HtmlTreeElementName::Section),
        ("article", HtmlTreeElementName::Article),
        ("aside", HtmlTreeElementName::Aside),
        ("footer", HtmlTreeElementName::Footer),
        ("header", HtmlTreeElementName::Header),
        ("main", HtmlTreeElementName::Main),
        ("nav", HtmlTreeElementName::Nav),
    ];
    for (name, expected) in names {
        let text = format!("<body><{name} k=v></{name}>");
        let report = analyze_text(&text);
        assert_complete(&report);
        // `<` at 6, name from 7, one space, then `k=v`.
        let k = 7 + name.len() + 1;
        let rows = report.selected_ordinary_attributes();
        assert_eq!(rows.len(), 1, "{name}");
        assert_eq!(element(&report, 4).name(), expected, "{name}");
        assert_row(
            &text,
            &rows[0],
            &Row {
                node: 4,
                complete: (k, k + 3),
                name: (k, k + 1),
                syntax: Syntax::Unquoted {
                    equals: (k + 1, k + 2),
                    value: (k + 2, k + 3),
                },
                interpreted_name: "k",
                interpreted_value: "v",
            },
        );
    }
}

#[test]
fn same_name_nested_nodes_keep_their_own_attributes() {
    // outer tag 6..26 (id="outer" 15..25), inner tag 26..46 (id="inner" 35..45)
    assert_rows(
        r#"<body><article id="outer"><article id="inner"></article></article>"#,
        &[
            Row {
                node: 4,
                complete: (15, 25),
                name: (15, 17),
                syntax: Syntax::DoubleQuoted {
                    equals: (17, 18),
                    open: (18, 19),
                    value: (19, 24),
                    close: (24, 25),
                },
                interpreted_name: "id",
                interpreted_value: "outer",
            },
            Row {
                node: 5,
                complete: (35, 45),
                name: (35, 37),
                syntax: Syntax::DoubleQuoted {
                    equals: (37, 38),
                    open: (38, 39),
                    value: (39, 44),
                    close: (44, 45),
                },
                interpreted_name: "id",
                interpreted_value: "inner",
            },
        ],
    );
}

#[test]
fn heterogeneous_nodes_keep_their_own_attributes() {
    // article tag 6..22 (id="a" 15..21), nav tag 22..37 (class="b" 27..36)
    let text = r#"<body><article id="a"><nav class="b"></nav></article>"#;
    assert_rows(
        text,
        &[
            Row {
                node: 4,
                complete: (15, 21),
                name: (15, 17),
                syntax: Syntax::DoubleQuoted {
                    equals: (17, 18),
                    open: (18, 19),
                    value: (19, 20),
                    close: (20, 21),
                },
                interpreted_name: "id",
                interpreted_value: "a",
            },
            Row {
                node: 5,
                complete: (27, 36),
                name: (27, 32),
                syntax: Syntax::DoubleQuoted {
                    equals: (32, 33),
                    open: (33, 34),
                    value: (34, 35),
                    close: (35, 36),
                },
                interpreted_name: "class",
                interpreted_value: "b",
            },
        ],
    );
    let report = analyze_text(text);
    assert_eq!(element(&report, 4).name(), HtmlTreeElementName::Article);
    assert_eq!(element(&report, 5).name(), HtmlTreeElementName::Nav);
}

#[test]
fn one_row_per_attributed_node_and_none_for_its_zero_attribute_neighbours() {
    // <div> 6..11 has none; <nav k=v> 11..20 has one (k 16..17 = 17..18 v 18..19).
    assert_rows(
        "<body><div><nav k=v></nav></div>",
        &[Row {
            node: 5,
            complete: (16, 19),
            name: (16, 17),
            syntax: Syntax::Unquoted {
                equals: (17, 18),
                value: (18, 19),
            },
            interpreted_name: "k",
            interpreted_value: "v",
        }],
    );
}

#[test]
fn paragraph_closing_attributed_selected_start_keeps_the_new_node_identity() {
    // <p>x then <div id=a>: node 4 = p, 5 = text, 6 = div. The start tag that
    // implicitly closes the paragraph still keys its row to the div.
    let text = "<body><p>x<div id=a>";
    let report = analyze_text(text);
    let rows = report.selected_ordinary_attributes();
    assert_eq!(rows.len(), 1);
    let node = report.node(rows[0].node()).expect("row node resolves");
    let HtmlTreeNodeKind::Element(div) = node.kind() else {
        panic!("row node is not an element")
    };
    assert_eq!(div.name(), HtmlTreeElementName::Div);
    // `<div` 10..14, space 14, id 15..17, = 17..18, a 18..19.
    assert_row(
        text,
        &rows[0],
        &Row {
            node: rows[0].node().value(),
            complete: (15, 19),
            name: (15, 17),
            syntax: Syntax::Unquoted {
                equals: (17, 18),
                value: (18, 19),
            },
            interpreted_name: "id",
            interpreted_value: "a",
        },
    );
}

#[test]
fn raw_nul_stays_authored_while_the_interpreted_value_is_replacement() {
    // <div a="\0"> : a 11..12 = 12..13 " 13..14 NUL 14..15 " 15..16
    let text = "<body><div a=\"\u{0}\"></div>";
    assert_rows(
        text,
        &[Row {
            node: 4,
            complete: (11, 16),
            name: (11, 12),
            syntax: Syntax::DoubleQuoted {
                equals: (12, 13),
                open: (13, 14),
                value: (14, 15),
                close: (15, 16),
            },
            interpreted_name: "a",
            interpreted_value: "\u{fffd}",
        }],
    );
    let report = analyze_text(text);
    let HtmlTreeAttributeValueSyntax::DoubleQuoted { value, .. } =
        report.selected_ordinary_attributes()[0].value_syntax()
    else {
        panic!("double quoted")
    };
    assert_eq!(value.fragment(), "\u{0}");
    // Diagnostic ownership stays with the tokenizer.
    let codes: Vec<_> = report
        .tokenizer_diagnostics()
        .iter()
        .map(HtmlTokenizerDiagnostic::code)
        .collect();
    assert_eq!(
        codes,
        [HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter]
    );
}

#[test]
fn attribute_value_character_reference_stays_a_lower_layer_stop_without_a_row() {
    let report = analyze_text("<body><div id=\"a&amp;b\"></div>");
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TokenizerUnsupported(unsupported)) =
        report.completion()
    else {
        panic!(
            "expected tokenizer unsupported, got {:?}",
            report.completion()
        );
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTokenizerUnsupportedCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::AttributeValue
        }
    );
    assert!(report.selected_ordinary_attributes().is_empty());
    assert_eq!(report.nodes().len(), 4, "no selected node was constructed");
}

#[test]
fn second_attribute_stays_a_resource_stop_without_a_row_or_node() {
    let report = analyze_text("<body><div x y></div>");
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::AttributesPerTag);
    assert_eq!((limit.limit(), limit.attempted()), (1, 2));
    assert!(report.selected_ordinary_attributes().is_empty());
    assert_eq!(report.nodes().len(), 4);
}

#[test]
fn a_later_unsupported_stop_keeps_the_earlier_committed_row() {
    // <div id="a"> 6..18 commits; `<span>` then stops the run.
    let text = r#"<body><div id="a"><span>"#;
    let report = analyze_text(text);
    assert!(matches!(
        report.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(_))
    ));
    assert_eq!(report.selected_ordinary_attributes().len(), 1);
    assert_row(
        text,
        &report.selected_ordinary_attributes()[0],
        &Row {
            node: 4,
            complete: (11, 17),
            name: (11, 13),
            syntax: Syntax::DoubleQuoted {
                equals: (13, 14),
                open: (14, 15),
                value: (15, 16),
                close: (16, 17),
            },
            interpreted_name: "id",
            interpreted_value: "a",
        },
    );
}

#[test]
fn out_of_domain_and_refused_tags_never_produce_rows() {
    for text in [
        "<body><p id=x>",
        "<body><span id=x>",
        "<body><style id=x>",
        "<body><title id=x>",
        "<html lang=en>",
        "<body a>",
        "<body><div a=\"b\"/>",
        "<body><div></div id=x>",
        "<body></p>",
    ] {
        let report = analyze_text(text);
        assert!(report.selected_ordinary_attributes().is_empty(), "{text:?}");
    }
}

#[test]
fn attributed_end_tag_leaves_the_zero_attribute_start_without_a_row() {
    let report = analyze_text("<body><div></div id=x>");
    assert!(matches!(
        report.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(u))
            if u.capability() == HtmlTreeUnsupportedCapability::SelectedOrdinaryTagAttribute
    ));
    assert!(report.selected_ordinary_attributes().is_empty());
}

#[test]
fn attribute_projection_is_deterministic_and_does_not_change_other_report_data() {
    let text = r#"<body><article id="o"><nav class=b></nav></article>"#;
    let first = analyze_text(text);
    let second = analyze_text(text);
    assert_eq!(
        format!("{:?}", first.selected_ordinary_attributes()),
        format!("{:?}", second.selected_ordinary_attributes())
    );
    assert_eq!(outline(&first), outline(&second));
    assert_complete(&first);
}

#[test]
fn value_syntax_domain_is_closed_to_exactly_the_five_forms() {
    // No wildcard arm: adding, removing, or making a variant non-exhaustive
    // fails to compile (within-crate matching would not detect
    // `#[non_exhaustive]`, which is checked by the public-API source audit).
    fn form(syntax: &HtmlTreeAttributeValueSyntax) -> &'static str {
        match syntax {
            HtmlTreeAttributeValueSyntax::Missing => "missing",
            HtmlTreeAttributeValueSyntax::MissingAfterEquals { .. } => "missing-after-equals",
            HtmlTreeAttributeValueSyntax::Unquoted { .. } => "unquoted",
            HtmlTreeAttributeValueSyntax::DoubleQuoted { .. } => "double-quoted",
            HtmlTreeAttributeValueSyntax::SingleQuoted { .. } => "single-quoted",
        }
    }
    let report = analyze_text("<body><div a></div>");
    assert_eq!(
        form(report.selected_ordinary_attributes()[0].value_syntax()),
        "missing"
    );
}
