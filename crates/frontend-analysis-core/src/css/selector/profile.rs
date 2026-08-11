//! Frozen first-slice selector grammar profile (#182).
//!
//! The registry is intentionally named `CoreV1`, not `Selectors4`: it is a
//! bounded browser-independent qualification profile pinned to the inspected
//! primary specifications and does not claim complete standards or user-agent
//! validity.

/// Selectors Level 4 Editor's Draft inspected for #182.
pub(crate) const SELECTORS_LEVEL_4_ED_REVISION: &str = "2026-07-22";
/// CSS Nesting Module Level 1 Editor's Draft inspected for #182.
pub(crate) const CSS_NESTING_LEVEL_1_ED_REVISION: &str = "2026-02-05";
/// CSS Namespaces Module Level 3 Editor's Draft inspected for #182.
pub(crate) const CSS_NAMESPACES_LEVEL_3_ED_REVISION: &str = "2024-10-25";
/// CSS Syntax Module Level 3 Editor's Draft inspected for #182.
pub(crate) const CSS_SYNTAX_LEVEL_3_ED_REVISION: &str = "2026-07-30";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorGrammarProfile {
    CoreV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorFunctionalPseudoClass {
    Is,
    Where,
    Not,
    Has,
}

/// Identifier-form pseudo-classes explicitly included by `CoreV1`.
///
/// Functional pseudo-classes (`:dir()`, `:lang()`, `:nth-*()`, and others)
/// are deliberately absent. Pseudo-elements are a separate deferred feature
/// family. Names are matched ASCII-case-insensitively by
/// [`is_core_v1_identifier_pseudo_class`].
pub(crate) const CORE_V1_IDENTIFIER_PSEUDO_CLASSES: &[&str] = &[
    "defined",
    "any-link",
    "link",
    "visited",
    "target",
    "scope",
    "hover",
    "active",
    "focus",
    "focus-visible",
    "focus-within",
    "playing",
    "paused",
    "seeking",
    "buffering",
    "stalled",
    "muted",
    "volume-locked",
    "open",
    "popover-open",
    "modal",
    "fullscreen",
    "picture-in-picture",
    "enabled",
    "disabled",
    "read-only",
    "read-write",
    "placeholder-shown",
    "autofill",
    "default",
    "checked",
    "unchecked",
    "indeterminate",
    "valid",
    "invalid",
    "in-range",
    "out-of-range",
    "required",
    "optional",
    "user-valid",
    "user-invalid",
    "root",
    "empty",
    "first-child",
    "last-child",
    "only-child",
    "first-of-type",
    "last-of-type",
    "only-of-type",
];

pub(crate) fn is_core_v1_identifier_pseudo_class(name: &str) -> bool {
    CORE_V1_IDENTIFIER_PSEUDO_CLASSES
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}

pub(crate) fn core_v1_functional_pseudo_class(
    name: &str,
) -> Option<CssSelectorFunctionalPseudoClass> {
    if name.eq_ignore_ascii_case("is") {
        Some(CssSelectorFunctionalPseudoClass::Is)
    } else if name.eq_ignore_ascii_case("where") {
        Some(CssSelectorFunctionalPseudoClass::Where)
    } else if name.eq_ignore_ascii_case("not") {
        Some(CssSelectorFunctionalPseudoClass::Not)
    } else if name.eq_ignore_ascii_case("has") {
        Some(CssSelectorFunctionalPseudoClass::Has)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn pinned_spec_revisions_are_explicit() {
        assert_eq!(SELECTORS_LEVEL_4_ED_REVISION, "2026-07-22");
        assert_eq!(CSS_NESTING_LEVEL_1_ED_REVISION, "2026-02-05");
        assert_eq!(CSS_NAMESPACES_LEVEL_3_ED_REVISION, "2024-10-25");
        assert_eq!(CSS_SYNTAX_LEVEL_3_ED_REVISION, "2026-07-30");
    }

    #[test]
    fn identifier_registry_is_unique_and_ascii_lowercase() {
        let mut seen = BTreeSet::new();
        for name in CORE_V1_IDENTIFIER_PSEUDO_CLASSES {
            assert_eq!(*name, name.to_ascii_lowercase());
            assert!(seen.insert(*name), "duplicate CoreV1 pseudo-class: {name}");
        }
    }

    #[test]
    fn identifier_registry_is_case_insensitive_but_bounded() {
        assert!(is_core_v1_identifier_pseudo_class("hover"));
        assert!(is_core_v1_identifier_pseudo_class("FOCUS-VISIBLE"));
        assert!(is_core_v1_identifier_pseudo_class("first-child"));
        assert!(!is_core_v1_identifier_pseudo_class("lang"));
        assert!(!is_core_v1_identifier_pseudo_class("nth-child"));
        assert!(!is_core_v1_identifier_pseudo_class("future-pseudo"));
    }

    #[test]
    fn exactly_four_functional_pseudo_classes_are_selected() {
        assert_eq!(
            core_v1_functional_pseudo_class("is"),
            Some(CssSelectorFunctionalPseudoClass::Is)
        );
        assert_eq!(
            core_v1_functional_pseudo_class("WHERE"),
            Some(CssSelectorFunctionalPseudoClass::Where)
        );
        assert_eq!(
            core_v1_functional_pseudo_class("not"),
            Some(CssSelectorFunctionalPseudoClass::Not)
        );
        assert_eq!(
            core_v1_functional_pseudo_class("has"),
            Some(CssSelectorFunctionalPseudoClass::Has)
        );
        assert_eq!(core_v1_functional_pseudo_class("nth-child"), None);
        assert_eq!(core_v1_functional_pseudo_class("lang"), None);
    }
}
