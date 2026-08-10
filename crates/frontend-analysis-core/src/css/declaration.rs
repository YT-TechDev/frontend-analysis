//! Source-backed CSS declaration-analysis domain contracts (#138).
//!
//! These types answer only an authored-syntax question for the #137-approved
//! first parser slice. They never claim known-property identity, grammar
//! validity, browser support, CSSOM membership, cascade participation, or
//! computed/used value meaning.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::parser::context::{CssParserContextId, CssParserDirectItemOrdinal};

/// A declaration's run-local ordinal among the consecutive declarations in
/// one owning context uninterrupted by a materialized child qualified-rule
/// context (#167).
///
/// Zero-based per owning context. Owns no authored [`SourceAnchor`]: run
/// identity is parser analysis metadata only, never a source range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssDeclarationRunOrdinal(usize);

impl CssDeclarationRunOrdinal {
    pub(crate) const fn new(ordinal: usize) -> Self {
        Self(ordinal)
    }

    pub(crate) const fn value(self) -> usize {
        self.0
    }
}

/// A declaration's structural placement within the context-aware #167
/// producer: the owning context, its direct-item ordinal within that
/// context (shared with sibling child qualified-rule contexts), and the
/// declaration-run ordinal.
///
/// Replaces the #137/#166 single-variant `CssDeclarationContext` zone
/// association. Placement is parser analysis metadata only: it owns no
/// authored source range and does not replace any existing declaration
/// evidence anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssDeclarationPlacement {
    context_id: CssParserContextId,
    item_ordinal: CssParserDirectItemOrdinal,
    run_ordinal: CssDeclarationRunOrdinal,
}

impl CssDeclarationPlacement {
    pub(crate) const fn new(
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
        run_ordinal: CssDeclarationRunOrdinal,
    ) -> Self {
        Self {
            context_id,
            item_ordinal,
            run_ordinal,
        }
    }

    pub(crate) const fn context_id(&self) -> CssParserContextId {
        self.context_id
    }

    pub(crate) const fn item_ordinal(&self) -> CssParserDirectItemOrdinal {
        self.item_ordinal
    }

    pub(crate) const fn run_ordinal(&self) -> CssDeclarationRunOrdinal {
        self.run_ordinal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssDeclarationEvidenceRole {
    Complete,
    PropertyName,
    Colon,
    Value,
    PriorityComplete,
    PriorityBang,
    PriorityImportantIdent,
    Semicolon,
    RightCurly,
    EofTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssDeclarationContractError {
    SourceIdentityMismatch {
        role: CssDeclarationEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    EmptyEvidence {
        role: CssDeclarationEvidenceRole,
    },
    FixedSpellingMismatch {
        role: CssDeclarationEvidenceRole,
        expected: &'static str,
    },
    CompleteStartMismatch {
        complete_start: usize,
        property_name_start: usize,
    },
    EvidenceOutsideComplete {
        role: CssDeclarationEvidenceRole,
    },
    EvidenceOutOfOrder {
        role: CssDeclarationEvidenceRole,
    },
    ZeroLengthValueNotAtColonEnd,
    ValueOverlapsPriority,
    PriorityContainmentViolation,
    InvalidPriorityIdentifier,
    CompleteEndMismatch {
        complete_end: usize,
        expected_end: usize,
    },
    FabricatedOmittedEofTerminal,
}

impl fmt::Display for CssDeclarationContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS declaration contract violation: {self:?}")
    }
}

impl Error for CssDeclarationContractError {}

/// Source-backed evidence of a valid, recognized authored `!important`
/// annotation.
///
/// Invalid priority-shaped syntax never constructs this type; it remains
/// ordinary value/recovery evidence instead.
#[derive(Clone)]
pub(crate) struct CssDeclarationPriorityEvidence {
    complete: SourceAnchor,
    bang: SourceAnchor,
    important_ident: SourceAnchor,
}

impl CssDeclarationPriorityEvidence {
    /// `decoded_important_ident` is the already-decoded tokenizer `Ident`
    /// value used only to validate an ASCII-case-insensitive match against
    /// `important`; it is not retained afterward.
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        bang: SourceAnchor,
        important_ident: SourceAnchor,
        decoded_important_ident: &str,
    ) -> Result<Self, CssDeclarationContractError> {
        let expected = source_text.id();
        require_source(
            expected,
            &complete,
            CssDeclarationEvidenceRole::PriorityComplete,
        )?;
        require_source(expected, &bang, CssDeclarationEvidenceRole::PriorityBang)?;
        require_source(
            expected,
            &important_ident,
            CssDeclarationEvidenceRole::PriorityImportantIdent,
        )?;

        non_empty(&complete, CssDeclarationEvidenceRole::PriorityComplete)?;
        non_empty(&bang, CssDeclarationEvidenceRole::PriorityBang)?;
        non_empty(
            &important_ident,
            CssDeclarationEvidenceRole::PriorityImportantIdent,
        )?;

        exact(&bang, CssDeclarationEvidenceRole::PriorityBang, "!")?;

        if !decoded_important_ident.eq_ignore_ascii_case("important") {
            return Err(CssDeclarationContractError::InvalidPriorityIdentifier);
        }

        if complete.range().start() != bang.range().start() {
            return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                role: CssDeclarationEvidenceRole::PriorityBang,
            });
        }
        if complete.range().end() != important_ident.range().end() {
            return Err(CssDeclarationContractError::CompleteEndMismatch {
                complete_end: complete.range().end(),
                expected_end: important_ident.range().end(),
            });
        }
        if bang.range().end() > important_ident.range().start() {
            return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                role: CssDeclarationEvidenceRole::PriorityImportantIdent,
            });
        }
        contained(
            &complete,
            &important_ident,
            CssDeclarationEvidenceRole::PriorityImportantIdent,
        )?;

        Ok(Self {
            complete,
            bang,
            important_ident,
        })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    pub(crate) const fn bang(&self) -> &SourceAnchor {
        &self.bang
    }

    pub(crate) const fn important_ident(&self) -> &SourceAnchor {
        &self.important_ident
    }
}

impl PartialEq for CssDeclarationPriorityEvidence {
    fn eq(&self, other: &Self) -> bool {
        same_anchor(&self.complete, &other.complete)
            && same_anchor(&self.bang, &other.bang)
            && same_anchor(&self.important_ident, &other.important_ident)
    }
}

impl Eq for CssDeclarationPriorityEvidence {}

impl fmt::Debug for CssDeclarationPriorityEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssDeclarationPriorityEvidence")
            .field("source_id", &self.complete.source_id())
            .field("complete", &self.complete.range())
            .field("bang", &self.bang.range())
            .field("important_ident", &self.important_ident.range())
            .finish()
    }
}

/// Explicit, source-aware declaration termination.
///
/// Never fabricates an empty semicolon anchor for omitted syntax.
#[derive(Clone)]
pub(crate) enum CssDeclarationTermination {
    AuthoredSemicolon { semicolon: SourceAnchor },
    OmittedBeforeRightCurly { right_curly: SourceAnchor },
    OmittedAtEndOfInput { terminal: SourceAnchor },
}

impl PartialEq for CssDeclarationTermination {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::AuthoredSemicolon { semicolon: left },
                Self::AuthoredSemicolon { semicolon: right },
            ) => same_anchor(left, right),
            (
                Self::OmittedBeforeRightCurly { right_curly: left },
                Self::OmittedBeforeRightCurly { right_curly: right },
            ) => same_anchor(left, right),
            (
                Self::OmittedAtEndOfInput { terminal: left },
                Self::OmittedAtEndOfInput { terminal: right },
            ) => same_anchor(left, right),
            _ => false,
        }
    }
}

impl Eq for CssDeclarationTermination {}

impl fmt::Debug for CssDeclarationTermination {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthoredSemicolon { semicolon } => formatter
                .debug_struct("AuthoredSemicolon")
                .field("source_id", &semicolon.source_id())
                .field("semicolon", &semicolon.range())
                .finish(),
            Self::OmittedBeforeRightCurly { right_curly } => formatter
                .debug_struct("OmittedBeforeRightCurly")
                .field("source_id", &right_curly.source_id())
                .field("right_curly", &right_curly.range())
                .finish(),
            Self::OmittedAtEndOfInput { terminal } => formatter
                .debug_struct("OmittedAtEndOfInput")
                .field("source_id", &terminal.source_id())
                .field("terminal", &terminal.range())
                .finish(),
        }
    }
}

/// One recognized authored `Ident : value` syntax occurrence's source-backed
/// evidence, shared between ordinary style-rule declarations
/// ([`CssDeclarationOccurrence`]) and #169 descriptor occurrences
/// ([`CssDescriptorOccurrence`]).
///
/// This is a syntax-occurrence claim only. It does not establish known
/// property/descriptor identity, property/value grammar validity, selector
/// validity, browser support, CSSOM membership, cascade participation, or
/// computed/used value. Owning a shared syntax shape does not make the two
/// occurrence meanings interchangeable: each wrapper retains its own
/// [`CssDeclarationPlacement`]/[`CssDescriptorPlacement`], and result-level
/// validation enforces the semantic owner boundary between them.
#[derive(Clone)]
pub(crate) struct CssDeclarationSyntaxEvidence {
    complete: SourceAnchor,
    property_name: SourceAnchor,
    colon: SourceAnchor,
    value: SourceAnchor,
    priority: Option<CssDeclarationPriorityEvidence>,
    termination: CssDeclarationTermination,
}

impl CssDeclarationSyntaxEvidence {
    fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        property_name: SourceAnchor,
        colon: SourceAnchor,
        value: SourceAnchor,
        priority: Option<CssDeclarationPriorityEvidence>,
        termination: CssDeclarationTermination,
    ) -> Result<Self, CssDeclarationContractError> {
        let expected = source_text.id();

        require_source(expected, &complete, CssDeclarationEvidenceRole::Complete)?;
        require_source(
            expected,
            &property_name,
            CssDeclarationEvidenceRole::PropertyName,
        )?;
        require_source(expected, &colon, CssDeclarationEvidenceRole::Colon)?;
        require_source(expected, &value, CssDeclarationEvidenceRole::Value)?;
        if let Some(priority) = &priority {
            require_source(
                expected,
                priority.complete(),
                CssDeclarationEvidenceRole::PriorityComplete,
            )?;
        }
        require_termination_source(expected, &termination)?;

        non_empty(&complete, CssDeclarationEvidenceRole::Complete)?;
        non_empty(&property_name, CssDeclarationEvidenceRole::PropertyName)?;
        non_empty(&colon, CssDeclarationEvidenceRole::Colon)?;
        exact(&colon, CssDeclarationEvidenceRole::Colon, ":")?;

        if complete.range().start() != property_name.range().start() {
            return Err(CssDeclarationContractError::CompleteStartMismatch {
                complete_start: complete.range().start(),
                property_name_start: property_name.range().start(),
            });
        }

        contained(
            &complete,
            &property_name,
            CssDeclarationEvidenceRole::PropertyName,
        )?;
        contained(&complete, &colon, CssDeclarationEvidenceRole::Colon)?;
        if !value.range().is_empty() {
            contained(&complete, &value, CssDeclarationEvidenceRole::Value)?;
        }

        if property_name.range().end() > colon.range().start() {
            return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                role: CssDeclarationEvidenceRole::Colon,
            });
        }

        if value.range().is_empty() {
            if value.range().start() != colon.range().end() {
                return Err(CssDeclarationContractError::ZeroLengthValueNotAtColonEnd);
            }
        } else if colon.range().end() > value.range().start() {
            return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                role: CssDeclarationEvidenceRole::Value,
            });
        }

        if let Some(priority_evidence) = &priority {
            contained(
                &complete,
                priority_evidence.complete(),
                CssDeclarationEvidenceRole::PriorityComplete,
            )?;
            if !value.range().is_empty()
                && value.range().end() > priority_evidence.complete().range().start()
            {
                return Err(CssDeclarationContractError::ValueOverlapsPriority);
            }
            if value.range().is_empty()
                && value.range().start() > priority_evidence.complete().range().start()
            {
                return Err(CssDeclarationContractError::PriorityContainmentViolation);
            }
        }

        validate_termination(
            source_text,
            &complete,
            &colon,
            &value,
            priority.as_ref(),
            &termination,
        )?;

        Ok(Self {
            complete,
            property_name,
            colon,
            value,
            priority,
            termination,
        })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    pub(crate) const fn name(&self) -> &SourceAnchor {
        &self.property_name
    }

    pub(crate) const fn colon(&self) -> &SourceAnchor {
        &self.colon
    }

    pub(crate) const fn value(&self) -> &SourceAnchor {
        &self.value
    }

    pub(crate) const fn priority(&self) -> Option<&CssDeclarationPriorityEvidence> {
        self.priority.as_ref()
    }

    pub(crate) const fn termination(&self) -> &CssDeclarationTermination {
        &self.termination
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        (self.complete.range().start(), self.complete.range().end())
    }
}

impl PartialEq for CssDeclarationSyntaxEvidence {
    fn eq(&self, other: &Self) -> bool {
        same_anchor(&self.complete, &other.complete)
            && same_anchor(&self.property_name, &other.property_name)
            && same_anchor(&self.colon, &other.colon)
            && same_anchor(&self.value, &other.value)
            && self.priority == other.priority
            && self.termination == other.termination
    }
}

impl Eq for CssDeclarationSyntaxEvidence {}

/// One recognized authored CSS declaration syntax occurrence within the
/// #137-approved parser context.
///
/// This is a syntax-occurrence claim only. It does not establish known
/// property identity, property/value grammar validity, selector validity,
/// browser support, CSSOM membership, cascade participation, or
/// computed/used value.
#[derive(Clone)]
pub(crate) struct CssDeclarationOccurrence {
    syntax: CssDeclarationSyntaxEvidence,
    placement: CssDeclarationPlacement,
}

impl CssDeclarationOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        property_name: SourceAnchor,
        colon: SourceAnchor,
        value: SourceAnchor,
        priority: Option<CssDeclarationPriorityEvidence>,
        termination: CssDeclarationTermination,
        placement: CssDeclarationPlacement,
    ) -> Result<Self, CssDeclarationContractError> {
        let syntax = CssDeclarationSyntaxEvidence::new(
            source_text,
            complete,
            property_name,
            colon,
            value,
            priority,
            termination,
        )?;
        Ok(Self { syntax, placement })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        self.syntax.complete()
    }

    pub(crate) const fn property_name(&self) -> &SourceAnchor {
        self.syntax.name()
    }

    pub(crate) const fn colon(&self) -> &SourceAnchor {
        self.syntax.colon()
    }

    pub(crate) const fn value(&self) -> &SourceAnchor {
        self.syntax.value()
    }

    pub(crate) const fn priority(&self) -> Option<&CssDeclarationPriorityEvidence> {
        self.syntax.priority()
    }

    pub(crate) const fn termination(&self) -> &CssDeclarationTermination {
        self.syntax.termination()
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        self.syntax.source_order_key()
    }
}

impl PartialEq for CssDeclarationOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.placement == other.placement
    }
}

impl Eq for CssDeclarationOccurrence {}

impl fmt::Debug for CssDeclarationOccurrence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssDeclarationOccurrence")
            .field("source_id", &self.complete().source_id())
            .field("complete", &self.complete().range())
            .field("property_name", &self.property_name().range())
            .field("colon", &self.colon().range())
            .field("value", &self.value().range())
            .field("priority", &self.priority())
            .field("termination", &self.termination())
            .field("placement", &self.placement)
            .finish()
    }
}

/// A descriptor occurrence's structural placement (#169): the owning
/// [`CssParserContextKind::DescriptorRuleBlock`](super::parser::context::CssParserContextKind::DescriptorRuleBlock)
/// context and its direct-item ordinal within that context, shared with
/// sibling explicit unsupported nested at-rule items.
///
/// Deliberately carries no declaration-run ordinal: `<declaration-list>`
/// admits no child rule whose interleaving requires the style-rule
/// declaration-run model, so descriptor placement is not
/// [`CssDeclarationPlacement`] with a field omitted -- it is a distinct,
/// narrower shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssDescriptorPlacement {
    context_id: CssParserContextId,
    item_ordinal: CssParserDirectItemOrdinal,
}

impl CssDescriptorPlacement {
    pub(crate) const fn new(
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
    ) -> Self {
        Self {
            context_id,
            item_ordinal,
        }
    }

    pub(crate) const fn context_id(&self) -> CssParserContextId {
        self.context_id
    }

    pub(crate) const fn item_ordinal(&self) -> CssParserDirectItemOrdinal {
        self.item_ordinal
    }
}

/// One recognized authored descriptor-shaped syntax occurrence retained
/// inside an explicitly-qualified supported descriptor-rule context (#169).
///
/// Means only: a declaration-shaped authored syntax occurrence retained
/// inside a supported `DescriptorRuleBlock` context. It does not mean the
/// descriptor name is recognized by the owning rule, that its value matches
/// its grammar, that it has CSSOM/runtime effect, that `@font-face` is
/// usable for font matching, or that `@property` is a valid registration.
/// Never interchangeable with [`CssDeclarationOccurrence`], despite sharing
/// [`CssDeclarationSyntaxEvidence`]: the two remain semantically distinct
/// types, and result-level validation enforces that a descriptor occurrence
/// can only be owned by a `DescriptorRuleBlock` context and an ordinary
/// declaration can never be owned by one.
#[derive(Clone)]
pub(crate) struct CssDescriptorOccurrence {
    syntax: CssDeclarationSyntaxEvidence,
    placement: CssDescriptorPlacement,
}

impl CssDescriptorOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        name: SourceAnchor,
        colon: SourceAnchor,
        value: SourceAnchor,
        priority: Option<CssDeclarationPriorityEvidence>,
        termination: CssDeclarationTermination,
        placement: CssDescriptorPlacement,
    ) -> Result<Self, CssDeclarationContractError> {
        let syntax = CssDeclarationSyntaxEvidence::new(
            source_text,
            complete,
            name,
            colon,
            value,
            priority,
            termination,
        )?;
        Ok(Self { syntax, placement })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        self.syntax.complete()
    }

    pub(crate) const fn name(&self) -> &SourceAnchor {
        self.syntax.name()
    }

    pub(crate) const fn colon(&self) -> &SourceAnchor {
        self.syntax.colon()
    }

    pub(crate) const fn value(&self) -> &SourceAnchor {
        self.syntax.value()
    }

    pub(crate) const fn priority(&self) -> Option<&CssDeclarationPriorityEvidence> {
        self.syntax.priority()
    }

    pub(crate) const fn termination(&self) -> &CssDeclarationTermination {
        self.syntax.termination()
    }

    pub(crate) const fn placement(&self) -> CssDescriptorPlacement {
        self.placement
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        self.syntax.source_order_key()
    }
}

impl PartialEq for CssDescriptorOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.placement == other.placement
    }
}

impl Eq for CssDescriptorOccurrence {}

impl fmt::Debug for CssDescriptorOccurrence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssDescriptorOccurrence")
            .field("source_id", &self.complete().source_id())
            .field("complete", &self.complete().range())
            .field("name", &self.name().range())
            .field("colon", &self.colon().range())
            .field("value", &self.value().range())
            .field("priority", &self.priority())
            .field("termination", &self.termination())
            .field("placement", &self.placement)
            .finish()
    }
}

/// A keyframe declaration's structural placement (#171): the owning
/// [`CssParserContextKind::KeyframeBlock`](super::parser::context::CssParserContextKind::KeyframeBlock)
/// context and its direct-item ordinal within that declaration-list body.
///
/// Deliberately carries no declaration-run ordinal: a keyframe block contains
/// a `<declaration-list>`, so no child-rule interleaving needs the ordinary
/// style-rule declaration-run model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssKeyframeDeclarationPlacement {
    context_id: CssParserContextId,
    item_ordinal: CssParserDirectItemOrdinal,
}

impl CssKeyframeDeclarationPlacement {
    pub(crate) const fn new(
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
    ) -> Self {
        Self {
            context_id,
            item_ordinal,
        }
    }

    pub(crate) const fn context_id(&self) -> CssParserContextId {
        self.context_id
    }

    pub(crate) const fn item_ordinal(&self) -> CssParserDirectItemOrdinal {
        self.item_ordinal
    }
}

/// One recognized authored declaration-shaped syntax occurrence retained
/// inside an explicitly qualified `KeyframeBlock` context (#171).
///
/// This is parser-owned source evidence only. It does not claim property
/// grammar validity, animation interpolation/runtime meaning, CSSOM
/// membership, or the animation-specific legality of `!important`.
/// Never interchangeable with ordinary, descriptor, page, or page-margin
/// occurrences despite sharing [`CssDeclarationSyntaxEvidence`].
#[derive(Clone)]
pub(crate) struct CssKeyframeDeclarationOccurrence {
    syntax: CssDeclarationSyntaxEvidence,
    placement: CssKeyframeDeclarationPlacement,
}

impl CssKeyframeDeclarationOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        name: SourceAnchor,
        colon: SourceAnchor,
        value: SourceAnchor,
        priority: Option<CssDeclarationPriorityEvidence>,
        termination: CssDeclarationTermination,
        placement: CssKeyframeDeclarationPlacement,
    ) -> Result<Self, CssDeclarationContractError> {
        let syntax = CssDeclarationSyntaxEvidence::new(
            source_text,
            complete,
            name,
            colon,
            value,
            priority,
            termination,
        )?;
        Ok(Self { syntax, placement })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        self.syntax.complete()
    }
    pub(crate) const fn name(&self) -> &SourceAnchor {
        self.syntax.name()
    }
    pub(crate) const fn colon(&self) -> &SourceAnchor {
        self.syntax.colon()
    }
    pub(crate) const fn value(&self) -> &SourceAnchor {
        self.syntax.value()
    }
    pub(crate) const fn priority(&self) -> Option<&CssDeclarationPriorityEvidence> {
        self.syntax.priority()
    }
    pub(crate) const fn termination(&self) -> &CssDeclarationTermination {
        self.syntax.termination()
    }
    pub(crate) const fn placement(&self) -> CssKeyframeDeclarationPlacement {
        self.placement
    }
    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        self.syntax.source_order_key()
    }
}

impl PartialEq for CssKeyframeDeclarationOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.placement == other.placement
    }
}
impl Eq for CssKeyframeDeclarationOccurrence {}

impl fmt::Debug for CssKeyframeDeclarationOccurrence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssKeyframeDeclarationOccurrence")
            .field("source_id", &self.complete().source_id())
            .field("complete", &self.complete().range())
            .field("name", &self.name().range())
            .field("colon", &self.colon().range())
            .field("value", &self.value().range())
            .field("priority", &self.priority())
            .field("termination", &self.termination())
            .field("placement", &self.placement)
            .finish()
    }
}

/// A page declaration's structural placement (#170): the owning
/// [`CssParserContextKind::PageRuleBlock`](super::parser::context::CssParserContextKind::PageRuleBlock)
/// context and its direct-item ordinal within that context, shared with
/// sibling `PageMarginRuleBlock` child contexts and nested unsupported
/// at-rule items.
///
/// Deliberately carries no declaration-run ordinal, mirroring
/// [`CssDescriptorPlacement`]: CSSWG #11271 means Page must not introduce a
/// new stable declaration-run semantic contract (Amendment A). This is not
/// [`CssDeclarationPlacement`] with a field omitted -- it is a distinct,
/// narrower shape reusing only `context_id`/`item_ordinal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPageDeclarationPlacement {
    context_id: CssParserContextId,
    item_ordinal: CssParserDirectItemOrdinal,
}

impl CssPageDeclarationPlacement {
    pub(crate) const fn new(
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
    ) -> Self {
        Self {
            context_id,
            item_ordinal,
        }
    }

    pub(crate) const fn context_id(&self) -> CssParserContextId {
        self.context_id
    }

    pub(crate) const fn item_ordinal(&self) -> CssParserDirectItemOrdinal {
        self.item_ordinal
    }
}

/// A page-margin declaration's structural placement (#170): the owning
/// [`CssParserContextKind::PageMarginRuleBlock`](super::parser::context::CssParserContextKind::PageMarginRuleBlock)
/// context and its direct-item ordinal within that context, shared with
/// sibling nested unsupported at-rule items. Carries no declaration-run
/// ordinal for the same Amendment A reason as [`CssPageDeclarationPlacement`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPageMarginDeclarationPlacement {
    context_id: CssParserContextId,
    item_ordinal: CssParserDirectItemOrdinal,
}

impl CssPageMarginDeclarationPlacement {
    pub(crate) const fn new(
        context_id: CssParserContextId,
        item_ordinal: CssParserDirectItemOrdinal,
    ) -> Self {
        Self {
            context_id,
            item_ordinal,
        }
    }

    pub(crate) const fn context_id(&self) -> CssParserContextId {
        self.context_id
    }

    pub(crate) const fn item_ordinal(&self) -> CssParserDirectItemOrdinal {
        self.item_ordinal
    }
}

/// One recognized authored declaration-shaped syntax occurrence retained
/// inside a root-owned `PageRuleBlock` context (#170).
///
/// Means only: a declaration-shaped authored syntax occurrence retained
/// inside a supported `PageRuleBlock` context. It does not mean the
/// property name is recognized by `@page`, that its value matches its
/// grammar, or that it has CSSOM/cascade/print effect. Never interchangeable
/// with [`CssDeclarationOccurrence`] or [`CssDescriptorOccurrence`], despite
/// sharing [`CssDeclarationSyntaxEvidence`]: result-level validation
/// enforces that a page declaration occurrence can only be owned by a
/// `PageRuleBlock` context.
#[derive(Clone)]
pub(crate) struct CssPageDeclarationOccurrence {
    syntax: CssDeclarationSyntaxEvidence,
    placement: CssPageDeclarationPlacement,
}

impl CssPageDeclarationOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        name: SourceAnchor,
        colon: SourceAnchor,
        value: SourceAnchor,
        priority: Option<CssDeclarationPriorityEvidence>,
        termination: CssDeclarationTermination,
        placement: CssPageDeclarationPlacement,
    ) -> Result<Self, CssDeclarationContractError> {
        let syntax = CssDeclarationSyntaxEvidence::new(
            source_text,
            complete,
            name,
            colon,
            value,
            priority,
            termination,
        )?;
        Ok(Self { syntax, placement })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        self.syntax.complete()
    }

    pub(crate) const fn name(&self) -> &SourceAnchor {
        self.syntax.name()
    }

    pub(crate) const fn colon(&self) -> &SourceAnchor {
        self.syntax.colon()
    }

    pub(crate) const fn value(&self) -> &SourceAnchor {
        self.syntax.value()
    }

    pub(crate) const fn priority(&self) -> Option<&CssDeclarationPriorityEvidence> {
        self.syntax.priority()
    }

    pub(crate) const fn termination(&self) -> &CssDeclarationTermination {
        self.syntax.termination()
    }

    pub(crate) const fn placement(&self) -> CssPageDeclarationPlacement {
        self.placement
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        self.syntax.source_order_key()
    }
}

impl PartialEq for CssPageDeclarationOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.placement == other.placement
    }
}

impl Eq for CssPageDeclarationOccurrence {}

impl fmt::Debug for CssPageDeclarationOccurrence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssPageDeclarationOccurrence")
            .field("source_id", &self.complete().source_id())
            .field("complete", &self.complete().range())
            .field("name", &self.name().range())
            .field("colon", &self.colon().range())
            .field("value", &self.value().range())
            .field("priority", &self.priority())
            .field("termination", &self.termination())
            .field("placement", &self.placement)
            .finish()
    }
}

/// One recognized authored declaration-shaped syntax occurrence retained
/// inside a `PageMarginRuleBlock` context (#170), whose parent is always a
/// retained `PageRuleBlock`.
///
/// Means only a declaration-shaped authored syntax occurrence retained
/// inside a supported `PageMarginRuleBlock` context; carries none of the
/// broader claims disclaimed by [`CssPageDeclarationOccurrence`]'s own doc.
/// Never interchangeable with [`CssDeclarationOccurrence`],
/// [`CssDescriptorOccurrence`], or [`CssPageDeclarationOccurrence`], despite
/// sharing [`CssDeclarationSyntaxEvidence`]: result-level validation
/// enforces that a page-margin declaration occurrence can only be owned by a
/// `PageMarginRuleBlock` context.
#[derive(Clone)]
pub(crate) struct CssPageMarginDeclarationOccurrence {
    syntax: CssDeclarationSyntaxEvidence,
    placement: CssPageMarginDeclarationPlacement,
}

impl CssPageMarginDeclarationOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        name: SourceAnchor,
        colon: SourceAnchor,
        value: SourceAnchor,
        priority: Option<CssDeclarationPriorityEvidence>,
        termination: CssDeclarationTermination,
        placement: CssPageMarginDeclarationPlacement,
    ) -> Result<Self, CssDeclarationContractError> {
        let syntax = CssDeclarationSyntaxEvidence::new(
            source_text,
            complete,
            name,
            colon,
            value,
            priority,
            termination,
        )?;
        Ok(Self { syntax, placement })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        self.syntax.complete()
    }

    pub(crate) const fn name(&self) -> &SourceAnchor {
        self.syntax.name()
    }

    pub(crate) const fn colon(&self) -> &SourceAnchor {
        self.syntax.colon()
    }

    pub(crate) const fn value(&self) -> &SourceAnchor {
        self.syntax.value()
    }

    pub(crate) const fn priority(&self) -> Option<&CssDeclarationPriorityEvidence> {
        self.syntax.priority()
    }

    pub(crate) const fn termination(&self) -> &CssDeclarationTermination {
        self.syntax.termination()
    }

    pub(crate) const fn placement(&self) -> CssPageMarginDeclarationPlacement {
        self.placement
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        self.syntax.source_order_key()
    }
}

impl PartialEq for CssPageMarginDeclarationOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.placement == other.placement
    }
}

impl Eq for CssPageMarginDeclarationOccurrence {}

impl fmt::Debug for CssPageMarginDeclarationOccurrence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssPageMarginDeclarationOccurrence")
            .field("source_id", &self.complete().source_id())
            .field("complete", &self.complete().range())
            .field("name", &self.name().range())
            .field("colon", &self.colon().range())
            .field("value", &self.value().range())
            .field("priority", &self.priority())
            .field("termination", &self.termination())
            .field("placement", &self.placement)
            .finish()
    }
}

fn validate_termination(
    source_text: &SourceText,
    complete: &SourceAnchor,
    colon: &SourceAnchor,
    value: &SourceAnchor,
    priority: Option<&CssDeclarationPriorityEvidence>,
    termination: &CssDeclarationTermination,
) -> Result<(), CssDeclarationContractError> {
    match termination {
        CssDeclarationTermination::AuthoredSemicolon { semicolon } => {
            non_empty(semicolon, CssDeclarationEvidenceRole::Semicolon)?;
            exact(semicolon, CssDeclarationEvidenceRole::Semicolon, ";")?;
            contained(complete, semicolon, CssDeclarationEvidenceRole::Semicolon)?;

            let last_semantic_end = priority.map_or_else(
                || {
                    if value.range().is_empty() {
                        colon.range().end()
                    } else {
                        value.range().end()
                    }
                },
                |priority| priority.complete().range().end(),
            );
            if semicolon.range().start() < last_semantic_end {
                return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                    role: CssDeclarationEvidenceRole::Semicolon,
                });
            }
            if complete.range().end() != semicolon.range().end() {
                return Err(CssDeclarationContractError::CompleteEndMismatch {
                    complete_end: complete.range().end(),
                    expected_end: semicolon.range().end(),
                });
            }
            Ok(())
        }
        CssDeclarationTermination::OmittedBeforeRightCurly { right_curly } => {
            non_empty(right_curly, CssDeclarationEvidenceRole::RightCurly)?;
            exact(right_curly, CssDeclarationEvidenceRole::RightCurly, "}")?;
            require_source(
                source_text.id(),
                right_curly,
                CssDeclarationEvidenceRole::RightCurly,
            )?;
            let expected_end = expected_omitted_end(colon, value, priority);
            if complete.range().end() != expected_end {
                return Err(CssDeclarationContractError::CompleteEndMismatch {
                    complete_end: complete.range().end(),
                    expected_end,
                });
            }
            if right_curly.range().start() < complete.range().end() {
                return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                    role: CssDeclarationEvidenceRole::RightCurly,
                });
            }
            Ok(())
        }
        CssDeclarationTermination::OmittedAtEndOfInput { terminal } => {
            non_empty_forbidden(terminal, CssDeclarationEvidenceRole::EofTerminal)?;
            require_source(
                source_text.id(),
                terminal,
                CssDeclarationEvidenceRole::EofTerminal,
            )?;
            let expected_end = expected_omitted_end(colon, value, priority);
            if complete.range().end() != expected_end {
                return Err(CssDeclarationContractError::CompleteEndMismatch {
                    complete_end: complete.range().end(),
                    expected_end,
                });
            }
            let source_len = source_text.as_str().len();
            if terminal.range().start() != source_len {
                return Err(CssDeclarationContractError::FabricatedOmittedEofTerminal);
            }
            if terminal.range().start() < complete.range().end() {
                return Err(CssDeclarationContractError::EvidenceOutOfOrder {
                    role: CssDeclarationEvidenceRole::EofTerminal,
                });
            }
            Ok(())
        }
    }
}

fn expected_omitted_end(
    colon: &SourceAnchor,
    value: &SourceAnchor,
    priority: Option<&CssDeclarationPriorityEvidence>,
) -> usize {
    if let Some(priority) = priority {
        priority.complete().range().end()
    } else if value.range().is_empty() {
        colon.range().end()
    } else {
        value.range().end()
    }
}

fn require_termination_source(
    expected: SourceId,
    termination: &CssDeclarationTermination,
) -> Result<(), CssDeclarationContractError> {
    match termination {
        CssDeclarationTermination::AuthoredSemicolon { semicolon } => {
            require_source(expected, semicolon, CssDeclarationEvidenceRole::Semicolon)
        }
        CssDeclarationTermination::OmittedBeforeRightCurly { right_curly } => require_source(
            expected,
            right_curly,
            CssDeclarationEvidenceRole::RightCurly,
        ),
        CssDeclarationTermination::OmittedAtEndOfInput { terminal } => {
            require_source(expected, terminal, CssDeclarationEvidenceRole::EofTerminal)
        }
    }
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssDeclarationEvidenceRole,
) -> Result<(), CssDeclarationContractError> {
    let actual = anchor.source_id();
    if actual != expected {
        return Err(CssDeclarationContractError::SourceIdentityMismatch {
            role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn non_empty(
    anchor: &SourceAnchor,
    role: CssDeclarationEvidenceRole,
) -> Result<(), CssDeclarationContractError> {
    if anchor.range().is_empty() {
        return Err(CssDeclarationContractError::EmptyEvidence { role });
    }
    Ok(())
}

fn non_empty_forbidden(
    anchor: &SourceAnchor,
    role: CssDeclarationEvidenceRole,
) -> Result<(), CssDeclarationContractError> {
    if !anchor.range().is_empty() {
        return Err(CssDeclarationContractError::EvidenceOutsideComplete { role });
    }
    Ok(())
}

fn exact(
    anchor: &SourceAnchor,
    role: CssDeclarationEvidenceRole,
    expected: &'static str,
) -> Result<(), CssDeclarationContractError> {
    if anchor.fragment() != expected {
        return Err(CssDeclarationContractError::FixedSpellingMismatch { role, expected });
    }
    Ok(())
}

fn contained(
    container: &SourceAnchor,
    nested: &SourceAnchor,
    role: CssDeclarationEvidenceRole,
) -> Result<(), CssDeclarationContractError> {
    if container.source_id() != nested.source_id()
        || nested.range().start() < container.range().start()
        || nested.range().end() > container.range().end()
    {
        return Err(CssDeclarationContractError::EvidenceOutsideComplete { role });
    }
    Ok(())
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    fn anchor(source: &SourceText, start: usize, end: usize) -> SourceAnchor {
        source.anchor(start, end).unwrap()
    }

    fn test_placement() -> CssDeclarationPlacement {
        CssDeclarationPlacement::new(
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssDeclarationRunOrdinal::new(0),
        )
    }

    #[test]
    fn basic_semicolon_declaration_constructs_with_exact_evidence() {
        let text = source(1, "color:red;");
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 10),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 9, 10),
            },
            test_placement(),
        )
        .unwrap();

        assert_eq!(occurrence.complete().range().start(), 0);
        assert_eq!(occurrence.complete().range().end(), 10);
        assert!(occurrence.priority().is_none());
        assert_eq!(occurrence.placement(), test_placement());
    }

    #[test]
    fn omitted_before_right_curly_requires_complete_end_at_value_end() {
        let text = source(2, "color:red}");
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 9),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::OmittedBeforeRightCurly {
                right_curly: anchor(&text, 9, 10),
            },
            test_placement(),
        )
        .unwrap();
        assert_eq!(occurrence.complete().range().end(), 9);

        let wrong = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 10),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::OmittedBeforeRightCurly {
                right_curly: anchor(&text, 9, 10),
            },
            test_placement(),
        );
        assert!(matches!(
            wrong,
            Err(CssDeclarationContractError::CompleteEndMismatch { .. })
        ));
    }

    #[test]
    fn omitted_at_end_of_input_requires_true_source_end() {
        let text = source(3, "color:red");
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 9),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::OmittedAtEndOfInput {
                terminal: anchor(&text, 9, 9),
            },
            test_placement(),
        )
        .unwrap();
        assert_eq!(occurrence.complete().range().end(), 9);
    }

    #[test]
    fn fabricated_eof_terminal_not_at_source_end_is_rejected() {
        let text = source(4, "color:red;more");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 9),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::OmittedAtEndOfInput {
                terminal: anchor(&text, 9, 9),
            },
            test_placement(),
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::FabricatedOmittedEofTerminal)
        );
    }

    #[test]
    fn zero_length_value_must_sit_exactly_at_colon_end() {
        let text = source(5, "color:;");
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 7),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 6),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 6, 7),
            },
            test_placement(),
        )
        .unwrap();
        assert!(occurrence.value().range().is_empty());

        let wrong = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 7),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 7, 7),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 6, 7),
            },
            test_placement(),
        );
        assert_eq!(
            wrong,
            Err(CssDeclarationContractError::ZeroLengthValueNotAtColonEnd)
        );
    }

    #[test]
    fn escaped_property_name_preserves_exact_raw_source_range() {
        let text = source(6, r"c\6F lor/**/:red;");
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 17),
            anchor(&text, 0, 8),
            anchor(&text, 12, 13),
            anchor(&text, 13, 16),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 16, 17),
            },
            test_placement(),
        )
        .unwrap();
        assert_eq!(occurrence.property_name().fragment(), r"c\6F lor");
    }

    #[test]
    fn valid_priority_evidence_preserves_authored_envelope_with_trivia() {
        let text = source(7, "red !/**/Im\\70 ortant");
        let priority = CssDeclarationPriorityEvidence::new(
            &text,
            anchor(&text, 4, 21),
            anchor(&text, 4, 5),
            anchor(&text, 9, 21),
            "Important",
        )
        .unwrap();
        assert_eq!(priority.complete().range(), anchor(&text, 4, 21).range());
        assert_eq!(priority.bang().fragment(), "!");
    }

    #[test]
    fn priority_rejects_wrong_decoded_identifier() {
        let text = source(8, "red !huge");
        let result = CssDeclarationPriorityEvidence::new(
            &text,
            anchor(&text, 4, 9),
            anchor(&text, 4, 5),
            anchor(&text, 5, 9),
            "huge",
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::InvalidPriorityIdentifier)
        );
    }

    #[test]
    fn priority_bang_must_be_exact_fixed_spelling() {
        let text = source(9, "red ~important");
        let result = CssDeclarationPriorityEvidence::new(
            &text,
            anchor(&text, 4, 14),
            anchor(&text, 4, 5),
            anchor(&text, 5, 14),
            "important",
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::FixedSpellingMismatch {
                role: CssDeclarationEvidenceRole::PriorityBang,
                expected: "!",
            })
        );
    }

    #[test]
    fn declaration_with_valid_priority_requires_no_value_priority_overlap() {
        let text = source(10, "color:red !important;");
        let priority = CssDeclarationPriorityEvidence::new(
            &text,
            anchor(&text, 10, 20),
            anchor(&text, 10, 11),
            anchor(&text, 11, 20),
            "important",
        )
        .unwrap();
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 21),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            Some(priority),
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 20, 21),
            },
            test_placement(),
        )
        .unwrap();
        assert!(occurrence.priority().is_some());
    }

    #[test]
    fn value_overlapping_priority_is_rejected() {
        let text = source(11, "color:red!important;");
        let priority = CssDeclarationPriorityEvidence::new(
            &text,
            anchor(&text, 9, 19),
            anchor(&text, 9, 10),
            anchor(&text, 10, 19),
            "important",
        )
        .unwrap();
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 20),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 10),
            Some(priority),
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 19, 20),
            },
            test_placement(),
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::ValueOverlapsPriority)
        );
    }

    #[test]
    fn cross_source_property_name_is_rejected() {
        let text = source(12, "color:red;");
        let other = source(13, "color:red;");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 10),
            anchor(&other, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 9, 10),
            },
            test_placement(),
        );
        assert!(matches!(
            result,
            Err(CssDeclarationContractError::SourceIdentityMismatch {
                role: CssDeclarationEvidenceRole::PropertyName,
                ..
            })
        ));
    }

    #[test]
    fn complete_start_must_equal_property_name_start() {
        let text = source(14, " color:red;");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 11),
            anchor(&text, 1, 6),
            anchor(&text, 6, 7),
            anchor(&text, 7, 10),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 10, 11),
            },
            test_placement(),
        );
        assert!(matches!(
            result,
            Err(CssDeclarationContractError::CompleteStartMismatch { .. })
        ));
    }

    #[test]
    fn colon_must_be_exact_fixed_spelling() {
        let text = source(15, "color;red;");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 10),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 9, 10),
            },
            test_placement(),
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::FixedSpellingMismatch {
                role: CssDeclarationEvidenceRole::Colon,
                expected: ":",
            })
        );
    }

    #[test]
    fn semicolon_must_be_exact_fixed_spelling() {
        let text = source(16, "color:red,");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 10),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 9, 10),
            },
            test_placement(),
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::FixedSpellingMismatch {
                role: CssDeclarationEvidenceRole::Semicolon,
                expected: ";",
            })
        );
    }

    #[test]
    fn right_curly_must_be_exact_fixed_spelling() {
        let text = source(17, "color:red;");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 9),
            anchor(&text, 0, 5),
            anchor(&text, 5, 6),
            anchor(&text, 6, 9),
            None,
            CssDeclarationTermination::OmittedBeforeRightCurly {
                right_curly: anchor(&text, 9, 10),
            },
            test_placement(),
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::FixedSpellingMismatch {
                role: CssDeclarationEvidenceRole::RightCurly,
                expected: "}",
            })
        );
    }

    #[test]
    fn empty_property_name_is_rejected() {
        let text = source(18, ":red;");
        let result = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, 5),
            anchor(&text, 0, 0),
            anchor(&text, 0, 1),
            anchor(&text, 1, 4),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, 4, 5),
            },
            test_placement(),
        );
        assert_eq!(
            result,
            Err(CssDeclarationContractError::EmptyEvidence {
                role: CssDeclarationEvidenceRole::PropertyName,
            })
        );
    }

    #[test]
    fn debug_output_never_discloses_authored_source_fragments() {
        const SECRET_PROPERTY: &str = "top-secret-property-name";
        // "zqx" (rather than a real value like "red") avoids false-positive
        // substring collisions with unrelated Debug output such as the
        // "AuthoredSemicolon" variant name, which itself contains "red".
        let source_text = format!("{SECRET_PROPERTY}:zqx;");
        let text = source(19, &source_text);
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, source_text.len()),
            anchor(&text, 0, SECRET_PROPERTY.len()),
            anchor(&text, SECRET_PROPERTY.len(), SECRET_PROPERTY.len() + 1),
            anchor(&text, SECRET_PROPERTY.len() + 1, SECRET_PROPERTY.len() + 4),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, SECRET_PROPERTY.len() + 4, SECRET_PROPERTY.len() + 5),
            },
            test_placement(),
        )
        .unwrap();

        let debug = format!("{occurrence:?}");
        assert!(!debug.contains(SECRET_PROPERTY));
        assert!(!debug.contains("zqx"));

        let error = CssDeclarationOccurrence::new(
            &text,
            anchor(&text, 0, source_text.len()),
            anchor(&text, 0, 0),
            anchor(&text, SECRET_PROPERTY.len(), SECRET_PROPERTY.len() + 1),
            anchor(&text, SECRET_PROPERTY.len() + 1, SECRET_PROPERTY.len() + 4),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: anchor(&text, SECRET_PROPERTY.len() + 4, SECRET_PROPERTY.len() + 5),
            },
            test_placement(),
        )
        .unwrap_err();
        assert!(!format!("{error:?}").contains(SECRET_PROPERTY));
        assert!(!error.to_string().contains(SECRET_PROPERTY));
    }
}
