from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VALUE = ROOT / "crates/frontend-analysis-core/src/css/value_qualification.rs"
MOD = ROOT / "crates/frontend-analysis-core/src/css/validation/mod.rs"
X_TEST = ROOT / "crates/frontend-analysis-core/src/css/validation/overscroll_behavior_x_value_qualification_tests.rs"
Y_TEST = ROOT / "crates/frontend-analysis-core/src/css/validation/overscroll_behavior_y_value_qualification_tests.rs"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def y_from_x(text: str) -> str:
    return (
        text.replace("CssOverscrollBehaviorX", "CssOverscrollBehaviorY")
        .replace("overscroll_behavior_x", "overscroll_behavior_y")
        .replace("overscroll-behavior-x", "overscroll-behavior-y")
        .replace("OVERSCROLL-BEHAVIOR-X", "OVERSCROLL-BEHAVIOR-Y")
    )


# Production implementation: add one sibling slice beside the accepted -x leaf.
text = VALUE.read_text()
if "CssOverscrollBehaviorYValue" in text or "overscroll_behavior_y_observations" in text:
    raise SystemExit("overscroll-behavior-y production slice already exists")

text = replace_once(text, "/#524).", "/#524/#526).", "module issue list")

derive = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n"
x_type_start = text.index(derive + "pub(crate) enum CssOverscrollBehaviorXValue")
word_type_start = text.index(derive + "pub(crate) enum CssWordSpacingValue", x_type_start)
x_type_block = text[x_type_start:word_type_start]
text = text[:word_type_start] + y_from_x(x_type_block) + text[word_type_start:]

text = replace_once(
    text,
    "    overscroll_behavior_x_observations: Vec<CssOverscrollBehaviorXQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "    overscroll_behavior_x_observations: Vec<CssOverscrollBehaviorXQualificationObservation>,\n"
    "    overscroll_behavior_y_observations: Vec<CssOverscrollBehaviorYQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "run-result field",
)

x_getter_start = text.index("    pub(crate) fn overscroll_behavior_x_observations(")
word_getter_start = text.index("    pub(crate) fn word_spacing_observations", x_getter_start)
x_getter_block = text[x_getter_start:word_getter_start]
text = text[:word_getter_start] + y_from_x(x_getter_block) + text[word_getter_start:]

tuple_anchor = "        overscroll_behavior_x_observations,\n        word_spacing_observations,"
tuple_count = text.count(tuple_anchor)
if tuple_count != 2:
    raise SystemExit(f"8-space tuple plumbing: expected two anchors, found {tuple_count}")
text = text.replace(
    tuple_anchor,
    "        overscroll_behavior_x_observations,\n"
    "        overscroll_behavior_y_observations,\n"
    "        word_spacing_observations,",
)

nested_tuple_anchor = "            overscroll_behavior_x_observations,\n            word_spacing_observations,"
text = replace_once(
    text,
    nested_tuple_anchor,
    "            overscroll_behavior_x_observations,\n"
    "            overscroll_behavior_y_observations,\n"
    "            word_spacing_observations,",
    "12-space tuple plumbing",
)

text = replace_once(
    text,
    "        let mut overscroll_behavior_x_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "        let mut overscroll_behavior_x_observations = Vec::new();\n"
    "        let mut overscroll_behavior_y_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "local observation collection",
)

x_dispatch_start = text.index(
    '            if property_name.eq_ignore_ascii_case("overscroll-behavior-x") {'
)
word_dispatch_start = text.index(
    '            if property_name.eq_ignore_ascii_case("word-spacing") {', x_dispatch_start
)
x_dispatch_block = text[x_dispatch_start:word_dispatch_start]
text = text[:word_dispatch_start] + y_from_x(x_dispatch_block) + text[word_dispatch_start:]

x_qualifier_start = text.index("fn qualify_overscroll_behavior_x_value(")
word_qualifier_start = text.index("fn qualify_word_spacing_value", x_qualifier_start)
x_qualifier_block = text[x_qualifier_start:word_qualifier_start]
text = text[:word_qualifier_start] + y_from_x(x_qualifier_block) + text[word_qualifier_start:]

if text.count("CssOverscrollBehaviorYValue") < 2:
    raise SystemExit("production Y value type/uses were not established")
if text.count('eq_ignore_ascii_case("overscroll-behavior-y")') != 1:
    raise SystemExit("production Y property dispatch count is not exactly one")
if text.count("fn qualify_overscroll_behavior_y_value(") != 1:
    raise SystemExit("production Y qualifier count is not exactly one")
VALUE.write_text(text)

# Validation module registration.
mod_text = MOD.read_text()
if "mod overscroll_behavior_y_value_qualification_tests;" in mod_text:
    raise SystemExit("overscroll-behavior-y validation module already registered")
mod_text = replace_once(
    mod_text,
    "mod overscroll_behavior_x_value_qualification_tests;\n#[cfg(test)]\nmod page_conformance_tests;",
    "mod overscroll_behavior_x_value_qualification_tests;\n#[cfg(test)]\n"
    "mod overscroll_behavior_y_value_qualification_tests;\n#[cfg(test)]\n"
    "mod page_conformance_tests;",
    "validation module registration",
)
MOD.write_text(mod_text)

# Candidate-independent sibling tests: reuse the accepted structural harness, but
# retain explicit handwritten outcomes and add an -x/-y cross-dispatch challenge.
if Y_TEST.exists():
    raise SystemExit("overscroll-behavior-y test file already exists")
y_test = y_from_x(X_TEST.read_text())
y_test = y_test.replace(r"overscroll-behavior-\78", r"overscroll-behavior-\79")

y_test = replace_once(
    y_test,
    '            "d{overscroll-behavior-y:chain;}",\n'
    '            "e{clip-rule:evenodd;}",',
    '            "d{overscroll-behavior-x:chain;}",\n'
    '            "e{overscroll-behavior-y:chain;}",\n'
    '            "f{clip-rule:evenodd;}",',
    "interleaving source",
)

y_test = replace_once(
    y_test,
    "    assert_eq!(result.clip_rule_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);\n"
    "    assert_eq!(\n"
    "        result.overscroll_behavior_y_observations()[0].occurrence_index(),\n"
    "        3\n"
    "    );",
    "    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);\n"
    "    assert_eq!(result.clip_rule_observations().len(), 1);\n"
    "    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);\n"
    "    assert_eq!(\n"
    "        result.overscroll_behavior_y_observations()[0].occurrence_index(),\n"
    "        4\n"
    "    );",
    "interleaving assertions",
)

if "CssOverscrollBehaviorX" in y_test:
    raise SystemExit("stale X type name remained in Y tests")
if y_test.count("overscroll-behavior-x") != 1:
    raise SystemExit(
        f"Y tests should contain exactly one deliberate x property occurrence, found {y_test.count('overscroll-behavior-x')}"
    )
if y_test.count("overscroll_behavior_x_observations") != 1:
    raise SystemExit("Y tests should contain exactly one deliberate x observation assertion")
if r"overscroll-behavior-\78" in y_test:
    raise SystemExit("stale escaped x property remained in Y tests")
if r"overscroll-behavior-\79" not in y_test:
    raise SystemExit("escaped y property challenge is missing")
Y_TEST.write_text(y_test)
