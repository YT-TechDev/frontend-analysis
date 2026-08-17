//! Bounded escaped `BindingIdentifier` support for the selected production slice.
//!
//! This is not a general ECMAScript Identifier abstraction. It owns only
//! `UnicodeEscapeSequence` formation, selected identifier-position membership,
//! and the existing unconditionally-reserved-word classifier needed by the
//! selected lexical/static-semantics owners.

use super::unicode::{is_id_continue, is_id_start};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FormedUnicodeEscape {
    pub(super) end: usize,
    pub(super) code_point: u32,
}

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn parse_braced_code_point(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() {
        return None;
    }

    let first_significant = digits.iter().position(|byte| *byte != b'0');
    let significant = match first_significant {
        Some(index) => &digits[index..],
        None => return Some(0),
    };

    // This bound follows from the mathematical maximum Code Point value after
    // leading zeroes are removed. It is not an authored UES digit-count limit.
    if significant.len() > 6 {
        return None;
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value
            .checked_mul(16)?
            .checked_add(ascii_hex_value(*byte)?)?;
    }

    (value <= 0x10_FFFF).then_some(value)
}

pub(super) fn formed_unicode_escape_at(source: &str, start: usize) -> Option<FormedUnicodeEscape> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return None;
    }

    let payload = start.checked_add(2)?;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload.checked_add(1)?;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end += 1;
        }
        if bytes.get(end) != Some(&b'}') {
            return None;
        }
        let code_point = parse_braced_code_point(bytes.get(digits_start..end)?)?;
        return Some(FormedUnicodeEscape {
            end: end + 1,
            code_point,
        });
    }

    let end = payload.checked_add(4)?;
    let digits = bytes.get(payload..end)?;
    if !digits.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }

    let mut code_point = 0_u32;
    for byte in digits {
        code_point = code_point * 16 + ascii_hex_value(*byte)?;
    }

    Some(FormedUnicodeEscape { end, code_point })
}

/// Returns the exact authored end of one #227-owned grammar-invalid escape
/// spelling once the spelling is stable enough to support a source verdict.
///
/// This deliberately does not classify every malformed `UnicodeEscapeSequence`.
/// Adjacent malformed forms deferred by #229 remain outside this helper.
pub(super) fn selected_grammar_escape_subject_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let u = start.checked_add(1)?;
    if bytes.get(start) != Some(&b'\\') || bytes.get(u) != Some(&b'u') {
        return None;
    }

    let payload = start.checked_add(2)?;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload.checked_add(1)?;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end = end.checked_add(1)?;
        }

        match bytes.get(end) {
            Some(b'}') => {
                let digits = bytes.get(digits_start..end)?;
                if digits.is_empty() || parse_braced_code_point(digits).is_none() {
                    return end.checked_add(1);
                }
                None
            }
            None if end > digits_start => Some(end),
            _ => None,
        }
    } else {
        let mut end = payload;
        while end
            .checked_sub(payload)
            .is_some_and(|digit_count| digit_count < 4)
            && bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit())
        {
            end = end.checked_add(1)?;
        }

        let digit_count = end.checked_sub(payload)?;
        if (1..4).contains(&digit_count) && bytes.get(end) == Some(&b';') {
            Some(end)
        } else {
            None
        }
    }
}

pub(super) fn is_selected_identifier_start(code_point: u32) -> bool {
    matches!(code_point, 0x24 | 0x5F) || is_id_start(code_point)
}

pub(super) fn is_selected_identifier_part(code_point: u32) -> bool {
    is_selected_identifier_start(code_point)
        || is_id_continue(code_point)
        || matches!(code_point, 0x200C | 0x200D)
}

pub(super) fn is_unconditionally_reserved_word(spelling: &str) -> bool {
    matches!(
        spelling,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formation_preserves_surrogate_values_before_positional_validation() {
        assert_eq!(
            formed_unicode_escape_at(r"\uD800", 0),
            Some(FormedUnicodeEscape {
                end: 6,
                code_point: 0xD800,
            })
        );
        assert_eq!(
            formed_unicode_escape_at(r"\u{D800}", 0),
            Some(FormedUnicodeEscape {
                end: 8,
                code_point: 0xD800,
            })
        );
        assert!(!is_selected_identifier_start(0xD800));
        assert!(!is_selected_identifier_part(0xD800));
    }

    #[test]
    fn malformed_and_non_code_point_escapes_do_not_form() {
        for source in [r"\u{}", r"\u0", r"\u{61", r"\u{110000}"] {
            assert_eq!(formed_unicode_escape_at(source, 0), None, "{source}");
        }
    }

    #[test]
    fn bounded_grammar_escape_classifier_retains_truthful_authored_end() {
        for (source, expected_end) in [
            (r"\u{};", 4),
            (r"\u0;", 3),
            (r"\u{61", 5),
            (r"\u{110000};", 10),
            (r"\u{000000110000};", 16),
        ] {
            assert_eq!(
                selected_grammar_escape_subject_end(source, 0),
                Some(expected_end),
                "{source}"
            );
        }
    }

    #[test]
    fn bounded_grammar_escape_classifier_does_not_widen_deferred_adjacent_forms() {
        for source in [r"\u{G}", r"\u00G0", r"\u{61;", r"\u0x", r"\u", r"\u{"] {
            assert_eq!(
                selected_grammar_escape_subject_end(source, 0),
                None,
                "{source}"
            );
        }
    }

    #[test]
    fn bounded_grammar_escape_classifier_does_not_claim_formed_ues() {
        for source in [r"\u0061;", r"\u0030;", r"\uD800;", r"\u{D800};"] {
            assert!(formed_unicode_escape_at(source, 0).is_some(), "{source}");
            assert_eq!(
                selected_grammar_escape_subject_end(source, 0),
                None,
                "{source}"
            );
        }
    }

    #[test]
    fn selected_identifier_membership_includes_ecmascript_additions() {
        assert!(is_selected_identifier_start('$' as u32));
        assert!(is_selected_identifier_start('_' as u32));
        assert!(is_selected_identifier_part('$' as u32));
        assert!(is_selected_identifier_part('_' as u32));
        assert!(is_selected_identifier_part(0x200C));
        assert!(is_selected_identifier_part(0x200D));
        assert!(is_selected_identifier_start(0x1D49C));
    }

    #[test]
    fn long_braced_escapes_have_no_authored_digit_cap() {
        let permanent = formed_unicode_escape_at(r"\u{00000061}", 0).expect("long UES");
        assert_eq!(permanent.code_point, 0x61);
        assert_eq!(permanent.end, 12);

        let zeros = "0".repeat(4096);
        let source = format!(r"\u{{{zeros}61}}");
        let large = formed_unicode_escape_at(&source, 0).expect("large leading-zero UES");
        assert_eq!(large.code_point, 0x61);
        assert_eq!(large.end, source.len());
    }
}
