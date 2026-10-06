//! Process-boundary tests for the `fa` CLI Product.
//!
//! Covers `fa css-selectors` (#857), `fa es-binding-refs` (#862), and
//! `fa html-tree` (#864) in one integration-test target.
//!
//! Every expected stdout/stderr text below is authored by hand from the
//! approved output contract and hand-counted source byte offsets. None of it
//! is captured from the binary under test.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Run {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

fn input_file(name: &str, bytes: &[u8]) -> PathBuf {
    static NEXT_INPUT_ID: AtomicUsize = AtomicUsize::new(0);

    let unique = NEXT_INPUT_ID.fetch_add(1, Ordering::Relaxed);
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("fa-stdin-{name}-{}-{unique}", std::process::id()));
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
                 fa es-binding-refs < source.js\n       fa html-tree < source.html\n"
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
                 fa es-binding-refs < source.js\n       fa html-tree < source.html\n"
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

// ---- #864: `fa html-tree` ----
//
// The workspace-state validator pins this crate to exactly one integration
// test target, so the HTML process tests live in this module. Every expected
// text is hand-authored from the approved output contract, the accepted tree
// and tokenizer golds, and hand-counted byte offsets; none is captured from
// the binary under test.
mod html_tree {
    use super::{Run, fa, finish};
    use std::process::{Command, Stdio};

    const PROFILE: &str = "profile: selected document construction; scripting disabled\n";

    const SYNTHESIZED_SHELL: &str = "\
#0 document
    authored evidence: none
  #1 element html
      authored evidence: none
      synthesized: implied by document structure
    #2 element head
        authored evidence: none
        synthesized: implied by document structure
";

    const SYNTHESIZED_BODY: &str = concat!(
        "    #3 element body\n",
        "        authored evidence: none\n",
        "        synthesized: implied by document structure\n",
    );

    const AUTHORED_BODY: &str = concat!(
        "    #3 element body\n",
        "        authored start tag: bytes 0..6, line 1, byte column 1: \"<body>\"\n",
        "        authored raw name: bytes 1..5, line 1, byte column 2: \"body\"\n",
    );

    const MISSING_DOCTYPE_AT_BODY: &str = "\
tree diagnostic 1: missing doctype
  recovery: continued in quirks document mode
  trigger: bytes 0..6, line 1, byte column 1: \"<body>\"
";

    fn html(name: &str, stdin: &[u8]) -> Run {
        fa(&["html-tree"], name, stdin)
    }

    fn head(bytes: usize) -> String {
        format!("capability: html-tree\nsource: id 0; {bytes} bytes\n{PROFILE}")
    }

    fn assert_report(run: &Run, expected: &str) {
        assert_eq!(run.status, Some(0));
        assert_eq!(run.stdout, expected);
        assert_eq!(run.stderr, "");
    }

    #[test]
    fn empty_stdin_is_a_complete_synthesized_document() {
        let output = Command::new(env!("CARGO_BIN_EXE_fa"))
            .arg("html-tree")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let run = finish(output);

        let expected = head(0)
            + "completion: complete
coverage: committed authored prefix bytes 0..0; processed tokens 1
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 4

" + SYNTHESIZED_SHELL
            + SYNTHESIZED_BODY
            + "
tree diagnostic 1: missing doctype
  recovery: continued in quirks document mode
  trigger: none (no authored boundary)
";
        assert_report(&run, &expected);
    }

    #[test]
    fn authored_and_synthesized_nodes_are_distinguished() {
        // <body> 0..6, <div> 6..11, </div> 11..17.
        let run = html("authored-synth", b"<body><div></div>");

        let expected = head(17)
            + "completion: complete
coverage: committed authored prefix bytes 0..17; processed tokens 4
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 5

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element div
          authored start tag: bytes 6..11, line 1, byte column 7: \"<div>\"
          authored raw name: bytes 7..10, line 1, byte column 8: \"div\"

selected ordinary relation 1: matching close
  node: #4 div
  trigger: bytes 11..17, line 1, byte column 12: \"</div>\"

" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn selected_ordinary_attribute_evidence_renders_core_provided_data_only() {
        // <body>0..6 <div id="a">6..18 (id="a" 11..17) </div>18..24.
        let run = html("attr-double", b"<body><div id=\"a\"></div>");

        let expected = head(24)
            + r##"completion: complete
coverage: committed authored prefix bytes 0..24; processed tokens 4
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 5

"## + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + r##"      #4 element div
          authored start tag: bytes 6..18, line 1, byte column 7: "<div id=\"a\">"
          authored raw name: bytes 7..10, line 1, byte column 8: "div"

selected ordinary relation 1: matching close
  node: #4 div
  trigger: bytes 18..24, line 1, byte column 19: "</div>"

selected ordinary attribute 1:
  node: #4 div
  authored attribute: bytes 11..17, line 1, byte column 12: "id=\"a\""
  authored name: bytes 11..13, line 1, byte column 12: "id"
  value syntax: double quoted
    equals: bytes 13..14, line 1, byte column 14: "="
    open quote: bytes 14..15, line 1, byte column 15: "\""
    value: bytes 15..16, line 1, byte column 16: "a"
    close quote: bytes 16..17, line 1, byte column 17: "\""
  interpreted name: "id"
  interpreted value: "a"

"## + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn character_reference_attribute_value_renders_the_core_decoded_value() {
        // <body>0..6 <div id="&amp;">6..22 (id="&amp;" 11..21, value 15..20)
        // </div>22..28. The CLI renders the authored spelling and the decoded
        // Core value; it decodes nothing itself.
        let run = html("attr-reference", b"<body><div id=\"&amp;\"></div>");

        let expected = head(28)
            + r##"completion: complete
coverage: committed authored prefix bytes 0..28; processed tokens 4
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 5

"## + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + r##"      #4 element div
          authored start tag: bytes 6..22, line 1, byte column 7: "<div id=\"&amp;\">"
          authored raw name: bytes 7..10, line 1, byte column 8: "div"

selected ordinary relation 1: matching close
  node: #4 div
  trigger: bytes 22..28, line 1, byte column 23: "</div>"

selected ordinary attribute 1:
  node: #4 div
  authored attribute: bytes 11..21, line 1, byte column 12: "id=\"&amp;\""
  authored name: bytes 11..13, line 1, byte column 12: "id"
  value syntax: double quoted
    equals: bytes 13..14, line 1, byte column 14: "="
    open quote: bytes 14..15, line 1, byte column 15: "\""
    value: bytes 15..20, line 1, byte column 16: "&amp;"
    close quote: bytes 20..21, line 1, byte column 21: "\""
  interpreted name: "id"
  interpreted value: "&"

"## + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn missing_after_equals_renders_its_empty_boundary_and_empty_interpreted_value() {
        // <body>0..6 <footer a=>6..17 (a= 14..16) </footer>17..26.
        let run = html("attr-missing-after-equals", b"<body><footer a=></footer>");

        let expected = head(26)
            + r##"completion: complete
coverage: committed authored prefix bytes 0..26; processed tokens 4
tokenizer diagnostics: 1
tree diagnostics: 1
nodes: 5

"## + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + r##"      #4 element footer
          authored start tag: bytes 6..17, line 1, byte column 7: "<footer a=>"
          authored raw name: bytes 7..13, line 1, byte column 8: "footer"

selected ordinary relation 1: matching close
  node: #4 footer
  trigger: bytes 17..26, line 1, byte column 18: "</footer>"

selected ordinary attribute 1:
  node: #4 footer
  authored attribute: bytes 14..16, line 1, byte column 15: "a="
  authored name: bytes 14..15, line 1, byte column 15: "a"
  value syntax: missing after equals
    equals: bytes 15..16, line 1, byte column 16: "="
    value boundary: bytes 16..16, line 1, byte column 17: ""
  interpreted name: "a"
  interpreted value: ""

tokenizer diagnostic 1: missing attribute value
  location: bytes 16..17, line 1, byte column 17: ">"

"## + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn raw_nul_is_rendered_as_authored_evidence_apart_from_the_interpreted_value() {
        // <body>0..6 <div a="NUL">6..17 (a="NUL" 11..16, NUL 14..15) </div>17..23.
        let run = html("attr-nul", b"<body><div a=\"\0\"></div>");

        let expected = head(23)
            + r##"completion: complete
coverage: committed authored prefix bytes 0..23; processed tokens 4
tokenizer diagnostics: 1
tree diagnostics: 1
nodes: 5

"## + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + r##"      #4 element div
          authored start tag: bytes 6..17, line 1, byte column 7: "<div a=\"\u{0}\">"
          authored raw name: bytes 7..10, line 1, byte column 8: "div"

selected ordinary relation 1: matching close
  node: #4 div
  trigger: bytes 17..23, line 1, byte column 18: "</div>"

selected ordinary attribute 1:
  node: #4 div
  authored attribute: bytes 11..16, line 1, byte column 12: "a=\"\u{0}\""
  authored name: bytes 11..12, line 1, byte column 12: "a"
  value syntax: double quoted
    equals: bytes 12..13, line 1, byte column 13: "="
    open quote: bytes 13..14, line 1, byte column 14: "\""
    value: bytes 14..15, line 1, byte column 15: "\u{0}"
    close quote: bytes 15..16, line 1, byte column 16: "\""
  interpreted name: "a"
  interpreted value: "\u{fffd}"

tokenizer diagnostic 1: unexpected null character
  location: bytes 14..15, line 1, byte column 15: "\u{0}"

"## + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn nested_selected_elements_render_in_final_parent_child_order() {
        // <body>0..6 <div>6..11 <section>11..20 <p>20..23 t23..24 </p>24..28
        // </section>28..38 </div>38..44 </body>44..51.
        let run = html(
            "nested",
            b"<body><div><section><p>t</p></section></div></body>",
        );

        let expected = head(51)
            + "completion: complete
coverage: committed authored prefix bytes 0..51; processed tokens 10
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 8

"
            + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element div
          authored start tag: bytes 6..11, line 1, byte column 7: \"<div>\"
          authored raw name: bytes 7..10, line 1, byte column 8: \"div\"
        #5 element section
            authored start tag: bytes 11..20, line 1, byte column 12: \"<section>\"
            authored raw name: bytes 12..19, line 1, byte column 13: \"section\"
          #6 element p
              authored start tag: bytes 20..23, line 1, byte column 21: \"<p>\"
              authored raw name: bytes 21..22, line 1, byte column 22: \"p\"
            #7 text \"t\"
                contribution 1: source bytes 23..24, line 1, byte column 24: \"t\"; interpreted \"t\"

selected ordinary relation 1: matching close
  node: #5 section
  trigger: bytes 28..38, line 1, byte column 29: \"</section>\"
selected ordinary relation 2: matching close
  node: #4 div
  trigger: bytes 38..44, line 1, byte column 39: \"</div>\"

paragraph relation 1: matching close
  node: #6 p
  trigger: bytes 24..28, line 1, byte column 25: \"</p>\"

"
            + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    /// The Paragraph relation section as printed: from the first
    /// `paragraph relation` line through the last of its blocks. Empty when
    /// the section is absent.
    fn paragraph_section(run: &Run) -> String {
        assert_eq!(run.status, Some(0));
        assert_eq!(run.stderr, "");
        let lines: Vec<&str> = run.stdout.lines().collect();
        let Some(first) = lines
            .iter()
            .position(|line| line.starts_with("paragraph relation "))
        else {
            return String::new();
        };
        let mut section = String::new();
        for line in &lines[first..] {
            if line.is_empty() {
                break;
            }
            section.push_str(line);
            section.push('\n');
        }
        section
    }

    #[test]
    fn paragraph_matching_and_start_triggered_close_render_core_relations() {
        // <body>0..6 <p>6..9 a9..10 <p>10..13 b13..14 </p>14..18.
        let run = html("paragraph-start", b"<body><p>a<p>b</p>");

        let expected = "\
paragraph relation 1: start-triggered close
  node: #4 p
  inserted: #6 p
  trigger: bytes 10..13, line 1, byte column 11: \"<p>\"
paragraph relation 2: matching close
  node: #6 p
  trigger: bytes 14..18, line 1, byte column 15: \"</p>\"
";
        assert_eq!(paragraph_section(&run), expected);
        assert!(run.stdout.contains("completion: complete\n"));
        assert!(!run.stdout.contains("selected ordinary relation"));
    }

    #[test]
    fn repeated_unmatched_paragraph_ends_render_distinct_nodes_and_triggers() {
        // <body>0..6 </p>6..10 </p>10..14.
        let run = html("paragraph-synth", b"<body></p></p>");

        let expected = "\
paragraph relation 1: synthesized close by unmatched end tag
  node: #4 p
  trigger: bytes 6..10, line 1, byte column 7: \"</p>\"
paragraph relation 2: synthesized close by unmatched end tag
  node: #5 p
  trigger: bytes 10..14, line 1, byte column 11: \"</p>\"
";
        assert_eq!(paragraph_section(&run), expected);
    }

    #[test]
    fn all_four_paragraph_meanings_render_in_their_own_slice_order() {
        // <body>0..6 <div>6..11 <p>11..14 a14..15 </p>15..19 <section>19..28
        // <p>28..31 b31..32 </div>32..38 </p>38..42.
        let run = html(
            "paragraph-all",
            b"<body><div><p>a</p><section><p>b</div></p>",
        );

        let expected = "\
paragraph relation 1: matching close
  node: #5 p
  trigger: bytes 15..19, line 1, byte column 16: \"</p>\"
paragraph relation 2: implied pop by selected ordinary end tag
  node: #8 p
  target: #4 div
  trigger: bytes 32..38, line 1, byte column 33: \"</div>\"
paragraph relation 3: synthesized close by unmatched end tag
  node: #10 p
  trigger: bytes 38..42, line 1, byte column 39: \"</p>\"
";
        assert_eq!(paragraph_section(&run), expected);
        // The selected ordinary slice keeps its own meanings and numbering;
        // the implied pop is not among them.
        let selected = run
            .stdout
            .find("selected ordinary relation 1: recovery pop")
            .unwrap();
        let paragraph = run.stdout.find("paragraph relation 1").unwrap();
        assert!(selected < paragraph);
        assert!(
            run.stdout
                .contains("selected ordinary relation 2: matching close\n  node: #4 div\n")
        );
        assert!(!run.stdout.contains("selected ordinary relation 3"));
    }

    #[test]
    fn paragraph_relation_survives_an_incomplete_report_without_upgrading_it() {
        // <body>0..6 <p>6..9 x9..10 </p>10..14 <span>14..20 (unsupported).
        let run = html("paragraph-incomplete", b"<body><p>x</p><span>");

        assert!(
            run.stdout
                .contains("completion: incomplete; tree unsupported: non-shell element tag\n")
        );
        let expected = "\
paragraph relation 1: matching close
  node: #4 p
  trigger: bytes 10..14, line 1, byte column 11: \"</p>\"
";
        assert_eq!(paragraph_section(&run), expected);
    }

    #[test]
    fn open_paragraph_at_end_of_file_prints_no_paragraph_section() {
        let run = html("paragraph-open", b"<body><p>x");

        assert_eq!(paragraph_section(&run), "");
        assert!(!run.stdout.contains("paragraph relation"));
    }

    #[test]
    fn block_container_family_names_render_from_core_retained_meaning() {
        // <body>0..6 <article>6..15 <nav>15..20 t20..21 </nav>21..27
        // </article>27..37 </body>37..44.
        let run = html(
            "family-nested",
            b"<body><article><nav>t</nav></article></body>",
        );

        let expected = head(44)
            + "completion: complete
coverage: committed authored prefix bytes 0..44; processed tokens 8
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 7

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element article
          authored start tag: bytes 6..15, line 1, byte column 7: \"<article>\"
          authored raw name: bytes 7..14, line 1, byte column 8: \"article\"
        #5 element nav
            authored start tag: bytes 15..20, line 1, byte column 16: \"<nav>\"
            authored raw name: bytes 16..19, line 1, byte column 17: \"nav\"
          #6 text \"t\"
              contribution 1: source bytes 20..21, line 1, byte column 21: \"t\"; interpreted \"t\"

selected ordinary relation 1: matching close
  node: #5 nav
  trigger: bytes 21..27, line 1, byte column 22: \"</nav>\"
selected ordinary relation 2: matching close
  node: #4 article
  trigger: bytes 27..37, line 1, byte column 28: \"</article>\"

" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn remaining_block_container_family_names_render_as_siblings() {
        // <body>0..6 <aside>6..13 </aside>13..21 <footer>21..29 </footer>29..38
        // <header>38..46 </header>46..55 <main>55..61 </main>61..68
        // </body>68..75.
        let run = html(
            "family-siblings",
            b"<body><aside></aside><footer></footer><header></header><main></main></body>",
        );

        let expected = head(75)
            + "completion: complete
coverage: committed authored prefix bytes 0..75; processed tokens 11
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 8

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element aside
          authored start tag: bytes 6..13, line 1, byte column 7: \"<aside>\"
          authored raw name: bytes 7..12, line 1, byte column 8: \"aside\"
      #5 element footer
          authored start tag: bytes 21..29, line 1, byte column 22: \"<footer>\"
          authored raw name: bytes 22..28, line 1, byte column 23: \"footer\"
      #6 element header
          authored start tag: bytes 38..46, line 1, byte column 39: \"<header>\"
          authored raw name: bytes 39..45, line 1, byte column 40: \"header\"
      #7 element main
          authored start tag: bytes 55..61, line 1, byte column 56: \"<main>\"
          authored raw name: bytes 56..60, line 1, byte column 57: \"main\"

selected ordinary relation 1: matching close
  node: #4 aside
  trigger: bytes 13..21, line 1, byte column 14: \"</aside>\"
selected ordinary relation 2: matching close
  node: #5 footer
  trigger: bytes 29..38, line 1, byte column 30: \"</footer>\"
selected ordinary relation 3: matching close
  node: #6 header
  trigger: bytes 46..55, line 1, byte column 47: \"</header>\"
selected ordinary relation 4: matching close
  node: #7 main
  trigger: bytes 61..68, line 1, byte column 62: \"</main>\"

" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn one_end_tag_renders_every_recovery_pop_then_the_matching_close() {
        // <body>0..6 <header>6..14 <main>14..20 <aside>20..27 </header>27..36.
        let run = html("recovery-multi", b"<body><header><main><aside></header>");

        let expected = head(36)
            + "completion: complete
coverage: committed authored prefix bytes 0..36; processed tokens 6
tokenizer diagnostics: 0
tree diagnostics: 2
nodes: 7

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element header
          authored start tag: bytes 6..14, line 1, byte column 7: \"<header>\"
          authored raw name: bytes 7..13, line 1, byte column 8: \"header\"
        #5 element main
            authored start tag: bytes 14..20, line 1, byte column 15: \"<main>\"
            authored raw name: bytes 15..19, line 1, byte column 16: \"main\"
          #6 element aside
              authored start tag: bytes 20..27, line 1, byte column 21: \"<aside>\"
              authored raw name: bytes 21..26, line 1, byte column 22: \"aside\"

selected ordinary relation 1: recovery pop by ancestor end tag
  node: #6 aside
  target: #4 header
  trigger: bytes 27..36, line 1, byte column 28: \"</header>\"
selected ordinary relation 2: recovery pop by ancestor end tag
  node: #5 main
  target: #4 header
  trigger: bytes 27..36, line 1, byte column 28: \"</header>\"
selected ordinary relation 3: matching close
  node: #4 header
  trigger: bytes 27..36, line 1, byte column 28: \"</header>\"

" + MISSING_DOCTYPE_AT_BODY
            + "tree diagnostic 2: misnested selected ordinary end tag
  recovery: popped intervening selected ordinary elements and closed target
  trigger: bytes 27..36, line 1, byte column 28: \"</header>\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn repeated_target_names_render_the_exact_target_identity() {
        // <body>0..6 <div>6..11 <div>11..16 <nav>16..21 </div>21..27 </div>27..33.
        let run = html("recovery-repeated", b"<body><div><div><nav></div></div>");

        let expected = head(33)
            + "completion: complete
coverage: committed authored prefix bytes 0..33; processed tokens 7
tokenizer diagnostics: 0
tree diagnostics: 2
nodes: 7

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element div
          authored start tag: bytes 6..11, line 1, byte column 7: \"<div>\"
          authored raw name: bytes 7..10, line 1, byte column 8: \"div\"
        #5 element div
            authored start tag: bytes 11..16, line 1, byte column 12: \"<div>\"
            authored raw name: bytes 12..15, line 1, byte column 13: \"div\"
          #6 element nav
              authored start tag: bytes 16..21, line 1, byte column 17: \"<nav>\"
              authored raw name: bytes 17..20, line 1, byte column 18: \"nav\"

selected ordinary relation 1: recovery pop by ancestor end tag
  node: #6 nav
  target: #5 div
  trigger: bytes 21..27, line 1, byte column 22: \"</div>\"
selected ordinary relation 2: matching close
  node: #5 div
  trigger: bytes 21..27, line 1, byte column 22: \"</div>\"
selected ordinary relation 3: matching close
  node: #4 div
  trigger: bytes 27..33, line 1, byte column 28: \"</div>\"

" + MISSING_DOCTYPE_AT_BODY
            + "tree diagnostic 2: misnested selected ordinary end tag
  recovery: popped intervening selected ordinary elements and closed target
  trigger: bytes 21..27, line 1, byte column 22: \"</div>\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn unmatched_end_and_eof_open_render_no_relation_section() {
        for (name, input) in [
            ("unmatched-end", &b"<body></article>"[..]),
            ("eof-open", &b"<body><article>"[..]),
        ] {
            let run = html(name, input);
            assert_eq!(run.status, Some(0), "{name}");
            assert!(
                !run.stdout.contains("selected ordinary relation"),
                "{name}: {}",
                run.stdout
            );
            assert_eq!(run.stderr, "", "{name}");
        }
    }

    #[test]
    fn relation_before_an_unsupported_stop_renders_beside_the_incomplete_state() {
        // <body>0..6 <div>6..11 </div>11..17 <span>17..23 (unsupported).
        let run = html("relation-incomplete", b"<body><div></div><span>");

        let expected = head(23)
            + "completion: incomplete; tree unsupported: non-shell element tag
  trigger: bytes 17..23, line 1, byte column 18: \"<span>\"
coverage: committed authored prefix bytes 0..17; processed tokens 3
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 5

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element div
          authored start tag: bytes 6..11, line 1, byte column 7: \"<div>\"
          authored raw name: bytes 7..10, line 1, byte column 8: \"div\"

selected ordinary relation 1: matching close
  node: #4 div
  trigger: bytes 11..17, line 1, byte column 12: \"</div>\"

" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn relation_rendering_is_deterministic_and_keeps_authored_end_tag_casing() {
        // <body>0..6 <ArTiClE>6..15 </aRtIcLe>15..25.
        let first = html("relation-case-a", b"<body><ArTiClE></aRtIcLe>");
        let second = html("relation-case-b", b"<body><ArTiClE></aRtIcLe>");

        assert_eq!(first.status, Some(0));
        assert_eq!(first.stdout, second.stdout);
        assert!(first.stdout.contains(
            "selected ordinary relation 1: matching close
  node: #4 article
  trigger: bytes 15..25, line 1, byte column 16: \"</aRtIcLe>\"
"
        ));
    }

    #[test]
    fn title_named_reference_separates_interpreted_text_from_authored_contributions() {
        // Accepted TC-S10 gold: `<title>a&amp;b</title>` is one text "a&b"
        // with contributions 7..8, 8..13 ("&amp;" -> "&"), 13..14.
        let run = html("title-reference", b"<title>a&amp;b</title>");

        let expected = head(22)
            + "completion: complete
coverage: committed authored prefix bytes 0..22; processed tokens 6
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 6

#0 document
    authored evidence: none
  #1 element html
      authored evidence: none
      synthesized: implied by document structure
    #2 element head
        authored evidence: none
        synthesized: implied by document structure
      #3 element title
          authored start tag: bytes 0..7, line 1, byte column 1: \"<title>\"
          authored raw name: bytes 1..6, line 1, byte column 2: \"title\"
        #4 text \"a&b\"
            contribution 1: source bytes 7..8, line 1, byte column 8: \"a\"; interpreted \"a\"
            contribution 2: source bytes 8..13, line 1, byte column 9: \"&amp;\"; interpreted \"&\"
            contribution 3: source bytes 13..14, line 1, byte column 14: \"b\"; interpreted \"b\"
    #5 element body
        authored evidence: none
        synthesized: implied by document structure

tree diagnostic 1: missing doctype
  recovery: continued in quirks document mode
  trigger: bytes 0..7, line 1, byte column 1: \"<title>\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn data_named_reference_separates_interpreted_text_from_authored_contributions() {
        let run = html("data-reference", b"<body>a&amp;b</body>");

        let expected = head(20)
            + "completion: complete
coverage: committed authored prefix bytes 0..20; processed tokens 6
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 5

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 text \"a&b\"
          contribution 1: source bytes 6..7, line 1, byte column 7: \"a\"; interpreted \"a\"
          contribution 2: source bytes 7..12, line 1, byte column 8: \"&amp;\"; interpreted \"&\"
          contribution 3: source bytes 12..13, line 1, byte column 13: \"b\"; interpreted \"b\"

" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn authored_data_nul_is_ignored_in_body_with_tokenizer_and_tree_evidence() {
        // `<body>` 0..6, authored U+0000 6..7: no text node, no U+FFFD.
        let run = html("data-nul", b"<body>\0");

        let expected = head(7)
            + "completion: complete
coverage: committed authored prefix bytes 0..7; processed tokens 3
tokenizer diagnostics: 1
tree diagnostics: 2
nodes: 4

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "
tokenizer diagnostic 1: unexpected null character
  location: bytes 6..7, line 1, byte column 7: \"\\u{0}\"

" + MISSING_DOCTYPE_AT_BODY
            + "tree diagnostic 2: null character in in-body
  recovery: ignored token
  trigger: bytes 6..7, line 1, byte column 7: \"\\u{0}\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn after_after_body_character_data_reports_a_distinct_recovery_diagnostic() {
        // `<body>` 0..6, `</body>` 6..13, `</html>` 13..20, `x` 20..21. The
        // character is reached in after-after-body, recovers into in-body, and
        // is inserted as text; five tokens are processed exactly once each.
        let run = html("after-after-body", b"<body></body></html>x");

        let expected = head(21)
            + "completion: complete
coverage: committed authored prefix bytes 0..21; processed tokens 5
tokenizer diagnostics: 0
tree diagnostics: 2
nodes: 5

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 text \"x\"
          contribution 1: source bytes 20..21, line 1, byte column 21: \"x\"; interpreted \"x\"

" + MISSING_DOCTYPE_AT_BODY
            + "tree diagnostic 2: after-after-body character data
  recovery: switched to in-body and reprocessed same token
  trigger: bytes 20..21, line 1, byte column 21: \"x\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn authored_nul_after_html_end_keeps_tokenizer_recovery_and_ignore_diagnostics_distinct() {
        // `</html>` ends at byte 20; the authored U+0000 is 20..21. The tree
        // reports the after-after-body recovery first, then the in-body NUL
        // ignore, for the same trigger; no text node and no U+FFFD exist.
        let run = html("after-after-body-nul", b"<body></body></html>\0");

        let expected = head(21)
            + "completion: complete
coverage: committed authored prefix bytes 0..21; processed tokens 5
tokenizer diagnostics: 1
tree diagnostics: 3
nodes: 4

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "
tokenizer diagnostic 1: unexpected null character
  location: bytes 20..21, line 1, byte column 21: \"\\u{0}\"

" + MISSING_DOCTYPE_AT_BODY
            + "tree diagnostic 2: after-after-body character data
  recovery: switched to in-body and reprocessed same token
  trigger: bytes 20..21, line 1, byte column 21: \"\\u{0}\"
tree diagnostic 3: null character in in-body
  recovery: ignored token
  trigger: bytes 20..21, line 1, byte column 21: \"\\u{0}\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn complete_report_with_a_tokenizer_diagnostic_exits_zero() {
        // `<title>` 0..7, `a` 7..8, U+0001 8..9.
        let run = html("complete-diagnostic", b"<title>a\x01b</title>");

        assert_eq!(run.status, Some(0));
        assert_eq!(run.stderr, "");
        assert!(
            run.stdout.contains("\ncompletion: complete\n"),
            "{}",
            run.stdout
        );
        assert!(run.stdout.contains("\ntokenizer diagnostics: 1\n"));
        assert!(run.stdout.contains(
            "tokenizer diagnostic 1: control character in input stream
  location: bytes 8..9, line 1, byte column 9: \"\\u{1}\"
"
        ));
    }

    #[test]
    fn numeric_character_reference_diagnostics_use_deterministic_wording() {
        // `<body>` 0..6, then each reference anchors at its `;`, or at the
        // offending unit for the no-digits recovery.
        let run = html(
            "numeric-diagnostics",
            b"<body>&#0;&#xD800;&#x110000;&#xFDD0;&#x80;&#;",
        );

        assert_eq!(run.status, Some(0));
        assert_eq!(run.stderr, "");
        assert!(
            run.stdout.contains("\ncompletion: complete\n"),
            "{}",
            run.stdout
        );
        assert!(run.stdout.contains("\ntokenizer diagnostics: 6\n"));
        for (index, wording, location) in [
            (1, "null character reference", "9..10"),
            (2, "surrogate character reference", "17..18"),
            (3, "character reference outside Unicode range", "27..28"),
            (4, "noncharacter character reference", "35..36"),
            (5, "control character reference", "41..42"),
            (
                6,
                "absence of digits in numeric character reference",
                "44..45",
            ),
        ] {
            let expected =
                format!("tokenizer diagnostic {index}: {wording}\n  location: bytes {location}, ");
            assert!(run.stdout.contains(&expected), "{expected}\n{}", run.stdout);
        }
    }

    #[test]
    fn recovery_synthesizes_a_paragraph_without_authored_evidence() {
        // <body> 0..6, </p> 6..10.
        let run = html("recovery", b"<body></p>");

        let expected = head(10)
            + "completion: complete
coverage: committed authored prefix bytes 0..10; processed tokens 3
tokenizer diagnostics: 0
tree diagnostics: 2
nodes: 5

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "      #4 element p
          authored evidence: none
          synthesized: unmatched paragraph end tag

paragraph relation 1: synthesized close by unmatched end tag
  node: #4 p
  trigger: bytes 6..10, line 1, byte column 7: \"</p>\"

" + MISSING_DOCTYPE_AT_BODY
            + "tree diagnostic 2: unmatched paragraph end tag
  recovery: synthesized paragraph element and closed it
  trigger: bytes 6..10, line 1, byte column 7: \"</p>\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn tree_unsupported_is_reported_with_its_trigger_and_exit_zero() {
        // <body> 0..6, <span> 6..12.
        let run = html("tree-unsupported", b"<body><span>");

        let expected = head(12)
            + "completion: incomplete; tree unsupported: non-shell element tag
  trigger: bytes 6..12, line 1, byte column 7: \"<span>\"
coverage: committed authored prefix bytes 0..6; processed tokens 1
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 4

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "
" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    // ---- #892: selected canonical DOCTYPE / Initial No-Quirks successor ----
    //
    // Every expectation is hand-authored from the selected theorem and
    // hand-counted source byte offsets. `<!DOCTYPE ` is 10 bytes and `html`
    // 4, so the name is bytes 10..14 on a single-line source.

    const DOCTYPE_SHELL_TAIL: &str = "  #2 element html
      authored evidence: none
      synthesized: implied by document structure
    #3 element head
        authored evidence: none
        synthesized: implied by document structure
    #4 element body
        authored evidence: none
        synthesized: implied by document structure
";

    #[test]
    fn canonical_doctype_with_an_implied_shell_has_no_missing_doctype() {
        let run = html("doctype-implied", b"<!DOCTYPE html>");

        let expected = head(15)
            + "completion: complete
coverage: committed authored prefix bytes 0..15; processed tokens 2
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 5

#0 document
    authored evidence: none
  #1 doctype html
      authored doctype: bytes 0..15, line 1, byte column 1: \"<!DOCTYPE html>\"
      authored name: bytes 10..14, line 1, byte column 11: \"html\"
" + DOCTYPE_SHELL_TAIL;
        assert_report(&run, &expected);
        assert!(!run.stdout.contains("missing doctype"));
    }

    #[test]
    fn canonical_doctype_with_an_explicit_shell_keeps_its_document_child_order() {
        // `<!DOCTYPE html>` 0..15, `<html>` 15..21, `<head>` 21..27,
        // `</head>` 27..34, `<body>` 34..40, `</body>` 40..47,
        // `</html>` 47..54; eight tokens including the end of file.
        let run = html(
            "doctype-explicit",
            b"<!DOCTYPE html><html><head></head><body></body></html>",
        );

        let expected = head(54)
            + "completion: complete
coverage: committed authored prefix bytes 0..54; processed tokens 8
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 5

#0 document
    authored evidence: none
  #1 doctype html
      authored doctype: bytes 0..15, line 1, byte column 1: \"<!DOCTYPE html>\"
      authored name: bytes 10..14, line 1, byte column 11: \"html\"
  #2 element html
      authored start tag: bytes 15..21, line 1, byte column 16: \"<html>\"
      authored raw name: bytes 16..20, line 1, byte column 17: \"html\"
    #3 element head
        authored start tag: bytes 21..27, line 1, byte column 22: \"<head>\"
        authored raw name: bytes 22..26, line 1, byte column 23: \"head\"
    #4 element body
        authored start tag: bytes 34..40, line 1, byte column 35: \"<body>\"
        authored raw name: bytes 35..39, line 1, byte column 36: \"body\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn canonical_doctype_renders_the_exact_authored_case_spelling() {
        let run = html("doctype-case", b"<!DoCtYpE HTML>");

        let expected = head(15)
            + "completion: complete
coverage: committed authored prefix bytes 0..15; processed tokens 2
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 5

#0 document
    authored evidence: none
  #1 doctype html
      authored doctype: bytes 0..15, line 1, byte column 1: \"<!DoCtYpE HTML>\"
      authored name: bytes 10..14, line 1, byte column 11: \"HTML\"
" + DOCTYPE_SHELL_TAIL;
        assert_report(&run, &expected);
    }

    #[test]
    fn canonical_doctype_with_a_line_feed_separator_keeps_raw_evidence_and_coordinates() {
        // `<!DOCTYPE` 0..9, LF 9..10, `html` 10..14 (line 2, column 1), `>`.
        let run = html("doctype-lf", b"<!DOCTYPE\nhtml>");

        let expected = head(15)
            + "completion: complete
coverage: committed authored prefix bytes 0..15; processed tokens 2
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 5

#0 document
    authored evidence: none
  #1 doctype html
      authored doctype: bytes 0..15, line 1, byte column 1: \"<!DOCTYPE\\u{a}html>\"
      authored name: bytes 10..14, line 2, byte column 1: \"html\"
" + DOCTYPE_SHELL_TAIL;
        assert_report(&run, &expected);
    }

    #[test]
    fn a_second_doctype_is_tree_unsupported_and_constructs_no_second_node() {
        // The second DOCTYPE is bytes 15..30 and is refused in before-html.
        let run = html("doctype-second", b"<!DOCTYPE html><!DOCTYPE html>");

        let expected = head(30)
            + "completion: incomplete; tree unsupported: doctype outside initial
  trigger: bytes 15..30, line 1, byte column 16: \"<!DOCTYPE html>\"
coverage: committed authored prefix bytes 0..15; processed tokens 1
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 2

#0 document
    authored evidence: none
  #1 doctype html
      authored doctype: bytes 0..15, line 1, byte column 1: \"<!DOCTYPE html>\"
      authored name: bytes 10..14, line 1, byte column 11: \"html\"
";
        assert_report(&run, &expected);
    }

    #[test]
    fn non_selected_markup_declarations_stay_tokenizer_unsupported() {
        for (name, source, bytes, trigger) in [
            (
                "md-xx",
                &b"<!xx>"[..],
                5,
                "bytes 0..2, line 1, byte column 1: \"<!\"",
            ),
            (
                "md-svg",
                &b"<!DOCTYPE svg>"[..],
                14,
                "bytes 0..2, line 1, byte column 1: \"<!\"",
            ),
            (
                "md-glued",
                &b"<!DOCTYPEhtml>"[..],
                14,
                "bytes 0..2, line 1, byte column 1: \"<!\"",
            ),
        ] {
            let run = html(name, source);

            let expected = head(bytes)
                + &format!(
                    "completion: incomplete; tokenizer unsupported: markup declaration (deferred)
  trigger: {trigger}
coverage: committed authored prefix bytes 0..0; processed tokens 0
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 1

#0 document
    authored evidence: none
"
                );
            assert_report(&run, &expected);
        }
    }

    #[test]
    fn canonical_doctype_output_is_byte_identical_across_invocations() {
        let first = html("doctype-repeat-1", b"<!DOCTYPE html><html></html>");
        let second = html("doctype-repeat-2", b"<!DOCTYPE html><html></html>");
        assert_eq!(first.status, Some(0));
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
        assert!(first.stdout.contains("#1 doctype html"));
    }

    #[test]
    fn tokenizer_unsupported_is_distinct_from_tree_unsupported() {
        // `<!xx>` is the lower-layer MarkupDeclaration boundary (#880 sentinel
        // migration; Numeric references are now supported). It follows the
        // 6-byte `<body>`, and coverage rolls back to the authored `<`.
        let run = html("tokenizer-unsupported", b"<body><!xx>");

        let expected = head(11)
            + "completion: incomplete; tokenizer unsupported: markup declaration (deferred)
  trigger: bytes 6..8, line 1, byte column 7: \"<!\"
coverage: committed authored prefix bytes 0..6; processed tokens 1
tokenizer diagnostics: 0
tree diagnostics: 1
nodes: 4

" + SYNTHESIZED_SHELL
            + AUTHORED_BODY
            + "
" + MISSING_DOCTYPE_AT_BODY;
        assert_report(&run, &expected);
    }

    #[test]
    fn resource_limited_is_distinct_and_reports_kind_limit_attempted_and_anchor() {
        // Accepted RES-005 shifted by `<body>`: `y` of `<div x y>` is at 13.
        let run = html("attributes", b"<body><div x y>");

        assert_eq!(run.status, Some(0));
        assert_eq!(run.stderr, "");
        assert!(
            run.stdout.contains(
                "completion: incomplete; resource limited: attributes per tag (limit 1, attempted 2)
  at: bytes 13..13, line 1, byte column 14: \"\"
"
            ),
            "{}",
            run.stdout
        );
    }

    fn core_source_bytes_refusal(bytes: usize) -> String {
        head(bytes)
            + &format!(
                "completion: incomplete; resource limited: source bytes (limit 36864, attempted {bytes})
  at: bytes 0..0, line 1, byte column 1: \"\"
coverage: committed authored prefix bytes 0..0; processed tokens 0
tokenizer diagnostics: 0
tree diagnostics: 0
nodes: 1

#0 document
    authored evidence: none
"
            )
    }

    #[test]
    fn core_source_bytes_refusal_is_an_analysis_report_not_input_failure() {
        // The lower edge of the 36865..=49152 acquisition/Core window.
        let run = html("core-window-low", &vec![b'a'; 36_865]);
        assert_report(&run, &core_source_bytes_refusal(36_865));
    }

    #[test]
    fn core_source_bytes_limit_admits_exactly_36864_bytes() {
        let mut input = b"<body>".to_vec();
        input.resize(36_864, b'x');
        let run = html("core-exact", &input);

        assert_eq!(run.status, Some(0));
        assert_eq!(run.stderr, "");
        assert!(run.stdout.starts_with(&head(36_864)));
        assert!(run.stdout.contains("\ncompletion: complete\n"));
        assert!(
            run.stdout
                .contains("coverage: committed authored prefix bytes 0..36864; ")
        );
    }

    #[test]
    fn input_exactly_at_product_cap_reaches_core() {
        let run = html("product-cap", &vec![b'a'; 49_152]);
        assert_report(&run, &core_source_bytes_refusal(49_152));
    }

    #[test]
    fn input_over_product_cap_is_rejected_before_core() {
        let run = html("over-product-cap", &vec![b'a'; 49_153]);
        assert_eq!(run.status, Some(1));
        assert_eq!(run.stdout, "");
        assert_eq!(run.stderr, "fa: stdin exceeds 49152 bytes\n");
    }

    #[test]
    fn invalid_utf8_is_rejected_before_core() {
        let run = html("invalid-utf8", b"<body>\xff");
        assert_eq!(run.status, Some(1));
        assert_eq!(run.stdout, "");
        assert_eq!(
            run.stderr,
            "fa: stdin is not valid UTF-8 (invalid sequence at byte 6)\n"
        );
    }

    #[test]
    fn usage_failures_exit_one_with_usage_on_stderr() {
        for (args, message) in [
            (&[][..], "missing command"),
            (&["html"][..], "unknown command"),
            (&["html-tree", "a.html"][..], "unexpected argument"),
            (&["html-tree", "--fragment"][..], "unexpected argument"),
        ] {
            let run = fa(args, "usage", b"<body>");
            assert_eq!(run.status, Some(1), "{args:?}");
            assert_eq!(run.stdout, "", "{args:?}");
            assert_eq!(
                run.stderr,
                format!(
                    "fa: {message}\nusage: fa css-selectors < style.css\n       \
                 fa es-binding-refs < source.js\n       fa html-tree < source.html\n"
                ),
                "{args:?}"
            );
        }
    }

    #[test]
    fn repeated_invocations_are_byte_identical() {
        let input = b"<body><div><section><p>t</p></div></section></body><title>";
        let first = html("repeat-1", input);
        let second = html("repeat-2", input);

        assert_eq!(first.status, Some(0));
        assert_eq!(first.status, second.status);
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }

    #[test]
    fn exact_source_is_not_normalized_by_the_product() {
        // A leading BOM is preserved by the Product and owned by the HTML
        // tokenizer's accepted preprocessing; the byte count is exact.
        let run = html("bom", b"\xef\xbb\xbf<body>");
        assert_eq!(run.status, Some(0));
        assert!(run.stdout.starts_with(&head(9)), "{}", run.stdout);
    }
}
