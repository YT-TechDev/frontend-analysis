//! Deterministic Test262 selection/effective-source model for Issue #197.
//!
//! Test262 metadata is an evidence-selection input. It never defines the
//! Frontend Analysis language profile or normative coverage universe. Exact
//! source bytes are materialized from the pinned external Git object only
//! after its blob and relevant frontmatter identities have been verified.

use super::model::TEST262_REVISION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NegativePhase {
    Parse,
    Resolution,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NegativeType {
    SyntaxError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Metadata {
    pub(crate) only_strict: bool,
    pub(crate) no_strict: bool,
    pub(crate) raw: bool,
    pub(crate) module: bool,
    pub(crate) non_deterministic: bool,
    pub(crate) host_only: bool,
    pub(crate) proposal_or_post_es2026: bool,
    pub(crate) negative_phase: Option<NegativePhase>,
    pub(crate) negative_type: Option<NegativeType>,
}

impl Metadata {
    pub(crate) const fn ordinary_parse_negative() -> Self {
        Self {
            only_strict: false,
            no_strict: false,
            raw: false,
            module: false,
            non_deterministic: false,
            host_only: false,
            proposal_or_post_es2026: false,
            negative_phase: Some(NegativePhase::Parse),
            negative_type: Some(NegativeType::SyntaxError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrontmatterIdentity<'a> {
    pub(crate) esid: Option<&'a str>,
    pub(crate) features: &'a [&'a str],
    pub(crate) metadata: Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectiveVariant {
    NonStrict,
    Strict,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EffectiveSource {
    pub(crate) variant: EffectiveVariant,
    /// Exact effective source content. Content equality is the authority here;
    /// no platform line-ending normalization or external parser coordinate is
    /// used to derive it.
    pub(crate) source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvidenceInput<'a> {
    pub(crate) revision: &'a str,
    pub(crate) path: &'a str,
    /// Git blob identity established by the external evidence materializer.
    pub(crate) source_blob_sha: &'a str,
    pub(crate) source: &'a str,
    pub(crate) metadata: Metadata,
    pub(crate) normative_mapping: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MaterializedSource<'a> {
    pub(crate) revision: &'a str,
    pub(crate) path: &'a str,
    pub(crate) source_blob_sha: &'a str,
    pub(crate) source: &'a str,
    pub(crate) frontmatter: FrontmatterIdentity<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedEvidence {
    pub(crate) revision: String,
    pub(crate) path: String,
    pub(crate) source_blob_sha: String,
    pub(crate) original_source: String,
    pub(crate) metadata: Metadata,
    pub(crate) normative_mapping: String,
    pub(crate) effective: EffectiveSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExclusionReason {
    WrongRevision,
    PathMismatch,
    OutsideLanguageTests,
    FixtureFile,
    SourceBlobMismatch,
    RelevantFrontmatterMismatch,
    Module,
    NotParseNegative,
    NonDeterministic,
    HostOnly,
    ProposalOrPostEs2026,
    MissingIndependentNormativeMapping,
    InvalidFlagCombination,
    InvalidNegativeMetadata,
}

pub(crate) const STRICT_PREFIX: &str = "\"use strict\";\n";

pub(crate) fn select_evidence(
    input: EvidenceInput<'_>,
) -> Result<Vec<SelectedEvidence>, ExclusionReason> {
    if input.revision != TEST262_REVISION {
        return Err(ExclusionReason::WrongRevision);
    }
    validate_path(input.path)?;

    let normative_mapping = input
        .normative_mapping
        .ok_or(ExclusionReason::MissingIndependentNormativeMapping)?;

    let effective_sources = select_effective_sources(input.source, input.metadata, true)?;

    Ok(effective_sources
        .into_iter()
        .map(|effective| SelectedEvidence {
            revision: input.revision.to_owned(),
            path: input.path.to_owned(),
            source_blob_sha: input.source_blob_sha.to_owned(),
            original_source: input.source.to_owned(),
            metadata: input.metadata,
            normative_mapping: normative_mapping.to_owned(),
            effective,
        })
        .collect())
}

pub(crate) fn select_effective_sources(
    source: &str,
    metadata: Metadata,
    has_independent_normative_mapping: bool,
) -> Result<Vec<EffectiveSource>, ExclusionReason> {
    validate_metadata(metadata)?;

    if metadata.module {
        return Err(ExclusionReason::Module);
    }
    if metadata.non_deterministic {
        return Err(ExclusionReason::NonDeterministic);
    }
    if metadata.host_only {
        return Err(ExclusionReason::HostOnly);
    }
    if metadata.proposal_or_post_es2026 {
        return Err(ExclusionReason::ProposalOrPostEs2026);
    }
    if !has_independent_normative_mapping {
        return Err(ExclusionReason::MissingIndependentNormativeMapping);
    }
    if metadata.negative_phase != Some(NegativePhase::Parse)
        || metadata.negative_type != Some(NegativeType::SyntaxError)
    {
        return Err(ExclusionReason::NotParseNegative);
    }

    if metadata.raw {
        return Ok(vec![EffectiveSource {
            variant: EffectiveVariant::Raw,
            source: source.to_owned(),
        }]);
    }

    if metadata.only_strict {
        return Ok(vec![strict_source(source)]);
    }

    if metadata.no_strict {
        return Ok(vec![EffectiveSource {
            variant: EffectiveVariant::NonStrict,
            source: source.to_owned(),
        }]);
    }

    Ok(vec![
        EffectiveSource {
            variant: EffectiveVariant::NonStrict,
            source: source.to_owned(),
        },
        strict_source(source),
    ])
}

fn validate_path(path: &str) -> Result<(), ExclusionReason> {
    if !path.starts_with("test/language/") {
        return Err(ExclusionReason::OutsideLanguageTests);
    }
    if path.contains("_FIXTURE") {
        return Err(ExclusionReason::FixtureFile);
    }
    Ok(())
}

fn validate_metadata(metadata: Metadata) -> Result<(), ExclusionReason> {
    if metadata.only_strict && metadata.no_strict
        || metadata.raw && (metadata.only_strict || metadata.no_strict)
    {
        return Err(ExclusionReason::InvalidFlagCombination);
    }
    if metadata.negative_phase.is_some() != metadata.negative_type.is_some() {
        return Err(ExclusionReason::InvalidNegativeMetadata);
    }
    Ok(())
}

fn strict_source(source: &str) -> EffectiveSource {
    let mut transformed = String::with_capacity(STRICT_PREFIX.len() + source.len());
    transformed.push_str(STRICT_PREFIX);
    transformed.push_str(source);
    EffectiveSource {
        variant: EffectiveVariant::Strict,
        source: transformed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PinnedCase {
    pub(crate) path: &'static str,
    pub(crate) source_blob_sha: &'static str,
    pub(crate) frontmatter: FrontmatterIdentity<'static>,
    pub(crate) normative_mapping: &'static str,
}

impl PinnedCase {
    pub(crate) fn materialize(
        self,
        materialized: MaterializedSource<'_>,
    ) -> Result<Vec<SelectedEvidence>, ExclusionReason> {
        if materialized.revision != TEST262_REVISION {
            return Err(ExclusionReason::WrongRevision);
        }
        if materialized.path != self.path {
            return Err(ExclusionReason::PathMismatch);
        }
        validate_path(materialized.path)?;
        if materialized.source_blob_sha != self.source_blob_sha {
            return Err(ExclusionReason::SourceBlobMismatch);
        }
        if materialized.frontmatter != self.frontmatter {
            return Err(ExclusionReason::RelevantFrontmatterMismatch);
        }

        select_evidence(EvidenceInput {
            revision: materialized.revision,
            path: materialized.path,
            source_blob_sha: materialized.source_blob_sha,
            source: materialized.source,
            metadata: materialized.frontmatter.metadata,
            normative_mapping: Some(self.normative_mapping),
        })
    }
}

const NO_FEATURES: &[&str] = &[];
const NEW_TARGET_FEATURE: &[&str] = &["new.target"];
const REGEXP_MODIFIERS_FEATURE: &[&str] = &["regexp-modifiers"];

const ONLY_STRICT_PARSE_NEGATIVE: Metadata = Metadata {
    only_strict: true,
    no_strict: false,
    raw: false,
    module: false,
    non_deterministic: false,
    host_only: false,
    proposal_or_post_es2026: false,
    negative_phase: Some(NegativePhase::Parse),
    negative_type: Some(NegativeType::SyntaxError),
};

pub(crate) const PINNED_CASES: &[PinnedCase] = &[
    PinnedCase {
        path: "test/language/statements/with/strict-script.js",
        source_blob_sha: "9514530c03705c8461a7aa42aeac52a0c027ed26",
        frontmatter: FrontmatterIdentity {
            esid: None,
            features: NO_FEATURES,
            metadata: ONLY_STRICT_PARSE_NEGATIVE,
        },
        normative_mapping: "EE-23-R01",
    },
    PinnedCase {
        path: "test/language/statements/class/syntax/early-errors/class-definition-evaluation-scriptbody-duplicate-binding.js",
        source_blob_sha: "c5f3dff7cc6e23dfd729954698301e156a76985b",
        frontmatter: FrontmatterIdentity {
            esid: None,
            features: NO_FEATURES,
            metadata: Metadata::ordinary_parse_negative(),
        },
        normative_mapping: "EE-36-R01",
    },
    PinnedCase {
        path: "test/language/global-code/new.target.js",
        source_blob_sha: "e8c55b1ce90046bda2201a51a49662a24b845430",
        frontmatter: FrontmatterIdentity {
            esid: Some("sec-scripts-static-semantics-early-errors"),
            features: NEW_TARGET_FEATURE,
            metadata: Metadata::ordinary_parse_negative(),
        },
        normative_mapping: "EE-36-R04",
    },
    PinnedCase {
        path: "test/language/literals/regexp/early-err-modifiers-code-point-repeat-i-1.js",
        source_blob_sha: "055cebc72a993126baf3517420fbc85d929d1a49",
        frontmatter: FrontmatterIdentity {
            esid: Some("sec-patterns-static-semantics-early-errors"),
            features: REGEXP_MODIFIERS_FEATURE,
            metadata: Metadata::ordinary_parse_negative(),
        },
        normative_mapping: "EE-37-R04",
    },
];
