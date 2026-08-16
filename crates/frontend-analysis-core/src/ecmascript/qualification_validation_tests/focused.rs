//! Candidate-independent focused rule evidence for Issue #216.
//!
//! Whole-standard qualification remains owned by `GoldFixture`. This layer
//! records only the additional rule ownership/support needed when multiple
//! Early Errors can trigger for the same source.

use super::model::GoldRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FocusedRuleEvidence {
    pub(crate) fixture_id: &'static str,
    pub(crate) primary_rule_id: &'static str,
    pub(crate) primary_subject: GoldRange,
    pub(crate) supporting_subjects: &'static [GoldRange],
    pub(crate) co_trigger_rule_ids: &'static [&'static str],
}

const NO_SUPPORTING_SUBJECTS: &[GoldRange] = &[];
const NO_CO_TRIGGERS: &[&str] = &[];

pub(crate) const FOCUSED_RULE_EVIDENCE: &[FocusedRuleEvidence] = &[
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        primary_rule_id: "EE-15-R02",
        primary_subject: GoldRange::new(7, 8),
        supporting_subjects: &[GoldRange::new(4, 5)],
        co_trigger_rule_ids: &["EE-36-R01"],
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        primary_rule_id: "EE-15-R03",
        primary_subject: GoldRange::new(6, 7),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001",
        primary_rule_id: "EE-36-R01",
        primary_subject: GoldRange::new(14, 15),
        supporting_subjects: &[GoldRange::new(7, 8)],
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-CONST-LET-MISSING-INIT-001",
        primary_rule_id: "EE-15-R01",
        primary_subject: GoldRange::new(6, 9),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: &["EE-15-R03"],
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-CONST-DUP-MISSING-INIT-001",
        primary_rule_id: "EE-15-R02",
        primary_subject: GoldRange::new(13, 14),
        supporting_subjects: &[GoldRange::new(6, 7)],
        co_trigger_rule_ids: &["EE-15-R03", "EE-36-R01"],
    },
];
