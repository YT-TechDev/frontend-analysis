//! Candidate-independent #171 `@keyframes` evidence gold model.
//!
//! This module is authored from the approved #171 architecture contract and
//! intentionally imports no production parser context/occurrence/evidence
//! types. It models only the bounded facts #171 claims: root-owned keyframes
//! contexts, qualified keyframe blocks, keyframe declaration syntax,
//! unsupported qualification boundaries, recovery, and honest EOF evidence.

use super::gold::GoldRange;
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyframeGoldContextTermination {
    AuthoredRightCurly(GoldRange),
    EndOfInput(GoldRange),
}

impl KeyframeGoldContextTermination {
    pub(super) const fn evidence(self) -> GoldRange {
        match self {
            Self::AuthoredRightCurly(range) | Self::EndOfInput(range) => range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyframeGoldOccurrenceTermination {
    AuthoredSemicolon(GoldRange),
    OmittedBeforeRightCurly(GoldRange),
    OmittedAtEndOfInput(GoldRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KeyframeGoldPriority {
    pub(super) complete: GoldRange,
    pub(super) bang: GoldRange,
    pub(super) important_ident: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KeyframeGoldOccurrence {
    pub(super) context_id: usize,
    pub(super) item_ordinal: usize,
    pub(super) complete: GoldRange,
    pub(super) name: GoldRange,
    pub(super) colon: GoldRange,
    pub(super) value: GoldRange,
    pub(super) priority: Option<KeyframeGoldPriority>,
    pub(super) termination: KeyframeGoldOccurrenceTermination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KeyframeGoldContext {
    Keyframes {
        id: usize,
        root_item_ordinal: usize,
        at_keyword: GoldRange,
        keyframes_name: GoldRange,
        header: GoldRange,
        block_opener: GoldRange,
        body: GoldRange,
        termination: KeyframeGoldContextTermination,
    },
    KeyframeBlock {
        id: usize,
        parent_id: usize,
        item_ordinal: usize,
        selector_list: GoldRange,
        header: GoldRange,
        block_opener: GoldRange,
        body: GoldRange,
        termination: KeyframeGoldContextTermination,
    },
}

impl KeyframeGoldContext {
    pub(super) const fn id(&self) -> usize {
        match self {
            Self::Keyframes { id, .. } | Self::KeyframeBlock { id, .. } => *id,
        }
    }

    const fn body(&self) -> GoldRange {
        match self {
            Self::Keyframes { body, .. } | Self::KeyframeBlock { body, .. } => *body,
        }
    }

    const fn header(&self) -> GoldRange {
        match self {
            Self::Keyframes { header, .. } | Self::KeyframeBlock { header, .. } => *header,
        }
    }

    const fn opener(&self) -> GoldRange {
        match self {
            Self::Keyframes { block_opener, .. }
            | Self::KeyframeBlock { block_opener, .. } => *block_opener,
        }
    }

    const fn termination(&self) -> KeyframeGoldContextTermination {
        match self {
            Self::Keyframes { termination, .. }
            | Self::KeyframeBlock { termination, .. } => *termination,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyframeGoldUnsupported {
    TopLevelAtRule {
        complete: GoldRange,
        at_keyword: GoldRange,
    },
    NestedAtRule {
        complete: GoldRange,
        at_keyword: GoldRange,
        context_id: usize,
        item_ordinal: usize,
    },
    UnqualifiedKeyframeBlock {
        complete: GoldRange,
        context_id: usize,
        item_ordinal: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyframeGoldRecoveryTermination {
    AuthoredSemicolon(GoldRange),
    EndOfInput(GoldRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KeyframeGoldRecovery {
    pub(super) region: GoldRange,
    pub(super) termination: KeyframeGoldRecoveryTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum KeyframeGoldCategory {
    OuterNameQualification,
    StringNameQualification,
    FromToSelectors,
    PercentageCommaSelectors,
    MultipleAndDuplicateBlocks,
    DistinctDeclarationMeaning,
    ImportantRemainsSyntaxEvidence,
    InvalidOuterIsUnsupported,
    InvalidChildIsUnsupported,
    MalformedDeclarationRecovery,
    TrueEofRetainsPartialContexts,
    NestedKeyframesNotPromoted,
    UpstreamIncompleteLifecycle,
    ParserResourceLifecycle,
    DeterministicRepeatAndCrossSourceId,
}

pub(super) const ALL_CATEGORIES: [KeyframeGoldCategory; 15] = [
    KeyframeGoldCategory::OuterNameQualification,
    KeyframeGoldCategory::StringNameQualification,
    KeyframeGoldCategory::FromToSelectors,
    KeyframeGoldCategory::PercentageCommaSelectors,
    KeyframeGoldCategory::MultipleAndDuplicateBlocks,
    KeyframeGoldCategory::DistinctDeclarationMeaning,
    KeyframeGoldCategory::ImportantRemainsSyntaxEvidence,
    KeyframeGoldCategory::InvalidOuterIsUnsupported,
    KeyframeGoldCategory::InvalidChildIsUnsupported,
    KeyframeGoldCategory::MalformedDeclarationRecovery,
    KeyframeGoldCategory::TrueEofRetainsPartialContexts,
    KeyframeGoldCategory::NestedKeyframesNotPromoted,
    KeyframeGoldCategory::UpstreamIncompleteLifecycle,
    KeyframeGoldCategory::ParserResourceLifecycle,
    KeyframeGoldCategory::DeterministicRepeatAndCrossSourceId,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KeyframeGoldFixture {
    pub(super) id: &'static str,
    pub(super) source_id: u64,
    pub(super) source: &'static str,
    pub(super) categories: Vec<KeyframeGoldCategory>,
    pub(super) contexts: Vec<KeyframeGoldContext>,
    pub(super) keyframe_occurrences: Vec<KeyframeGoldOccurrence>,
    pub(super) ordinary_occurrences: Vec<KeyframeGoldOccurrence>,
    pub(super) unsupported: Vec<KeyframeGoldUnsupported>,
    pub(super) recoveries: Vec<KeyframeGoldRecovery>,
    pub(super) diagnostics: Vec<GoldRange>,
}

fn range_ok(source: &SourceText, range: GoldRange) -> bool {
    source.anchor(range.start, range.end).is_ok()
}

fn contained(inner: GoldRange, outer: GoldRange) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn validate_context_boundary(
    source: &SourceText,
    context: &KeyframeGoldContext,
) -> Result<(), &'static str> {
    let header = context.header();
    let opener = context.opener();
    let body = context.body();
    if !range_ok(source, header) || !range_ok(source, opener) || !range_ok(source, body) {
        return Err("context range is not a valid source range");
    }
    if header.end != opener.start || opener.end != body.start {
        return Err("context header/opener/body boundaries disagree");
    }
    if source.anchor(opener.start, opener.end).unwrap().fragment() != "{" {
        return Err("context opener is not an authored left curly");
    }
    let termination = context.termination();
    let evidence = termination.evidence();
    if !range_ok(source, evidence) || body.end != evidence.start {
        return Err("context body/termination boundary disagrees");
    }
    match termination {
        KeyframeGoldContextTermination::AuthoredRightCurly(range) => {
            if source.anchor(range.start, range.end).unwrap().fragment() != "}" {
                return Err("authored context closer is not a right curly");
            }
        }
        KeyframeGoldContextTermination::EndOfInput(range) => {
            if range.start != range.end || range.end != source.as_str().len() {
                return Err("EOF context termination is not true source end");
            }
        }
    }
    Ok(())
}

pub(super) fn validate_fixture(fixture: &KeyframeGoldFixture) -> Result<(), &'static str> {
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let mut ids = std::collections::BTreeSet::new();
    for context in &fixture.contexts {
        if !ids.insert(context.id()) {
            return Err("duplicate context id");
        }
        validate_context_boundary(&source, context)?;
        match context {
            KeyframeGoldContext::Keyframes {
                at_keyword,
                keyframes_name,
                header,
                ..
            } => {
                if !range_ok(&source, *at_keyword)
                    || !range_ok(&source, *keyframes_name)
                    || !contained(*at_keyword, *header)
                    || !contained(*keyframes_name, *header)
                {
                    return Err("keyframes header evidence is inconsistent");
                }
                if source.anchor(at_keyword.start, at_keyword.end).unwrap().fragment()
                    != "@keyframes"
                {
                    return Err("gold keyframes at-keyword spelling is not exact");
                }
            }
            KeyframeGoldContext::KeyframeBlock {
                id,
                parent_id,
                selector_list,
                header,
                ..
            } => {
                if parent_id >= id
                    || !fixture.contexts.iter().any(|candidate| {
                        matches!(candidate, KeyframeGoldContext::Keyframes { id, .. } if id == parent_id)
                    })
                    || !range_ok(&source, *selector_list)
                    || !contained(*selector_list, *header)
                {
                    return Err("keyframe-block parent or selector evidence is inconsistent");
                }
                let parent_body = fixture
                    .contexts
                    .iter()
                    .find(|candidate| candidate.id() == *parent_id)
                    .unwrap()
                    .body();
                let child_extent = GoldRange::new(header.start, context.termination().evidence().end);
                if !contained(child_extent, parent_body) {
                    return Err("keyframe block is outside parent body");
                }
            }
        }
    }

    for occurrence in fixture
        .keyframe_occurrences
        .iter()
        .chain(&fixture.ordinary_occurrences)
    {
        for range in [occurrence.complete, occurrence.name, occurrence.colon, occurrence.value] {
            if !range_ok(&source, range) || !contained(range, occurrence.complete) {
                return Err("occurrence source evidence is inconsistent");
            }
        }
        if occurrence.name.end > occurrence.colon.start || occurrence.colon.end > occurrence.value.start {
            return Err("occurrence component order is inconsistent");
        }
    }

    for occurrence in &fixture.keyframe_occurrences {
        let Some(owner) = fixture
            .contexts
            .iter()
            .find(|context| context.id() == occurrence.context_id)
        else {
            return Err("keyframe occurrence owner is missing");
        };
        if !matches!(owner, KeyframeGoldContext::KeyframeBlock { .. })
            || !contained(occurrence.complete, owner.body())
        {
            return Err("keyframe occurrence is not contained by a keyframe block");
        }
    }

    for unsupported in &fixture.unsupported {
        match unsupported {
            KeyframeGoldUnsupported::TopLevelAtRule { complete, at_keyword } => {
                if !range_ok(&source, *complete)
                    || !range_ok(&source, *at_keyword)
                    || !contained(*at_keyword, *complete)
                {
                    return Err("top-level unsupported evidence is inconsistent");
                }
            }
            KeyframeGoldUnsupported::NestedAtRule {
                complete,
                at_keyword,
                ..
            } => {
                if !range_ok(&source, *complete)
                    || !range_ok(&source, *at_keyword)
                    || !contained(*at_keyword, *complete)
                {
                    return Err("nested unsupported evidence is inconsistent");
                }
            }
            KeyframeGoldUnsupported::UnqualifiedKeyframeBlock {
                complete,
                context_id,
                ..
            } => {
                let Some(owner) = fixture.contexts.iter().find(|context| context.id() == *context_id)
                else {
                    return Err("unqualified keyframe-block owner is missing");
                };
                if !matches!(owner, KeyframeGoldContext::Keyframes { .. })
                    || !range_ok(&source, *complete)
                    || !contained(*complete, owner.body())
                {
                    return Err("unqualified keyframe-block evidence is inconsistent");
                }
            }
        }
    }

    for recovery in &fixture.recoveries {
        if !range_ok(&source, recovery.region) {
            return Err("recovery region is invalid");
        }
    }
    for diagnostic in &fixture.diagnostics {
        if !range_ok(&source, *diagnostic) {
            return Err("diagnostic location is invalid");
        }
    }
    Ok(())
}
