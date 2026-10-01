//! Process-boundary tests for `fa css-selectors` (#857).
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
            format!("fa: {message}\nusage: fa css-selectors < style.css\n"),
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
