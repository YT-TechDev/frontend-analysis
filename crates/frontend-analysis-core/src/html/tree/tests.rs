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
