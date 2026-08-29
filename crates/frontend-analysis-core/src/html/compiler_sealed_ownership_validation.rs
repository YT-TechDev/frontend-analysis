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
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The tripwire text the generated data tests carry. Authored here, so matching
/// it is recognizing our own message rather than guessing a compiler string.
const TRIPWIRE_MESSAGE: &str = "generated Named Character Reference data tests are test-only";

/// The exact toolchain this evidence is only meaningful against.
const ACCEPTED_RUSTC_VERSION: &str = "1.97.1";

/// The three names the ownership theorem is *about*. A diagnostic that does not
/// name the relevant one is not evidence about this theorem, whatever its code.
const OWNER_TRAIT: &str = "OwnershipRegistration";
const OWNER_TOKEN: &str = "OwnerToken";
const OWNER_MODULE: &str = "named_data_owner";

/// The repository directory to resolve the pinned toolchain from.
///
/// Compile-time absolute, so resolution never depends on the process working
/// directory — which is the whole point of [`ResolvedCompiler`].
fn repository_context() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// One concrete compiler executable, resolved once in repository context.
///
/// ## Why this type exists
///
/// The obvious spelling — run `rustc` and let `rustup` sort it out — is wrong
/// here, and was a real defect in the first version of this harness. The
/// `rustup` shim picks a toolchain by walking up from the *working directory*
/// looking for `rust-toolchain.toml`. Fixtures compile from a temporary
/// directory, where that file is not discoverable, so the shim silently falls
/// back to the machine's default toolchain. On the host that first ran this
/// harness that meant the version probe saw 1.97.1 (probed from the repository)
/// while every fixture was compiled by 1.94.1 (the ambient default).
///
/// Same command spelling is not the same compiler. So the identity is resolved
/// to a concrete absolute executable *before* any fixture directory is entered,
/// and that one executable is used for version verification and for every
/// fixture. A concrete binary performs no toolchain lookup at all, so no
/// working directory, `RUSTUP_TOOLCHAIN` value, or ambient default can redirect
/// it.
#[derive(Debug, Clone)]
struct ResolvedCompiler {
    executable: PathBuf,
}

impl ResolvedCompiler {
    /// Resolves the repository-pinned compiler and proves it is the accepted
    /// version. Panics rather than returning a compiler it cannot vouch for.
    fn repository_pinned() -> Self {
        let resolved = Self::resolve();
        if let Err(reason) = resolved.verify_accepted_version() {
            panic!("the repository-pinned compiler could not be established: {reason}");
        }
        resolved
    }

    /// Resolves without verifying, so the falsification cells can inspect the
    /// resolution and the verification separately.
    fn resolve() -> Self {
        let launcher = std::env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"));
        let sysroot = Command::new(&launcher)
            .current_dir(repository_context())
            .arg("--print")
            .arg("sysroot")
            .output()
            .expect("a Rust compiler must be invocable from the repository");
        let sysroot = String::from_utf8_lossy(&sysroot.stdout).trim().to_owned();
        let concrete = Path::new(&sysroot).join("bin").join("rustc");
        Self {
            // A non-`rustup` environment may have no sysroot-local binary; the
            // launcher is then the only candidate, and verification still
            // decides whether it is acceptable.
            executable: if concrete.is_file() {
                concrete
            } else {
                PathBuf::from(launcher)
            },
        }
    }

    /// A compiler invocation that carries this exact identity.
    ///
    /// `RUSTUP_TOOLCHAIN` is removed deliberately. Cargo sets it in the test
    /// process, which is what made the first version of this harness *look*
    /// correct: a bare `rustc` inherited that variable and so happened to be
    /// the pinned compiler. Nothing in the harness bound it, and clearing the
    /// variable dropped the same code to the machine default. A concrete
    /// executable needs no such variable, so removing it turns "pinned by
    /// luck" into "pinned by construction" — and lets the falsification cells
    /// prove the difference.
    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.env_remove("RUSTUP_TOOLCHAIN");
        command
    }

    /// This compiler's own version string, asked from `directory`.
    ///
    /// The directory is a parameter so the falsification cells can ask the same
    /// question from inside a fixture directory and prove the answer does not
    /// change.
    fn version_from(&self, directory: &Path) -> Result<String, String> {
        let output = self
            .command()
            .current_dir(directory)
            .arg("--version")
            .output()
            .map_err(|error| format!("{} is not invocable: {error}", self.executable.display()))?;
        if !output.status.success() {
            return Err(format!(
                "{} did not report a version",
                self.executable.display()
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    /// `Ok` only when this exact executable is the accepted stable compiler.
    fn verify_accepted_version(&self) -> Result<(), String> {
        let version = self.version_from(repository_context())?;
        if version.contains(ACCEPTED_RUSTC_VERSION) {
            Ok(())
        } else {
            Err(format!(
                "{} reports {version:?}, which is not Rust {ACCEPTED_RUSTC_VERSION}",
                self.executable.display()
            ))
        }
    }
}

/// One compiler error diagnostic, as `--error-format=short` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompilerDiagnostic {
    /// The stable error code, when the diagnostic carries one.
    code: Option<String>,
    /// The diagnostic text after the code.
    message: String,
}

/// The evidence a cell requires from the compiler.
///
/// This is an *expectation*, not a classification of whatever happened. The
/// difference matters: classifying "the first error line" by code alone accepts
/// any `E0119` as an ownership conflict, including one between two types this
/// theorem has never heard of. Each variant below therefore binds a stable
/// error code *and* the identity the theorem is about.
///
/// Anything a variant does not match — a missing fixture file, an unrelated
/// privacy slip, a stray syntax error, a second unexpected error beside the
/// intended one — fails closed, because acceptance requires that *every* error
/// the compiler emitted matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedEvidence {
    /// Compiles cleanly, with no error diagnostics at all.
    Success,
    /// `E0119` naming both the owner trait and the owner token.
    OwnershipCoherence,
    /// `E0433`/`E0425` naming the canonical owner module specifically.
    MissingCanonicalOwner,
    /// `E0603` naming the owner trait or the owner token specifically.
    OwnerPrivacy,
    /// The exact authored `compile_error!` text, not source that quotes it.
    TestModuleTripwire,
    /// The dedicated malformed-Rust cell. Never an ownership result.
    RustSyntax,
}

impl ExpectedEvidence {
    /// Whether one diagnostic is evidence of this expectation.
    fn matches(self, diagnostic: &CompilerDiagnostic) -> bool {
        let code = diagnostic.code.as_deref();
        let message = diagnostic.message.as_str();
        match self {
            Self::Success => false,
            Self::OwnershipCoherence => {
                code == Some("E0119")
                    && message.contains(OWNER_TRAIT)
                    && message.contains(OWNER_TOKEN)
            }
            Self::MissingCanonicalOwner => {
                matches!(code, Some("E0433") | Some("E0425")) && message.contains(OWNER_MODULE)
            }
            Self::OwnerPrivacy => {
                code == Some("E0603")
                    && (message.contains(OWNER_TRAIT) || message.contains(OWNER_TOKEN))
            }
            Self::TestModuleTripwire => code.is_none() && message == TRIPWIRE_MESSAGE,
            Self::RustSyntax => code.is_none() && message.starts_with("unknown character escape"),
        }
    }
}

/// One compilation's verdict: whether it succeeded, and every error it emitted.
#[derive(Debug)]
struct Outcome {
    succeeded: bool,
    diagnostics: Vec<CompilerDiagnostic>,
    stderr: String,
}

impl Outcome {
    /// Whether this outcome is acceptable evidence for `expected`.
    ///
    /// Requires the intended diagnostic to be present **and** every error the
    /// compiler emitted to match it. An unrelated second error therefore fails
    /// closed rather than hiding behind the expected first one.
    fn satisfies(&self, expected: ExpectedEvidence) -> bool {
        if expected == ExpectedEvidence::Success {
            return self.succeeded && self.diagnostics.is_empty();
        }
        !self.succeeded
            && !self.diagnostics.is_empty()
            && self
                .diagnostics
                .iter()
                .all(|diagnostic| expected.matches(diagnostic))
    }

    /// Panics with the full compiler output unless the verdict is exactly this.
    fn expect(&self, succeeded: bool, expected: ExpectedEvidence, cell: &str) {
        assert_eq!(
            self.succeeded, succeeded,
            "{cell}: expected succeeded={succeeded}, got {}\n{}",
            self.succeeded, self.stderr
        );
        assert!(
            self.satisfies(expected),
            "{cell}: compiler output is not {expected:?} evidence.\n\
             diagnostics: {:#?}\nraw stderr:\n{}",
            self.diagnostics,
            self.stderr
        );
    }
}

/// Every error diagnostic in one `--error-format=short` compiler run.
///
/// Short format prints one line per diagnostic as
/// `<file>:<line>:<column>: error[<code>]: <message>`, which is why this
/// harness selects it: it exposes *all* errors in a bounded, line-oriented
/// shape, with the offending symbol names inside the message. The trailing
/// `error: aborting due to N previous errors` summary has no `<file>:` prefix,
/// so it carries no `": error"` and is skipped without a special case.
///
/// This reads compiler output. It does not read Rust source.
fn parse_diagnostics(stderr: &str) -> Vec<CompilerDiagnostic> {
    stderr.lines().filter_map(parse_diagnostic).collect()
}

fn parse_diagnostic(line: &str) -> Option<CompilerDiagnostic> {
    let after_location = &line[line.find(": error")? + ": error".len()..];
    let (code, remainder) = match after_location.strip_prefix('[') {
        Some(coded) => {
            let end = coded.find(']')?;
            (Some(coded[..end].to_owned()), &coded[end + 1..])
        }
        None => (None, after_location),
    };
    Some(CompilerDiagnostic {
        code,
        message: remainder.strip_prefix(": ")?.trim().to_owned(),
    })
}

/// A disposable fixture crate in its own temporary directory.
///
/// Nothing is written inside the repository: the approved workspace policy
/// permits no `.rs` file outside `crates/frontend-analysis-core/src`, and a
/// committed fixture would also add a second compilation target.
struct Fixture {
    directory: PathBuf,
    compiler: ResolvedCompiler,
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
        Self {
            directory,
            // Resolved and verified before this fixture's directory is ever
            // entered, so the identity cannot be redirected by that directory.
            compiler: ResolvedCompiler::repository_pinned(),
        }
    }

    fn write(&self, file: &str, contents: &str) -> &Self {
        fs::write(self.directory.join(file), contents).expect("fixture source");
        self
    }

    /// Compiles `root` with this fixture's resolved compiler.
    ///
    /// `--emit=metadata` stops before code generation and linking: it is
    /// faster, and it keeps every fixture free of a `main` symbol, removing one
    /// of the accidental failure causes this gate must not confuse for
    /// ownership.
    fn compile(&self, root: &str, mode: BuildMode) -> Outcome {
        self.compile_with(root, mode, &[])
    }

    fn compile_with(&self, root: &str, mode: BuildMode, extra: &[&str]) -> Outcome {
        let mut command = self.compiler.command();
        command
            .current_dir(&self.directory)
            .arg("--edition")
            .arg("2024")
            .arg("--emit=metadata")
            .arg("--error-format=short")
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
        for argument in extra {
            command.arg(argument);
        }
        let output = command
            .arg(root)
            .output()
            .expect("the resolved Rust compiler must be invocable");
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Outcome {
            succeeded: output.status.success(),
            diagnostics: parse_diagnostics(&stderr),
            stderr,
        }
    }
}

/// How a fixture crate is configured for a compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildMode {
    /// An ordinary library build: `cfg(test)` is **not** set.
    Production,
    /// A test build: `cfg(test)` is set, as `cargo test` configures it.
    Test,
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
fn f1a_the_resolved_compiler_identity_is_independent_of_fixture_cwd() {
    // The identity must answer the same from the repository and from a fixture
    // directory. A `rustup` shim does not: it re-picks a toolchain per working
    // directory, which is exactly how the first version of this harness probed
    // one compiler and compiled fixtures with another.
    let fixture = Fixture::new("f1a");
    let compiler = &fixture.compiler;
    let from_repository = compiler
        .version_from(repository_context())
        .expect("resolved compiler reports a version in the repository");
    let from_fixture = compiler
        .version_from(&fixture.directory)
        .expect("resolved compiler reports a version in a fixture directory");
    assert_eq!(
        from_repository, from_fixture,
        "the resolved compiler identity changed with the working directory"
    );
    assert!(
        from_fixture.contains(ACCEPTED_RUSTC_VERSION),
        "fixtures would be compiled by {from_fixture:?}, not Rust {ACCEPTED_RUSTC_VERSION}"
    );
}

#[test]
fn f1b_version_verification_uses_the_same_identity_that_compiles_fixtures() {
    // Not "two calls to `rustc --version`": the executable path asserted here is
    // the exact path `Fixture::compile` invokes, because both read the same
    // resolved value.
    let fixture = Fixture::new("f1b");
    let resolved = ResolvedCompiler::repository_pinned();
    assert_eq!(
        fixture.compiler.executable, resolved.executable,
        "fixtures do not compile with the verified compiler"
    );
    assert!(fixture.compiler.verify_accepted_version().is_ok());

    // And that identity really is what runs: a compilation from the fixture
    // directory succeeds with it.
    fixture.write("fixture.rs", "pub fn probe() -> u32 { 7 }\n");
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "F1-B",
    );
}

#[test]
fn f1c_a_compiler_reporting_a_wrong_version_cannot_produce_accepted_evidence() {
    // Simulated with a local wrapper rather than by touching the machine's
    // rustup configuration, so the test is portable and leaves no global state.
    let fixture = Fixture::new("f1c");
    let Some(impostor) = wrapper_reporting_version(&fixture.directory, "rustc 1.0.0 (deadbeef)")
    else {
        return;
    };
    let wrong = ResolvedCompiler {
        executable: impostor,
    };
    let verdict = wrong.verify_accepted_version();
    assert!(
        verdict.is_err(),
        "a non-{ACCEPTED_RUSTC_VERSION} compiler was accepted: {verdict:?}"
    );
}

#[test]
fn f1d_a_nonexistent_compiler_identity_fails_closed() {
    let missing = ResolvedCompiler {
        executable: PathBuf::from("/nonexistent/frontend-analysis/rustc"),
    };
    assert!(missing.version_from(repository_context()).is_err());
    assert!(missing.verify_accepted_version().is_err());
}

#[test]
fn f1e_a_fixture_directory_cannot_redirect_the_harness_to_an_ambient_compiler() {
    // The threat is concrete: from a temporary directory the repository's
    // `rust-toolchain.toml` is not discoverable, so a bare `rustc` may resolve
    // to the machine default. This records whatever the ambient spelling does
    // resolve to there, and requires the harness's own identity to be pinned
    // regardless of whether the two agree on this host.
    let fixture = Fixture::new("f1e");

    // The ambient spelling, asked the way a fixture would ask it: from the
    // fixture directory, without the toolchain variable Cargo happens to set.
    // On a host whose default toolchain differs from the pin, this is a
    // *different compiler* — which is exactly the trap being closed.
    let ambient = Command::new("rustc")
        .current_dir(&fixture.directory)
        .env_remove("RUSTUP_TOOLCHAIN")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned());

    let resolved = fixture
        .compiler
        .version_from(&fixture.directory)
        .expect("the resolved compiler reports a version from the fixture directory");
    assert!(
        resolved.contains(ACCEPTED_RUSTC_VERSION),
        "the harness fell back to an ambient compiler: resolved {resolved:?}, \
         while the ambient spelling in the fixture directory reports {ambient:?}"
    );
}

/// Writes an executable that answers `--version` with `version` and refuses
/// everything else. `None` when this platform cannot mark a file executable,
/// in which case the calling cell declines rather than asserting nothing.
fn wrapper_reporting_version(directory: &Path, version: &str) -> Option<PathBuf> {
    let path = directory.join("impostor-rustc");
    fs::write(&path, format!("#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo '{version}'\n  exit 0\nfi\nexit 1\n")).ok()?;
    make_executable(&path)?;
    Some(path)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Option<()> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).ok()
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Option<()> {
    None
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
        ExpectedEvidence::Success,
        "CELL 01",
    );
}

#[test]
fn cell02_canonical_owner_absent_breaks_the_consumer() {
    let fixture = Fixture::new("cell02");
    fixture.write("fixture.rs", CONSUMER);
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        false,
        ExpectedEvidence::MissingCanonicalOwner,
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
        ExpectedEvidence::MissingCanonicalOwner,
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
        ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::OwnershipCoherence,
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
            ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::OwnershipCoherence,
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
        ExpectedEvidence::Success,
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
        ExpectedEvidence::MissingCanonicalOwner,
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
    fixture.compile("fixture.rs", BuildMode::Test).expect(
        true,
        ExpectedEvidence::Success,
        "CELL 14",
    );
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
        .expect(
            false,
            ExpectedEvidence::OwnershipCoherence,
            "R10 with duplicate",
        );

    let without_duplicate = Fixture::new("r10-clean");
    without_duplicate.write(
        "fixture.rs",
        &format!("{}{CONSUMER}{noise}", owner_module(None)),
    );
    without_duplicate
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, ExpectedEvidence::Success, "R10 without duplicate");
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
        ExpectedEvidence::Success,
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
        ExpectedEvidence::OwnershipCoherence,
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
    outcome.expect(false, ExpectedEvidence::RustSyntax, "R12");
    assert!(
        !outcome.satisfies(ExpectedEvidence::OwnershipCoherence),
        "a syntax failure must never be reported as an ownership result: {:#?}",
        outcome.diagnostics
    );
    assert!(
        !outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.is_some()),
        "a lexical failure carries no ownership error code: {:#?}",
        outcome.diagnostics
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
        .expect(true, ExpectedEvidence::Success, "T01");
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
        ExpectedEvidence::TestModuleTripwire,
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
        ExpectedEvidence::TestModuleTripwire,
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
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "T04",
    );
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
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "E01",
    );
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
        ExpectedEvidence::OwnerPrivacy,
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
        .expect(true, ExpectedEvidence::Success, "E02 wrapper path");
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
        ExpectedEvidence::OwnerPrivacy,
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
        .expect(true, ExpectedEvidence::Success, "E04 baseline");

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
            ExpectedEvidence::OwnershipCoherence,
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
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "M1",
    );
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
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "M2",
    );
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
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "M3",
    );
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
    fixture.compile("fixture.rs", BuildMode::Production).expect(
        true,
        ExpectedEvidence::Success,
        "M4",
    );
}

#[test]
fn m5_making_the_duplicate_cfg_inactive_restores_exactly_one_owner() {
    let active = canonical_alias_fixture(
        "m5-active",
        "#[cfg(feature_that_is_on)]\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_gated;",
    );
    active
        .compile_with(
            "fixture.rs",
            BuildMode::Production,
            &[
                "--cfg",
                "feature_that_is_on",
                "--check-cfg",
                "cfg(feature_that_is_on)",
            ],
        )
        .expect(false, ExpectedEvidence::OwnershipCoherence, "M5 active");

    // The same source with the predicate unset is a single-owner crate.
    let inactive = canonical_alias_fixture(
        "m5-inactive",
        "#[cfg(feature_that_is_on)]\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_gated;",
    );
    inactive
        .compile("fixture.rs", BuildMode::Production)
        .expect(true, ExpectedEvidence::Success, "M5 inactive");
}

#[test]
fn m6_a_raw_byte_string_before_the_duplicate_keeps_the_diagnostic_ownership_related() {
    let fixture = canonical_alias_fixture(
        "m6",
        "const MASK: &[u8] = br#\"x\" y\"#;\npub fn mask_len() -> usize { MASK.len() }\n#[path = \"named_registration_generated.rs\"]\nmod duplicate_after_mask;",
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    outcome.expect(false, ExpectedEvidence::OwnershipCoherence, "M6");
    assert!(
        !outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("byte string")),
        "M6: the raw byte string must not be the cause, got {:#?}",
        outcome.diagnostics
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
        ExpectedEvidence::MissingCanonicalOwner,
        "M7",
    );
}

// ---------------------------------------------------------------------------
// F2-A - F2-F — wrong-reason failures the first harness would have accepted
// ---------------------------------------------------------------------------
//
// The first version reduced stderr to its first `error` line and classified it
// by error code alone. Any `E0119` counted as an ownership conflict, any
// `E0433` as a missing owner, any `E0603` as the private-owner boundary, and
// every diagnostic after the first was discarded. These cells are the
// counterexamples: each one fails to compile with the *right code for the wrong
// reason*, and none of them may be accepted as evidence.
//
// They run the real compiler. A classifier unit test alone would prove only
// that the matcher agrees with strings this file made up.

#[test]
fn f2a_an_unrelated_coherence_conflict_is_not_ownership_evidence() {
    let fixture = Fixture::new("f2a");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{CONSUMER}
struct Widget;
trait Render {{
    fn render(&self);
}}
impl Render for Widget {{
    fn render(&self) {{}}
}}
impl Render for Widget {{
    fn render(&self) {{}}
}}
",
            owner_module(None)
        ),
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    assert!(!outcome.succeeded, "F2-A must fail to compile");
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_deref() == Some("E0119")),
        "F2-A must actually produce E0119: {:#?}",
        outcome.diagnostics
    );
    assert!(
        !outcome.satisfies(ExpectedEvidence::OwnershipCoherence),
        "an E0119 between unrelated types was accepted as ownership evidence: {:#?}",
        outcome.diagnostics
    );
}

#[test]
fn f2b_an_unrelated_unresolved_name_is_not_missing_owner_evidence() {
    let fixture = Fixture::new("f2b");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{CONSUMER}pub fn unrelated() {{ let _ = totally_unrelated_symbol(); }}\n",
            owner_module(None)
        ),
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    assert!(!outcome.succeeded, "F2-B must fail to compile");
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic.code.as_deref(), Some("E0433") | Some("E0425"))),
        "F2-B must actually produce E0433/E0425: {:#?}",
        outcome.diagnostics
    );
    assert!(
        !outcome.satisfies(ExpectedEvidence::MissingCanonicalOwner),
        "an unrelated unresolved name was accepted as owner absence: {:#?}",
        outcome.diagnostics
    );
}

#[test]
fn f2c_an_unrelated_privacy_violation_is_not_owner_privacy_evidence() {
    let fixture = Fixture::new("f2c");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{CONSUMER}
mod unrelated_module {{
    struct Hidden;
    impl Hidden {{
        pub(crate) fn value() -> u32 {{
            1
        }}
    }}
}}
pub fn reach() -> u32 {{ unrelated_module::Hidden::value() }}
",
            owner_module(None)
        ),
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    assert!(!outcome.succeeded, "F2-C must fail to compile");
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_deref() == Some("E0603")),
        "F2-C must actually produce E0603: {:#?}",
        outcome.diagnostics
    );
    assert!(
        !outcome.satisfies(ExpectedEvidence::OwnerPrivacy),
        "an unrelated private item was accepted as the owner boundary: {:#?}",
        outcome.diagnostics
    );
}

#[test]
fn f2d_an_intended_conflict_beside_an_unrelated_error_fails_closed() {
    // The intended E0119 really is present. The old harness would have read it
    // first and accepted the fixture, silently discarding a second error that
    // means the fixture is no longer isolating the theorem.
    let fixture = canonical_alias_fixture(
        "f2d",
        "#[path = \"named_registration_generated.rs\"]\nmod duplicate_via_path;\npub fn unrelated() { let _ = totally_unrelated_symbol(); }",
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    assert!(!outcome.succeeded, "F2-D must fail to compile");
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| ExpectedEvidence::OwnershipCoherence.matches(diagnostic)),
        "F2-D must still contain the intended conflict: {:#?}",
        outcome.diagnostics
    );
    assert!(
        outcome.diagnostics.len() >= 2,
        "F2-D must produce more than the intended error: {:#?}",
        outcome.diagnostics
    );
    assert!(
        !outcome.satisfies(ExpectedEvidence::OwnershipCoherence),
        "an unrelated second error was hidden behind the expected first: {:#?}",
        outcome.diagnostics
    );
}

#[test]
fn f2e_error_like_source_text_on_a_successful_build_is_not_a_diagnostic() {
    let fixture = Fixture::new("f2e");
    fixture.write(
        "fixture.rs",
        &format!(
            "{}{CONSUMER}
pub const LOOKS_LIKE_A_DIAGNOSTIC: &str =
    \"error[E0119]: conflicting implementations of trait `OwnershipRegistration` \\
      for type `OwnerToken`\";
// error[E0603]: trait `OwnershipRegistration` is private
",
            owner_module(None)
        ),
    );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    outcome.expect(true, ExpectedEvidence::Success, "F2-E");
    for expected in [
        ExpectedEvidence::OwnershipCoherence,
        ExpectedEvidence::MissingCanonicalOwner,
        ExpectedEvidence::OwnerPrivacy,
        ExpectedEvidence::TestModuleTripwire,
        ExpectedEvidence::RustSyntax,
    ] {
        assert!(
            !outcome.satisfies(expected),
            "a successful build was read as {expected:?} evidence"
        );
    }
}

#[test]
fn f2f_quoted_tripwire_text_without_an_active_compile_error_is_not_the_tripwire() {
    // The generated data tests are exposed to a production build, but with the
    // tripwire removed and its text left behind as ordinary source. Nothing
    // stops the build, so nothing may be read as the tripwire firing.
    let fixture = Fixture::new("f2f");
    fixture
        .write(
            "generated_data_tests.rs",
            &format!(
                "pub(crate) const NOTE: &str = \"{TRIPWIRE_MESSAGE}\";\n// {TRIPWIRE_MESSAGE}\n{}",
                generated_data_tests(false)
            ),
        )
        .write(
            "fixture.rs",
            "#[path = \"generated_data_tests.rs\"]\nmod generated_data_tests;\npub fn library_symbol() -> u32 { 7 }\n",
        );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    outcome.expect(true, ExpectedEvidence::Success, "F2-F");
    assert!(
        !outcome.satisfies(ExpectedEvidence::TestModuleTripwire),
        "quoted tripwire text was read as the tripwire firing"
    );
}

#[test]
fn f2g_a_different_compile_error_that_quotes_the_tripwire_is_not_the_tripwire() {
    // A build that really does fail with a `compile_error!`, whose message
    // merely *contains* the authored tripwire text. Matching by containment
    // would accept this as the generated-data tripwire firing; the tripwire is
    // a specific authored message, so acceptance requires the exact text.
    let fixture = Fixture::new("f2g");
    fixture
        .write(
            "generated_data_tests.rs",
            &format!(
                "#[cfg(not(test))]\ncompile_error!(\"unrelated build policy: {TRIPWIRE_MESSAGE}, and other rules\");\n{}",
                generated_data_tests(false)
            ),
        )
        .write(
            "fixture.rs",
            "#[path = \"generated_data_tests.rs\"]\nmod generated_data_tests;\npub fn library_symbol() -> u32 { 7 }\n",
        );
    let outcome = fixture.compile("fixture.rs", BuildMode::Production);
    assert!(!outcome.succeeded, "F2-G must fail to compile");
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains(TRIPWIRE_MESSAGE)),
        "F2-G must actually quote the tripwire text: {:#?}",
        outcome.diagnostics
    );
    assert!(
        !outcome.satisfies(ExpectedEvidence::TestModuleTripwire),
        "a different compile_error! quoting the tripwire was accepted as the tripwire: {:#?}",
        outcome.diagnostics
    );
}
