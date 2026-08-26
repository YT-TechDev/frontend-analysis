from pathlib import Path
import re

ROOT = Path("crates/frontend-analysis-core/src/html")


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    assert count == 1, (path, "replace_once count", count, old[:80])
    path.write_text(text.replace(old, new, 1))


p_test = ROOT / "tree_construction/in_body_p_successor_production.rs"
text = p_test.read_text()
assert text.count("construct_document_shell") == 2
p_test.write_text(text.replace("construct_document_shell", "construct_html_document_shell"))

result = ROOT / "tree_construction/result.rs"
old = '''                    let next_matches = match (expected, next.kind()) {
                        (
                            Some("p"),
                            HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. },
                        ) => true,
                        (
                            Some("div"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Div,
                                ..
                            },
                        ) => true,
                        (
                            Some("section"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Section,
                                ..
                            },
                        ) => true,
                        _ => false,
                    };'''
new = '''                    let next_matches = matches!(
                        (expected, next.kind()),
                        (
                            Some("p"),
                            HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. },
                        ) | (
                            Some("div"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Div,
                                ..
                            },
                        ) | (
                            Some("section"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Section,
                                ..
                            },
                        )
                    );'''
replace_once(result, old, new)

for rel, expected_parts, label in [
    ("tree_construction/in_body_div_successor_production.rs", 5, "TC-S3"),
    ("tree_construction/in_body_div_section_successor_production.rs", 7, "TC-S4"),
]:
    path = ROOT / rel
    text = path.read_text()
    node_marker = "        HtmlTreeNodeKind::Text(text) => ExpectedNode::Text {"
    assert text.count(node_marker) == 1, (path, "node marker")
    text = text.replace(
        node_marker,
        f'''        HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)) => {{
            panic!("{label} predecessor fixtures must not construct a TC-S5 Paragraph")
        }}
{node_marker}''',
        1,
    )
    action_marker = (
        "                HtmlTreeActionKind::ReprocessedToken => ExpectedAction::Reprocessed,"
    )
    assert text.count(action_marker) == 1, (path, "action marker")
    text = text.replace(
        action_marker,
        f'''                HtmlTreeActionKind::InsertedAuthoredParagraphElement {{ .. }}
                | HtmlTreeActionKind::InsertedSynthesizedParagraphElement {{ .. }}
                | HtmlTreeActionKind::ClosedParagraphElement {{ .. }} => {{
                    panic!("{label} predecessor fixtures must not record a TC-S5 Paragraph action")
                }}
{action_marker}''',
        1,
    )
    text, count = re.subn(
        r"(?m)^(\s*)final_open_selected_ordinary: ([^\n]+),\n",
        r"\1final_open_selected_ordinary: \2,\n\1final_open_paragraph: None,\n",
        text,
    )
    assert count == expected_parts, (path, "parts count", count, expected_parts)
    path.write_text(text)

validation = ROOT / "tree_construction/validation.rs"
text = validation.read_text()
marker = "        HtmlTreeNodeKind::Text(text) => GoldNode::Text {"
assert text.count(marker) == 1
text = text.replace(
    marker,
    '''        HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)) => {
            panic!("the TC-S1 predecessor GOLD never constructs a TC-S5 Paragraph")
        }
'''
    + marker,
    1,
)
old_diag = '''            HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
            | HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile
            | HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag => {
                panic!("the TC-S1 predecessor GOLD never produces a selected ordinary diagnostic")
            }'''
new_diag = (
    old_diag
    + '''
            HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag => {
                panic!("the TC-S1 predecessor GOLD never produces a TC-S5 Paragraph diagnostic")
            }'''
)
assert text.count(old_diag) == 1
text = text.replace(old_diag, new_diag, 1)
old_provenance = '''                HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(_)) => {
                    unreachable!("no TC-S1 GOLD case constructs a selected ordinary element")
                }'''
new_provenance = (
    old_provenance
    + '''
                HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)) => {
                    unreachable!("no TC-S1 GOLD case constructs a TC-S5 Paragraph")
                }'''
)
assert text.count(old_provenance) == 1
text = text.replace(old_provenance, new_provenance, 1)
text, count = re.subn(
    r"(?m)^(\s*)final_open_selected_ordinary: ([^\n]+),\n",
    r"\1final_open_selected_ordinary: \2,\n\1final_open_paragraph: None,\n",
    text,
)
assert count == 1, ("validation parts count", count)
validation.write_text(text)

gate = ROOT / "tokenizer/validation/tree_construction_gate.rs"
text = gate.read_text()
old_gate = '''                HtmlElement::SelectedOrdinary(selected) => match selected.name() {
                    HtmlSelectedOrdinaryElementName::Div => "div",
                    HtmlSelectedOrdinaryElementName::Section => "section",
                },'''
new_gate = old_gate + '''
                HtmlElement::Paragraph(_) => "p",'''
assert text.count(old_gate) == 1
gate.write_text(text.replace(old_gate, new_gate, 1))
