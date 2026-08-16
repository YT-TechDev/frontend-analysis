//! Candidate-independent focused rule evidence for Issue #216/#222.
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
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        primary_rule_id: "EE-01-R01",
        primary_subject: GoldRange::new(4, 10),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-PART-HYPHEN-001",
        primary_rule_id: "EE-01-R02",
        primary_subject: GoldRange::new(5, 11),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-FIXED-001",
        primary_rule_id: "EE-01-R01",
        primary_subject: GoldRange::new(4, 10),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-BRACED-001",
        primary_rule_id: "EE-01-R01",
        primary_subject: GoldRange::new(4, 12),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        primary_rule_id: "EE-04-R08",
        primary_subject: GoldRange::new(4, 11),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-ESCAPED-LET-BINDING-001",
        primary_rule_id: "EE-15-R01",
        primary_subject: GoldRange::new(4, 12),
        supporting_subjects: NO_SUPPORTING_SUBJECTS,
        co_trigger_rule_ids: NO_CO_TRIGGERS,
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-ESCAPED-DUPBOUNDNAMES-001",
        primary_rule_id: "EE-15-R02",
        primary_subject: GoldRange::new(12, 13),
        supporting_subjects: &[GoldRange::new(4, 10)],
        co_trigger_rule_ids: &["EE-36-R01"],
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-DOLLAR-ESCAPED-DUPBOUNDNAMES-001",
        primary_rule_id: "EE-15-R02",
        primary_subject: GoldRange::new(7, 13),
        supporting_subjects: &[GoldRange::new(4, 5)],
        co_trigger_rule_ids: &["EE-36-R01"],
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-UNDERSCORE-ESCAPED-DUPBOUNDNAMES-001",
        primary_rule_id: "EE-15-R02",
        primary_subject: GoldRange::new(7, 13),
        supporting_subjects: &[GoldRange::new(4, 5)],
        co_trigger_rule_ids: &["EE-36-R01"],
    },
    FocusedRuleEvidence {
        fixture_id: "JS-GOLD-LEXDECL-SUPPLEMENTARY-ESCAPED-DUPBOUNDNAMES-001",
        primary_rule_id: "EE-15-R02",
        primary_subject: GoldRange::new(15, 19),
        supporting_subjects: &[GoldRange::new(4, 13)],
        co_trigger_rule_ids: &["EE-36-R01"],
    },
];
