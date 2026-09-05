from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VALUE = ROOT / "crates/frontend-analysis-core/src/css/value_qualification.rs"
MOD = ROOT / "crates/frontend-analysis-core/src/css/validation/mod.rs"
INLINE_TEST = ROOT / "crates/frontend-analysis-core/src/css/validation/overscroll_behavior_inline_value_qualification_tests.rs"
BLOCK_TEST = ROOT / "crates/frontend-analysis-core/src/css/validation/overscroll_behavior_block_value_qualification_tests.rs"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def block_from_inline(text: str) -> str:
    return (
        text.replace("CssOverscrollBehaviorInline", "CssOverscrollBehaviorBlock")
        .replace("overscroll_behavior_inline", "overscroll_behavior_block")
        .replace("overscroll-behavior-inline", "overscroll-behavior-block")
        .replace("OVERSCROLL-BEHAVIOR-INLINE", "OVERSCROLL-BEHAVIOR-BLOCK")
    )


# Production implementation: add the remaining logical-axis sibling beside inline.
text = VALUE.read_text()
if "CssOverscrollBehaviorBlockValue" in text or "overscroll_behavior_block_observations" in text:
    raise SystemExit("overscroll-behavior-block production slice already exists")

text = replace_once(text, "/#528).", "/#528/#530).", "module issue list")

derive = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n"
inline_type_start = text.index(derive + "pub(crate) enum CssOverscrollBehaviorInlineValue")
word_type_start = text.index(derive + "pub(crate) enum CssWordSpacingValue", inline_type_start)
inline_type_block = text[inline_type_start:word_type_start]
text = text[:word_type_start] + block_from_inline(inline_type_block) + text[word_type_start:]

text = replace_once(
    text,
    "    overscroll_behavior_inline_observations:\n"
    "        Vec<CssOverscrollBehaviorInlineQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "    overscroll_behavior_inline_observations:\n"
    "        Vec<CssOverscrollBehaviorInlineQualificationObservation>,\n"
    "    overscroll_behavior_block_observations: Vec<CssOverscrollBehaviorBlockQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "run-result field",
)

inline_getter_start = text.index("    pub(crate) fn overscroll_behavior_inline_observations(")
word_getter_start = text.index("    pub(crate) fn word_spacing_observations", inline_getter_start)
inline_getter_block = text[inline_getter_start:word_getter_start]
text = text[:word_getter_start] + block_from_inline(inline_getter_block) + text[word_getter_start:]

tuple_anchor = "        overscroll_behavior_inline_observations,\n        word_spacing_observations,"
tuple_count = text.count(tuple_anchor)
if tuple_count != 2:
    raise SystemExit(f"8-space tuple plumbing: expected two anchors, found {tuple_count}")
text = text.replace(
    tuple_anchor,
    "        overscroll_behavior_inline_observations,\n"
    "        overscroll_behavior_block_observations,\n"
    "        word_spacing_observations,",
)

nested_tuple_anchor = "            overscroll_behavior_inline_observations,\n            word_spacing_observations,"
text = replace_once(
    text,
    nested_tuple_anchor,
    "            overscroll_behavior_inline_observations,\n"
    "            overscroll_behavior_block_observations,\n"
    "            word_spacing_observations,",
    "12-space tuple plumbing",
)

text = replace_once(
    text,
    "        let mut overscroll_behavior_inline_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "        let mut overscroll_behavior_inline_observations = Vec::new();\n"
    "        let mut overscroll_behavior_block_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "local observation collection",
)

inline_dispatch_start = text.index(
    '            if property_name.eq_ignore_ascii_case("overscroll-behavior-inline") {'
)
word_dispatch_start = text.index(
    '            if property_name.eq_ignore_ascii_case("word-spacing") {', inline_dispatch_start
)
inline_dispatch_block = text[inline_dispatch_start:word_dispatch_start]
text = text[:word_dispatch_start] + block_from_inline(inline_dispatch_block) + text[word_dispatch_start:]

inline_qualifier_start = text.index("fn qualify_overscroll_behavior_inline_value(")
word_qualifier_start = text.index("fn qualify_word_spacing_value", inline_qualifier_start)
inline_qualifier_block = text[inline_qualifier_start:word_qualifier_start]
text = text[:word_qualifier_start] + block_from_inline(inline_qualifier_block) + text[word_qualifier_start:]

if text.count("CssOverscrollBehaviorBlockValue") < 2:
    raise SystemExit("production block value type/uses were not established")
if text.count('eq_ignore_ascii_case("overscroll-behavior-block")') != 1:
    raise SystemExit("production block property dispatch count is not exactly one")
if text.count("fn qualify_overscroll_behavior_block_value(") != 1:
    raise SystemExit("production block qualifier count is not exactly one")
VALUE.write_text(text)

# Validation module registration.
mod_text = MOD.read_text()
if "mod overscroll_behavior_block_value_qualification_tests;" in mod_text:
    raise SystemExit("overscroll-behavior-block validation module already registered")
mod_text = replace_once(
    mod_text,
    "mod overscroll_behavior_inline_value_qualification_tests;\n#[cfg(test)]\nmod page_conformance_tests;",
    "mod overscroll_behavior_inline_value_qualification_tests;\n#[cfg(test)]\n"
    "mod overscroll_behavior_block_value_qualification_tests;\n#[cfg(test)]\n"
    "mod page_conformance_tests;",
    "validation module registration",
)
MOD.write_text(mod_text)

# Candidate-independent sibling tests: reuse the accepted harness structure while
# keeping explicit handwritten outcomes and adding all four longhand cross-dispatch.
if BLOCK_TEST.exists():
    raise SystemExit("overscroll-behavior-block test file already exists")
block_test = block_from_inline(INLINE_TEST.read_text())
block_test = block_test.replace(
    r"f{overscroll-behavior-\69 nline:none;}",
    r"f{overscroll-behavior-\62 lock:none;}",
)

# Keep SourceId values unique across validation modules. The analyzer caches by
# SourceId, so copied fixture IDs must never overlap under full-suite parallelism.
for old, new in {
    "3380": "3420",
    "3381": "3421",
    "3382": "3422",
    "3383": "3423",
    "3384": "3424",
    "3385": "3425",
    "3386": "3426",
    "3387": "3427",
    "3390": "3430",
    "3391": "3431",
    "3392": "3432",
    "3393": "3433",
    "3400": "3440",
    "3410": "3450",
    "3411": "3451",
}.items():
    block_test = block_test.replace(old, new)

block_test = replace_once(
    block_test,
    '            "d{overscroll-behavior-x:chain;}",\n'
    '            "e{overscroll-behavior-y:chain;}",\n'
    '            "f{overscroll-behavior-block:chain;}",\n'
    '            "g{clip-rule:evenodd;}",',
    '            "d{overscroll-behavior-x:chain;}",\n'
    '            "e{overscroll-behavior-y:chain;}",\n'
    '            "f{overscroll-behavior-inline:chain;}",\n'
    '            "g{overscroll-behavior-block:chain;}",\n'
    '            "h{clip-rule:evenodd;}",',
    "interleaving source",
)

block_test = replace_once(
    block_test,
    "    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);\n"
    "    assert_eq!(result.clip_rule_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_block_observations().len(), 1);\n"
    "    assert_eq!(\n"
    "        result.overscroll_behavior_block_observations()[0].occurrence_index(),\n"
    "        5\n"
    "    );",
    "    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_inline_observations().len(), 1);\n"
    "    assert_eq!(result.clip_rule_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_block_observations().len(), 1);\n"
    "    assert_eq!(\n"
    "        result.overscroll_behavior_block_observations()[0].occurrence_index(),\n"
    "        6\n"
    "    );",
    "interleaving assertions",
)

if "CssOverscrollBehaviorInline" in block_test:
    raise SystemExit("stale Inline type name remained in block tests")
if block_test.count("overscroll-behavior-x") != 1:
    raise SystemExit("block tests should contain exactly one deliberate x property occurrence")
if block_test.count("overscroll_behavior_x_observations") != 1:
    raise SystemExit("block tests should contain exactly one deliberate x observation assertion")
if block_test.count("overscroll-behavior-y") != 1:
    raise SystemExit("block tests should contain exactly one deliberate y property occurrence")
if block_test.count("overscroll_behavior_y_observations") != 1:
    raise SystemExit("block tests should contain exactly one deliberate y observation assertion")
if block_test.count("overscroll-behavior-inline") != 1:
    raise SystemExit("block tests should contain exactly one deliberate inline property occurrence")
if block_test.count("overscroll_behavior_inline_observations") != 1:
    raise SystemExit("block tests should contain exactly one deliberate inline observation assertion")
if r"overscroll-behavior-\69 nline" in block_test:
    raise SystemExit("stale escaped inline property remained in block tests")
if r"overscroll-behavior-\62 lock" not in block_test:
    raise SystemExit("escaped block property challenge is missing")
for stale_id in ["3380", "3381", "3382", "3383", "3384", "3385", "3386", "3387", "3390", "3391", "3392", "3393", "3400", "3410", "3411"]:
    if stale_id in block_test:
        raise SystemExit(f"stale copied SourceId remained in block tests: {stale_id}")
BLOCK_TEST.write_text(block_test)
