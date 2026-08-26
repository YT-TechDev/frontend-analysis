from pathlib import Path

path = Path("crates/frontend-analysis-core/src/html/tree_construction/in_body_html_end_open_stack_successor_validation.rs")
text = path.read_text()

def replace_once(old: str, new: str) -> None:
    global text
    assert text.count(old) == 1, f"expected exactly one match: {old[:80]!r}"
    text = text.replace(old, new, 1)

replace_once(
    "struct RefusalRecord {\n    capability: Unsupported,\n    token_index: usize,\n    before: Snapshot,",
    "struct RefusalRecord {\n    capability: Unsupported,\n    token_index: usize,\n    trigger: Evidence,\n    before: Snapshot,",
)

replace_once(
    "                refusal = Some(RefusalRecord {\n                    capability,\n                    token_index,\n                    before,",
    "                refusal = Some(RefusalRecord {\n                    capability,\n                    token_index,\n                    trigger: token_evidence(token),\n                    before,",
)

replace_once(
    "struct Oracle {\n    audit_diagnostics: usize,\n    final_stack: Vec<Name>,\n    mode_path: Vec<Phase>,\n    reprocess_count: usize,\n}",
    "struct Oracle {\n    audit_diagnostics: usize,\n    final_stack: Vec<Name>,\n    mode_path: Vec<Phase>,\n    reprocess_count: usize,\n    expected_next_id: usize,\n}",
)

replace_once(
    "        mode_path: vec![Phase::InBody, Phase::AfterBody, Phase::AfterAfterBody],\n        reprocess_count: 1,\n    }",
    "        mode_path: vec![Phase::InBody, Phase::AfterBody, Phase::AfterAfterBody],\n        reprocess_count: 1,\n        expected_next_id: 2 + blocks.len() + usize::from(p) * 2,\n    }",
)

replace_once(
    "    let token_index = candidate_token_index(&observation);\n    let actions = candidate_actions(&observation);\n\n    let expected_kinds",
    "    let token_index = candidate_token_index(&observation);\n    let actions = candidate_actions(&observation);\n\n    let source_text = SourceText::new(SourceId::new(77), source.to_owned());\n    let run = tokenize(&source_text, limits());\n    let HtmlToken::Tag(candidate_tag) = &run.tokens()[token_index] else {\n        panic!(\"candidate token is an html end tag\")\n    };\n    assert_eq!(candidate_tag.kind(), HtmlTagKind::End);\n    assert_eq!(candidate_tag.name().interpreted(), \"html\");\n    let raw_name = candidate_tag.name().source();\n    assert_eq!(raw_name.source_id(), SourceId::new(77));\n    assert_eq!((raw_name.range().start(), raw_name.range().end()), (14, 18));\n    assert_eq!(&source[raw_name.range().start()..raw_name.range().end()], \"HtMl\");\n\n    let expected_kinds",
)

old_cases = '''    let cases = [
        (
            "<body><div><body>",
            Unsupported::BodyStartWithOpenBoundedStack,
        ),
        ("<body><div></html x>", Unsupported::HtmlEndAttribute),
        ("<body><div></html/>", Unsupported::HtmlEndSelfClosing),
        ("<body><div></head>", Unsupported::OtherShellEnd),
        (
            "<body><div></html> ",
            Unsupported::AfterAfterBodyCharacterData,
        ),
        (
            "<body><div></html>x",
            Unsupported::AfterAfterBodyCharacterData,
        ),
        ("<body><div></html></div>", Unsupported::AfterAfterBodyTag),
    ];

    for (source, capability) in cases {
        let observation = observe(source);
        assert_transactional_refusal(&observation, capability);
        assert!(
            matches!(observation.completion, Completion::Unsupported { capability: actual, .. } if actual == capability)
        );
    }'''
new_cases = '''    let cases = [
        (
            "<body><div><body>",
            Unsupported::BodyStartWithOpenBoundedStack,
            (11usize, 17usize),
        ),
        (
            "<body><div></html x>",
            Unsupported::HtmlEndAttribute,
            (11, 20),
        ),
        (
            "<body><div></html/>",
            Unsupported::HtmlEndSelfClosing,
            (11, 19),
        ),
        (
            "<body><div></head>",
            Unsupported::OtherShellEnd,
            (11, 18),
        ),
        (
            "<body><div></html> ",
            Unsupported::AfterAfterBodyCharacterData,
            (18, 19),
        ),
        (
            "<body><div></html>x",
            Unsupported::AfterAfterBodyCharacterData,
            (18, 19),
        ),
        (
            "<body><div></html></div>",
            Unsupported::AfterAfterBodyTag,
            (18, 24),
        ),
    ];

    for (source, capability, expected_range) in cases {
        let observation = observe(source);
        assert_transactional_refusal(&observation, capability);
        let refusal = observation.refusal.as_ref().expect("refusal evidence");
        assert_eq!(refusal.trigger.source_id, SourceId::new(1), "{source}");
        assert_eq!(refusal.trigger.range, expected_range, "{source}");
        assert_eq!(refusal.token_index, observation.processed_tokens, "{source}");
        assert!(
            matches!(observation.completion, Completion::Unsupported { capability: actual, .. } if actual == capability)
        );
    }'''
replace_once(old_cases, new_cases)

replace_once(
    "                assert_eq!(open_names(&observation), oracle.final_stack, \"{source}\");\n                assert_eq!(\n                    observation.reprocess_count, oracle.reprocess_count,\n                    \"{source}\"\n                );\n\n                let token_index = candidate_token_index(&observation);",
    "                assert_eq!(open_names(&observation), oracle.final_stack, \"{source}\");\n                assert_eq!(observation.next_id, oracle.expected_next_id, \"{source}\");\n                assert_eq!(observation.diagnostics.len(), oracle.audit_diagnostics, \"{source}\");\n                assert_eq!(\n                    observation.reprocess_count, oracle.reprocess_count,\n                    \"{source}\"\n                );\n\n                let token_index = candidate_token_index(&observation);\n                assert!(candidate_actions(&observation).iter().all(|action| !matches!(\n                    action,\n                    Action::Insert { .. } | Action::TextInsert { .. }\n                )), \"{source}\");",
)

path.write_text(text)
