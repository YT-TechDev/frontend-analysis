//! Process-boundary tests for `fa css-selectors` (#857) and, in the
//! `es_binding_refs` module, `fa es-binding-refs` (#862).
//!
//! Every expected stdout/stderr text below is authored by hand from the
//! approved output contract and hand-counted source byte offsets. None of it
//! is captured from the binary under test.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct Run {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

fn input_file(name: &str, bytes: &[u8]) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("fa-stdin-{name}"));
    fs::write(&path, bytes).unwrap();
    path
}

fn finish(output: Output) -> Run {
    Run {
        status: output.status.code(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn fa(args: &[&str], name: &str, stdin: &[u8]) -> Run {
    let path = input_file(name, stdin);
    let output = Command::new(env!("CARGO_BIN_EXE_fa"))
        .args(args)
        .stdin(fs::File::open(&path).unwrap())
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    finish(output)
}

fn css_selectors(name: &str, stdin: &[u8]) -> Run {
    fa(&["css-selectors"], name, stdin)
}

const COMPLETE_STAGES: &str = "\
tokenizer: complete; end of input; diagnostics 0
parser: complete; end of tokenizer input; coverage supported for selected question; \
diagnostics 0; recovery records 0; unsupported regions 0; discard records 0
selector: complete; all retained qualified contexts processed
";

#[test]
fn qualified_stylesheet_reports_exit_zero() {
    let run = css_selectors("qualified", b"a{}");

    let expected = format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; 3 bytes
{COMPLETE_STAGES}observations: 1
observation 1: qualified by selected grammar
  context: bytes 0..1, line 1, byte column 1: \"a\"
  grammar: normal selector list
"
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn empty_stdin_is_a_valid_empty_source() {
    // Null stdin exercises the non-file empty-input path.
    let output = Command::new(env!("CARGO_BIN_EXE_fa"))
        .arg("css-selectors")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let run = finish(output);

    let expected = format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; 0 bytes
{COMPLETE_STAGES}observations: 0
"
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn invalid_unsupported_and_indeterminate_remain_distinct_with_exit_zero() {
    // "a,,b{}" = 0..6, LF at 6; "svg|a{}" = 7..14, LF at 14;
    // "::before{}" = 15..25, LF at 25; 26 bytes total.
    let run = css_selectors("mixed", b"a,,b{}\nsvg|a{}\n::before{}\n");

    let expected = format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; 26 bytes
{COMPLETE_STAGES}observations: 3
observation 1: invalid for selected grammar (unexpected comma)
  context: bytes 0..4, line 1, byte column 1: \"a,,b\"
  subject: bytes 2..3, line 1, byte column 3: \",\"
  grammar: normal selector list
observation 2: indeterminate (missing namespace environment)
  context: bytes 7..12, line 2, byte column 1: \"svg|a\"
  subject: bytes 7..10, line 2, byte column 1: \"svg\"
  grammar: normal selector list
observation 3: unsupported by selected grammar profile (pseudo-element)
  context: bytes 15..23, line 3, byte column 1: \"::before\"
  subject: bytes 17..23, line 3, byte column 3: \"before\"
  grammar: normal selector list
"
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn bom_crlf_unicode_and_escape_spelling_are_preserved_exactly() {
    // BOM = 0..3; ".\66 oo" = 3..10; "," = 10..11; CRLF = 11..13;
    // ".é" = 13..16; "{}" = 16..18.
    let run = css_selectors("exact-source", "\u{feff}.\\66 oo,\r\n.é{}".as_bytes());

    let expected = format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; 18 bytes
{COMPLETE_STAGES}observations: 1
observation 1: qualified by selected grammar
  context: bytes 3..16, line 1, byte column 4: \".\\\\66 oo,\\u{{d}}\\u{{a}}.\\u{{e9}}\"
  grammar: normal selector list
"
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn nested_context_reports_relative_grammar() {
    // ".a" = 0..2 normal; ".b" = 3..5 nested.
    let run = css_selectors("nested", b".a{.b{}}");

    let expected = format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; 8 bytes
{COMPLETE_STAGES}observations: 2
observation 1: qualified by selected grammar
  context: bytes 0..2, line 1, byte column 1: \".a\"
  grammar: normal selector list
observation 2: qualified by selected grammar
  context: bytes 3..5, line 1, byte column 4: \".b\"
  grammar: nested relative selector list
"
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
}

fn source_bytes_refusal_report(bytes: usize) -> String {
    format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; {bytes} bytes
tokenizer: incomplete; resource limit source bytes (limit 32768, attempted {bytes}) at byte 0, \
line 1, byte column 1; diagnostics 0
parser: incomplete; upstream tokenizer incomplete; coverage supported for selected question; \
diagnostics 0; recovery records 0; unsupported regions 0; discard records 0
selector: complete; all retained qualified contexts processed
observations: 0
"
    )
}

#[test]
fn core_source_bytes_refusal_is_an_analysis_report_not_input_failure() {
    // The lower edge of the 32769..=49152 acquisition/Core window.
    let run = css_selectors("core-window-low", &vec![b'a'; 32_769]);
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, source_bytes_refusal_report(32_769));
    assert_eq!(run.stderr, "");
}

#[test]
fn input_exactly_at_product_cap_reaches_core() {
    let run = css_selectors("product-cap", &vec![b'a'; 49_152]);
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, source_bytes_refusal_report(49_152));
    assert_eq!(run.stderr, "");
}

#[test]
fn input_over_product_cap_is_rejected_before_core() {
    let run = css_selectors("over-product-cap", &vec![b'a'; 49_153]);
    assert_eq!(run.status, Some(1));
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "fa: stdin exceeds 49152 bytes\n");
}

#[test]
fn invalid_utf8_is_rejected_before_core() {
    let run = css_selectors("invalid-utf8", b"a\xff{}");
    assert_eq!(run.status, Some(1));
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        "fa: stdin is not valid UTF-8 (invalid sequence at byte 1)\n"
    );
}

#[test]
fn usage_failures_exit_one_with_usage_on_stderr() {
    for (args, message) in [
        (&[][..], "missing command"),
        (&["css"][..], "unknown command"),
        (&["--help"][..], "unknown command"),
        (&["css-selectors", "style.css"][..], "unexpected argument"),
    ] {
        let run = fa(args, "usage", b"a{}");
        assert_eq!(run.status, Some(1), "{args:?}");
        assert_eq!(run.stdout, "", "{args:?}");
        assert_eq!(
            run.stderr,
            format!(
                "fa: {message}\nusage: fa css-selectors < style.css\n       \
                 fa es-binding-refs < source.js\n"
            ),
            "{args:?}"
        );
    }
}

#[test]
fn repeated_invocations_are_byte_identical() {
    let input = b"a,,b{}\n.a{&Bar{} > .b{}}\nsvg|a{}\n";
    let first = css_selectors("repeat-1", input);
    let second = css_selectors("repeat-2", input);

    assert_eq!(first.status, Some(0));
    assert_eq!(first.status, second.status);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

// ---- #860: source-locatable parser partial-coverage evidence ----

fn report_head(bytes: usize) -> String {
    format!(
        "capability: css-selectors
profile: CoreV1
source: id 0; {bytes} bytes
tokenizer: complete; end of input; diagnostics 0
"
    )
}

#[test]
fn recovery_record_is_source_locatable() {
    // `a{` 0..2; `color red;` 2..12; `background:blue;` 12..28; `}` 28..29.
    let run = css_selectors("recovery", b"a{color red;background:blue;}");

    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage supported for selected question; \
diagnostics 1; recovery records 1; unsupported regions 0; discard records 0
recovery 1: malformed block item; authored semicolon
  source: bytes 2..12, line 1, byte column 3: \"color red;\"
selector: complete; all retained qualified contexts processed
observations: 1
observation 1: qualified by selected grammar
  context: bytes 0..1, line 1, byte column 1: \"a\"
  grammar: normal selector list
",
        report_head(29)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn recovery_termination_kinds_are_distinguished() {
    // `a{` 0..2; `color red` 2..11; authored `}` 11..12; 12 bytes.
    let run = css_selectors("recovery-block-end", b"a{color red}");
    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage supported for selected question; \
diagnostics 1; recovery records 1; unsupported regions 0; discard records 0
recovery 1: malformed block item; enclosing block end
  source: bytes 2..11, line 1, byte column 3: \"color red\"
selector: complete; all retained qualified contexts processed
observations: 1
observation 1: qualified by selected grammar
  context: bytes 0..1, line 1, byte column 1: \"a\"
  grammar: normal selector list
",
        report_head(12)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");

    // True end of input, no authored delimiter: `a{` 0..2; `color red`
    // 2..11; 11 bytes.
    let run = css_selectors("recovery-eof", b"a{color red");
    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage supported for selected question; \
diagnostics 1; recovery records 1; unsupported regions 0; discard records 0
recovery 1: malformed block item; end of input
  source: bytes 2..11, line 1, byte column 3: \"color red\"
selector: complete; all retained qualified contexts processed
observations: 1
observation 1: qualified by selected grammar
  context: bytes 0..1, line 1, byte column 1: \"a\"
  grammar: normal selector list
",
        report_head(11)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn unsupported_region_is_source_locatable() {
    // `a{color:red;` is 12 bytes; the nested at-rule is 12..38.
    let run = css_selectors("unsupported", b"a{color:red;@unknown-rule{color:blue;}}");

    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage contains unsupported contexts; \
diagnostics 0; recovery records 0; unsupported regions 1; discard records 0
unsupported 1: nested at-rule
  source: bytes 12..38, line 1, byte column 13: \"@unknown-rule{{color:blue;}}\"
selector: complete; all retained qualified contexts processed
observations: 1
observation 1: qualified by selected grammar
  context: bytes 0..1, line 1, byte column 1: \"a\"
  grammar: normal selector list
",
        report_head(39)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn unsupported_kinds_are_distinguished() {
    // The whole 28-byte source is one structurally consumed top-level
    // at-rule; no descendant context is extracted from its block.
    let run = css_selectors("unsupported-top-level", b"@media screen{a{color:red;}}");
    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage contains unsupported contexts; \
diagnostics 0; recovery records 0; unsupported regions 1; discard records 0
unsupported 1: top-level at-rule
  source: bytes 0..28, line 1, byte column 1: \"@media screen{{a{{color:red;}}}}\"
selector: complete; all retained qualified contexts processed
observations: 0
",
        report_head(28)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");

    // CSS-KEYFRAMES-INVALID-CHILD-001: the invalid child block is 13..24;
    // the source is 35 bytes and retains no qualified-rule context.
    let run = css_selectors(
        "unsupported-keyframe",
        b"@keyframes x{bogus{x:y;}from{a:b;}}",
    );
    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage contains unsupported contexts; \
diagnostics 0; recovery records 0; unsupported regions 1; discard records 0
unsupported 1: unqualified keyframe block
  source: bytes 13..24, line 1, byte column 14: \"bogus{{x:y;}}\"
selector: complete; all retained qualified contexts processed
observations: 0
",
        report_head(35)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn discard_record_is_source_locatable() {
    let run = css_selectors("discard", b"--foo:bar{color:red;}");

    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage supported for selected question; \
diagnostics 0; recovery records 0; unsupported regions 0; discard records 1
discard 1: top-level custom-property-like qualified rule
  source: bytes 0..21, line 1, byte column 1: \"--foo:bar{{color:red;}}\"
selector: complete; all retained qualified contexts processed
observations: 0
",
        report_head(21)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn records_keep_family_local_order_in_separate_sections() {
    // Lines start at bytes 0, 8, 14, 22, 28.
    // `--a:b{}` 0..7; `@one;` 8..13; `--c:d{}` 14..21; `@two;` 22..27;
    // `a{` 28..30; `x;` 30..32; `y` 32..33; `}` 33..34; LF 34; 35 bytes.
    let run = css_selectors("family-order", b"--a:b{}\n@one;\n--c:d{}\n@two;\na{x;y}\n");

    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage contains unsupported contexts; \
diagnostics 2; recovery records 2; unsupported regions 2; discard records 2
recovery 1: malformed block item; authored semicolon
  source: bytes 30..32, line 5, byte column 3: \"x;\"
recovery 2: malformed block item; enclosing block end
  source: bytes 32..33, line 5, byte column 5: \"y\"
unsupported 1: top-level at-rule
  source: bytes 8..13, line 2, byte column 1: \"@one;\"
unsupported 2: top-level at-rule
  source: bytes 22..27, line 4, byte column 1: \"@two;\"
discard 1: top-level custom-property-like qualified rule
  source: bytes 0..7, line 1, byte column 1: \"--a:b{{}}\"
discard 2: top-level custom-property-like qualified rule
  source: bytes 14..21, line 3, byte column 1: \"--c:d{{}}\"
selector: complete; all retained qualified contexts processed
observations: 1
observation 1: qualified by selected grammar
  context: bytes 28..29, line 5, byte column 1: \"a\"
  grammar: normal selector list
",
        report_head(35)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn exact_source_spelling_is_escaped_without_normalization_in_records() {
    // BOM 0..3; `a{` 3..5; `x`, CRLF, `;` is 5..9; `}` 9..10; and the
    // escaped-keyword at-rule `@\66 oo{}` is 10..19; 19 bytes. U+FEFF counts
    // as bytes on line 1, so `a` is byte column 4 and the recovery starts at
    // byte column 6; the CRLF ends at byte 8, so line 2 starts there and `@`
    // is byte column 3.
    let run = css_selectors(
        "records-exact-source",
        "\u{feff}a{x\r\n;}@\\66 oo{}".as_bytes(),
    );

    let expected = format!(
        "{}parser: complete; end of tokenizer input; coverage contains unsupported contexts; \
diagnostics 1; recovery records 1; unsupported regions 1; discard records 0
recovery 1: malformed block item; authored semicolon
  source: bytes 5..9, line 1, byte column 6: \"x\\u{{d}}\\u{{a}};\"
unsupported 1: top-level at-rule
  source: bytes 10..19, line 2, byte column 3: \"@\\\\66 oo{{}}\"
selector: complete; all retained qualified contexts processed
observations: 1
observation 1: qualified by selected grammar
  context: bytes 3..4, line 1, byte column 4: \"a\"
  grammar: normal selector list
",
        report_head(19)
    );
    assert_eq!(run.status, Some(0));
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, "");
}

#[test]
fn resource_limited_parser_run_reports_only_committed_records_with_exit_zero() {
    // The 65th `@x;` exceeds the approved 64 unsupported-region limit.
    let run = css_selectors("records-resource-limit", "@x;".repeat(65).as_bytes());

    assert_eq!(run.status, Some(0));
    assert_eq!(run.stderr, "");
    assert!(run.stdout.contains(
        "parser: incomplete; resource limit unsupported regions (limit 64, attempted 65) at byte"
    ));
    assert!(run.stdout.contains("unsupported regions 64;"));
    let listed = run
        .stdout
        .lines()
        .filter(|line| line.starts_with("unsupported "))
        .count();
    assert_eq!(listed, 64);
    assert!(run.stdout.contains(
        "unsupported 64: top-level at-rule
  source: bytes 189..192, line 1, byte column 190: \"@x;\"
"
    ));
    assert!(!run.stdout.contains("unsupported 65:"));
}

// ---- #862: `fa es-binding-refs` process tests ----
//
// The workspace-state validator pins this crate to exactly one integration
// test target, so the ES process tests live in this module rather than a new
// target. Every expected text is hand-authored from the approved output
// contract and hand-counted offsets (accepted `selected_binding_scope`
// fixtures for relation cases), never captured from the binary under test.
mod es_binding_refs {
    use super::{Run, fa, finish};
    use std::process::{Command, Stdio};

    fn es(name: &str, stdin: &[u8]) -> Run {
        fa(&["es-binding-refs"], name, stdin)
    }

    fn head(bytes: usize) -> String {
        format!(
            "capability: es-binding-refs
source: id 0; {bytes} bytes
goal: Script
scope: selected flat lexical binding initializers
"
        )
    }

    fn assert_report(run: &Run, expected: &str) {
        assert_eq!(run.status, Some(0));
        assert_eq!(run.stdout, expected);
        assert_eq!(run.stderr, "");
    }

    #[test]
    fn complete_positive_report() {
        // "let a=1; " 0..9; "let " 9..13; x 13..14; "=" 14; a 15..16; ";" 16.
        let run = es("positive", b"let a=1; let x=a;");
        let expected = head(17)
            + "analysis: complete
relations: 1
relation 1:
  name: \"a\"
  containing binding: bytes 13..14, line 1, byte column 14: \"x\"
  reference: bytes 15..16, line 1, byte column 16: \"a\"
  target: same-source selected lexical binding (before)
    binding: bytes 4..5, line 1, byte column 5: \"a\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn absent_and_after_targets_render_distinctly() {
        // "let x=y; let y=1;": x 4..5, y 6..7, second y 13..14.
        let run = es("after", b"let x=y; let y=1;");
        let expected = head(17)
            + "analysis: complete
relations: 1
relation 1:
  name: \"y\"
  containing binding: bytes 4..5, line 1, byte column 5: \"x\"
  reference: bytes 6..7, line 1, byte column 7: \"y\"
  target: same-source selected lexical binding (after)
    binding: bytes 13..14, line 1, byte column 14: \"y\"
";
        assert_report(&run, &expected);

        let run = es("none", b"let x=y;");
        let expected = head(8)
            + "analysis: complete
relations: 1
relation 1:
  name: \"y\"
  containing binding: bytes 4..5, line 1, byte column 5: \"x\"
  reference: bytes 6..7, line 1, byte column 7: \"y\"
  target: no same-source selected lexical binding
";
        assert_report(&run, &expected);
    }

    #[test]
    fn complete_zero_relations_report() {
        let run = es("zero", b"let x=1;");
        let expected = head(8) + "analysis: complete\nrelations: 0\n";
        assert_report(&run, &expected);
    }

    #[test]
    fn unsupported_coverage_is_not_a_complete_zero() {
        let run = es("unsupported", b"x;");
        let expected = head(2) + "analysis: unsupported coverage for selected scope\n";
        assert_report(&run, &expected);
    }

    #[test]
    fn selected_grammar_rejection_reports_the_authored_subject() {
        // "let " 0..4; "\u{}" 4..8; ";" 8.
        let run = es("grammar", b"let \\u{};");
        let expected = head(9)
            + "analysis: selected grammar rejection
  subject: bytes 4..8, line 1, byte column 5: \"\\\\u{}\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn selected_static_rejection_reports_its_evidence() {
        let run = es("const", b"const x;");
        let expected = head(8)
            + "analysis: selected static rejection (const binding missing initializer)
  binding: bytes 6..7, line 1, byte column 7: \"x\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn duplicate_static_rejection_shows_both_anchors() {
        // "let a=1; " 0..9; "let " 9..13; second a 13..14.
        let run = es("duplicate", b"let a=1; let a=2;");
        let expected = head(17)
            + "analysis: selected static rejection (duplicate lexical name)
  first binding: bytes 4..5, line 1, byte column 5: \"a\"
  duplicate binding: bytes 13..14, line 1, byte column 14: \"a\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn escaped_reference_shows_authored_fragment_and_decoded_name() {
        // "let a=1; let x=" 0..15; "a" 15..21; ";" 21.
        let run = es("escaped", b"let a=1; let x=\\u0061;");
        let expected = head(22)
            + "analysis: complete
relations: 1
relation 1:
  name: \"a\"
  containing binding: bytes 13..14, line 1, byte column 14: \"x\"
  reference: bytes 15..21, line 1, byte column 16: \"\\\\u0061\"
  target: same-source selected lexical binding (before)
    binding: bytes 4..5, line 1, byte column 5: \"a\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn bom_and_crlf_are_preserved_exactly() {
        // BOM 0..3; "let a=1;" 3..11; CRLF 11..13; "let " 13..17; x 17..18;
        // "=" 18; a 19..20; ";" 20. Target a 7..8 on line 1 (BOM bytes count).
        let run = es("bom-crlf", "\u{feff}let a=1;\r\nlet x=a;".as_bytes());
        let expected = head(21)
            + "analysis: complete
relations: 1
relation 1:
  name: \"a\"
  containing binding: bytes 17..18, line 2, byte column 5: \"x\"
  reference: bytes 19..20, line 2, byte column 7: \"a\"
  target: same-source selected lexical binding (before)
    binding: bytes 7..8, line 1, byte column 8: \"a\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn input_exactly_at_product_cap_reaches_core() {
        let mut input = b"let x=1;".to_vec();
        input.resize(49_152, b' ');
        let run = es("cap", &input);
        let expected = head(49_152) + "analysis: complete\nrelations: 0\n";
        assert_report(&run, &expected);
    }

    #[test]
    fn input_over_product_cap_is_rejected_before_core() {
        let mut input = b"let x=1;".to_vec();
        input.resize(49_153, b' ');
        let run = es("over-cap", &input);
        assert_eq!(run.status, Some(1));
        assert_eq!(run.stdout, "");
        assert_eq!(run.stderr, "fa: stdin exceeds 49152 bytes\n");
    }

    #[test]
    fn invalid_utf8_is_rejected_before_core() {
        let run = es("invalid-utf8", b"let a=1;\xff");
        assert_eq!(run.status, Some(1));
        assert_eq!(run.stdout, "");
        assert_eq!(
            run.stderr,
            "fa: stdin is not valid UTF-8 (invalid sequence at byte 8)\n"
        );
    }

    #[test]
    fn usage_failures_exit_one_with_usage_on_stderr() {
        for (args, message) in [
            (&[][..], "missing command"),
            (&["es"][..], "unknown command"),
            (&["es-binding-refs", "a.js"][..], "unexpected argument"),
            (&["es-binding-refs", "--goal"][..], "unexpected argument"),
        ] {
            let run = fa(args, "usage", b"let x=1;");
            assert_eq!(run.status, Some(1), "{args:?}");
            assert_eq!(run.stdout, "", "{args:?}");
            assert_eq!(
                run.stderr,
                format!(
                    "fa: {message}\nusage: fa css-selectors < style.css\n       \
                 fa es-binding-refs < source.js\n"
                ),
                "{args:?}"
            );
        }
    }

    #[test]
    fn repeated_invocations_are_byte_identical() {
        let input = b"let a=1,b=2; let x=a+b+c, y=x;";
        let first = es("repeat-1", input);
        let second = es("repeat-2", input);

        assert_eq!(first.status, Some(0));
        assert_eq!(first.status, second.status);
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }

    #[test]
    fn empty_stdin_follows_the_core_recognizer() {
        let output = Command::new(env!("CARGO_BIN_EXE_fa"))
            .arg("es-binding-refs")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let run = finish(output);
        let expected = head(0) + "analysis: unsupported coverage for selected scope\n";
        assert_report(&run, &expected);
    }
}
