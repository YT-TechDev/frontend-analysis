use std::cmp::Ordering;

use super::unicode_generated::{
    BINARY_PROPERTIES_OF_STRINGS, BINARY_PROPERTY_ALIASES, GENERAL_CATEGORY_ALIASES,
    ID_CONTINUE_RANGES, ID_START_RANGES, NONBINARY_PROPERTY_ALIASES, SCRIPT_ALIASES,
    SPACE_SEPARATOR_RANGES,
};

fn contains_code_point(ranges: &[(u32, u32)], code_point: u32) -> bool {
    if code_point > 0x10_FFFF {
        return false;
    }

    ranges
        .binary_search_by(|&(start, end)| {
            if end < code_point {
                Ordering::Less
            } else if start > code_point {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
        .is_ok()
}

fn canonical_alias(
    aliases: &[(&'static str, &'static str)],
    spelling: &str,
) -> Option<&'static str> {
    aliases
        .binary_search_by(|entry| entry.0.cmp(spelling))
        .ok()
        .map(|index| aliases[index].1)
}

pub(super) fn is_id_start(code_point: u32) -> bool {
    contains_code_point(ID_START_RANGES, code_point)
}

pub(super) fn is_id_continue(code_point: u32) -> bool {
    contains_code_point(ID_CONTINUE_RANGES, code_point)
}

pub(super) fn is_space_separator(code_point: u32) -> bool {
    contains_code_point(SPACE_SEPARATOR_RANGES, code_point)
}

pub(super) fn canonical_general_category_value(spelling: &str) -> Option<&'static str> {
    canonical_alias(GENERAL_CATEGORY_ALIASES, spelling)
}

pub(super) fn canonical_script_value(spelling: &str) -> Option<&'static str> {
    canonical_alias(SCRIPT_ALIASES, spelling)
}

pub(super) fn canonical_nonbinary_property(spelling: &str) -> Option<&'static str> {
    canonical_alias(NONBINARY_PROPERTY_ALIASES, spelling)
}

pub(super) fn canonical_binary_property(spelling: &str) -> Option<&'static str> {
    canonical_alias(BINARY_PROPERTY_ALIASES, spelling)
}

pub(super) fn is_binary_property_of_strings(spelling: &str) -> bool {
    BINARY_PROPERTIES_OF_STRINGS.contains(&spelling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_is_unicode_only() {
        assert!(is_id_start('A' as u32));
        assert!(is_id_start('π' as u32));
        assert!(!is_id_start('$' as u32));
        assert!(!is_id_start('_' as u32));
        assert!(!is_id_start('0' as u32));

        assert!(is_id_continue('0' as u32));
        assert!(is_id_continue('_' as u32));
        assert!(is_id_continue('\u{0301}' as u32));
        assert!(!is_id_continue('$' as u32));
        assert!(!is_id_continue(0x11_0000));
    }

    #[test]
    fn space_separator_is_not_generic_whitespace() {
        assert!(is_space_separator(' ' as u32));
        assert!(is_space_separator('\u{00A0}' as u32));
        assert!(is_space_separator('\u{3000}' as u32));
        assert!(!is_space_separator('\t' as u32));
        assert!(!is_space_separator('\n' as u32));
    }

    #[test]
    fn aliases_are_exact_and_case_sensitive() {
        assert_eq!(
            canonical_general_category_value("Lu"),
            Some("Uppercase_Letter")
        );
        assert_eq!(
            canonical_general_category_value("Uppercase_Letter"),
            Some("Uppercase_Letter")
        );
        assert_eq!(canonical_general_category_value("lu"), None);

        assert_eq!(canonical_script_value("Latn"), Some("Latin"));
        assert_eq!(canonical_script_value("Latin"), Some("Latin"));
        assert_eq!(canonical_script_value("latn"), None);

        assert_eq!(canonical_nonbinary_property("gc"), Some("General_Category"));
        assert_eq!(
            canonical_nonbinary_property("scx"),
            Some("Script_Extensions")
        );
        assert_eq!(canonical_nonbinary_property("GC"), None);

        assert_eq!(canonical_binary_property("Alpha"), Some("Alphabetic"));
        assert_eq!(canonical_binary_property("Alphabetic"), Some("Alphabetic"));
        assert_eq!(canonical_binary_property("alpha"), None);
    }

    #[test]
    fn string_property_classification_is_exact() {
        for spelling in [
            "Basic_Emoji",
            "Emoji_Keycap_Sequence",
            "RGI_Emoji_Modifier_Sequence",
            "RGI_Emoji_Flag_Sequence",
            "RGI_Emoji_Tag_Sequence",
            "RGI_Emoji_ZWJ_Sequence",
            "RGI_Emoji",
        ] {
            assert!(is_binary_property_of_strings(spelling));
        }
        assert!(!is_binary_property_of_strings("Emoji"));
        assert!(!is_binary_property_of_strings("rgi_emoji"));
    }

    #[test]
    fn generated_data_records_frozen_source_versions() {
        assert_eq!(super::super::unicode_generated::UNICODE_VERSION, "17.0.0");
        assert_eq!(
            super::super::unicode_generated::ECMA262_SNAPSHOT,
            "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
        );
    }
}
