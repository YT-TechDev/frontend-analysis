//! Production correspondence for the selected in-body block-container family
//! expansion (Issue #896).
//!
//! The selected ordinary domain is the closed set
//! `{div, section, article, aside, footer, header, main, nav}`. Every
//! expectation here is hand-authored from the accepted theorem and the exact
//! source bytes each case names. Nothing in this module imports, calls, or
//! reads any value from
//! [`super::in_body_block_container_family_successor_validation`]: that
//! module keeps its own candidate machine, fixtures, and GOLD, and stays
//! unchanged.
//!
//! The expected vocabulary is plain strings. A production result is projected
//! into the same strings by `Run`, which carries no expectation of its own.
//! Elements are named by their normalized closed name plus the exact authored
//! raw name text, and relations are named by the authored start offset of each
//! endpoint, so a recovery pop stays distinguishable from a matching closure
//! without promising any identity representation.

use crate::html::tree::{
    HtmlTreeElementName, HtmlTreeIncompleteCause, HtmlTreeNodeKind, HtmlTreeNodeProvenance,
    HtmlTreeUnsupportedCapability, analyze_selected_document_tree,
};
use crate::{SourceAnchor, SourceId, SourceText};

use super::driver::construct_html_document_shell;
use super::result::{
    HtmlDocumentShellAnalysis, HtmlElement, HtmlShellElementOrigin, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeIncompleteCause as InternalIncompleteCause,
    HtmlTreeNodeKind as InternalNodeKind,
};

use super::super::tokenizer::resource::HtmlTokenizerLimits;

const NEW_NAMES: [&str; 6] = ["article", "aside", "footer", "header", "main", "nav"];
const ALL_NAMES: [&str; 8] = [
    "div", "section", "article", "aside", "footer", "header", "main", "nav",
];

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze_with(source: &str, id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(id), source.to_owned());
    construct_html_document_shell(&source, limits()).expect("no boundary failure")
}

fn text_of<'a>(source: &'a str, anchor: &SourceAnchor) -> (&'a str, usize) {
    let range = anchor.range();
    (&source[range.start()..range.end()], range.start())
}

/// A production result projected into plain strings.
struct Run {
    tree: String,
    closures: Vec<String>,
    recoveries: Vec<String>,
    ignored: Vec<String>,
    diagnostics: Vec<String>,
    completion: String,
}

fn render_node(
    analysis: &HtmlDocumentShellAnalysis,
    source: &str,
    id: super::result::HtmlConstructedNodeId,
) -> String {
    let node = analysis.node(id).expect("child resolves");
    let children: String = node
        .children()
        .iter()
        .map(|child| render_node(analysis, source, *child))
        .collect::<Vec<_>>()
        .join(",");
    match node.kind() {
        InternalNodeKind::Document => format!("#({children})"),
        InternalNodeKind::Element(HtmlElement::Shell(shell)) => {
            let synthesized = matches!(shell.origin(), HtmlShellElementOrigin::Synthesized(_));
            let name = match shell.name() {
                super::result::HtmlShellElementName::Html => "html",
                super::result::HtmlShellElementName::Head => "head",
                super::result::HtmlShellElementName::Body => "body",
            };
            format!("{name}{}({children})", if synthesized { "*" } else { "" })
        }
        InternalNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => {
            let (raw, _) = text_of(source, selected.raw_name());
            let (complete, _) = text_of(source, selected.complete());
            assert!(
                complete.starts_with('<') && complete.ends_with('>'),
                "complete evidence is the whole authored start tag: {complete}"
            );
            assert!(
                complete[1..].starts_with(raw),
                "raw name is the authored spelling inside the start tag"
            );
            assert!(raw.eq_ignore_ascii_case(selected.name().interpreted()));
            format!("{}:{raw}({children})", selected.name().interpreted())
        }
        InternalNodeKind::Element(HtmlElement::Paragraph(_)) => format!("p({children})"),
        InternalNodeKind::Element(_) => panic!("unexpected element kind in family fixture"),
        InternalNodeKind::DocumentType(_) => panic!("family fixtures carry no DOCTYPE"),
        InternalNodeKind::Text(text) => format!("{:?}", text.interpreted()),
    }
}

fn start_of(
    analysis: &HtmlDocumentShellAnalysis,
    id: super::result::HtmlConstructedNodeId,
) -> String {
    let node = analysis.node(id).expect("endpoint resolves");
    let InternalNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) = node.kind() else {
        panic!("relation endpoint must be a selected ordinary element")
    };
    format!(
        "{}@{}",
        selected.name().interpreted(),
        selected.complete().range().start()
    )
}

fn trigger_text(action: &super::result::HtmlTreeAction, source: &str) -> String {
    let (text, at) = text_of(
        source,
        action
            .trigger()
            .authored_boundary()
            .expect("authored end tag trigger"),
    );
    format!("{text}@{at}")
}

fn run_with(source: &str, id: u64) -> Run {
    let analysis = analyze_with(source, id);
    let tree = render_node(&analysis, source, analysis.root());
    let mut closures = Vec::new();
    let mut recoveries = Vec::new();
    let mut ignored = Vec::new();
    for action in analysis.actions() {
        match action.kind() {
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, .. } => {
                closures.push(format!(
                    "{} by {}",
                    start_of(&analysis, *node),
                    trigger_text(action, source)
                ))
            }
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, target } => {
                assert_ne!(node, target);
                recoveries.push(format!(
                    "{}=>{} by {}",
                    start_of(&analysis, *node),
                    start_of(&analysis, *target),
                    trigger_text(action, source)
                ))
            }
            HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name } => ignored.push(
                format!("{} by {}", name.interpreted(), trigger_text(action, source)),
            ),
            _ => {}
        }
    }
    let diagnostics = analysis
        .diagnostics()
        .iter()
        .map(|d| format!("{:?}", d.code()))
        .filter(|code| code != "MissingDoctype")
        .collect();
    let completion = match analysis.completion() {
        HtmlTreeCompletion::Complete => "complete".to_owned(),
        HtmlTreeCompletion::Incomplete(InternalIncompleteCause::UnsupportedCapability(u)) => {
            format!("unsupported:{:?}", u.capability())
        }
        HtmlTreeCompletion::Incomplete(other) => format!("incomplete:{other:?}"),
    };
    Run {
        tree,
        closures,
        recoveries,
        ignored,
        diagnostics,
        completion,
    }
}

fn run(source: &str) -> Run {
    run_with(source, 1)
}

fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn each_new_name_is_inserted_and_closed_by_its_own_end_tag() {
    for name in NEW_NAMES {
        let source = format!("<body><{name}></{name}>");
        let got = run(&source);
        assert_eq!(
            got.tree,
            format!("#(html*(head*(),body({name}:{name}())))"),
            "{source}"
        );
        assert_eq!(
            got.closures,
            vec![format!("{name}@6 by </{name}>@{}", 6 + name.len() + 2)],
            "{source}"
        );
        assert!(
            got.recoveries.is_empty() && got.ignored.is_empty(),
            "{source}"
        );
        assert!(
            got.diagnostics.is_empty(),
            "{source}: {:?}",
            got.diagnostics
        );
        assert_eq!(got.completion, "complete", "{source}");
    }
}

#[test]
fn mixed_case_keeps_normalized_name_and_exact_raw_spelling_distinct() {
    let got = run("<body><ArTiClE>x</aRtIcLe>");
    assert_eq!(got.tree, "#(html*(head*(),body(article:ArTiClE(\"x\"))))");
    assert_eq!(got.closures, strs(&["article@6 by </aRtIcLe>@16"]));
    let got = run("<body><NaV>x</nAv>");
    assert_eq!(got.tree, "#(html*(head*(),body(nav:NaV(\"x\"))))");
    assert_eq!(got.closures, strs(&["nav@6 by </nAv>@12"]));
}

#[test]
fn same_name_nesting_closes_innermost_first() {
    let got = run("<body><article><article>x</article></article>");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(article:article(article:article(\"x\")))))"
    );
    assert_eq!(
        got.closures,
        strs(&["article@15 by </article>@25", "article@6 by </article>@35"])
    );
    assert!(got.recoveries.is_empty());
    let got = run("<body><nav><nav>x</nav></nav>");
    assert_eq!(got.tree, "#(html*(head*(),body(nav:nav(nav:nav(\"x\")))))");
    assert_eq!(
        got.closures,
        strs(&["nav@11 by </nav>@17", "nav@6 by </nav>@23"])
    );
}

#[test]
fn existing_and_new_names_compose_in_both_directions() {
    let cases = [
        (
            "<body><div><article></article></div>",
            "#(html*(head*(),body(div:div(article:article()))))",
        ),
        (
            "<body><section><main></main></section>",
            "#(html*(head*(),body(section:section(main:main()))))",
        ),
        (
            "<body><article><section></section></article>",
            "#(html*(head*(),body(article:article(section:section()))))",
        ),
        (
            "<body><nav><div></div></nav>",
            "#(html*(head*(),body(nav:nav(div:div()))))",
        ),
    ];
    for (source, tree) in cases {
        let got = run(source);
        assert_eq!(got.tree, tree, "{source}");
        assert_eq!(got.closures.len(), 2, "{source}");
        assert!(
            got.recoveries.is_empty() && got.diagnostics.is_empty(),
            "{source}"
        );
    }
}

#[test]
fn heterogeneous_recovery_pop_is_distinct_from_target_closure() {
    let got = run("<body><article><nav></article>");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(article:article(nav:nav()))))"
    );
    assert_eq!(
        got.recoveries,
        strs(&["nav@15=>article@6 by </article>@20"])
    );
    assert_eq!(got.closures, strs(&["article@6 by </article>@20"]));
    assert_eq!(got.diagnostics, strs(&["MisnestedSelectedOrdinaryEndTag"]));
    assert_eq!(got.completion, "complete");

    let got = run("<body><section><main></section>");
    assert_eq!(
        got.recoveries,
        strs(&["main@15=>section@6 by </section>@21"])
    );
    assert_eq!(got.closures, strs(&["section@6 by </section>@21"]));

    let got = run("<body><nav><div></nav>");
    assert_eq!(got.recoveries, strs(&["div@11=>nav@6 by </nav>@16"]));
    assert_eq!(got.closures, strs(&["nav@6 by </nav>@16"]));
}

#[test]
fn multiple_recovery_pops_run_current_to_target_before_one_closure() {
    let got = run("<body><header><main><aside></header>");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(header:header(main:main(aside:aside())))))"
    );
    assert_eq!(
        got.recoveries,
        strs(&[
            "aside@20=>header@6 by </header>@27",
            "main@14=>header@6 by </header>@27"
        ])
    );
    assert_eq!(got.closures, strs(&["header@6 by </header>@27"]));
    assert_eq!(got.diagnostics, strs(&["MisnestedSelectedOrdinaryEndTag"]));
}

#[test]
fn nearest_same_name_open_target_is_used() {
    let got = run("<body><nav><main><nav><div></nav></nav>");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(nav:nav(main:main(nav:nav(div:div()))))))"
    );
    assert_eq!(
        got.recoveries,
        strs(&["div@22=>nav@17 by </nav>@27", "main@11=>nav@6 by </nav>@33"])
    );
    assert_eq!(
        got.closures,
        strs(&["nav@17 by </nav>@27", "nav@6 by </nav>@33"])
    );
}

#[test]
fn new_names_close_a_current_paragraph_before_insertion() {
    let got = run("<body><p>x<article>y</article>");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(p(\"x\"),article:article(\"y\"))))"
    );
    let got = run("<body><p>x<nav>y</nav>");
    assert_eq!(got.tree, "#(html*(head*(),body(p(\"x\"),nav:nav(\"y\"))))");
}

#[test]
fn selected_end_over_current_paragraph_pops_it_then_closes() {
    let got = run("<body><article><p>x</article>");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(article:article(p(\"x\")))))"
    );
    assert_eq!(got.closures, strs(&["article@6 by </article>@19"]));
    assert!(
        got.recoveries.is_empty(),
        "a paragraph is not a selected recovery pop"
    );
}

#[test]
fn unmatched_selected_ends_are_ignored_without_nodes() {
    let got = run("<body></article>");
    assert_eq!(got.tree, "#(html*(head*(),body()))");
    assert_eq!(got.ignored, strs(&["article by </article>@6"]));
    assert_eq!(got.diagnostics, strs(&["UnmatchedSelectedOrdinaryEndTag"]));
    let got = run("<body></nav>");
    assert_eq!(got.ignored, strs(&["nav by </nav>@6"]));
    let got = run("<body><div></aside></div>");
    assert_eq!(got.tree, "#(html*(head*(),body(div:div())))");
    assert_eq!(got.ignored, strs(&["aside by </aside>@11"]));
    assert_eq!(got.closures, strs(&["div@6 by </div>@19"]));
}

#[test]
fn eof_open_suffix_fabricates_no_closure() {
    let got = run("<body><article>x");
    assert_eq!(got.tree, "#(html*(head*(),body(article:article(\"x\"))))");
    assert!(got.closures.is_empty() && got.recoveries.is_empty());
    assert_eq!(
        got.diagnostics,
        strs(&["OpenSelectedOrdinaryElementAtEndOfFile"])
    );
    let got = run("<body><div><nav>x");
    assert_eq!(got.tree, "#(html*(head*(),body(div:div(nav:nav(\"x\")))))");
    assert!(got.closures.is_empty() && got.recoveries.is_empty());
}

#[test]
fn text_stays_under_the_current_selected_element() {
    let got = run("<body>a<main>b<nav>c</nav>d</main>e");
    assert_eq!(
        got.tree,
        "#(html*(head*(),body(\"a\",main:main(\"b\",nav:nav(\"c\"),\"d\"),\"e\")))"
    );
}

#[test]
fn attributes_self_closing_wrong_mode_and_nearby_names_are_refused() {
    let cases = [
        ("<body><article id=x>", "SelectedOrdinaryTagAttribute"),
        ("<body><nav/>", "SelfClosingSelectedOrdinaryTag"),
        ("<body></body><article>", "SelectedOrdinaryTagOutsideInBody"),
        ("<body><span>", "NonShellElementTag"),
        ("<body><search>", "NonShellElementTag"),
        ("<body><ul>", "NonShellElementTag"),
    ];
    for (source, capability) in cases {
        let got = run(source);
        assert_eq!(
            got.completion,
            format!("unsupported:{capability}"),
            "{source}"
        );
        assert!(
            got.closures.is_empty() && got.recoveries.is_empty(),
            "{source}"
        );
    }
}

#[test]
fn equal_bytes_under_distinct_source_ids_keep_distinct_evidence() {
    let source = "<body><aside><footer></aside>";
    let (a, b) = (run_with(source, 7), run_with(source, 8));
    assert_eq!(a.tree, b.tree);
    assert_eq!(a.closures, b.closures);
    assert_eq!(a.recoveries, b.recoveries);
    let ids = |id: u64| -> Vec<SourceId> {
        let text = SourceText::new(SourceId::new(id), source.to_owned());
        let report = analyze_selected_document_tree(&text).expect("report");
        report
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                HtmlTreeNodeKind::Element(e) if e.name() != HtmlTreeElementName::Body => {
                    match e.provenance() {
                        HtmlTreeNodeProvenance::AuthoredStartTag { complete, .. } => {
                            Some(complete.source_id())
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(ids(7), vec![SourceId::new(7); 2]);
    assert_eq!(ids(8), vec![SourceId::new(8); 2]);
}

#[test]
fn every_ordered_distinct_name_pair_recovers_and_nests_by_name_alone() {
    let mut pairs = 0;
    for outer in ALL_NAMES {
        for inner in ALL_NAMES {
            if outer == inner {
                continue;
            }
            pairs += 1;
            let tree = format!("#(html*(head*(),body({outer}:{outer}({inner}:{inner}()))))");
            let mismatched = run(&format!("<body><{outer}><{inner}></{outer}>"));
            assert_eq!(mismatched.tree, tree, "{outer}/{inner}");
            let outer_end = 6 + outer.len() + 2 + inner.len() + 2;
            assert_eq!(
                mismatched.recoveries,
                vec![format!(
                    "{inner}@{}=>{outer}@6 by </{outer}>@{outer_end}",
                    6 + outer.len() + 2
                )],
                "{outer}/{inner}"
            );
            assert_eq!(
                mismatched.closures,
                vec![format!("{outer}@6 by </{outer}>@{outer_end}")],
                "{outer}/{inner}"
            );
            let nested = run(&format!("<body><{outer}><{inner}></{inner}></{outer}>"));
            assert_eq!(nested.tree, tree, "{outer}/{inner}");
            assert_eq!(nested.closures.len(), 2);
            assert!(nested.recoveries.is_empty() && nested.diagnostics.is_empty());
        }
    }
    assert_eq!(pairs, 56);
}

#[test]
fn deep_mixed_stack_recovers_every_intervening_element_once() {
    let source = "<body><div><section><article><aside><footer><header><main><nav></div>";
    let got = run(source);
    assert_eq!(got.recoveries.len(), 7);
    assert!(got.recoveries[0].starts_with("nav@"));
    assert!(got.recoveries[6].starts_with("section@"));
    assert_eq!(got.closures, strs(&["div@6 by </div>@63"]));
    assert_eq!(got.completion, "complete");
}

#[test]
fn public_projection_maps_each_internal_name_to_its_public_variant() {
    let expected = [
        ("div", HtmlTreeElementName::Div),
        ("section", HtmlTreeElementName::Section),
        ("article", HtmlTreeElementName::Article),
        ("aside", HtmlTreeElementName::Aside),
        ("footer", HtmlTreeElementName::Footer),
        ("header", HtmlTreeElementName::Header),
        ("main", HtmlTreeElementName::Main),
        ("nav", HtmlTreeElementName::Nav),
    ];
    for (name, variant) in expected {
        let source_text = format!("<body><{name}></{name}>");
        let source = SourceText::new(SourceId::new(3), source_text.clone());
        let report = analyze_selected_document_tree(&source).expect("report");
        assert!(matches!(
            report.completion(),
            crate::html::tree::HtmlTreeCompletion::Complete
        ));
        let found: Vec<_> = report
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                HtmlTreeNodeKind::Element(e)
                    if matches!(
                        e.provenance(),
                        HtmlTreeNodeProvenance::AuthoredStartTag { .. }
                    ) && !matches!(e.name(), HtmlTreeElementName::Body) =>
                {
                    Some(e.name())
                }
                _ => None,
            })
            .collect();
        assert_eq!(found, vec![variant], "{name}");
    }
    let refused = SourceText::new(SourceId::new(3), "<body><nav id=x>".to_owned());
    let report = analyze_selected_document_tree(&refused).expect("report");
    assert!(matches!(
        report.completion(),
        crate::html::tree::HtmlTreeCompletion::Incomplete(
            HtmlTreeIncompleteCause::TreeUnsupported(u)
        ) if u.capability() == HtmlTreeUnsupportedCapability::SelectedOrdinaryTagAttribute
    ));
    let _ = HtmlTreeCapability::SelectedOrdinaryTagAttribute;
}
