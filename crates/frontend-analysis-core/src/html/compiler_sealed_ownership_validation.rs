//! Candidate-independent mechanical validation of the compiler-sealed
//! generated-data ownership theorem (#396).
//!
//! This module proves **nothing about HTML semantics**. It exists to answer one
//! question with the repository's own pinned compiler rather than by argument:
//!
//! > Can `rustc` alone own "exactly one canonical owner of the generated Named
//! > Character Reference data", such that no source spelling can establish a
//! > second one and no configuration can silently remove the first?
//!
//! ## Why this is a compiler question
//!
//! The previously attempted repository-local scanner was rejected six times
//! (R6 `cfg`/`cfg_attr`, R7 `#[path]`/`include!`, R8 `concat!` and raw strings,
//! R9 ordinary string escapes, R10 character literals, R11 raw byte strings,
//! R12 malformed escapes). Every remediation closed one spelling and the next
//! review found another, because a scanner must re-derive lexical and
//! configuration semantics that only `rustc` actually owns.
//!
//! So this harness never inspects Rust source. It writes disposable fixtures to
//! a temporary directory, runs the pinned compiler over them, and asserts the
//! compiler's own exit status and diagnostic. Reading a `rustc` diagnostic is
//! not parsing Rust; it is reading the compiler's verdict.
//!
//! ## The theorem under test
//!
//! ```text
//! consumer
//!    -> private canonical owner wrapper      (the only access path)
//!       -> generated artifact registration   (impl of a private trait)
//! ```
//!
//! A duplicate registration — however it is spelled or loaded — is a second
//! `impl` of the same private trait for the same private token, which is a
//! coherence violation (`E0119`). Removing the canonical owner breaks the
//! consumer's only access path (`E0433`). Both are compiler-owned.
//!
//! ## What this module deliberately does not do
//!
//! It does not implement the production seal, touch the real generated table,
//! or model entity semantics. The fake dataset is two rows, because ownership
//! mechanics are what is under test.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The tripwire text the generated data tests carry. Authored here, so matching
/// it is recognizing our own message rather than guessing a compiler string.
const TRIPWIRE_MESSAGE: &str = "generated Named Character Reference data tests are test-only";

/// The exact toolchain this evidence is only meaningful against.
const ACCEPTED_RUSTC_VERSION: &str = "1.97.1";

/// How a fixture crate is configured for a compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildMode {
    /// An ordinary library build: `cfg(test)` is **not** set.
    Production,
    /// A test build: `cfg(test)` is set, as `cargo test` configures it.
    Test,
}

/// Why a compilation failed, classified from the compiler's own diagnostic.
///
/// Classification exists because "nonzero exit" is not evidence. A fixture that
/// fails because a file is missing, or because of an unrelated syntax slip,
/// proves nothing about ownership — and would silently look like success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Diagnosis {
    /// Compiled cleanly.
    Success,
    /// `E0119` — a second registration for the same owner token.
    OwnershipCoherence,
    /// `E0433`/`E0425` — the canonical owner or its access path is absent.
    MissingCanonicalOwner,
    /// `E0603` — the owner's trait or token is private, so the item naming it
    /// is outside the owner module. Seals ownership by visibility rather than
    /// by coherence.
    OwnerPrivacy,
    /// The authored `compile_error!` in the generated data tests fired.
    TestModuleTripwire,
    /// A lexical or syntactic error. Never an ownership result.
    RustSyntax,
    /// A module file could not be read. Always a broken fixture.
    MissingFixtureFile,
    /// Anything else — always a fixture defect, never accepted as evidence.
    Other,
}

/// One compilation's verdict.
#[derive(Debug)]
struct Outcome {
    succeeded: bool,
    diagnosis: Diagnosis,
    primary: String,
}

impl Outcome {
    /// Panics with the full compiler output unless the verdict is exactly this.
    fn expect(&self, succeeded: bool, diagnosis: Diagnosis, cell: &str) {
        assert_eq!(
            self.succeeded, succeeded,
            "{cell}: expected succeeded={succeeded}, got {}\nprimary: {}",
            self.succeeded, self.primary
        );
        assert_eq!(
            self.diagnosis, diagnosis,
            "{cell}: expected {diagnosis:?}, got {:?}\nprimary: {}",
            self.diagnosis, self.primary
        );
    }
}

/// A disposable fixture crate in its own temporary directory.
///
/// Nothing is written inside the repository: the approved workspace policy
/// permits no `.rs` file outside `crates/frontend-analysis-core/src`, and a
/// committed fixture would also add a second compilation target.
struct Fixture {
    directory: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

impl Fixture {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.subsec_nanos())
            .unwrap_or_default();
        let directory = std::env::temp_dir().join(format!(
            "frontend-analysis-seal-{name}-{}-{unique}-{nanos}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("fixture directory");
        Self { directory }
    }

    fn write(&self, file: &str, contents: &str) -> &Self {
        fs::write(self.directory.join(file), contents).expect("fixture source");
        self
    }

    /// Compiles `root` with the pinned compiler and classifies the verdict.
    ///
    /// `--emit=metadata` stops before code generation and linking: it is
    /// faster, and it keeps every fixture free of a `main` symbol, removing one
    /// of the accidental failure causes this gate must not confuse for
    /// ownership.
    fn compile(&self, root: &str, mode: BuildMode) -> Outcome {
        let mut command = Command::new(rustc_program());
        command
            .current_dir(&self.directory)
            .arg("--edition")
            .arg("2024")
            .arg("--emit=metadata")
            .arg("-o")
            .arg(self.directory.join("fixture.meta"));
        match mode {
            BuildMode::Production => {
                command.arg("--crate-type").arg("lib");
            }
            BuildMode::Test => {
                command.arg("--test");
            }
        }
        let output = command
            .arg(root)
            .output()
            .expect("the pinned Rust compiler must be invocable");
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let succeeded = output.status.success();
        let primary = primary_diagnostic(&stderr);
        Outcome {
            succeeded,
            diagnosis: if succeeded {
                Diagnosis::Success
            } else {
                classify(&primary)
            },
            primary,
        }
    }
}

/// The compiler to drive: whatever Cargo selected, else the `rustup` shim,
/// which resolves `rust-toolchain.toml` from the working directory.
fn rustc_program() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"))
}

/// The first `error` line of a compiler run, which is the failure's cause.
fn primary_diagnostic(stderr: &str) -> String {
    stderr
        .lines()
        .find(|line| line.starts_with("error"))
        .unwrap_or("")
        .trim()
        .to_owned()
}

/// Maps a compiler diagnostic to the reason it represents.
///
/// Keyed on `rustc`'s stable error codes wherever one exists, so the
/// classification tracks the compiler's own taxonomy rather than prose.
fn classify(primary: &str) -> Diagnosis {
    if primary.starts_with("error[E0119]") {
        return Diagnosis::OwnershipCoherence;
    }
    if primary.starts_with("error[E0433]") || primary.starts_with("error[E0425]") {
        return Diagnosis::MissingCanonicalOwner;
    }
    if primary.starts_with("error[E0603]") {
        return Diagnosis::OwnerPrivacy;
    }
    if primary.starts_with("error[E0583]") || primary.contains("couldn't read") {
        return Diagnosis::MissingFixtureFile;
    }
    if primary.contains(TRIPWIRE_MESSAGE) {
        return Diagnosis::TestModuleTripwire;
    }
    if primary.contains("escape")
        || primary.contains("unterminated")
        || primary.starts_with("error: expected")
    {
        return Diagnosis::RustSyntax;
    }
    Diagnosis::Other
}

// ---------------------------------------------------------------------------
// Fixture source
// ---------------------------------------------------------------------------

/// The private canonical owner: trait, token, the generated registration, and
/// the wrapper that is the only consumer-visible access path.
///
/// `gate` optionally places a `cfg` attribute on the whole owner module, which
/// is how a "canonical owner removed by configuration" defect is modelled.
fn owner_module(gate: Option<&str>) -> String {
    owner_module_with_visibility(gate, "pub(crate)")
}

/// The canonical owner at a chosen visibility for its trait and token.
///
/// Visibility is a parameter because it turns out to select *which* compiler
/// rule seals ownership — see the encoding-selection tests below.
fn owner_module_with_visibility(gate: Option<&str>, visibility: &str) -> String {
    let mut text = String::new();
    if let Some(predicate) = gate {
        let _ = writeln!(text, "#[cfg({predicate})]");
    }
    let space = if visibility.is_empty() { "" } else { " " };
    let _ = write!(
        text,
        r#"mod named_data_owner {{
    {visibility}{space}trait OwnershipRegistration {{
        const ROWS: &'static [(&'static str, &'static str)];
    }}
    {visibility}{space}struct OwnerToken;

    impl OwnershipRegistration for OwnerToken {{
        const ROWS: &'static [(&'static str, &'static str)] = &[("amp;", "&"), ("lt;", "<")];
    }}

    pub(crate) struct NamedData;
    impl NamedData {{
        pub(crate) fn rows() -> &'static [(&'static str, &'static str)] {{
            <OwnerToken as OwnershipRegistration>::ROWS
        }}
    }}
}}
"#
    );
    text
}

/// A consumer that reaches past the wrapper to the registration directly.
const BYPASSING_CONSUMER: &str = r#"
pub fn bypassing_consumer() -> &'static str {
    <named_data_owner::OwnerToken as named_data_owner::OwnershipRegistration>::ROWS[0].0
}
"#;

/// The consumer, which reaches the data only through the canonical wrapper.
const CONSUMER: &str = r#"
pub fn consumer_first_row() -> &'static str {
    named_data_owner::NamedData::rows()[0].0
}
"#;

/// The generated artifact as a separately loadable file: a registration of the
/// canonical trait for the canonical token, and nothing else.
///
/// `token` is a parameter only so mutation M2 can retarget it.
fn generated_registration(token: &str) -> String {
    format!(
        r#"use crate::named_data_owner::{{OwnershipRegistration, {token}}};
impl OwnershipRegistration for {token} {{
    const ROWS: &'static [(&'static str, &'static str)] = &[("amp;", "&"), ("lt;", "<")];
}}
"#
    )
}

/// The generated Rust data tests, carrying the compiler-owned tripwire.
fn generated_data_tests(with_tripwire: bool) -> String {
    let mut text = String::new();
    if with_tripwire {
        let _ = writeln!(text, "#[cfg(not(test))]");
        let _ = writeln!(text, "compile_error!(\"{TRIPWIRE_MESSAGE}\");");
    }
    text.push_str(
        r#"
#[cfg(test)]
mod generated_rows {
    #[test]
    fn the_generated_rows_are_present() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    );
    text
}

/// A canonical crate plus one extra alias declaration under test.
fn canonical_with_alias(alias: &str) -> String {
    format!("{}{CONSUMER}{alias}\n", owner_module(None))
}

/// Writes a canonical fixture whose alias loads the generated registration.
fn canonical_alias_fixture(name: &str, alias: &str) -> Fixture {
    let fixture = Fixture::new(name);
    fixture
        .write(
            "named_registration_generated.rs",
            &generated_registration("OwnerToken"),
        )
        .write("fixture.rs", &canonical_with_alias(alias));
    fixture
}

// ---------------------------------------------------------------------------
// Toolchain evidence
// ---------------------------------------------------------------------------

#[test]
fn the_validated_compiler_is_the_repository_pinned_toolchain() {
    let output = Command::new(rustc_program())
        .arg("--version")
        .output()
        .expect("the pinned Rust compiler must be invocable");
    let version = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        version.contains(ACCEPTED_RUSTC_VERSION),
        "this evidence is only valid against Rust {ACCEPTED_RUSTC_VERSION}; got {version:?}"
    );
}

// ---------------------------------------------------------------------------
// CELL 01-14 — mechanical ownership cells
// ---------------------------------------------------------------------------

#[test]
fn cell01_canonical_owner_exactly_once_compiles() {
    let fixture = Fixture::new("cell01");
    fixture.write("fixture.rs", &format!("{}{CONSUMER}", owner_module(None)));
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        Diagnosis::Success,
        "CELL 01",
    );
}

#[test]
fn cell02_canonical_owner_absent_breaks_the_consumer() {
    let fixture = Fixture::new("cell02");
    fixture.write("fixture.rs", CONSUMER);
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::MissingCanonicalOwner,
        "CELL 02",
    );
}

#[test]
fn cell03_a_generated_alias_cannot_substitute_for_the_canonical_owner() {
    // The generated artifact is present and loaded, under its own module. The
    // canonical owner is not. This is the theorem's sharpest edge: artifact
    // presence is not owner presence, so the consumer must still fail.
    let fixture = Fixture::new("cell03");
    fixture.write(
        "fixture.rs",
        &format!(
            r#"mod generated_alias {{
    pub(crate) trait OwnershipRegistration {{
        const ROWS: &'static [(&'static str, &'static str)];
    }}
    pub(crate) struct OwnerToken;
    impl OwnershipRegistration for OwnerToken {{
        const ROWS: &'static [(&'static str, &'static str)] = &[("amp;", "&")];
    }}
}}
{CONSUMER}"#
        ),
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::MissingCanonicalOwner,
        "CELL 03",
    );
}

#[test]
fn cell04_a_direct_duplicate_registration_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell04",
        "mod duplicate_direct { include!(\"named_registration_generated.rs\"); }",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 04",
    );
}

#[test]
fn cell05_a_nested_module_duplicate_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell05",
        r#"mod outer_alias {
    pub(crate) mod inner_alias {
        include!("named_registration_generated.rs");
    }
}"#,
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 05",
    );
}

#[test]
fn cell06_a_path_attribute_duplicate_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell06",
        "#[path = \"named_registration_generated.rs\"]\nmod duplicate_via_path;",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 06",
    );
}

#[test]
fn cell07_an_include_duplicate_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell07",
        "mod duplicate_via_include {\n    include!(\"named_registration_generated.rs\");\n}",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 07",
    );
}

#[test]
fn cell08_an_include_of_concat_duplicate_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell08",
        "mod duplicate_via_concat {\n    include!(concat!(\"named_registration\", \"_generated.rs\"));\n}",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 08",
    );
}

#[test]
fn cell09_a_raw_string_path_duplicate_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell09",
        "#[path = r\"named_registration_generated.rs\"]\nmod duplicate_via_raw_string;",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 09",
    );
}

#[test]
fn cell10_escaped_string_path_duplicates_are_coherence_conflicts() {
    // `\x5f` and `\u{5f}` are both `_`. The scanner strategy had to decode these
    // by hand and got it wrong twice (R9); the compiler simply knows.
    for (label, alias) in [
        (
            "CELL 10 (\\x5f)",
            r#"#[path = "named_registration\x5fgenerated.rs"]
mod duplicate_via_hex_escape;"#,
        ),
        (
            "CELL 10 (\\u{5f})",
            r#"#[path = "named_registration\u{5f}generated.rs"]
mod duplicate_via_unicode_escape;"#,
        ),
    ] {
        let fixture = canonical_alias_fixture("cell10", alias);
        fixture.compile("fixture.rs", BuildMode::Production).expect(
            false,
            Diagnosis::OwnershipCoherence,
            label,
        );
    }
}

#[test]
fn cell11_an_active_cfg_attr_duplicate_is_a_coherence_conflict() {
    let fixture = canonical_alias_fixture(
        "cell11",
        "#[cfg_attr(all(), path = \"named_registration_generated.rs\")]\nmod duplicate_via_cfg_attr;",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "CELL 11",
    );
}

#[test]
fn cell12_an_inactive_cfg_duplicate_is_not_a_second_owner() {
    // The compiler decides which registrations are active. An unreachable one
    // is not a duplicate, and must not be reported as one.
    let fixture = canonical_alias_fixture(
        "cell12",
        "#[cfg(any())]\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_inactive;",
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        Diagnosis::Success,
        "CELL 12",
    );
}

#[test]
fn cell13_a_canonical_owner_gated_out_of_production_breaks_the_consumer() {
    // The consumer is deliberately unconditional. Gating it too would let the
    // fixture compile vacuously and prove nothing.
    let fixture = Fixture::new("cell13");
    fixture.write(
        "fixture.rs",
        &format!("{}{CONSUMER}", owner_module(Some("test"))),
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::MissingCanonicalOwner,
        "CELL 13",
    );
}

#[test]
fn cell14_the_same_owner_remains_valid_under_a_test_build() {
    let fixture = Fixture::new("cell14");
    fixture.write(
        "fixture.rs",
        &format!("{}{CONSUMER}", owner_module(Some("test"))),
    );
    fixture
        .compile("fixture.rs", BuildMode::Test)
        .expect(true, Diagnosis::Success, "CELL 14");
}

// ---------------------------------------------------------------------------
// R6-R12 — the rejected scanner's findings, closed by compiler semantics
// ---------------------------------------------------------------------------

#[test]
fn r10_character_literals_and_lifetimes_do_not_change_the_ownership_result() {
    // R10 desynchronized a hand-written scanner. Syntax that is merely nearby
    // must not move the compiler's verdict in either direction.
    let noise = r#"
const BRACKET: char = ']';
const QUOTE: char = '"';
const ESCAPED: char = '\x5d';
fn borrow<'a>(value: &'a str) -> &'a str { value }
fn labelled() { 'outer: loop { break 'outer; } }
"#;
    let with_duplicate = canonical_alias_fixture(
        "r10-duplicate",
        &format!(
            "{noise}\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_after_noise;"
        ),
    );
    with_duplicate
        .compile("fixture.rs", BuildMode::Production)
        .expect(false, Diagnosis::OwnershipCoherence, "R10 with duplicate");

    let without_duplicate = Fixture::new("r10-clean");
    without_duplicate.write(
        "fixture.rs",
        &format!("{}{CONSUMER}{noise}", owner_module(None)),
    );
    without_duplicate
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "R10 without duplicate");
}

#[test]
fn r11_a_raw_byte_string_does_not_hide_a_duplicate_alias() {
    let raw_byte_string =
        "const MASK: &[u8] = br#\"x\" y\"#;\npub fn mask_len() -> usize { MASK.len() }\n";

    // Control: the raw byte string alone must compile, so a failure below can
    // only be the duplicate alias.
    let control = Fixture::new("r11-control");
    control.write(
        "fixture.rs",
        &format!("{}{CONSUMER}{raw_byte_string}", owner_module(None)),
    );
    control.compile("fixture.rs", BuildMode::Production).expect(
        true,
        Diagnosis::Success,
        "R11 control",
    );

    let fixture = canonical_alias_fixture(
        "r11",
        &format!(
            "{raw_byte_string}#[path = \"named_registration_generated.rs\"]\nmod duplicate_after_raw_byte_string;"
        ),
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnershipCoherence,
        "R11 duplicate",
    );
}

#[test]
fn r12_a_malformed_escape_is_a_syntax_failure_and_never_an_ownership_result() {
    // The rejected scanner treated an unparseable file as clean. The compiler
    // rejects it, and this gate classifies it as syntax — never as integrity
    // success, and never as an ownership conflict.
    let fixture = Fixture::new("r12");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{CONSUMER}pub fn malformed() {{ let _value = '\\q'; }}\n",
            owner_module(None)
        ),
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    outcome.expect(false, Diagnosis::RustSyntax, "R12");
    assert_ne!(
        outcome.diagnosis,
        Diagnosis::OwnershipCoherence,
        "a syntax failure must never be reported as an ownership result"
    );
}

// ---------------------------------------------------------------------------
// T01-T04 — generated-data test-module tripwire
// ---------------------------------------------------------------------------

#[test]
fn t01_the_generated_data_tests_compile_under_a_test_build() {
    let fixture = Fixture::new("t01");
    fixture
        .write("generated_data_tests.rs", &generated_data_tests(true))
        .write(
            "fixture.rs",
            "#[cfg(test)]\n#[path = \"generated_data_tests.rs\"]\nmod generated_data_tests;\npub fn library_symbol() -> u32 { 7 }\n",
        );
    fixture
        .compile("fixture.rs", BuildMode::Test)
        .expect(true, Diagnosis::Success, "T01");
}

#[test]
fn t02_direct_production_exposure_trips_the_compile_error() {
    let fixture = Fixture::new("t02");
    fixture
        .write("generated_data_tests.rs", &generated_data_tests(true))
        .write(
            "fixture.rs",
            "#[path = \"generated_data_tests.rs\"]\nmod generated_data_tests;\npub fn library_symbol() -> u32 { 7 }\n",
        );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::TestModuleTripwire,
        "T02",
    );
}

#[test]
fn t03_production_exposure_through_an_alias_still_trips_the_compile_error() {
    // The tripwire is file-local, so it follows the source through whatever
    // loading form reaches it.
    let fixture = Fixture::new("t03");
    fixture
        .write("generated_data_tests.rs", &generated_data_tests(true))
        .write(
            "fixture.rs",
            "mod exposed_alias { include!(\"generated_data_tests.rs\"); }\npub fn library_symbol() -> u32 { 7 }\n",
        );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::TestModuleTripwire,
        "T03",
    );
}

#[test]
fn t04_a_genuinely_inactive_test_only_branch_compiles_in_production() {
    let fixture = Fixture::new("t04");
    fixture
        .write("generated_data_tests.rs", &generated_data_tests(true))
        .write(
            "fixture.rs",
            "#[cfg(test)]\n#[path = \"generated_data_tests.rs\"]\nmod generated_data_tests;\npub fn library_symbol() -> u32 { 7 }\n",
        );
    fixture
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "T04");
}

// ---------------------------------------------------------------------------
// E01-E04 — which visibility the production encoding must use
// ---------------------------------------------------------------------------
//
// Governance selected the shape (owner + registration + wrapper + tripwire) but
// not the exact Rust spelling. M3 shows why the spelling matters: coherence
// seals how many registrations exist, but it says nothing about who may read
// them. These four cells settle the remaining choice mechanically.

/// The generated registration as it appears *inside* the owner module, where it
/// needs no import because it is already in scope.
const REGISTRATION_INSIDE_OWNER: &str = r#"impl OwnershipRegistration for OwnerToken {
    const ROWS: &'static [(&'static str, &'static str)] = &[("amp;", "&"), ("lt;", "<")];
}
"#;

/// The canonical owner with its trait and token private, and the generated
/// registration included inside it.
fn private_owner_with_included_registration(extra_inside: &str) -> String {
    format!(
        r#"mod named_data_owner {{
    trait OwnershipRegistration {{
        const ROWS: &'static [(&'static str, &'static str)];
    }}
    struct OwnerToken;
    include!("registration_inside_owner.rs");
{extra_inside}
    pub(crate) struct NamedData;
    impl NamedData {{
        pub(crate) fn rows() -> &'static [(&'static str, &'static str)] {{
            <OwnerToken as OwnershipRegistration>::ROWS
        }}
    }}
}}
"#
    )
}

#[test]
fn e01_a_pub_crate_owner_does_not_seal_wrapper_only_access() {
    // Recorded as a limitation, not a success: with a crate-visible trait and
    // token, any module in the crate can read the registration directly, so
    // "consumers go through the wrapper" would be convention, not enforcement.
    let fixture = Fixture::new("e01");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{BYPASSING_CONSUMER}",
            owner_module_with_visibility(None, "pub(crate)")
        ),
    );
    fixture
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "E01");
}

#[test]
fn e02_a_private_owner_seals_wrapper_only_access() {
    let fixture = Fixture::new("e02");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{BYPASSING_CONSUMER}",
            owner_module_with_visibility(None, "")
        ),
    );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnerPrivacy,
        "E02",
    );

    // The wrapper path through the same private owner still compiles, so the
    // seal rejects the bypass specifically rather than breaking the design.
    let permitted = Fixture::new("e02-wrapper");
    permitted.write(
        "fixture.rs",
        &format!("{}{CONSUMER}", owner_module_with_visibility(None, "")),
    );
    permitted
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "E02 wrapper path");
}

#[test]
fn e03_a_private_owner_rejects_an_external_registration_by_privacy() {
    // An alias outside the owner module cannot even name the trait, so it can
    // never register. Ownership is still sealed — by a different compiler rule
    // than coherence, which the production prerequisite must record.
    let fixture = Fixture::new("e03");
    fixture
        .write(
            "named_registration_generated.rs",
            &generated_registration("OwnerToken"),
        )
        .write(
            "fixture.rs",
            &format!(
                "{}{CONSUMER}#[path = \"named_registration_generated.rs\"]\nmod duplicate_outside;\n",
                owner_module_with_visibility(None, "")
            ),
        );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::OwnerPrivacy,
        "E03",
    );
}

#[test]
fn e04_a_private_owner_rejects_an_internal_duplicate_by_coherence() {
    // Inside the owner module the trait is nameable, so a second registration
    // is reached and rejected as a coherence conflict. Together with E03 this
    // closes both directions: outside cannot register, inside cannot register
    // twice.
    let baseline = Fixture::new("e04-baseline");
    baseline
        .write("registration_inside_owner.rs", REGISTRATION_INSIDE_OWNER)
        .write(
            "fixture.rs",
            &format!("{}{CONSUMER}", private_owner_with_included_registration("")),
        );
    baseline
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "E04 baseline");

    for (label, extra) in [
        (
            "E04 duplicate include",
            "    include!(\"registration_inside_owner.rs\");",
        ),
        (
            "E04 duplicate nested module",
            "    mod duplicate_nested {\n        use super::{OwnershipRegistration, OwnerToken};\n        include!(\"registration_inside_owner.rs\");\n    }",
        ),
    ] {
        let fixture = Fixture::new("e04");
        fixture
            .write("registration_inside_owner.rs", REGISTRATION_INSIDE_OWNER)
            .write(
                "fixture.rs",
                &format!(
                    "{}{CONSUMER}",
                    private_owner_with_included_registration(extra)
                ),
            );
        fixture.compile("fixture.rs", BuildMode::Production).expect(
            false,
            Diagnosis::OwnershipCoherence,
            label,
        );
    }
}

// ---------------------------------------------------------------------------
// M1-M7 — mutation pressure
// ---------------------------------------------------------------------------
//
// Each mutation changes exactly one thing about a fixture and asserts the
// verdict moves as the theorem predicts. If a cell still failed the same way
// after its mechanism was removed, the cell was never testing that mechanism.
//
// The mutations are expressed as fixture variants rather than as temporary
// edits to this file, so nothing is left mutated anywhere and the evidence is
// re-runnable rather than a claim about a step someone once performed.

#[test]
fn m1_removing_the_generated_registration_removes_the_coherence_conflict() {
    let fixture = Fixture::new("m1");
    fixture.write("named_registration_generated.rs", "").write(
        "fixture.rs",
        &canonical_with_alias(
            "#[path = \"named_registration_generated.rs\"]\nmod duplicate_via_path;",
        ),
    );
    fixture
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "M1");
}

#[test]
fn m2_retargeting_the_duplicate_registration_removes_the_coherence_conflict() {
    let fixture = Fixture::new("m2");
    fixture
        .write(
            "named_registration_generated.rs",
            &generated_registration("OtherToken"),
        )
        .write(
            "fixture.rs",
            &format!(
                "{}{CONSUMER}\nmod extra {{ }}\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_via_path;\n",
                owner_module(None).replace(
                    "pub(crate) struct OwnerToken;",
                    "pub(crate) struct OwnerToken;\n    pub(crate) struct OtherToken;",
                )
            ),
        );
    fixture
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "M2");
}

#[test]
fn m3_a_consumer_that_bypasses_the_wrapper_compiles_which_is_why_the_contract_exists() {
    // This mutation is expected to COMPILE. That is the evidence: coherence
    // seals how many registrations exist, but it cannot by itself force a
    // consumer to go through the wrapper. The wrapper-only consumer topology is
    // a real, separate obligation of the production design, not something the
    // compiler supplies for free.
    let fixture = Fixture::new("m3");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}\npub fn bypassing_consumer() -> &'static str {{\n    <named_data_owner::OwnerToken as named_data_owner::OwnershipRegistration>::ROWS[0].0\n}}\n",
            owner_module(None)
        ),
    );
    fixture
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "M3");
}

#[test]
fn m4_removing_the_tripwire_lets_production_exposure_compile() {
    let fixture = Fixture::new("m4");
    fixture
        .write("generated_data_tests.rs", &generated_data_tests(false))
        .write(
            "fixture.rs",
            "#[path = \"generated_data_tests.rs\"]\nmod generated_data_tests;\npub fn library_symbol() -> u32 { 7 }\n",
        );
    fixture
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "M4");
}

#[test]
fn m5_making_the_duplicate_cfg_inactive_restores_exactly_one_owner() {
    let active = canonical_alias_fixture(
        "m5-active",
        "#[cfg(feature_that_is_on)]\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_gated;",
    );
    let mut command = Command::new(rustc_program());
    command
        .current_dir(&active.directory)
        .arg("--edition")
        .arg("2024")
        .arg("--emit=metadata")
        .arg("-o")
        .arg(active.directory.join("fixture.meta"))
        .arg("--crate-type")
        .arg("lib")
        .arg("--cfg")
        .arg("feature_that_is_on")
        .arg("--check-cfg")
        .arg("cfg(feature_that_is_on)")
        .arg("fixture.rs");
    let output = command.output().expect("compiler must be invocable");
    let primary = primary_diagnostic(&String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success(), "M5: gate on must duplicate");
    assert_eq!(
        classify(&primary),
        Diagnosis::OwnershipCoherence,
        "M5: gate on must conflict, got {primary}"
    );

    // The same source with the predicate unset is a single-owner crate.
    let inactive = canonical_alias_fixture(
        "m5-inactive",
        "#[cfg(feature_that_is_on)]\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_gated;",
    );
    inactive
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, Diagnosis::Success, "M5 inactive");
}

#[test]
fn m6_a_raw_byte_string_before_the_duplicate_keeps_the_diagnostic_ownership_related() {
    let fixture = canonical_alias_fixture(
        "m6",
        "const MASK: &[u8] = br#\"x\" y\"#;\npub fn mask_len() -> usize { MASK.len() }\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_after_mask;",
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    outcome.expect(false, Diagnosis::OwnershipCoherence, "M6");
    assert!(
        !outcome.primary.contains("byte string"),
        "M6: the raw byte string must not be the cause, got {}",
        outcome.primary
    );
}

#[test]
fn m7_a_generated_alias_alone_does_not_satisfy_the_owner_seam() {
    // The registration file is present and loaded; only the canonical owner is
    // gone. The consumer must still fail, and for the owner reason.
    let fixture = Fixture::new("m7");
    fixture
        .write(
            "named_registration_generated.rs",
            &generated_registration("OwnerToken"),
        )
        .write(
            "fixture.rs",
            &format!(
                "mod stand_in {{\n    pub(crate) trait OwnershipRegistration {{\n        const ROWS: &'static [(&'static str, &'static str)];\n    }}\n    pub(crate) struct OwnerToken;\n}}\n{CONSUMER}"
            ),
        );
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        Diagnosis::MissingCanonicalOwner,
        "M7",
    );
}
