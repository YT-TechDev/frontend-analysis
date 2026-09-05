from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VALUE = ROOT / "crates/frontend-analysis-core/src/css/value_qualification.rs"
MOD = ROOT / "crates/frontend-analysis-core/src/css/validation/mod.rs"
Y_TEST = ROOT / "crates/frontend-analysis-core/src/css/validation/overscroll_behavior_y_value_qualification_tests.rs"
INLINE_TEST = ROOT / "crates/frontend-analysis-core/src/css/validation/overscroll_behavior_inline_value_qualification_tests.rs"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def inline_from_y(text: str) -> str:
    return (
        text.replace("CssOverscrollBehaviorY", "CssOverscrollBehaviorInline")
        .replace("overscroll_behavior_y", "overscroll_behavior_inline")
        .replace("overscroll-behavior-y", "overscroll-behavior-inline")
        .replace("OVERSCROLL-BEHAVIOR-Y", "OVERSCROLL-BEHAVIOR-INLINE")
    )


# Production implementation: add one logical-axis sibling beside accepted -y.
text = VALUE.read_text()
if "CssOverscrollBehaviorInlineValue" in text or "overscroll_behavior_inline_observations" in text:
    raise SystemExit("overscroll-behavior-inline production slice already exists")

text = replace_once(text, "/#526).", "/#526/#528).", "module issue list")

derive = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n"
y_type_start = text.index(derive + "pub(crate) enum CssOverscrollBehaviorYValue")
word_type_start = text.index(derive + "pub(crate) enum CssWordSpacingValue", y_type_start)
y_type_block = text[y_type_start:word_type_start]
text = text[:word_type_start] + inline_from_y(y_type_block) + text[word_type_start:]

text = replace_once(
    text,
    "    overscroll_behavior_y_observations: Vec<CssOverscrollBehaviorYQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "    overscroll_behavior_y_observations: Vec<CssOverscrollBehaviorYQualificationObservation>,\n"
    "    overscroll_behavior_inline_observations: Vec<CssOverscrollBehaviorInlineQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "run-result field",
)

y_getter_start = text.index("    pub(crate) fn overscroll_behavior_y_observations(")
word_getter_start = text.index("    pub(crate) fn word_spacing_observations", y_getter_start)
y_getter_block = text[y_getter_start:word_getter_start]
text = text[:word_getter_start] + inline_from_y(y_getter_block) + text[word_getter_start:]

tuple_anchor = "        overscroll_behavior_y_observations,\n        word_spacing_observations,"
tuple_count = text.count(tuple_anchor)
if tuple_count != 2:
    raise SystemExit(f"8-space tuple plumbing: expected two anchors, found {tuple_count}")
text = text.replace(
    tuple_anchor,
    "        overscroll_behavior_y_observations,\n"
    "        overscroll_behavior_inline_observations,\n"
    "        word_spacing_observations,",
)

nested_tuple_anchor = "            overscroll_behavior_y_observations,\n            word_spacing_observations,"
text = replace_once(
    text,
    nested_tuple_anchor,
    "            overscroll_behavior_y_observations,\n"
    "            overscroll_behavior_inline_observations,\n"
    "            word_spacing_observations,",
    "12-space tuple plumbing",
)

text = replace_once(
    text,
    "        let mut overscroll_behavior_y_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "        let mut overscroll_behavior_y_observations = Vec::new();\n"
    "        let mut overscroll_behavior_inline_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "local observation collection",
)

y_dispatch_start = text.index(
    '            if property_name.eq_ignore_ascii_case("overscroll-behavior-y") {'
)
word_dispatch_start = text.index(
    '            if property_name.eq_ignore_ascii_case("word-spacing") {', y_dispatch_start
)
y_dispatch_block = text[y_dispatch_start:word_dispatch_start]
text = text[:word_dispatch_start] + inline_from_y(y_dispatch_block) + text[word_dispatch_start:]

y_qualifier_start = text.index("fn qualify_overscroll_behavior_y_value(")
word_qualifier_start = text.index("fn qualify_word_spacing_value", y_qualifier_start)
y_qualifier_block = text[y_qualifier_start:word_qualifier_start]
text = text[:word_qualifier_start] + inline_from_y(y_qualifier_block) + text[word_qualifier_start:]

if text.count("CssOverscrollBehaviorInlineValue") < 2:
    raise SystemExit("production inline value type/uses were not established")
if text.count('eq_ignore_ascii_case("overscroll-behavior-inline")') != 1:
    raise SystemExit("production inline property dispatch count is not exactly one")
if text.count("fn qualify_overscroll_behavior_inline_value(") != 1:
    raise SystemExit("production inline qualifier count is not exactly one")
VALUE.write_text(text)

# Validation module registration.
mod_text = MOD.read_text()
if "mod overscroll_behavior_inline_value_qualification_tests;" in mod_text:
    raise SystemExit("overscroll-behavior-inline validation module already registered")
mod_text = replace_once(
    mod_text,
    "mod overscroll_behavior_y_value_qualification_tests;\n#[cfg(test)]\nmod page_conformance_tests;",
    "mod overscroll_behavior_y_value_qualification_tests;\n#[cfg(test)]\n"
    "mod overscroll_behavior_inline_value_qualification_tests;\n#[cfg(test)]\n"
    "mod page_conformance_tests;",
    "validation module registration",
)
MOD.write_text(mod_text)

# Candidate-independent sibling tests: reuse the accepted harness structure while
# keeping explicit handwritten outcomes and adding physical/logical cross-dispatch.
if INLINE_TEST.exists():
    raise SystemExit("overscroll-behavior-inline test file already exists")
inline_test = inline_from_y(Y_TEST.read_text())
inline_test = inline_test.replace(
    r"f{overscroll-behavior-\79 :none;}",
    r"f{overscroll-behavior-\69 nline:none;}",
)

# Keep SourceId values unique across validation modules. The analyzer caches by
# SourceId, so copied fixture IDs must never overlap under full-suite parallelism.
for old, new in {
    "3340": "3380",
    "3341": "3381",
    "3342": "3382",
    "3343": "3383",
    "3344": "3384",
    "3345": "3385",
    "3346": "3386",
    "3347": "3387",
    "3350": "3390",
    "3351": "3391",
    "3352": "3392",
    "3353": "3393",
    "3360": "3400",
    "3370": "3410",
    "3371": "3411",
}.items():
    inline_test = inline_test.replace(old, new)

inline_test = replace_once(
    inline_test,
    '            "d{overscroll-behavior-x:chain;}",\n'
    '            "e{overscroll-behavior-inline:chain;}",\n'
    '            "f{clip-rule:evenodd;}",',
    '            "d{overscroll-behavior-x:chain;}",\n'
    '            "e{overscroll-behavior-y:chain;}",\n'
    '            "f{overscroll-behavior-inline:chain;}",\n'
    '            "g{clip-rule:evenodd;}",',
    "interleaving source",
)

inline_test = replace_once(
    inline_test,
    "    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);\n"
    "    assert_eq!(result.clip_rule_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_inline_observations().len(), 1);\n"
    "    assert_eq!(\n"
    "        result.overscroll_behavior_inline_observations()[0].occurrence_index(),\n"
    "        4\n"
    "    );",
    "    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);\n"
    "    assert_eq!(result.clip_rule_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_inline_observations().len(), 1);\n"
    "    assert_eq!(\n"
    "        result.overscroll_behavior_inline_observations()[0].occurrence_index(),\n"
    "        5\n"
    "    );",
    "interleaving assertions",
)

if "CssOverscrollBehaviorY" in inline_test:
    raise SystemExit("stale Y type name remained in inline tests")
if inline_test.count("overscroll-behavior-x") != 1:
    raise SystemExit("inline tests should contain exactly one deliberate x property occurrence")
if inline_test.count("overscroll_behavior_x_observations") != 1:
    raise SystemExit("inline tests should contain exactly one deliberate x observation assertion")
if inline_test.count("overscroll-behavior-y") != 1:
    raise SystemExit("inline tests should contain exactly one deliberate y property occurrence")
if inline_test.count("overscroll_behavior_y_observations") != 1:
    raise SystemExit("inline tests should contain exactly one deliberate y observation assertion")
if r"overscroll-behavior-\79" in inline_test:
    raise SystemExit("stale escaped y property remained in inline tests")
if r"overscroll-behavior-\69 nline" not in inline_test:
    raise SystemExit("escaped inline property challenge is missing")
for stale_id in ["3340", "3341", "3342", "3343", "3344", "3345", "3346", "3347", "3350", "3351", "3352", "3353", "3360", "3370", "3371"]:
    if stale_id in inline_test:
        raise SystemExit(f"stale copied SourceId remained in inline tests: {stale_id}")
INLINE_TEST.write_text(inline_test)
