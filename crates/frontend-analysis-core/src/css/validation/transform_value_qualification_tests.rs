use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssExponentSign, CssNumberSign, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransformFunction, CssTransformQualificationOutcome, CssTransformRotate3dAngleArgument,
    CssTransformRotate3dFunction, CssTransformScaleArgumentKind,
    CssTransformTranslate3dXyArgumentKind, CssTransformTranslateArgumentKind,
    CssTransformUnsupportedReason, CssTransformValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
}

fn parser_limits() -> CssParserLimits {
    parser_limits_with_occurrences(8192)
}

fn parser_limits_with_occurrences(max_declaration_occurrences: usize) -> CssParserLimits {
    CssParserLimits::new(
        100_000,
        256,
        256,
        max_declaration_occurrences,
        1024,
        1024,
        1024,
        1024,
        8192,
    )
    .unwrap()
}

fn qualify(source_id: u64, css: &str) -> CssValueQualificationRunResult {
    qualify_with_limits(source_id, css, parser_limits())
}

fn qualify_with_limits(
    source_id: u64,
    css: &str,
    parser_limits: CssParserLimits,
) -> CssValueQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), css.to_owned());
    let parser_result = analyze_css_source(&source, tokenizer_limits(), parser_limits).unwrap();
    run(parser_result).unwrap()
}

fn outcome_at(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &CssTransformQualificationOutcome {
    result
        .transform_observations()
        .get(index)
        .unwrap_or_else(|| panic!("missing transform observation at {index}"))
        .outcome()
}

fn assert_whole_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformQualificationOutcome::Qualified(CssTransformValue::None),
        "expected whole-value none at {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected selected-profile invalidity at {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssTransformUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected unsupported selected-profile coverage at {index}"
    );
}

fn qualified_functions(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssTransformFunction] {
    match outcome_at(result, index) {
        CssTransformQualificationOutcome::Qualified(CssTransformValue::Functions(functions)) => {
            functions
        }
        other => panic!("expected qualified transform components at {index}, got {other:?}"),
    }
}

/// Reconstructs one retained `Number` token's authored numeric structure
/// from tokenizer-owned evidence alone -- sign spelling, integer digits,
/// fraction digits, and exponent sign/digits -- so `1`, `1.0`, `1e0`, `+1`,
/// and `-0` stay distinguishable in assertions. The exponent's authored
/// `e`/`E` letter case is not part of retained numeric structure and is
/// therefore not reconstructed here. No machine float is ever produced.
fn authored_number_spelling(token: &CssTokenKind) -> String {
    let CssTokenKind::Number { value, .. } = token else {
        panic!("matrix argument evidence did not resolve to a Number token");
    };

    let mut spelling = String::new();
    match value.sign() {
        Some(CssNumberSign::Plus) => spelling.push('+'),
        Some(CssNumberSign::Minus) => spelling.push('-'),
        None => {}
    }
    spelling.push_str(value.decimal().integer_digits());
    let fraction_digits = value.decimal().fraction_digits();
    if !fraction_digits.is_empty() {
        spelling.push('.');
        spelling.push_str(fraction_digits);
    }
    if let Some(exponent) = value.decimal().exponent() {
        spelling.push('e');
        match exponent.sign() {
            Some(CssExponentSign::Plus) => spelling.push('+'),
            Some(CssExponentSign::Minus) => spelling.push('-'),
            None => {}
        }
        spelling.push_str(exponent.digits());
    }
    spelling
}

fn matrix_argument_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> Vec<String> {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Matrix(matrix) = function else {
        panic!("expected matrix component {function_index} at {index}, got {function:?}");
    };
    matrix
        .arguments()
        .iter()
        .map(|evidence| {
            let token = result
                .transform_matrix_argument_token(*evidence)
                .expect("matrix argument evidence did not resolve");
            authored_number_spelling(token)
        })
        .collect()
}

/// Reconstructs one retained `Number` or `Percentage` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_number_spelling`, with a trailing `%` marker distinguishing a
/// `Percentage` token so `2` and `2%` stay distinguishable in assertions
/// and a `Percentage` is never collapsed into a `Number`. No machine
/// number is ever produced.
fn authored_scale_argument_spelling(token: &CssTokenKind) -> String {
    let (value, is_percentage) = match token {
        CssTokenKind::Number { value, .. } => (value, false),
        CssTokenKind::Percentage { value } => (value, true),
        other => panic!(
            "scale argument evidence did not resolve to a Number/Percentage token, got {other:?}"
        ),
    };

    let mut spelling = String::new();
    match value.sign() {
        Some(CssNumberSign::Plus) => spelling.push('+'),
        Some(CssNumberSign::Minus) => spelling.push('-'),
        None => {}
    }
    spelling.push_str(value.decimal().integer_digits());
    let fraction_digits = value.decimal().fraction_digits();
    if !fraction_digits.is_empty() {
        spelling.push('.');
        spelling.push_str(fraction_digits);
    }
    if let Some(exponent) = value.decimal().exponent() {
        spelling.push('e');
        match exponent.sign() {
            Some(CssExponentSign::Plus) => spelling.push('+'),
            Some(CssExponentSign::Minus) => spelling.push('-'),
            None => {}
        }
        spelling.push_str(exponent.digits());
    }
    if is_percentage {
        spelling.push('%');
    }
    spelling
}

/// Resolves one qualified `scale()` transform component at `function_index`
/// within observation `index`, preserving authored one-vs-two cardinality
/// as an ordered `(kind, spelling)` vector of length 1 or 2 -- never
/// synthesizing a second entry for an authored one-argument `scale()`.
fn scale_argument_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> Vec<(CssTransformScaleArgumentKind, String)> {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Scale(scale) = function else {
        panic!("expected scale component {function_index} at {index}, got {function:?}");
    };

    let mut arguments = vec![scale.arguments().first()];
    if let Some(second) = scale.arguments().second() {
        arguments.push(second);
    }

    arguments
        .into_iter()
        .map(|argument| {
            let token = result
                .transform_scale_argument_token(argument.evidence_ref())
                .expect("scale argument evidence did not resolve");
            (argument.kind(), authored_scale_argument_spelling(token))
        })
        .collect()
}

/// Reconstructs one retained `Number`, `Dimension`, or `Percentage` token's
/// authored numeric structure from tokenizer-owned evidence alone, exactly
/// like `authored_scale_argument_spelling`, with the authored unit spelling
/// appended for a `Dimension` and a trailing `%` marker for a `Percentage`,
/// so `0`, `0px`, and `0%` stay pairwise distinguishable in assertions and
/// none is ever collapsed into another. No machine number or unit
/// conversion is ever produced.
fn authored_translate3d_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        CssTokenKind::Percentage { value } => (value, "%".to_string()),
        other => panic!(
            "translate3d argument evidence did not resolve to a Number/Dimension/Percentage token, got {other:?}"
        ),
    };

    let mut spelling = String::new();
    match value.sign() {
        Some(CssNumberSign::Plus) => spelling.push('+'),
        Some(CssNumberSign::Minus) => spelling.push('-'),
        None => {}
    }
    spelling.push_str(value.decimal().integer_digits());
    let fraction_digits = value.decimal().fraction_digits();
    if !fraction_digits.is_empty() {
        spelling.push('.');
        spelling.push_str(fraction_digits);
    }
    if let Some(exponent) = value.decimal().exponent() {
        spelling.push('e');
        match exponent.sign() {
            Some(CssExponentSign::Plus) => spelling.push('+'),
            Some(CssExponentSign::Minus) => spelling.push('-'),
            None => {}
        }
        spelling.push_str(exponent.digits());
    }
    spelling.push_str(&suffix);
    spelling
}

/// Resolves one qualified `translate3d()` transform component at
/// `function_index` within observation `index` into its ordered
/// `(x_kind, x, y_kind, y, z)` authored evidence, preserving exact X/Y/Z
/// positional order and X/Y Length-vs-Percentage kind. Z carries no kind
/// because a Z `Percentage` is rejected by `classify_transform_component`
/// before a `CssTransformTranslate3dFunction` is ever constructed, so a Z
/// `Percentage` can never reach this resolver.
fn translate3d_argument_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (
    CssTransformTranslate3dXyArgumentKind,
    String,
    CssTransformTranslate3dXyArgumentKind,
    String,
    String,
) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Translate3d(translate3d) = function else {
        panic!("expected translate3d component {function_index} at {index}, got {function:?}");
    };

    let x = translate3d.x();
    let y = translate3d.y();
    let x_token = result
        .transform_translate3d_argument_token(x.evidence_ref())
        .expect("translate3d X evidence did not resolve");
    let y_token = result
        .transform_translate3d_argument_token(y.evidence_ref())
        .expect("translate3d Y evidence did not resolve");
    let z_token = result
        .transform_translate3d_argument_token(translate3d.z())
        .expect("translate3d Z evidence did not resolve");

    (
        x.kind(),
        authored_translate3d_argument_spelling(x_token),
        y.kind(),
        authored_translate3d_argument_spelling(y_token),
        authored_translate3d_argument_spelling(z_token),
    )
}

/// Reconstructs one retained `Number`, `Dimension`, or `Percentage` token's
/// authored numeric structure from tokenizer-owned evidence alone, exactly
/// like `authored_translate3d_argument_spelling`, so `0`, `0px`, and `0%`
/// stay pairwise distinguishable in assertions and none is ever collapsed
/// into another. This is a dedicated `translate()` test helper, distinct
/// from `authored_translate3d_argument_spelling`: `translate()` and
/// `translate3d()` argument placement remain distinct semantic roles even
/// though the underlying scalar spelling logic is identical. No machine
/// number or unit conversion is ever produced.
fn authored_translate_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        CssTokenKind::Percentage { value } => (value, "%".to_string()),
        other => panic!(
            "translate argument evidence did not resolve to a Number/Dimension/Percentage token, got {other:?}"
        ),
    };

    let mut spelling = String::new();
    match value.sign() {
        Some(CssNumberSign::Plus) => spelling.push('+'),
        Some(CssNumberSign::Minus) => spelling.push('-'),
        None => {}
    }
    spelling.push_str(value.decimal().integer_digits());
    let fraction_digits = value.decimal().fraction_digits();
    if !fraction_digits.is_empty() {
        spelling.push('.');
        spelling.push_str(fraction_digits);
    }
    if let Some(exponent) = value.decimal().exponent() {
        spelling.push('e');
        match exponent.sign() {
            Some(CssExponentSign::Plus) => spelling.push('+'),
            Some(CssExponentSign::Minus) => spelling.push('-'),
            None => {}
        }
        spelling.push_str(exponent.digits());
    }
    spelling.push_str(&suffix);
    spelling
}

/// Resolves one qualified `translate()` transform component at
/// `function_index` within observation `index`, preserving authored
/// one-vs-two cardinality as an ordered `(kind, spelling)` vector of length
/// 1 or 2 -- never synthesizing a second entry for an authored
/// one-argument `translate()`, mirroring `scale_argument_spellings`.
fn translate_argument_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> Vec<(CssTransformTranslateArgumentKind, String)> {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Translate(translate) = function else {
        panic!("expected translate component {function_index} at {index}, got {function:?}");
    };

    let mut arguments = vec![translate.arguments().first()];
    if let Some(second) = translate.arguments().second() {
        arguments.push(second);
    }

    arguments
        .into_iter()
        .map(|argument| {
            let token = result
                .transform_translate_argument_token(argument.evidence_ref())
                .expect("translate argument evidence did not resolve");
            (argument.kind(), authored_translate_argument_spelling(token))
        })
        .collect()
}

/// Reconstructs one retained `Number` or `Dimension` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_number_spelling`, with the authored unit spelling appended for
/// a `Dimension` so `0` and `0deg` stay distinguishable in assertions. No
/// machine number or unit conversion is ever produced.
fn authored_rotate3d_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        other => panic!(
            "rotate3d argument evidence did not resolve to a Number/Dimension token, got {other:?}"
        ),
    };

    let mut spelling = String::new();
    match value.sign() {
        Some(CssNumberSign::Plus) => spelling.push('+'),
        Some(CssNumberSign::Minus) => spelling.push('-'),
        None => {}
    }
    spelling.push_str(value.decimal().integer_digits());
    let fraction_digits = value.decimal().fraction_digits();
    if !fraction_digits.is_empty() {
        spelling.push('.');
        spelling.push_str(fraction_digits);
    }
    if let Some(exponent) = value.decimal().exponent() {
        spelling.push('e');
        match exponent.sign() {
            Some(CssExponentSign::Plus) => spelling.push('+'),
            Some(CssExponentSign::Minus) => spelling.push('-'),
            None => {}
        }
        spelling.push_str(exponent.digits());
    }
    spelling.push_str(&suffix);
    spelling
}

fn rotate3d_function(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> CssTransformRotate3dFunction {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Rotate3d(rotate3d) = function else {
        panic!("expected rotate3d component {function_index} at {index}, got {function:?}");
    };
    *rotate3d
}

/// Resolves one qualified `rotate3d()` transform component's ordered axis
/// evidence at `function_index` within observation `index`, preserving
/// exact X/Y/Z positional order. Axis Numbers are never normalized into a
/// unit vector.
fn rotate3d_axis_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (String, String, String) {
    let rotate3d = rotate3d_function(result, index, function_index);
    let x = result
        .transform_rotate3d_argument_token(rotate3d.x())
        .expect("rotate3d X evidence did not resolve");
    let y = result
        .transform_rotate3d_argument_token(rotate3d.y())
        .expect("rotate3d Y evidence did not resolve");
    let z = result
        .transform_rotate3d_argument_token(rotate3d.z())
        .expect("rotate3d Z evidence did not resolve");
    (
        authored_rotate3d_argument_spelling(x),
        authored_rotate3d_argument_spelling(y),
        authored_rotate3d_argument_spelling(z),
    )
}

/// Resolves one qualified `rotate3d()` transform component's fourth-slot
/// authored role and evidence at `function_index` within observation
/// `index`, returning `(is_angle, spelling)` -- `is_angle` is `true` for
/// the `Angle` branch and `false` for the `Zero` branch, so `0` and `0deg`
/// stay distinguishable by authored role, never merely by spelling.
fn rotate3d_angle_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (bool, String) {
    let rotate3d = rotate3d_function(result, index, function_index);
    match rotate3d.angle() {
        CssTransformRotate3dAngleArgument::Angle(evidence_ref) => {
            let token = result
                .transform_rotate3d_argument_token(evidence_ref)
                .expect("rotate3d angle evidence did not resolve");
            (true, authored_rotate3d_argument_spelling(token))
        }
        CssTransformRotate3dAngleArgument::Zero(evidence_ref) => {
            let token = result
                .transform_rotate3d_argument_token(evidence_ref)
                .expect("rotate3d zero evidence did not resolve");
            (false, authored_rotate3d_argument_spelling(token))
        }
    }
}

fn assert_all_invalid(source_id: u64, values: &[&str]) {
    for (offset, value) in values.iter().enumerate() {
        let css = format!("a{{transform:{value};}}");
        let result = qualify(source_id + offset as u64, &css);
        assert_eq!(
            result.transform_observations().len(),
            1,
            "expected one observation for {value:?}"
        );
        assert_eq!(
            outcome_at(&result, 0),
            &CssTransformQualificationOutcome::InvalidForSelectedValueGrammar,
            "expected selected-profile invalidity for {value:?}"
        );
    }
}

fn assert_all_unsupported(source_id: u64, reason: CssTransformUnsupportedReason, values: &[&str]) {
    for (offset, value) in values.iter().enumerate() {
        let css = format!("a{{transform:{value};}}");
        let result = qualify(source_id + offset as u64, &css);
        assert_eq!(
            result.transform_observations().len(),
            1,
            "expected one observation for {value:?}"
        );
        assert_eq!(
            outcome_at(&result, 0),
            &CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
            "expected unsupported selected-profile coverage for {value:?}"
        );
    }
}

// 1. Alternatives and exclusivity: `transform = none | <transform-list>`
// is an exclusive alternation, so `none` is a whole-value branch and never
// a `<transform-list>` component in either authored order.

#[test]
fn whole_none_qualifies_and_remains_exclusive() {
    let result = qualify(
        418100,
        concat!(
            "a{transform:none;}",
            "b{transform:NONE;}",
            "c{transform:n\\6f ne;}",
            "d{transform: /**/ none /**/ ;}",
            "e{transform:none matrix(1,0,0,1,0,0);}",
            "f{transform:matrix(1,0,0,1,0,0) none;}",
            "g{transform:none none;}",
            "h{transform:none scale(2);}",
            "i{transform:scale(2) none;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 9);
    assert_whole_none(&result, 0);
    assert_whole_none(&result, 1);
    assert_whole_none(&result, 2);
    assert_whole_none(&result, 3);
    assert_invalid(&result, 4);
    assert_invalid(&result, 5);
    assert_invalid(&result, 6);
    assert_invalid(&result, 7);
    assert_invalid(&result, 8);
}

// 2. `matrix() = matrix(<number>#{6})`: exactly six ordered direct
// `<number>` arguments qualify, and each argument's authored numeric
// identity stays source-backed tokenizer evidence -- never normalized
// through machine floating point, so `1`, `1.0`, and `1e0` stay distinct.

#[test]
fn canonical_matrix_qualifies_with_six_ordered_arguments() {
    let result = qualify(418110, "a{transform:matrix(1,0,0,1,0,0);}");

    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(qualified_functions(&result, 0).len(), 1);
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
}

#[test]
fn authored_numeric_identity_is_preserved_without_normalization() {
    let result = qualify(
        418111,
        concat!(
            "a{transform:matrix(+1,-0,0.0,1e0,10,-10);}",
            "b{transform:matrix(1,1.0,1e0,1e+2,1e-2,.5);}",
            "c{transform:matrix(0,-0,+0,0.0,0e0,00);}",
        ),
    );

    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["+1", "-0", "0.0", "1e0", "10", "-10"]
    );
    // The final argument is authored `.5`: the absent leading integer digit
    // is canonicalized to `0` by the tokenizer's own retained numeric
    // contract, upstream of this leaf. This slice preserves exactly what
    // the tokenizer retained and normalizes nothing further itself.
    assert_eq!(
        matrix_argument_spellings(&result, 1, 0),
        ["1", "1.0", "1e0", "1e+2", "1e-2", "0.5"]
    );
    assert_eq!(
        matrix_argument_spellings(&result, 2, 0),
        ["0", "-0", "+0", "0.0", "0e0", "00"]
    );

    // The five authored spellings the profile must keep distinguishable
    // are pairwise distinct as retained evidence, never collapsed to one
    // interpreted magnitude.
    let distinct = matrix_argument_spellings(&result, 1, 0);
    assert_ne!(distinct[0], distinct[1]);
    assert_ne!(distinct[1], distinct[2]);
    assert_ne!(distinct[0], distinct[2]);
}

// 3. `<transform-list> = <transform-function>+`: repeated components are
// whitespace-separated, preserved in exact authored order, and -- because
// CSS tokenization needs no separating whitespace after `)` -- also
// partition correctly when authored adjacently.

#[test]
fn repeated_matrix_components_preserve_authored_order() {
    let result = qualify(
        418120,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) matrix(2,0,0,2,10,-10);}",
            "b{transform:matrix(2,0,0,2,10,-10) matrix(1,0,0,1,0,0);}",
            "c{transform:matrix(1,0,0,1,0,0) matrix(1,0,0,1,0,0) matrix(3,0,0,3,0,0);}",
        ),
    );

    assert_eq!(qualified_functions(&result, 0).len(), 2);
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_eq!(
        matrix_argument_spellings(&result, 0, 1),
        ["2", "0", "0", "2", "10", "-10"]
    );

    // The reversed authoring order yields the reversed component order:
    // repetition order is authored evidence, never sorted or deduplicated.
    assert_eq!(
        matrix_argument_spellings(&result, 1, 0),
        ["2", "0", "0", "2", "10", "-10"]
    );
    assert_eq!(
        matrix_argument_spellings(&result, 1, 1),
        ["1", "0", "0", "1", "0", "0"]
    );

    assert_eq!(qualified_functions(&result, 2).len(), 3);
    assert_eq!(
        matrix_argument_spellings(&result, 2, 2),
        ["3", "0", "0", "3", "0", "0"]
    );
}

#[test]
fn adjacent_matrix_components_without_whitespace_partition_correctly() {
    let result = qualify(
        418121,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0)matrix(2,0,0,2,10,-10);}",
            "b{transform:matrix(1,0,0,1,0,0)/**/matrix(2,0,0,2,10,-10);}",
        ),
    );

    for index in 0..2 {
        assert_eq!(qualified_functions(&result, index).len(), 2);
        assert_eq!(
            matrix_argument_spellings(&result, index, 0),
            ["1", "0", "0", "1", "0", "0"]
        );
        assert_eq!(
            matrix_argument_spellings(&result, index, 1),
            ["2", "0", "0", "2", "10", "-10"]
        );
    }
}

// 4. `#{6}` is an exact arity: every other directly visible matrix-level
// argument count is decisive selected-profile invalidity.

#[test]
fn matrix_argument_cardinality_other_than_six_is_invalid() {
    assert_all_invalid(
        418130,
        &[
            "matrix()",
            "matrix(1)",
            "matrix(1,2)",
            "matrix(1,2,3,4,5)",
            "matrix(1,2,3,4,5,6,7)",
            "matrix(1,2,3,4,5,6,7,8)",
        ],
    );

    // The adjacent accepted arity is the only qualifying one, sealing the
    // off-by-one boundary from both sides.
    let result = qualify(418139, "a{transform:matrix(1,2,3,4,5,6);}");
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "2", "3", "4", "5", "6"]
    );
}

// 5. `#` is a comma-separated repetition: whitespace is not a separator,
// and an authored-empty argument position is preserved as its own ordered
// slot and then rejected -- never collapsed away into a shorter list.

#[test]
fn matrix_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        418140,
        &[
            "matrix(1 0 0 1 0 0)",
            "matrix(1,0 0,1,0,0)",
            "matrix(,1,0,0,1,0,0)",
            "matrix(1,,0,1,0,0)",
            "matrix(1,0,0,1,0,0,)",
            "matrix(1,0,,,1,0,0)",
            "matrix(,,,,,)",
        ],
    );
}

// 6. `<number>` admits only a direct retained `Number` token: every other
// direct token category at an argument position is a decisive direct
// token-category failure.

#[test]
fn direct_non_number_argument_categories_are_invalid() {
    assert_all_invalid(
        418150,
        &[
            "matrix(1px,0,0,1,0,0)",
            "matrix(50%,0,0,1,0,0)",
            "matrix(foo,0,0,1,0,0)",
            "matrix(\"1\",0,0,1,0,0)",
            "matrix(1,0,0,1,0,1px)",
            "matrix(1,0,0,1,0,50%)",
            "matrix(1,0,0,1,0,none)",
            "matrix(1,0,0,1,0,#1)",
            "matrix(1deg,0,0,1,0,0)",
        ],
    );
}

// 7. An opaque non-deferred Function at an argument position is
// structurally feasible but its validity depends on calculated-value
// semantics this slice does not own, so it stays unsupported. Critically,
// a comma retained inside that nested Function is at a deeper relative
// depth and must never become a matrix argument separator.

#[test]
fn opaque_matrix_argument_function_is_unsupported() {
    assert_all_unsupported(
        418160,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "matrix(calc(1),0,0,1,0,0)",
            "matrix(1,0,0,1,0,calc(1))",
            "matrix(calc(1),calc(2),0,1,0,0)",
            "matrix(min(1,2),0,0,1,0,0)",
            "matrix(1,0,0,1,0,0) matrix(calc(1),0,0,1,0,0)",
        ],
    );
}

#[test]
fn nested_commas_never_change_matrix_arity() {
    // `calc(1,2)` contributes exactly ONE matrix-level slot: its inner
    // comma is at matrix-body relative depth one. Were it leaking, this
    // would be a seven-slot shell and therefore decisively Invalid, so the
    // Unsupported outcome is itself the depth-isolation proof.
    assert_all_unsupported(
        418170,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "matrix(calc(1,2),0,0,1,0,0)",
            "matrix(1,0,0,1,0,calc(1,2))",
            "matrix(calc(1,2,3,4,5,6,7),0,0,1,0,0)",
            "matrix(calc(calc(1,2),3),0,0,1,0,0)",
        ],
    );

    // Nested non-parenthesis blocks isolate their commas identically.
    assert_all_unsupported(
        418180,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "matrix(calc([1,2]),0,0,1,0,0)",
            "matrix(calc({1,2}),0,0,1,0,0)",
        ],
    );

    // A nested comma that genuinely does add a matrix-level slot once the
    // nesting closes is still counted: this rules out "ignore every comma
    // after a Function" as an accidental passing implementation.
    assert_all_invalid(
        418190,
        &["matrix(calc(1,2),0,0,1,0,0,7)", "matrix(calc(1,2),0)"],
    );
}

// 8. Mixed shapes: directly visible structural shell (arity, empty slot)
// and direct token-category failure both outrank opaque unsupported
// semantics found in a sibling slot.

#[test]
fn directly_visible_structure_outranks_opaque_argument_semantics() {
    assert_all_invalid(
        418200,
        &[
            "matrix(calc(1),0)",
            "matrix(calc(1),0,0,1,0,0,2)",
            "matrix(1,,calc(1),1,0,0)",
            "matrix(calc(1),,0,1,0,0)",
            "matrix(1px,calc(1),0,1,0,0)",
            "matrix(calc(1),1px,0,1,0,0)",
            "matrix(calc(1),0,0,1,0,foo)",
            "matrix(calc(1) 1,0,0,1,0,0)",
            "matrix(1 calc(1),0,0,1,0,0)",
            "matrix(calc(1)calc(2),0,0,1,0,0)",
            // Near-miss arity beside an opaque argument: the directly
            // visible five- and seven-slot shells stay decisive, so a
            // structurally feasible but unevaluated `calc()` can never
            // relax the exact `#{6}` boundary by one position in either
            // direction.
            "matrix(calc(1),0,0,1,0)",
            "matrix(1,0,0,1,0,calc(1),0)",
            "matrix(calc(1),0,0,1,0,0,calc(2))",
        ],
    );
}

// 9. Deferred substitution can still change the surrounding token
// sequence, separators, and cardinality, so it is resolved before any
// surrounding shape conclusion -- including an arity that looks decisive.

#[test]
fn deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        418210,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "var(--x)",
            "matrix(var(--x),0)",
            "matrix(var(--x),0,0,1,0,0)",
            "matrix(1,0,0,1,0,0,var(--x))",
            "matrix(1,0,0,1,0,0) var(--x)",
            "var(--x) matrix(1,0,0,1,0,0)",
            "none var(--x)",
            "matrix(calc(var(--x)),0,0,1,0,0)",
            "rotate(var(--x))",
            "env(--x)",
        ],
    );
}

// 10. Outer coverage boundary: every `<transform-function>` other than
// selected `matrix()`/`scale()` stays outside selected-profile coverage
// rather than being decided here, in either authored order and regardless
// of a sibling qualified matrix/scale.

#[test]
fn unselected_transform_functions_remain_outside_selected_profile() {
    assert_all_unsupported(
        418220,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate(1deg)",
            "matrix(1,0,0,1,0,0) rotate(1deg)",
            "rotate(1deg) matrix(1,0,0,1,0,0)",
            "translateY(10px)",
            "scaleX(2)",
            "scaleY(2)",
            "scaleZ(2)",
            "scale3d(1,1,1)",
            "skew(1deg)",
            "perspective(1px)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "unknownfunction(1,0,0,1,0,0)",
            "rotate(1deg) rotate(2deg)",
            "scale(2) rotate(1deg)",
            "rotate(1deg) scale(2)",
        ],
    );
}

// 11. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, so it is decisive invalidity --
// including in the `none, matrix(...)` shape.

#[test]
fn top_level_comma_is_never_a_transform_list_separator() {
    assert_all_invalid(
        418230,
        &[
            "matrix(1,0,0,1,0,0), matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0),matrix(1,0,0,1,0,0)",
            "none, matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0), none",
            "none,none",
            ",matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0),",
            "scale(2), scale(2)",
            "scale(2),matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,0),scale(2)",
            "none, scale(2)",
            "scale(2), none",
            ",scale(2)",
            "scale(2),",
        ],
    );
}

// 12. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `matrix()` extent is qualified from retained
// interior evidence alone. No closing `)` is synthesized, searched for, or
// reconstructed, and this never generalizes into a blanket missing-close
// acceptance: retained material that lands inside a slot still decides it.

#[test]
fn true_stylesheet_eof_ended_matrix_extent_follows_parser_authority() {
    let result = qualify(418240, "a{transform:matrix(1,0,0,1,0,0");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );

    // Negative side: without a closer, later authored material is absorbed
    // into the final slot by real retained structure, so it no longer
    // satisfies exactly one direct Number.
    let absorbed = qualify(418241, "a{transform:matrix(1,0,0,1,0,0 7");
    assert_eq!(
        absorbed.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_invalid(&absorbed, 0);

    // A short EOF-ended extent stays short: EOF never fills missing slots.
    let short = qualify(418242, "a{transform:matrix(1,0,0");
    assert_invalid(&short, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a matrix".
    let trailing = qualify(418243, "a{transform:matrix(1,0,0,1,0,0) 7;}");
    assert_invalid(&trailing, 0);
}

// Same true-EOF parser-authority theorem applied to `scale()` (#645): a
// one- or two-argument body ending at true stylesheet EOF with no authored
// closer still qualifies from retained interior evidence, while a short or
// wrong-arity EOF-ended extent stays invalid -- EOF never fills or repairs
// missing slots.

#[test]
fn true_stylesheet_eof_ended_scale_extent_follows_parser_authority() {
    let one_argument = qualify(645250, "a{transform:scale(2");
    assert_eq!(
        one_argument.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(one_argument.transform_observations().len(), 1);
    assert_eq!(
        scale_argument_spellings(&one_argument, 0, 0),
        vec![(CssTransformScaleArgumentKind::Number, "2".to_string())]
    );

    let two_arguments = qualify(645251, "a{transform:scale(2,100%");
    assert_eq!(
        two_arguments.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(
        scale_argument_spellings(&two_arguments, 0, 0),
        vec![
            (CssTransformScaleArgumentKind::Number, "2".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "100%".to_string()
            ),
        ]
    );

    // Without a closer, later authored material is absorbed into the final
    // slot by real retained structure, so it no longer satisfies exactly
    // one direct Number/Percentage.
    let absorbed = qualify(645252, "a{transform:scale(2 7");
    assert_eq!(
        absorbed.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_invalid(&absorbed, 0);

    // An EOF-ended empty body stays empty: EOF never fills missing slots.
    let empty = qualify(645253, "a{transform:scale(");
    assert_invalid(&empty, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a scale".
    let trailing = qualify(645254, "a{transform:scale(2) 7;}");
    assert_invalid(&trailing, 0);
}

#[test]
fn trivia_never_changes_matrix_slot_interpretation() {
    let result = qualify(
        418250,
        concat!(
            "a{transform:matrix(1,/**/0,0,1,0,0);}",
            "b{transform:matrix(1/**/,0,0,1,0,0);}",
            "c{transform:matrix(1,0,0,1,0,0/**/);}",
            "d{transform:matrix(/**/1,0,0,1,0,0);}",
            "e{transform:matrix( 1 , 0 , 0 , 1 , 0 , 0 );}",
            "f{transform:matrix(1,/*,*/0,0,1,0,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        assert_eq!(
            matrix_argument_spellings(&result, index, 0),
            ["1", "0", "0", "1", "0", "0"],
            "trivia changed slot interpretation at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(418260, &["matrix(1,/**/,0,1,0,0)", "matrix(1,0,0,1,0/**/)"]);
}

#[test]
fn trivia_never_changes_scale_slot_interpretation() {
    let result = qualify(
        645260,
        concat!(
            "a{transform:scale(2,/**/100%);}",
            "b{transform:scale(2/**/,100%);}",
            "c{transform:scale(2,100%/**/);}",
            "d{transform:scale(/**/2,100%);}",
            "e{transform:scale( 2 , 100% );}",
            "f{transform:scale(2,/*,*/100%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        assert_eq!(
            scale_argument_spellings(&result, index, 0),
            vec![
                (CssTransformScaleArgumentKind::Number, "2".to_string()),
                (
                    CssTransformScaleArgumentKind::Percentage,
                    "100%".to_string()
                ),
            ],
            "trivia changed slot interpretation at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(645270, &["scale(2,/**/,100%)", "scale(2,100/**/%)"]);
}

// 13. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser and is never upgraded, reconstructed, or re-scanned here.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(
        418265,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) !important;}",
            "b{transform:matrix(1,0,0,1,0,0)!important;}",
            "c{transform:none !important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_eq!(
        matrix_argument_spellings(&result, 1, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_whole_none(&result, 2);

    for index in 0..3 {
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
}

#[test]
fn nonordinary_contexts_do_not_enter_transform_dispatch() {
    for (source_id, css) in [
        (418270, "@font-face{transform:matrix(1,0,0,1,0,0);}"),
        (418271, "@page{transform:matrix(1,0,0,1,0,0);}"),
        (418272, "@keyframes k{from{transform:matrix(1,0,0,1,0,0);}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transform_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_termination_preserves_committed_prefix_only() {
    let incomplete = qualify_with_limits(
        418280,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0);}",
            "b{transform:matrix(2,0,0,2,0,0);}",
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.transform_observations().len(), 1);
    assert_eq!(
        matrix_argument_spellings(&incomplete, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
}

#[test]
fn unsupported_region_material_never_produces_transform_observations() {
    let result = qualify(
        418290,
        concat!(
            "@media screen{a{transform:matrix(9,0,0,9,0,0);}}",
            "b{transform:matrix(1,0,0,1,0,0);}",
        ),
    );

    assert!(
        !result
            .upstream_parser_result()
            .unsupported_regions()
            .is_empty(),
        "expected retained unsupported-region evidence"
    );

    // Only the ordinary declaration outside the unsupported region is
    // qualified: unsupported-region material is propagated unchanged, never
    // reconstructed into a qualified occurrence.
    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
}

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:none;}",
        "b{transform:matrix(1,0,0,1,0,0) matrix(2,0,0,2,0,0);}",
        "c{transform:matrix(calc(1,2),0,0,1,0,0);}",
        "d{transform:rotate(1deg);}",
        "e{transform:matrix(var(--x),0);}",
        "f{transform:matrix(1,2);}",
        "g{transform:matrix(1,0,0,1,0,0) none;}",
        "h{transform:scale(2) matrix(1,0,0,1,0,0);}",
        "i{transform:scale(calc(1),2);}",
        "j{transform:scaleX(2);}",
        "k{transform:scale(1,2,3);}",
    );

    let first = qualify(418300, css);
    let repeated = qualify(418300, css);
    let another_source = qualify(418301, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_whole_none(&first, 0);
    assert_eq!(qualified_functions(&first, 1).len(), 2);
    assert_unsupported(
        &first,
        2,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_unsupported(
        &first,
        3,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
    );
    assert_unsupported(
        &first,
        4,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
    );
    assert_invalid(&first, 5);
    assert_invalid(&first, 6);
    assert_eq!(qualified_functions(&first, 7).len(), 2);
    assert_unsupported(
        &first,
        8,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_unsupported(
        &first,
        9,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
    );
    assert_invalid(&first, 10);

    // Scale evidence lookup remains deterministic and resolves to the exact
    // retained Number/Percentage tokens across repeated and cross-source
    // runs, exactly like matrix evidence lookup above.
    assert_eq!(
        scale_argument_spellings(&first, 7, 0),
        scale_argument_spellings(&repeated, 7, 0)
    );
    assert_eq!(
        scale_argument_spellings(&first, 7, 0),
        scale_argument_spellings(&another_source, 7, 0)
    );
}

// Adversarial sealing: outcome precedence must follow evidence authority,
// never which component or token the walk happened to reach first.

#[test]
fn invalid_unsupported_precedence_is_scan_order_independent() {
    // Decisive invalidity outranks unsupported coverage in both orders.
    assert_all_invalid(
        418310,
        &[
            "rotate(1deg) matrix(1,2)",
            "matrix(1,2) rotate(1deg)",
            "calc(1) matrix(1,2)",
            "matrix(1,2) calc(1)",
            "matrix(calc(1),0,0,1,0,0) matrix(1,2)",
            "matrix(1,2) matrix(calc(1),0,0,1,0,0)",
        ],
    );

    // Between the two unsupported reasons the coarser outer grammar level
    // is reported first, identically in both authored orders.
    assert_all_unsupported(
        418320,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "matrix(calc(1),0,0,1,0,0) rotate(1deg)",
            "rotate(1deg) matrix(calc(1),0,0,1,0,0)",
        ],
    );

    // Deferred substitution outranks both, in every authored position.
    assert_all_unsupported(
        418330,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "var(--x) rotate(1deg) matrix(1,2)",
            "matrix(1,2) rotate(1deg) var(--x)",
            "rotate(1deg) matrix(var(--x),0) matrix(1,2)",
        ],
    );
}

#[test]
fn whole_value_boundaries_are_preserved() {
    assert_all_unsupported(
        418340,
        CssTransformUnsupportedReason::CssWideKeyword,
        &["inherit", "initial", "unset", "revert", "REVERT-LAYER"],
    );

    assert_all_unsupported(
        418350,
        CssTransformUnsupportedReason::WholeValueFunction,
        &["first-valid(matrix(1,0,0,1,0,0))", "cycle(none)"],
    );

    // A whole-value-only Function has no independent meaning embedded in a
    // `<transform-list>` or at a matrix/scale argument position.
    assert_all_invalid(
        418360,
        &[
            "matrix(1,0,0,1,0,0) first-valid(none)",
            "matrix(first-valid(1),0,0,1,0,0)",
            "scale(2) first-valid(none)",
            "scale(first-valid(1))",
        ],
    );

    // A CSS-wide keyword is a whole-value branch only.
    assert_all_invalid(
        418370,
        &[
            "inherit matrix(1,0,0,1,0,0)",
            "matrix(1,0,0,1,0,inherit)",
            "inherit scale(2)",
            "scale(inherit)",
        ],
    );
}

#[test]
fn matrix_function_name_recognition_is_ascii_case_insensitive() {
    let result = qualify(
        418380,
        concat!(
            "a{transform:MATRIX(1,0,0,1,0,0);}",
            "b{transform:MaTrIx(1,0,0,1,0,0);}",
            "c{transform:m\\61 trix(1,0,0,1,0,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            matrix_argument_spellings(&result, index, 0),
            ["1", "0", "0", "1", "0", "0"],
            "matrix name recognition failed at {index}"
        );
    }

    // A name that merely starts with `matrix` is a different function and
    // stays outside selected-profile coverage.
    assert_all_unsupported(
        418390,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &["matrix3d(1,0,0,1,0,0)", "matrixx(1,0,0,1,0,0)"],
    );
}

#[test]
fn scale_function_name_recognition_is_ascii_case_insensitive() {
    let result = qualify(
        645280,
        concat!(
            "a{transform:SCALE(2);}",
            "b{transform:ScAlE(2);}",
            "c{transform:s\\63 ale(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            scale_argument_spellings(&result, index, 0),
            vec![(CssTransformScaleArgumentKind::Number, "2".to_string())],
            "scale name recognition failed at {index}"
        );
    }

    // A name that merely starts with `scale` is a different function and
    // stays outside selected-profile coverage.
    assert_all_unsupported(
        645290,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &["scaleX(2)", "scale3d(1,1,1)", "scalex(2)"],
    );
}

// 14. `scale() = scale([<number> | <percentage>]#{1,2})` (#645): one
// direct `<number>` argument qualifies as `CssTransformScaleArguments::One`,
// and each argument's authored numeric identity stays source-backed
// tokenizer evidence -- never normalized through machine floating point.

#[test]
fn direct_one_argument_number_scale_qualifies() {
    let result = qualify(
        645300,
        concat!(
            "a{transform:scale(0);}",
            "b{transform:scale(+0);}",
            "c{transform:scale(-0);}",
            "d{transform:scale(1);}",
            "e{transform:scale(-1);}",
            "f{transform:scale(.5);}",
            "g{transform:scale(1e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 7);
    assert_eq!(
        scale_argument_spellings(&result, 0, 0),
        vec![(CssTransformScaleArgumentKind::Number, "0".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 1, 0),
        vec![(CssTransformScaleArgumentKind::Number, "+0".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 2, 0),
        vec![(CssTransformScaleArgumentKind::Number, "-0".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 3, 0),
        vec![(CssTransformScaleArgumentKind::Number, "1".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 4, 0),
        vec![(CssTransformScaleArgumentKind::Number, "-1".to_string())]
    );
    // Authored `.5`: the absent leading integer digit is canonicalized to
    // `0` by the tokenizer's own retained numeric contract, upstream of
    // this leaf, exactly as for `matrix()` arguments.
    assert_eq!(
        scale_argument_spellings(&result, 5, 0),
        vec![(CssTransformScaleArgumentKind::Number, "0.5".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 6, 0),
        vec![(CssTransformScaleArgumentKind::Number, "1e100".to_string())]
    );
}

// 15. Direct `<percentage>` arguments qualify identically to `<number>`
// arguments, and Number vs Percentage token kind is preserved distinctly
// -- never collapsed into an interpreted scale factor (#645).

#[test]
fn direct_percentage_scale_arguments_qualify_and_preserve_kind() {
    let result = qualify(
        645310,
        concat!(
            "a{transform:scale(0%);}",
            "b{transform:scale(+0%);}",
            "c{transform:scale(-0%);}",
            "d{transform:scale(100%);}",
            "e{transform:scale(-2%);}",
            "f{transform:scale(.5%);}",
            "g{transform:scale(1e100%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 7);
    assert_eq!(
        scale_argument_spellings(&result, 0, 0),
        vec![(CssTransformScaleArgumentKind::Percentage, "0%".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 3, 0),
        vec![(
            CssTransformScaleArgumentKind::Percentage,
            "100%".to_string()
        )]
    );
    assert_eq!(
        scale_argument_spellings(&result, 6, 0),
        vec![(
            CssTransformScaleArgumentKind::Percentage,
            "1e100%".to_string()
        )]
    );

    // `100%` and `1` remain distinct authored evidence -- never collapsed
    // into an equivalent interpreted scale factor.
    let percent = qualify(645320, "a{transform:scale(100%);}");
    let number = qualify(645321, "a{transform:scale(1);}");
    assert_ne!(
        scale_argument_spellings(&percent, 0, 0),
        scale_argument_spellings(&number, 0, 0)
    );
}

// 16. Two ordered `scale()` arguments qualify as
// `CssTransformScaleArguments::Two`, preserving authored order and every
// Number/Percentage combination (#645).

#[test]
fn two_argument_scale_preserves_order_and_kind_combinations() {
    let result = qualify(
        645330,
        concat!(
            "a{transform:scale(1,2);}",
            "b{transform:scale(100%,200%);}",
            "c{transform:scale(1,200%);}",
            "d{transform:scale(150%,-2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    assert_eq!(
        scale_argument_spellings(&result, 0, 0),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (CssTransformScaleArgumentKind::Number, "2".to_string()),
        ]
    );
    assert_eq!(
        scale_argument_spellings(&result, 1, 0),
        vec![
            (
                CssTransformScaleArgumentKind::Percentage,
                "100%".to_string()
            ),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
    assert_eq!(
        scale_argument_spellings(&result, 2, 0),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
    assert_eq!(
        scale_argument_spellings(&result, 3, 0),
        vec![
            (
                CssTransformScaleArgumentKind::Percentage,
                "150%".to_string()
            ),
            (CssTransformScaleArgumentKind::Number, "-2".to_string()),
        ]
    );
}

// 17. Authored one-vs-two cardinality is preserved structurally:
// `CssTransformScaleArguments` cannot represent zero, three, or a
// synthesized argument, and an authored one-argument `scale()` never gains
// a materialized second factor (#645).

#[test]
fn authored_scale_cardinality_is_never_synthesized() {
    let result = qualify(
        645340,
        concat!("a{transform:scale(2);}", "b{transform:scale(2,3);}"),
    );

    assert_eq!(scale_argument_spellings(&result, 0, 0).len(), 1);
    assert_eq!(scale_argument_spellings(&result, 1, 0).len(), 2);

    let functions = qualified_functions(&result, 0);
    let CssTransformFunction::Scale(scale) = &functions[0] else {
        panic!("expected a qualified scale component");
    };
    assert!(scale.arguments().second().is_none());
}

// 18. `#{1,2}` bounds scale arity to one or two: zero, three, or more
// directly visible arguments are decisive `InvalidForSelectedValueGrammar`
// before any argument content is consulted, so an opaque Function in
// another slot never relaxes the directly visible arity boundary (#645).

#[test]
fn scale_argument_cardinality_outside_one_or_two_is_invalid() {
    assert_all_invalid(
        645350,
        &[
            "scale()",
            "scale(1,2,3)",
            "scale(1,2,3,4)",
            "scale(calc(1),2,3)",
            "scale(1,calc(2),3)",
            "scale(1,2,calc(3))",
        ],
    );
}

// 19. `#` is comma-separated repetition, never whitespace-separated SVG
// transform-attribute syntax, and an authored-empty position is preserved
// as its own ordered slot and rejected -- never collapsed away (#645).

#[test]
fn scale_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        645360,
        &[
            "scale(1 2)",
            "scale(,1)",
            "scale(1,)",
            "scale(1,,2)",
            "scale(,,)",
            "scale( , )",
        ],
    );
}

// 20. `<number> | <percentage>` admits only a direct retained `Number` or
// `Percentage` token: every other direct token category at a scale
// argument position is a decisive direct token-category failure. CSSWG
// #5273 (`<length>` support) remains an open proposal, so a direct
// `Dimension` such as `1px` stays decisively Invalid under the current
// pinned grammar (#645) -- the proposal is not pre-implemented.

#[test]
fn direct_non_number_percentage_scale_argument_categories_are_invalid() {
    assert_all_invalid(
        645370,
        &[
            "scale(1px)",
            "scale(1px,2)",
            "scale(1,2px)",
            "scale(foo)",
            "scale(none)",
            "scale(\"1\")",
            "scale(#abc)",
            "scale(1deg)",
        ],
    );
}

// 21. An opaque non-deferred Function at a scale argument position is
// structurally feasible but its validity depends on calculated-value
// semantics this leaf does not own, so it stays unsupported under the
// same shared `FunctionValuedTransformArgument` reason a `matrix()` opaque
// argument uses -- both sit at the same evidence-authority level (#645). A
// comma nested inside that Function is at a deeper relative depth and
// never becomes a scale argument separator.

#[test]
fn opaque_scale_argument_function_is_unsupported() {
    assert_all_unsupported(
        645380,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "scale(calc(1))",
            "scale(calc(100%))",
            "scale(calc(1),2)",
            "scale(1,calc(2))",
            "scale(min(1,2))",
            "matrix(1,0,0,1,0,0) scale(calc(1))",
        ],
    );

    // `calc(1,2)` contributes exactly ONE scale-level slot: its inner comma
    // is at scale-body relative depth one. Were it leaking, `scale(calc(1,2),3)`
    // would be a three-slot shell and therefore decisively Invalid, so the
    // Unsupported outcome is itself the depth-isolation proof.
    assert_all_unsupported(
        645390,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "scale(calc(1,2),3)",
            "scale(calc([1,2]))",
            "scale(calc({1,2}))",
        ],
    );

    // A nested comma that genuinely does add a scale-level slot once the
    // nesting closes is still counted, ruling out "ignore every comma
    // after a Function" as an accidental passing implementation.
    assert_all_invalid(645400, &["scale(calc(1,2),3,4)"]);
}

// 22. Outcome precedence is scan-order independent for `scale()` exactly
// as for `matrix()`: decisive Invalid outranks opaque Unsupported content
// regardless of authored position, and the coarser outer unselected-
// function coverage outranks an inner opaque scale argument (#645).

#[test]
fn scale_invalid_unsupported_precedence_is_scan_order_independent() {
    assert_all_invalid(
        645410,
        &[
            "scale(1px,calc(1))",
            "scale(calc(1),1px)",
            "scale(calc(1) 2)",
            "scale(calc(1),2,3)",
            "scale(1,2,calc(3))",
        ],
    );

    assert_all_unsupported(
        645420,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &["scale(calc(1)) rotate(1deg)", "rotate(1deg) scale(calc(1))"],
    );
}

// 23. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// scale shape conclusion -- including an arity that looks decisive (#645).

#[test]
fn scale_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        645430,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "scale(var(--x))",
            "scale(var(--x),0,0)",
            "scale(1,var(--x))",
            "matrix(1,0,0,1,0,0) scale(var(--x))",
            "scale(var(--x)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 24. Selected `matrix()` and `scale()` components mix freely and preserve
// exact authored order and repetition, retained through the heterogeneous
// `CssTransformFunction` alternation (#645).

#[test]
fn heterogeneous_matrix_and_scale_components_preserve_authored_order() {
    let result = qualify(
        645440,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) scale(2);}",
            "b{transform:scale(2) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) matrix(1,0,0,1,0,0) scale(100%,200%);}",
            "d{transform:matrix(1,0,0,1,0,0) matrix(2,0,0,2,0,0);}",
            "e{transform:scale(1) scale(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 5);

    let functions_a = qualified_functions(&result, 0);
    assert_eq!(functions_a.len(), 2);
    assert!(matches!(functions_a[0], CssTransformFunction::Matrix(_)));
    assert!(matches!(functions_a[1], CssTransformFunction::Scale(_)));

    let functions_b = qualified_functions(&result, 1);
    assert_eq!(functions_b.len(), 2);
    assert!(matches!(functions_b[0], CssTransformFunction::Scale(_)));
    assert!(matches!(functions_b[1], CssTransformFunction::Matrix(_)));

    let functions_c = qualified_functions(&result, 2);
    assert_eq!(functions_c.len(), 3);
    assert!(matches!(functions_c[0], CssTransformFunction::Scale(_)));
    assert!(matches!(functions_c[1], CssTransformFunction::Matrix(_)));
    assert!(matches!(functions_c[2], CssTransformFunction::Scale(_)));
    assert_eq!(
        scale_argument_spellings(&result, 2, 2),
        vec![
            (
                CssTransformScaleArgumentKind::Percentage,
                "100%".to_string()
            ),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );

    assert_eq!(qualified_functions(&result, 3).len(), 2);
    assert_eq!(qualified_functions(&result, 4).len(), 2);
    assert_eq!(
        scale_argument_spellings(&result, 4, 0),
        vec![(CssTransformScaleArgumentKind::Number, "1".to_string())]
    );
    assert_eq!(
        scale_argument_spellings(&result, 4, 1),
        vec![(CssTransformScaleArgumentKind::Number, "2".to_string())]
    );
}

#[test]
fn structurally_malformed_components_are_invalid() {
    assert_all_invalid(
        418400,
        &[
            "1",
            "matrix",
            "scale",
            "(1,0,0,1,0,0)",
            "(2)",
            "[matrix(1,0,0,1,0,0)]",
            "[scale(2)]",
            "matrix(1,0,0,1,0,0))",
            "scale(2))",
            "matrix((1,0,0,1,0,0)",
            "scale((2)",
            "1 matrix(1,0,0,1,0,0)",
            "1 scale(2)",
        ],
    );
}

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        418410,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0);transform:none;}",
            "b{transform-box:border-box;}",
            "c{transform-origin:left top;}",
            "d{transform-style:flat;}",
            "e{scale:2;}",
            "f{rotate:45deg;}",
            "g{translate:10px;}",
            "h{counter-reset:reversed(foo);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(result.transform_observations()[0].occurrence_index(), 0);
    assert_eq!(result.transform_observations()[1].occurrence_index(), 1);
    assert_eq!(
        matrix_argument_spellings(&result, 0, 0),
        ["1", "0", "0", "1", "0", "0"]
    );
    assert_whole_none(&result, 1);

    // The longer `transform-*` property names never enter this dispatch,
    // and the new matrix-body depth scoping never leaks into the accepted
    // single-value transform leaves or the accepted `reversed()` branch.
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.counter_reset_observations().len(), 1);
}

// 25. `translate3d() = translate3d(<length-percentage>, <length-percentage>,
// <length>)` (#418 / #647): exactly three ordered, position-sensitive
// arguments qualify. X and Y accept a direct `<length>` or `<percentage>`;
// Z accepts a direct `<length>` only. Canonical direct-`<length>` forms in
// every slot qualify, and authored numeric/unit identity stays
// source-backed tokenizer evidence -- never normalized through machine
// floating point or unit conversion.

#[test]
fn canonical_translate3d_lengths_qualify_with_three_ordered_arguments() {
    let result = qualify(
        647100,
        concat!(
            "a{transform:translate3d(0,0,0);}",
            "b{transform:translate3d(10px,20px,30px);}",
            "c{transform:translate3d(-1px,-2em,-3rem);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(qualified_functions(&result, 0).len(), 1);

    assert_eq!(
        translate3d_argument_spellings(&result, 0, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Length,
            "0".to_string(),
            CssTransformTranslate3dXyArgumentKind::Length,
            "0".to_string(),
            "0".to_string(),
        )
    );
    assert_eq!(
        translate3d_argument_spellings(&result, 1, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Length,
            "10px".to_string(),
            CssTransformTranslate3dXyArgumentKind::Length,
            "20px".to_string(),
            "30px".to_string(),
        )
    );
    assert_eq!(
        translate3d_argument_spellings(&result, 2, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Length,
            "-1px".to_string(),
            CssTransformTranslate3dXyArgumentKind::Length,
            "-2em".to_string(),
            "-3rem".to_string(),
        )
    );
}

// 26. XY `<percentage>` and mixed Length/Percentage combinations qualify in
// either X or Y position, preserving authored kind distinctly (#647).

#[test]
fn xy_percentage_and_mixed_combinations_qualify() {
    let result = qualify(
        647110,
        concat!(
            "a{transform:translate3d(10%,20%,30px);}",
            "b{transform:translate3d(10%,20px,30px);}",
            "c{transform:translate3d(10px,20%,30px);}",
            "d{transform:translate3d(0%,0%,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    assert_eq!(
        translate3d_argument_spellings(&result, 0, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Percentage,
            "10%".to_string(),
            CssTransformTranslate3dXyArgumentKind::Percentage,
            "20%".to_string(),
            "30px".to_string(),
        )
    );
    assert_eq!(
        translate3d_argument_spellings(&result, 1, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Percentage,
            "10%".to_string(),
            CssTransformTranslate3dXyArgumentKind::Length,
            "20px".to_string(),
            "30px".to_string(),
        )
    );
    assert_eq!(
        translate3d_argument_spellings(&result, 2, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Length,
            "10px".to_string(),
            CssTransformTranslate3dXyArgumentKind::Percentage,
            "20%".to_string(),
            "30px".to_string(),
        )
    );
    assert_eq!(
        translate3d_argument_spellings(&result, 3, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Percentage,
            "0%".to_string(),
            CssTransformTranslate3dXyArgumentKind::Percentage,
            "0%".to_string(),
            "0".to_string(),
        )
    );
}

// 27. Z is restricted to a direct `<length>`: a direct `<percentage>` in
// the Z position is decisive `InvalidForSelectedValueGrammar` even when it
// is mathematically zero -- this boundary is load-bearing and is never
// softened by X/Y accepting `<percentage>` (#418 / #647).

#[test]
fn z_percentage_is_decisively_invalid() {
    assert_all_invalid(
        647120,
        &[
            "translate3d(1px,2px,3%)",
            "translate3d(1px,2px,0%)",
            "translate3d(10%,20%,30%)",
            "translate3d(0px,0px,0%)",
        ],
    );
}

// 28. A direct exact-zero `Number` satisfies `<length>` in every slot,
// reusing the accepted `translate` (#606) exact-zero-Number-as-`Length`
// theorem; the retained evidence stays a `Number` token, never converted
// to a `Dimension` or interpreted magnitude (#647).

#[test]
fn exact_zero_number_qualifies_as_length_in_every_slot() {
    let result = qualify(
        647130,
        concat!(
            "a{transform:translate3d(0,1px,2px);}",
            "b{transform:translate3d(1px,+0,2px);}",
            "c{transform:translate3d(1px,2px,-0);}",
            "d{transform:translate3d(.0,0.0,0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    let (x_kind, x, _, _, _) = translate3d_argument_spellings(&result, 0, 0);
    assert_eq!(x_kind, CssTransformTranslate3dXyArgumentKind::Length);
    assert_eq!(x, "0");

    let (_, _, y_kind, y, _) = translate3d_argument_spellings(&result, 1, 0);
    assert_eq!(y_kind, CssTransformTranslate3dXyArgumentKind::Length);
    assert_eq!(y, "+0");

    let (.., z) = translate3d_argument_spellings(&result, 2, 0);
    assert_eq!(z, "-0");

    // Authored `.0`: the absent leading integer digit is canonicalized to
    // `0` by the tokenizer's own retained numeric contract, upstream of
    // this leaf, exactly as for `matrix()`/`scale()` arguments.
    assert_eq!(
        translate3d_argument_spellings(&result, 3, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Length,
            "0.0".to_string(),
            CssTransformTranslate3dXyArgumentKind::Length,
            "0.0".to_string(),
            "0e100".to_string(),
        )
    );
}

// 29. A non-zero unitless `Number` never satisfies `<length>` or
// `<percentage>` in any slot: this leaf never interprets an arbitrary
// `Number` as `Length` (#418 / #647).

#[test]
fn non_zero_unitless_number_is_invalid_in_every_slot() {
    assert_all_invalid(
        647140,
        &[
            "translate3d(1,2px,3px)",
            "translate3d(1px,2,3px)",
            "translate3d(1px,2px,3)",
            "translate3d(1px,2px,-3)",
            "translate3d(1px,2px,0.5)",
        ],
    );
}

// 30. `0`, `0px`, and `0%` remain three distinct authored evidences: none
// is ever normalized, synthesized, or collapsed into another, and a `0%`
// Z is invalid precisely because it is a `Percentage`, not because it is
// mathematically zero (#418 / #647).

#[test]
fn zero_number_dimension_and_percentage_identity_is_preserved() {
    let result = qualify(
        647150,
        concat!(
            "a{transform:translate3d(0,0,0);}",
            "b{transform:translate3d(0px,0px,0px);}",
            "c{transform:translate3d(0%,0%,0px);}",
        ),
    );

    let all_number = translate3d_argument_spellings(&result, 0, 0);
    let all_px = translate3d_argument_spellings(&result, 1, 0);
    let percentage_xy = translate3d_argument_spellings(&result, 2, 0);

    assert_eq!(all_number.1, "0");
    assert_eq!(all_px.1, "0px");
    assert_eq!(percentage_xy.1, "0%");
    assert_ne!(all_number.1, all_px.1);
    assert_ne!(all_number.1, percentage_xy.1);
    assert_ne!(all_px.1, percentage_xy.1);
    assert_eq!(
        percentage_xy.0,
        CssTransformTranslate3dXyArgumentKind::Percentage
    );

    // `0%` remains decisively Invalid in the Z position even paired with
    // every other Z form qualifying.
    assert_all_invalid(647151, &["translate3d(0px,0px,0%)"]);
}

// 31. Every other direct token category -- an angle `Dimension`, `Ident`,
// `String`, or `Hash` -- is a decisive direct token-category failure in
// any of the three positions (#418 / #647).

#[test]
fn wrong_direct_token_categories_are_invalid() {
    assert_all_invalid(
        647160,
        &[
            "translate3d(1deg,2px,3px)",
            "translate3d(1px,2deg,3px)",
            "translate3d(1px,2px,3deg)",
            "translate3d(foo,2px,3px)",
            "translate3d(1px,foo,3px)",
            "translate3d(1px,2px,foo)",
            "translate3d(\"1\",2px,3px)",
            "translate3d(1px,\"2\",3px)",
            "translate3d(1px,2px,\"3\")",
            "translate3d(#abc,2px,3px)",
            "translate3d(1px,#abc,3px)",
            "translate3d(1px,2px,#abc)",
        ],
    );
}

// 32. Exact arity is three: zero, one, two, four, or more directly visible
// arguments are decisive `InvalidForSelectedValueGrammar` before any
// argument content is consulted (#418 / #647).

#[test]
fn translate3d_argument_cardinality_other_than_three_is_invalid() {
    assert_all_invalid(
        647170,
        &[
            "translate3d()",
            "translate3d(1px)",
            "translate3d(1px,2px)",
            "translate3d(1px,2px,3px,4px)",
            "translate3d(1px,2px,3px,4px,5px)",
        ],
    );

    // The adjacent accepted arity is the only qualifying one, sealing the
    // off-by-one boundary from both sides.
    let result = qualify(647179, "a{transform:translate3d(1px,2px,3px);}");
    let (.., z) = translate3d_argument_spellings(&result, 0, 0);
    assert_eq!(z, "3px");
}

// 33. `#` is comma-separated repetition, never whitespace-separated SVG
// transform-attribute syntax, and an authored-empty argument position is
// preserved as its own ordered slot and rejected -- never collapsed away
// (#418 / #647).

#[test]
fn translate3d_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        647180,
        &[
            "translate3d(1px 2px 3px)",
            "translate3d(1 2 3)",
            "translate3d(,2px,3px)",
            "translate3d(1px,,3px)",
            "translate3d(1px,2px,)",
            "translate3d(1px,,,3px)",
            "translate3d(,,)",
        ],
    );
}

// 34. An opaque non-deferred Function occupying an otherwise structurally
// feasible argument slot is selected-profile Unsupported, mirroring the
// shared `FunctionValuedTransformArgument` reason `matrix()`/`scale()`
// opaque arguments use -- this leaf never evaluates `calc()` (#418 /
// #647).

#[test]
fn opaque_translate3d_argument_function_is_unsupported() {
    assert_all_unsupported(
        647190,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translate3d(calc(1px),2px,3px)",
            "translate3d(1px,calc(2%),3px)",
            "translate3d(1px,2px,calc(3px))",
            "translate3d(min(1px,2px),2px,3px)",
            "matrix(1,0,0,1,0,0) translate3d(calc(1px),2px,3px)",
        ],
    );
}

// 35. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a translate3d-level argument separator -- the
// Unsupported outcome for exactly three slots is itself the
// depth-isolation proof, and a nested comma that genuinely does add a
// translate3d-level slot once the nesting closes is still counted (#647).

#[test]
fn nested_commas_never_change_translate3d_arity() {
    assert_all_unsupported(
        647200,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translate3d(calc(1px,2px),2px,3px)",
            "translate3d(2px,calc(1px,2px),3px)",
            "translate3d(2px,3px,calc(1px,2px))",
            "translate3d(calc([1px,2px]),2px,3px)",
            "translate3d(calc({1px,2px}),2px,3px)",
        ],
    );

    // Once the nesting closes, a genuine fourth translate3d-level slot is
    // still counted and makes the shell decisively Invalid.
    assert_all_invalid(
        647210,
        &[
            "translate3d(calc(1px,2px),2px,3px,4px)",
            "translate3d(calc(1px,2px),2px)",
        ],
    );
}

// 36. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot, in either authored order -- including the
// position-sensitive Z `Percentage` boundary, which never depends on
// which slot the walk reaches first (#418 / #647).

#[test]
fn decisive_invalid_outranks_opaque_argument_in_either_order() {
    assert_all_invalid(
        647220,
        &[
            "translate3d(calc(1px),2px,30%)",
            "translate3d(30%,calc(2px),30%)",
            "translate3d(calc(1px),2px)",
            "translate3d(calc(1px),2px,3px,4px)",
            "translate3d(1px,calc(2px),3%)",
            "translate3d(1%,calc(2px),3%)",
        ],
    );
}

// 37. Deferred substitution can still change the surrounding token
// sequence, separators, and cardinality, so it is resolved before any
// surrounding translate3d shape conclusion -- including an arity that
// looks decisive (#418 / #647).

#[test]
fn translate3d_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        647230,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "translate3d(var(--x),2px)",
            "translate3d(1px,var(--y),3px)",
            "translate3d(var(--x),2px,3px)",
            "translate3d(1px,2px,3px,var(--x))",
            "matrix(1,0,0,1,0,0) translate3d(var(--x),2px)",
            "translate3d(var(--x),2px) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 38. Selected `matrix()`, `scale()`, and `translate3d()` components mix
// freely, preserve exact authored order and repetition, and stay
// distinguishable through the heterogeneous `CssTransformFunction`
// alternation (#418 / #645 / #647).

#[test]
fn mixed_matrix_scale_translate3d_order_is_preserved() {
    let result = qualify(
        647240,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) translate3d(10%,20px,30px) scale(2);}",
            "b{transform:scale(2) translate3d(10px,20%,30px) matrix(1,0,0,1,0,0);}",
            "c{transform:translate3d(1px,2px,3px) translate3d(4%,5px,6px);}",
            "d{transform:translate3d(1px,2px,3px) matrix(1,0,0,1,0,0);}",
            "e{transform:matrix(1,0,0,1,0,0) translate3d(1px,2px,3px);}",
            "f{transform:scale(2) translate3d(1px,2px,3px);}",
            "g{transform:translate3d(1px,2px,3px) scale(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 7);

    let functions_a = qualified_functions(&result, 0);
    assert_eq!(functions_a.len(), 3);
    assert!(matches!(functions_a[0], CssTransformFunction::Matrix(_)));
    assert!(matches!(
        functions_a[1],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(functions_a[2], CssTransformFunction::Scale(_)));

    let functions_b = qualified_functions(&result, 1);
    assert_eq!(functions_b.len(), 3);
    assert!(matches!(functions_b[0], CssTransformFunction::Scale(_)));
    assert!(matches!(
        functions_b[1],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(functions_b[2], CssTransformFunction::Matrix(_)));

    let functions_c = qualified_functions(&result, 2);
    assert_eq!(functions_c.len(), 2);
    assert!(matches!(
        functions_c[0],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        functions_c[1],
        CssTransformFunction::Translate3d(_)
    ));
    assert_eq!(
        translate3d_argument_spellings(&result, 2, 0).4,
        "3px".to_string()
    );
    assert_eq!(
        translate3d_argument_spellings(&result, 2, 1).4,
        "6px".to_string()
    );

    for index in 3..7 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
}

// 39. Every `<transform-function>` other than selected `matrix()`/
// `scale()`/`translate3d()` stays outside selected-profile coverage, in
// either authored order and regardless of a sibling qualified selected
// function, and the coarser outer unselected-function coverage outranks
// an inner opaque `translate3d()` argument (#418 / #647).

#[test]
fn unselected_outer_function_precedence_covers_translate3d() {
    assert_all_unsupported(
        647250,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate(1deg)",
            "translate3d(1px,2px,3px) rotate(1deg)",
            "rotate(1deg) translate3d(1px,2px,3px)",
            "translate3dx(1px,2px,3px)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translate3d argument, identically in both authored orders.
    assert_all_unsupported(
        647260,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translate3d(calc(1px),2px,3px) rotate(1deg)",
            "rotate(1deg) translate3d(calc(1px),2px,3px)",
        ],
    );
}

// 40. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two
// `translate3d()` components or a `translate3d()` and a sibling selected
// function (#418 / #647).

#[test]
fn top_level_comma_is_invalid_around_translate3d() {
    assert_all_invalid(
        647270,
        &[
            "translate3d(1px,2px,3px), matrix(1,0,0,1,0,0)",
            "translate3d(1px,2px,3px), scale(2)",
            "translate3d(1px,2px,3px), translate3d(1px,2px,3px)",
            ",translate3d(1px,2px,3px)",
            "translate3d(1px,2px,3px),",
            "none, translate3d(1px,2px,3px)",
            "translate3d(1px,2px,3px), none",
        ],
    );
}

// 41. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `translate3d()` extent is qualified from
// retained interior evidence alone. EOF never fills or repairs missing
// slots, empty slots, or a decisive Z `Percentage` (#418 / #647).

#[test]
fn true_stylesheet_eof_ended_translate3d_extent_follows_parser_authority() {
    let complete = qualify(647280, "a{transform:translate3d(1px,2px,3px");
    assert_eq!(
        complete.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(complete.transform_observations().len(), 1);
    let (.., z) = translate3d_argument_spellings(&complete, 0, 0);
    assert_eq!(z, "3px");

    // A short EOF-ended extent stays short: EOF never fills missing slots.
    let short = qualify(647281, "a{transform:translate3d(1px,2px");
    assert_invalid(&short, 0);

    // EOF never repairs a decisive Z Percentage.
    let z_percentage = qualify(647282, "a{transform:translate3d(1px,2px,3%");
    assert_invalid(&z_percentage, 0);

    // Without a closer, later authored material is absorbed into the final
    // slot by real retained structure, so it no longer satisfies exactly
    // one direct Length.
    let absorbed = qualify(647283, "a{transform:translate3d(1px,2px,3px 7");
    assert_eq!(
        absorbed.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_invalid(&absorbed, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a translate3d".
    let trailing = qualify(647284, "a{transform:translate3d(1px,2px,3px) 7;}");
    assert_invalid(&trailing, 0);
}

// 42. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, and
// `!important` remains outside the semantic value window (#418 / #647).

#[test]
fn trivia_and_important_never_change_translate3d_interpretation() {
    let result = qualify(
        647290,
        concat!(
            "a{transform:translate3d(1px,/**/2px,3px);}",
            "b{transform:translate3d(1px/**/,2px,3px);}",
            "c{transform:translate3d(1px,2px,3px/**/);}",
            "d{transform:translate3d(/**/1px,2px,3px);}",
            "e{transform:translate3d( 1px , 2px , 3px );}",
            "f{transform:translate3d(1px,/*,*/2px,3px);}",
            "g{transform:translate3d(1px,2px,3px) !important;}",
            "h{transform:translate3d(1px,2px,3px)!important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..6 {
        assert_eq!(
            translate3d_argument_spellings(&result, index, 0),
            (
                CssTransformTranslate3dXyArgumentKind::Length,
                "1px".to_string(),
                CssTransformTranslate3dXyArgumentKind::Length,
                "2px".to_string(),
                "3px".to_string(),
            ),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 6..8 {
        let (.., z) = translate3d_argument_spellings(&result, index, 0);
        assert_eq!(z, "3px");
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(
        647300,
        &["translate3d(1px,/**/,2px,3px)", "translate3d(1px,2px,/**/)"],
    );
}

// 43. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension/Percentage
// tokens identically across runs, exactly like matrix/scale evidence
// lookup (#418 / #647).

#[test]
fn translate3d_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:translate3d(10%,20px,30px);}",
        "b{transform:translate3d(calc(1px,2px),2px,3px);}",
        "c{transform:translate3d(1px,2px,3%);}",
        "d{transform:matrix(1,0,0,1,0,0) translate3d(1px,2px,3px);}",
    );

    let first = qualify(647310, css);
    let repeated = qualify(647310, css);
    let another_source = qualify(647311, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_eq!(qualified_functions(&first, 0).len(), 1);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        translate3d_argument_spellings(&first, 0, 0),
        translate3d_argument_spellings(&repeated, 0, 0)
    );
    assert_eq!(
        translate3d_argument_spellings(&first, 0, 0),
        translate3d_argument_spellings(&another_source, 0, 0)
    );
}

// 44. `translate3d` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale` boundary; a name that merely
// starts with `translate3d` is a different function and stays outside
// selected-profile coverage (#647).

#[test]
fn translate3d_function_name_recognition_is_ascii_case_insensitive() {
    let result = qualify(
        647320,
        concat!(
            "a{transform:TRANSLATE3D(1px,2px,3px);}",
            "b{transform:TrAnSlAtE3d(1px,2px,3px);}",
            "c{transform:t\\72 anslate3d(1px,2px,3px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        let (.., z) = translate3d_argument_spellings(&result, index, 0);
        assert_eq!(z, "3px", "translate3d name recognition failed at {index}");
    }
}

// 45. Structurally malformed `translate3d` components -- a bare Function
// name, an unbalanced/duplicated closer, or stray leading material -- stay
// decisively `Invalid`, mirroring the accepted `matrix`/`scale` structural
// boundary (#647).

#[test]
fn structurally_malformed_translate3d_components_are_invalid() {
    assert_all_invalid(
        647330,
        &[
            "translate3d",
            "translate3d(1px,2px,3px))",
            "translate3d((1px,2px,3px)",
            "1 translate3d(1px,2px,3px)",
            "[translate3d(1px,2px,3px)]",
        ],
    );
}

// 46. Cross-leaf isolation: `translate3d()` recognition never leaks into
// the accepted longhand `translate`/`scale`/`rotate` leaves or the other
// `transform-*` single-value leaves, and coexists with a sibling matrix
// declaration in the same ordinary declaration list (#418 / #606 / #645 /
// #647).

#[test]
fn translate3d_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        647340,
        concat!(
            "a{transform:translate3d(1px,2px,3px);transform:none;}",
            "b{translate:10px;}",
            "c{scale:2;}",
            "d{rotate:45deg;}",
            "e{transform-origin:left top;}",
            "f{transform-box:border-box;}",
            "g{transform-style:flat;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    let (.., z) = translate3d_argument_spellings(&result, 0, 0);
    assert_eq!(z, "3px");
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);
}

// 47. `rotate3d() = rotate3d(<number>, <number>, <number>, [<angle> |
// <zero>])` (#418 / #645 / #647 / #649): exactly four ordered,
// position-sensitive arguments qualify. The first three (axis X, Y, Z)
// accept a direct `<number>` with no value-range restriction; the fourth
// accepts a direct `<angle>` or a direct literal `<zero>`. Canonical direct
// forms qualify, and authored numeric/unit identity stays source-backed
// tokenizer evidence -- never normalized through machine floating point,
// unit conversion, or axis-vector normalization.

#[test]
fn canonical_rotate3d_qualifies_with_four_ordered_arguments() {
    let result = qualify(
        649100,
        concat!(
            "a{transform:rotate3d(1,0,0,90deg);}",
            "b{transform:rotate3d(0,1,0,180deg);}",
            "c{transform:rotate3d(0,0,1,1rad);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(qualified_functions(&result, 0).len(), 1);

    assert_eq!(
        rotate3d_axis_spellings(&result, 0, 0),
        ("1".to_string(), "0".to_string(), "0".to_string())
    );
    assert_eq!(
        rotate3d_angle_spelling(&result, 0, 0),
        (true, "90deg".to_string())
    );

    assert_eq!(
        rotate3d_axis_spellings(&result, 1, 0),
        ("0".to_string(), "1".to_string(), "0".to_string())
    );
    assert_eq!(
        rotate3d_angle_spelling(&result, 1, 0),
        (true, "180deg".to_string())
    );

    assert_eq!(
        rotate3d_axis_spellings(&result, 2, 0),
        ("0".to_string(), "0".to_string(), "1".to_string())
    );
    assert_eq!(
        rotate3d_angle_spelling(&result, 2, 0),
        (true, "1rad".to_string())
    );
}

// 48. Signed, fractional, and exponential axis `<number>` spellings qualify
// in every axis slot, preserving exact tokenizer-owned evidence without
// normalization (#649).

#[test]
fn axis_number_spellings_are_preserved_without_normalization() {
    let result = qualify(
        649110,
        concat!(
            "a{transform:rotate3d(-1,+2,.5,45deg);}",
            "b{transform:rotate3d(1e2,-3e-2,+0,45deg);}",
            "c{transform:rotate3d(-0,+0,.0,90deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        rotate3d_axis_spellings(&result, 0, 0),
        ("-1".to_string(), "+2".to_string(), "0.5".to_string())
    );
    assert_eq!(
        rotate3d_axis_spellings(&result, 1, 0),
        ("1e2".to_string(), "-3e-2".to_string(), "+0".to_string())
    );
    assert_eq!(
        rotate3d_axis_spellings(&result, 2, 0),
        ("-0".to_string(), "+0".to_string(), "0.0".to_string())
    );
}

// 49. The all-zero axis vector `rotate3d(0, 0, 0, <angle>)` is directly
// Qualified: axis-vector normalization and all-zero-vector rejection are
// downstream transform-matrix/interpolation semantics this leaf does not
// own. This is load-bearing (#649).

#[test]
fn all_zero_axis_vector_is_qualified() {
    let result = qualify(649120, "a{transform:rotate3d(0,0,0,45deg);}");
    assert_eq!(result.transform_observations().len(), 1);
    assert_eq!(
        rotate3d_axis_spellings(&result, 0, 0),
        ("0".to_string(), "0".to_string(), "0".to_string())
    );
    assert_eq!(
        rotate3d_angle_spelling(&result, 0, 0),
        (true, "45deg".to_string())
    );
}

// 50. Every recognized CSS angle unit qualifies the fourth slot as a direct
// `<angle>`, ASCII-case-insensitively, reusing the accepted
// `is_css_angle_unit` theorem without a second independent unit table
// (#649).

#[test]
fn recognized_angle_units_qualify_the_fourth_slot() {
    let result = qualify(
        649130,
        concat!(
            "a{transform:rotate3d(1,0,0,90deg);}",
            "b{transform:rotate3d(1,0,0,100grad);}",
            "c{transform:rotate3d(1,0,0,1rad);}",
            "d{transform:rotate3d(1,0,0,0.25turn);}",
            "e{transform:rotate3d(1,0,0,90DEG);}",
            "f{transform:rotate3d(1,0,0,1RAD);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        let (is_angle, _) = rotate3d_angle_spelling(&result, index, 0);
        assert!(is_angle, "expected Angle role at {index}");
    }
    assert_eq!(rotate3d_angle_spelling(&result, 0, 0).1, "90deg");
    assert_eq!(rotate3d_angle_spelling(&result, 1, 0).1, "100grad");
    assert_eq!(rotate3d_angle_spelling(&result, 2, 0).1, "1rad");
    assert_eq!(rotate3d_angle_spelling(&result, 3, 0).1, "0.25turn");
    assert_eq!(rotate3d_angle_spelling(&result, 4, 0).1, "90DEG");
    assert_eq!(rotate3d_angle_spelling(&result, 5, 0).1, "1RAD");
}

// 51. Representative exact-zero `Number` spellings qualify the fourth slot
// through the `<zero>` branch, reusing the accepted
// `is_direct_zero_numeric_value` theorem; the retained evidence stays a
// `Number` token (#649).

#[test]
fn exact_zero_spellings_qualify_the_fourth_slot_as_zero() {
    let result = qualify(
        649140,
        concat!(
            "a{transform:rotate3d(1,0,0,0);}",
            "b{transform:rotate3d(1,0,0,+0);}",
            "c{transform:rotate3d(1,0,0,-0);}",
            "d{transform:rotate3d(1,0,0,.0);}",
            "e{transform:rotate3d(1,0,0,0.0);}",
            "f{transform:rotate3d(1,0,0,0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        let (is_angle, _) = rotate3d_angle_spelling(&result, index, 0);
        assert!(!is_angle, "expected Zero role at {index}");
    }
    assert_eq!(rotate3d_angle_spelling(&result, 0, 0).1, "0");
    assert_eq!(rotate3d_angle_spelling(&result, 1, 0).1, "+0");
    assert_eq!(rotate3d_angle_spelling(&result, 2, 0).1, "-0");
    assert_eq!(rotate3d_angle_spelling(&result, 3, 0).1, "0.0");
    assert_eq!(rotate3d_angle_spelling(&result, 4, 0).1, "0.0");
    assert_eq!(rotate3d_angle_spelling(&result, 5, 0).1, "0e100");
}

// 52. `0` and `0deg` remain distinct authored roles -- `<zero>` and
// `<angle>` respectively -- never collapsed into one generic scalar, never
// normalized into each other, and a `<zero>` never gains a synthesized
// `deg` unit. This is load-bearing (#649).

#[test]
fn zero_and_zero_deg_authored_roles_remain_distinct() {
    let result = qualify(
        649150,
        concat!(
            "a{transform:rotate3d(1,0,0,0);}",
            "b{transform:rotate3d(1,0,0,0deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    let (zero_is_angle, zero_spelling) = rotate3d_angle_spelling(&result, 0, 0);
    let (angle_is_angle, angle_spelling) = rotate3d_angle_spelling(&result, 1, 0);
    assert!(!zero_is_angle, "expected authored `0` to qualify as Zero");
    assert!(
        angle_is_angle,
        "expected authored `0deg` to qualify as Angle"
    );
    assert_eq!(zero_spelling, "0");
    assert_eq!(angle_spelling, "0deg");
}

// 53. Every direct token category other than `<number>` -- `<percentage>`,
// an angle `Dimension`, a length `Dimension`, `Ident`, `String`, `Hash` --
// is a decisive direct token-category failure in any axis position (#649).

#[test]
fn wrong_axis_token_categories_are_invalid() {
    assert_all_invalid(
        649160,
        &[
            "rotate3d(1%,0,0,90deg)",
            "rotate3d(1deg,0,0,90deg)",
            "rotate3d(1px,0,0,90deg)",
            "rotate3d(foo,0,0,90deg)",
            "rotate3d(\"1\",0,0,90deg)",
            "rotate3d(#abc,0,0,90deg)",
            "rotate3d(0,1%,0,90deg)",
            "rotate3d(0,1deg,0,90deg)",
            "rotate3d(0,1px,0,90deg)",
            "rotate3d(0,0,1%,90deg)",
            "rotate3d(0,0,1deg,90deg)",
            "rotate3d(0,0,1px,90deg)",
        ],
    );
}

// 54. Every direct token category other than a recognized `<angle>` or a
// literal `<zero>` -- a nonzero `Number`, `<percentage>`, a length
// `Dimension` (including `0px`), a non-angle time `Dimension`, `Ident`,
// `String`, `Hash` -- is a decisive direct token-category failure in the
// fourth slot (#649).

#[test]
fn wrong_fourth_slot_categories_are_invalid() {
    assert_all_invalid(
        649170,
        &[
            "rotate3d(1,0,0,1)",
            "rotate3d(1,0,0,-1)",
            "rotate3d(1,0,0,.5)",
            "rotate3d(1,0,0,0px)",
            "rotate3d(1,0,0,90px)",
            "rotate3d(1,0,0,0%)",
            "rotate3d(1,0,0,90%)",
            "rotate3d(1,0,0,0s)",
            "rotate3d(1,0,0,foo)",
            "rotate3d(1,0,0,\"90deg\")",
            "rotate3d(1,0,0,#abc)",
        ],
    );
}

// 55. Exact arity is four: zero, one, two, three, five, or more directly
// visible arguments are decisive `InvalidForSelectedValueGrammar` before
// any argument content is consulted (#649).

#[test]
fn rotate3d_argument_cardinality_other_than_four_is_invalid() {
    assert_all_invalid(
        649180,
        &[
            "rotate3d()",
            "rotate3d(1)",
            "rotate3d(1,0)",
            "rotate3d(1,0,0)",
            "rotate3d(1,0,0,90deg,2)",
            "rotate3d(1,0,0,90deg,2,3)",
        ],
    );

    // The adjacent accepted arity is the only qualifying one, sealing the
    // off-by-one boundary from both sides.
    let result = qualify(649189, "a{transform:rotate3d(1,0,0,90deg);}");
    let (is_angle, spelling) = rotate3d_angle_spelling(&result, 0, 0);
    assert!(is_angle);
    assert_eq!(spelling, "90deg");
}

// 56. `,` is comma-separated repetition, never whitespace-separated SVG
// transform-attribute syntax, and an authored-empty argument position is
// preserved as its own ordered slot and rejected -- never collapsed away
// (#649).

#[test]
fn rotate3d_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        649190,
        &[
            "rotate3d(1 0 0 90deg)",
            "rotate3d(,0,0,90deg)",
            "rotate3d(1,,0,90deg)",
            "rotate3d(1,0,,90deg)",
            "rotate3d(1,0,0,)",
            "rotate3d(1,,,90deg)",
            "rotate3d(,,,)",
        ],
    );
}

// 57. An opaque non-deferred Function occupying an otherwise structurally
// feasible argument slot is selected-profile Unsupported, mirroring the
// shared `FunctionValuedTransformArgument` reason the other selected
// functions use -- this leaf never evaluates `calc()`, so `calc(0)` is
// never direct `<zero>` (#649).

#[test]
fn opaque_rotate3d_argument_function_is_unsupported() {
    assert_all_unsupported(
        649200,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "rotate3d(calc(1),0,0,90deg)",
            "rotate3d(1,calc(0),0,90deg)",
            "rotate3d(1,0,calc(0),90deg)",
            "rotate3d(1,0,0,calc(0))",
            "rotate3d(1,0,0,calc(90deg))",
            "matrix(1,0,0,1,0,0) rotate3d(calc(1),0,0,90deg)",
        ],
    );
}

// 58. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a rotate3d-level argument separator -- the
// Unsupported outcome for exactly four slots is itself the depth-isolation
// proof, and a nested comma that genuinely does add or remove a
// rotate3d-level slot once the nesting closes is still counted (#649).

#[test]
fn nested_commas_never_change_rotate3d_arity() {
    assert_all_unsupported(
        649210,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "rotate3d(calc(1,2),0,0,90deg)",
            "rotate3d(0,calc(1,2),0,90deg)",
            "rotate3d(0,0,calc(1,2),90deg)",
            "rotate3d(calc([1,2]),0,0,90deg)",
            "rotate3d(calc({1,2}),0,0,90deg)",
        ],
    );

    // Once the nesting closes, the genuinely different rotate3d-level slot
    // count is still counted and makes the shell decisively Invalid.
    assert_all_invalid(
        649220,
        &["rotate3d(calc(1,2),0,0,90deg,2)", "rotate3d(calc(1,2),0,0)"],
    );
}

// 59. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot, in either authored order (#649).

#[test]
fn decisive_invalid_outranks_opaque_rotate3d_argument_in_either_order() {
    assert_all_invalid(
        649230,
        &[
            "rotate3d(calc(1),0,0,1)",
            "rotate3d(1%,calc(0),0,90deg)",
            "rotate3d(calc(1),0,90deg)",
            "rotate3d(calc(1),0,0,90deg,2)",
        ],
    );
}

// 60. Deferred substitution can still change the surrounding token
// sequence, separators, and cardinality, so it is resolved before any
// surrounding rotate3d shape conclusion -- including an arity that looks
// decisive (#649).

#[test]
fn rotate3d_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        649240,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "rotate3d(var(--x),0,0,90deg)",
            "rotate3d(1,var(--y),0,90deg)",
            "rotate3d(1,0,var(--z),90deg)",
            "rotate3d(1,0,0,var(--angle))",
            "rotate3d(1,0,0,90deg,var(--extra))",
            "matrix(1,0,0,1,0,0) rotate3d(var(--x),0,0,90deg)",
        ],
    );
}

// 61. Selected `matrix()`, `scale()`, `translate3d()`, and `rotate3d()`
// components mix freely, preserve exact authored order and repetition, and
// stay distinguishable through the heterogeneous `CssTransformFunction`
// alternation (#418 / #645 / #647 / #649).

#[test]
fn mixed_selected_function_order_including_rotate3d_is_preserved() {
    let result = qualify(
        649250,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) rotate3d(1,0,0,90deg);}",
            "b{transform:rotate3d(1,0,0,90deg) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) rotate3d(1,0,0,90deg);}",
            "d{transform:rotate3d(1,0,0,90deg) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) rotate3d(1,0,0,90deg);}",
            "f{transform:rotate3d(1,0,0,90deg) translate3d(1px,2px,3px);}",
            "g{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg);}",
            "h{transform:rotate3d(1,0,0,90deg) rotate3d(0,1,0,45deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);

    let functions_a = qualified_functions(&result, 0);
    assert_eq!(functions_a.len(), 2);
    assert!(matches!(functions_a[0], CssTransformFunction::Matrix(_)));
    assert!(matches!(functions_a[1], CssTransformFunction::Rotate3d(_)));

    let functions_b = qualified_functions(&result, 1);
    assert_eq!(functions_b.len(), 2);
    assert!(matches!(functions_b[0], CssTransformFunction::Rotate3d(_)));
    assert!(matches!(functions_b[1], CssTransformFunction::Matrix(_)));

    let functions_g = qualified_functions(&result, 6);
    assert_eq!(functions_g.len(), 4);
    assert!(matches!(functions_g[0], CssTransformFunction::Matrix(_)));
    assert!(matches!(functions_g[1], CssTransformFunction::Scale(_)));
    assert!(matches!(
        functions_g[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(functions_g[3], CssTransformFunction::Rotate3d(_)));

    let functions_h = qualified_functions(&result, 7);
    assert_eq!(functions_h.len(), 2);
    assert!(matches!(functions_h[0], CssTransformFunction::Rotate3d(_)));
    assert!(matches!(functions_h[1], CssTransformFunction::Rotate3d(_)));
    assert_eq!(
        rotate3d_axis_spellings(&result, 7, 0),
        ("1".to_string(), "0".to_string(), "0".to_string())
    );
    assert_eq!(
        rotate3d_axis_spellings(&result, 7, 1),
        ("0".to_string(), "1".to_string(), "0".to_string())
    );

    for index in 2..6 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
}

// 62. Every `<transform-function>` other than selected `matrix()`/
// `scale()`/`translate3d()`/`rotate3d()` stays outside selected-profile
// coverage, in either authored order and regardless of a sibling qualified
// selected function, and the coarser outer unselected-function coverage
// outranks an inner opaque `rotate3d()` argument (#649).

#[test]
fn unselected_outer_function_precedence_covers_rotate3d() {
    assert_all_unsupported(
        649260,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate(1deg)",
            "rotate3d(1,0,0,90deg) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1) rotate3d(1,0,0,90deg)",
            "rotate3dx(1,0,0,90deg)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // rotate3d argument, identically in both authored orders.
    assert_all_unsupported(
        649270,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate3d(calc(1),0,0,90deg) rotate(1deg)",
            "rotate(1deg) rotate3d(calc(1),0,0,90deg)",
        ],
    );
}

// 63. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two
// `rotate3d()` components or a `rotate3d()` and a sibling selected
// function (#649).

#[test]
fn top_level_comma_is_invalid_around_rotate3d() {
    assert_all_invalid(
        649280,
        &[
            "rotate3d(1,0,0,90deg), matrix(1,0,0,1,0,0)",
            "rotate3d(1,0,0,90deg), scale(2)",
            "rotate3d(1,0,0,90deg), translate3d(1px,2px,3px)",
            "rotate3d(1,0,0,90deg), rotate3d(1,0,0,90deg)",
            ",rotate3d(1,0,0,90deg)",
            "rotate3d(1,0,0,90deg),",
            "none, rotate3d(1,0,0,90deg)",
        ],
    );
}

// 64. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `rotate3d()` extent is qualified from
// retained interior evidence alone. EOF never fills or repairs missing
// slots, empty slots, or a decisive fourth-slot category (#649).

#[test]
fn true_stylesheet_eof_ended_rotate3d_extent_follows_parser_authority() {
    let complete = qualify(649290, "a{transform:rotate3d(1,0,0,90deg");
    assert_eq!(
        complete.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(complete.transform_observations().len(), 1);
    let (is_angle, spelling) = rotate3d_angle_spelling(&complete, 0, 0);
    assert!(is_angle);
    assert_eq!(spelling, "90deg");

    // A short EOF-ended extent stays short: EOF never fills missing slots.
    let short = qualify(649291, "a{transform:rotate3d(1,0,0");
    assert_invalid(&short, 0);

    // EOF never repairs a decisive fourth-slot category.
    let wrong_angle = qualify(649292, "a{transform:rotate3d(1,0,0,1");
    assert_invalid(&wrong_angle, 0);

    // A trailing comma at EOF is still invalid: EOF never fills the empty
    // slot it leaves behind.
    let trailing_comma = qualify(649293, "a{transform:rotate3d(1,0,0,");
    assert_invalid(&trailing_comma, 0);

    // Without a closer, later authored material is absorbed into the final
    // slot by real retained structure, so it no longer satisfies exactly
    // one direct Angle/Zero.
    let absorbed = qualify(649294, "a{transform:rotate3d(1,0,0,90deg 7");
    assert_eq!(
        absorbed.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_invalid(&absorbed, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a rotate3d".
    let trailing = qualify(649295, "a{transform:rotate3d(1,0,0,90deg) 7;}");
    assert_invalid(&trailing, 0);
}

// 65. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, and
// `!important` remains outside the semantic value window (#649).

#[test]
fn trivia_and_important_never_change_rotate3d_interpretation() {
    let result = qualify(
        649300,
        concat!(
            "a{transform:rotate3d(1,/**/0,0,90deg);}",
            "b{transform:rotate3d(1/**/,0,0,90deg);}",
            "c{transform:rotate3d(1,0,0,90deg/**/);}",
            "d{transform:rotate3d(/**/1,0,0,90deg);}",
            "e{transform:rotate3d( 1 , 0 , 0 , 90deg );}",
            "f{transform:rotate3d(1,/*,*/0,0,90deg);}",
            "g{transform:rotate3d(1,0,0,90deg) !important;}",
            "h{transform:rotate3d(1,0,0,90deg)!important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..6 {
        assert_eq!(
            rotate3d_axis_spellings(&result, index, 0),
            ("1".to_string(), "0".to_string(), "0".to_string()),
            "trivia changed axis interpretation at {index}"
        );
        assert_eq!(
            rotate3d_angle_spelling(&result, index, 0),
            (true, "90deg".to_string()),
            "trivia changed angle interpretation at {index}"
        );
    }
    for index in 6..8 {
        let (is_angle, spelling) = rotate3d_angle_spelling(&result, index, 0);
        assert!(is_angle);
        assert_eq!(spelling, "90deg");
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(
        649310,
        &["rotate3d(1,/**/,0,0,90deg)", "rotate3d(1,0,0,/**/)"],
    );
}

// 66. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension tokens
// identically across runs, exactly like matrix/scale/translate3d evidence
// lookup (#649).

#[test]
fn rotate3d_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:rotate3d(1,0,0,90deg);}",
        "b{transform:rotate3d(calc(1,2),0,0,90deg);}",
        "c{transform:rotate3d(1,0,0,1);}",
        "d{transform:matrix(1,0,0,1,0,0) rotate3d(1,0,0,90deg);}",
    );

    let first = qualify(649320, css);
    let repeated = qualify(649320, css);
    let another_source = qualify(649321, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_eq!(qualified_functions(&first, 0).len(), 1);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        rotate3d_axis_spellings(&first, 0, 0),
        rotate3d_axis_spellings(&repeated, 0, 0)
    );
    assert_eq!(
        rotate3d_axis_spellings(&first, 0, 0),
        rotate3d_axis_spellings(&another_source, 0, 0)
    );
}

// 67. `rotate3d` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d` boundary; a name
// that merely starts with `rotate3d` is a different function and stays
// outside selected-profile coverage (#649).

#[test]
fn rotate3d_function_name_recognition_is_ascii_case_insensitive() {
    let result = qualify(
        649330,
        concat!(
            "a{transform:ROTATE3D(1,0,0,90deg);}",
            "b{transform:RoTaTe3d(1,0,0,90deg);}",
            "c{transform:r\\6f tate3d(1,0,0,90deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        let (is_angle, spelling) = rotate3d_angle_spelling(&result, index, 0);
        assert!(is_angle, "rotate3d name recognition failed at {index}");
        assert_eq!(spelling, "90deg");
    }
}

// 68. Structurally malformed `rotate3d` components -- a bare Function
// name, an unbalanced/duplicated closer, or stray leading material -- stay
// decisively `Invalid`, mirroring the accepted `matrix`/`scale`/
// `translate3d` structural boundary (#649).

#[test]
fn structurally_malformed_rotate3d_components_are_invalid() {
    assert_all_invalid(
        649340,
        &[
            "rotate3d",
            "rotate3d(1,0,0,90deg))",
            "rotate3d((1,0,0,90deg)",
            "1 rotate3d(1,0,0,90deg)",
            "[rotate3d(1,0,0,90deg)]",
        ],
    );
}

// 69. Cross-leaf isolation: `rotate3d()` recognition never leaks into the
// accepted longhand `rotate`/`translate`/`scale` leaves or the other
// `transform-*` single-value leaves, and coexists with a sibling matrix
// declaration in the same ordinary declaration list (#418 / #606 / #645 /
// #647 / #649).

#[test]
fn rotate3d_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        649350,
        concat!(
            "a{transform:rotate3d(1,0,0,90deg);transform:none;}",
            "b{translate:10px;}",
            "c{scale:2;}",
            "d{rotate:45deg;}",
            "e{transform-origin:left top;}",
            "f{transform-box:border-box;}",
            "g{transform-style:flat;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    let (is_angle, spelling) = rotate3d_angle_spelling(&result, 0, 0);
    assert!(is_angle);
    assert_eq!(spelling, "90deg");
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);
}

// 70. `none` remains an exclusive whole-value branch even when combined
// with `rotate3d()`, in either authored order (#418 / #649).

#[test]
fn whole_none_remains_exclusive_with_rotate3d() {
    assert_all_invalid(
        649360,
        &["none rotate3d(1,0,0,90deg)", "rotate3d(1,0,0,90deg) none"],
    );
}

// 71. `translate() = translate(<length-percentage>, <length-percentage>?)`
// (#651): a canonical single direct `<length>` argument qualifies as
// `CssTransformTranslateArguments::One`, preserving the `Length` role and
// exact tokenizer-owned evidence.

#[test]
fn canonical_one_argument_length_qualifies() {
    let result = qualify(
        651100,
        concat!(
            "a{transform:translate(10px);}",
            "b{transform:translate(1em);}",
            "c{transform:translate(-2rem);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        translate_argument_spellings(&result, 0, 0),
        vec![(
            CssTransformTranslateArgumentKind::Length,
            "10px".to_string()
        )]
    );
    assert_eq!(
        translate_argument_spellings(&result, 1, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "1em".to_string())]
    );
    assert_eq!(
        translate_argument_spellings(&result, 2, 0),
        vec![(
            CssTransformTranslateArgumentKind::Length,
            "-2rem".to_string()
        )]
    );
}

// 72. A canonical single direct `<percentage>` argument qualifies as
// `CssTransformTranslateArguments::One`, preserving the `Percentage` role
// and exact tokenizer-owned evidence; a `Percentage` is never collapsed
// into a `Length` (#651).

#[test]
fn canonical_one_argument_percentage_qualifies() {
    let result = qualify(
        651110,
        concat!(
            "a{transform:translate(50%);}",
            "b{transform:translate(-20%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(
        translate_argument_spellings(&result, 0, 0),
        vec![(
            CssTransformTranslateArgumentKind::Percentage,
            "50%".to_string()
        )]
    );
    assert_eq!(
        translate_argument_spellings(&result, 1, 0),
        vec![(
            CssTransformTranslateArgumentKind::Percentage,
            "-20%".to_string()
        )]
    );
}

// 73. Two ordered `translate()` arguments qualify as
// `CssTransformTranslateArguments::Two`, preserving authored order and
// every Length/Percentage combination (#651).

#[test]
fn two_argument_translate_preserves_order_and_kind_combinations() {
    let result = qualify(
        651120,
        concat!(
            "a{transform:translate(10px,20px);}",
            "b{transform:translate(10px,20%);}",
            "c{transform:translate(10%,20px);}",
            "d{transform:translate(10%,20%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    assert_eq!(
        translate_argument_spellings(&result, 0, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Length,
                "10px".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Length,
                "20px".to_string()
            ),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 1, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Length,
                "10px".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Percentage,
                "20%".to_string()
            ),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 2, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Percentage,
                "10%".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Length,
                "20px".to_string()
            ),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 3, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Percentage,
                "10%".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Percentage,
                "20%".to_string()
            ),
        ]
    );
}

// 74. A direct exact-zero `Number` satisfies `<length>` in either slot,
// reusing the accepted `translate` (#606) / `translate3d` (#647)
// exact-zero-Number-as-`Length` theorem; the retained evidence stays a
// `Number` token, never converted to a `Dimension` or interpreted
// magnitude (#651).

#[test]
fn exact_zero_number_qualifies_as_length_in_either_slot() {
    let result = qualify(
        651130,
        concat!(
            "a{transform:translate(0);}",
            "b{transform:translate(+0);}",
            "c{transform:translate(-0);}",
            "d{transform:translate(.0);}",
            "e{transform:translate(0.0);}",
            "f{transform:translate(0e100);}",
            "g{transform:translate(1px,0);}",
            "h{transform:translate(0,1px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    assert_eq!(
        translate_argument_spellings(&result, 0, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "0".to_string())]
    );
    assert_eq!(
        translate_argument_spellings(&result, 1, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "+0".to_string())]
    );
    assert_eq!(
        translate_argument_spellings(&result, 2, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "-0".to_string())]
    );
    // Authored `.0`: the absent leading integer digit is canonicalized to
    // `0` by the tokenizer's own retained numeric contract, upstream of
    // this leaf, exactly as for `matrix()`/`scale()`/`translate3d()`
    // arguments.
    assert_eq!(
        translate_argument_spellings(&result, 3, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "0.0".to_string())]
    );
    assert_eq!(
        translate_argument_spellings(&result, 4, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "0.0".to_string())]
    );
    assert_eq!(
        translate_argument_spellings(&result, 5, 0),
        vec![(
            CssTransformTranslateArgumentKind::Length,
            "0e100".to_string()
        )]
    );
    assert_eq!(
        translate_argument_spellings(&result, 6, 0),
        vec![
            (CssTransformTranslateArgumentKind::Length, "1px".to_string()),
            (CssTransformTranslateArgumentKind::Length, "0".to_string()),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 7, 0),
        vec![
            (CssTransformTranslateArgumentKind::Length, "0".to_string()),
            (CssTransformTranslateArgumentKind::Length, "1px".to_string()),
        ]
    );
}

// 75. `0`, `0px`, and `0%` remain three distinct authored evidences in
// every `translate()` slot: none is ever normalized, synthesized, or
// collapsed into another (#651).

#[test]
fn translate_zero_number_dimension_and_percentage_identity_is_preserved() {
    let result = qualify(
        651140,
        concat!(
            "a{transform:translate(0);}",
            "b{transform:translate(0px);}",
            "c{transform:translate(0%);}",
            "d{transform:translate(0,0px);}",
            "e{transform:translate(0px,0%);}",
        ),
    );

    let number = translate_argument_spellings(&result, 0, 0);
    let px = translate_argument_spellings(&result, 1, 0);
    let percentage = translate_argument_spellings(&result, 2, 0);

    assert_eq!(
        number,
        vec![(CssTransformTranslateArgumentKind::Length, "0".to_string())]
    );
    assert_eq!(
        px,
        vec![(CssTransformTranslateArgumentKind::Length, "0px".to_string())]
    );
    assert_eq!(
        percentage,
        vec![(
            CssTransformTranslateArgumentKind::Percentage,
            "0%".to_string()
        )]
    );
    assert_ne!(number, px);
    assert_ne!(number, percentage);
    assert_ne!(px, percentage);

    assert_eq!(
        translate_argument_spellings(&result, 3, 0),
        vec![
            (CssTransformTranslateArgumentKind::Length, "0".to_string()),
            (CssTransformTranslateArgumentKind::Length, "0px".to_string()),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 4, 0),
        vec![
            (CssTransformTranslateArgumentKind::Length, "0px".to_string()),
            (
                CssTransformTranslateArgumentKind::Percentage,
                "0%".to_string()
            ),
        ]
    );
}

// 76. Representative recognized CSS length units qualify identically to
// `translate3d()` X/Y arguments, including a useful ASCII-case unit
// variant, reusing the same recognized-length-unit theorem (#651).

#[test]
fn recognized_css_length_units_qualify() {
    let result = qualify(
        651150,
        concat!(
            "a{transform:translate(1px);}",
            "b{transform:translate(1em);}",
            "c{transform:translate(1rem);}",
            "d{transform:translate(1vh);}",
            "e{transform:translate(1vw);}",
            "f{transform:translate(1cm);}",
            "g{transform:translate(1PX);}",
            "h{transform:translate(1Px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..8 {
        assert_eq!(
            translate_argument_spellings(&result, index, 0).len(),
            1,
            "expected one qualified argument at {index}"
        );
    }
}

// 77. Signed direct `Length`, `Percentage`, and exact-zero `Number`
// arguments preserve exact authored sign/fraction/exponent evidence
// without any machine-number conversion (#651).

#[test]
fn signed_direct_values_preserve_authored_evidence() {
    let result = qualify(
        651160,
        concat!(
            "a{transform:translate(-10px,+20px);}",
            "b{transform:translate(+10%,-20%);}",
            "c{transform:translate(-0,+0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        translate_argument_spellings(&result, 0, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Length,
                "-10px".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Length,
                "+20px".to_string()
            ),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 1, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Percentage,
                "+10%".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Percentage,
                "-20%".to_string()
            ),
        ]
    );
    assert_eq!(
        translate_argument_spellings(&result, 2, 0),
        vec![
            (CssTransformTranslateArgumentKind::Length, "-0".to_string()),
            (CssTransformTranslateArgumentKind::Length, "+0".to_string()),
        ]
    );
}

// 78. Authored one-vs-two cardinality is preserved structurally:
// `CssTransformTranslateArguments` cannot represent zero, three, or a
// synthesized argument, and an authored one-argument `translate()` never
// gains a materialized second argument -- `translate(10px)` and
// `translate(10px,0)` stay structurally distinct (#651).

#[test]
fn authored_translate_cardinality_is_never_synthesized() {
    let result = qualify(
        651170,
        concat!(
            "a{transform:translate(10px);}",
            "b{transform:translate(10px,0);}",
        ),
    );

    assert_eq!(translate_argument_spellings(&result, 0, 0).len(), 1);
    assert_eq!(translate_argument_spellings(&result, 1, 0).len(), 2);

    let functions = qualified_functions(&result, 0);
    let CssTransformFunction::Translate(translate) = &functions[0] else {
        panic!("expected a qualified translate component");
    };
    assert!(translate.arguments().second().is_none());

    let functions_two = qualified_functions(&result, 1);
    let CssTransformFunction::Translate(translate_two) = &functions_two[0] else {
        panic!("expected a qualified translate component");
    };
    assert!(translate_two.arguments().second().is_some());
}

// 79. Translate arity is bounded to one or two: zero, three, or more
// directly visible arguments are decisive `InvalidForSelectedValueGrammar`
// before any argument content is consulted, so an opaque Function in
// another slot never relaxes the directly visible arity boundary (#651).

#[test]
fn translate_argument_cardinality_outside_one_or_two_is_invalid() {
    assert_all_invalid(
        651180,
        &[
            "translate()",
            "translate(10px,20px,30px)",
            "translate(1px,2px,3px,4px)",
            "translate(calc(1px),2px,3px)",
            "translate(1px,calc(2px),3px)",
        ],
    );
}

// 80. A comma is the only accepted inner separator, an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away -- and whitespace alone is never the inner argument
// separator (#651).

#[test]
fn translate_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        651190,
        &[
            "translate(10px 20px)",
            "translate(,20px)",
            "translate(10px,)",
            "translate(10px,,20px)",
            "translate(,)",
        ],
    );
}

// 81. `<length-percentage>` admits only a direct recognized-length
// `Dimension`, an exact-zero `Number`, or a `Percentage`: every other
// direct token category at a translate argument position is a decisive
// direct token-category failure, in either position (#651).

#[test]
fn wrong_direct_categories_are_invalid() {
    assert_all_invalid(
        651200,
        &[
            "translate(1)",
            "translate(-1)",
            "translate(.5)",
            "translate(1,2px)",
            "translate(2px,1)",
            "translate(45deg)",
            "translate(1s)",
            "translate(1hz)",
            "translate(1dpi)",
            "translate(1fr)",
            "translate(1unknownunit)",
            "translate(foo)",
            "translate(\"1\")",
            "translate(#abc)",
            "translate(10px,45deg)",
            "translate(45deg,10px)",
        ],
    );
}

// 82. A complete non-deferred Function occupying an otherwise structurally
// feasible translate argument slot is selected-profile Unsupported,
// mirroring the shared `FunctionValuedTransformArgument` reason
// `matrix()`/`scale()`/`translate3d()` opaque arguments use -- this leaf
// never evaluates `calc()` (#651).

#[test]
fn opaque_translate_argument_function_is_unsupported() {
    assert_all_unsupported(
        651210,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translate(calc(10px))",
            "translate(calc(10px),20%)",
            "translate(10px,calc(20%))",
            "translate(min(10px,20px))",
            "translate(clamp(0px,10px,20px))",
            "matrix(1,0,0,1,0,0) translate(calc(10px))",
        ],
    );
}

// 83. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`:
// a structurally feasible complete opaque Function is never softened by
// an unevaluated sibling in the same slot (#651).

#[test]
fn function_plus_junk_in_same_slot_is_invalid() {
    assert_all_invalid(
        651220,
        &[
            "translate(calc(10px) 1px)",
            "translate(calc(10px) foo)",
            "translate(calc(10px) 20%)",
        ],
    );
}

// 84. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a translate-level argument separator -- the
// Unsupported outcome for exactly one or two slots is itself the
// depth-isolation proof, and a nested comma that genuinely does add a
// translate-level slot once the nesting closes is still counted (#651).

#[test]
fn nested_commas_never_change_translate_arity() {
    assert_all_unsupported(
        651230,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translate(calc(10px,20px))",
            "translate(calc(10px,20px),30%)",
            "translate(30%,calc(10px,20px))",
        ],
    );

    // Once the nesting closes, a genuine third translate-level slot is
    // still counted and makes the shell decisively Invalid.
    assert_all_invalid(651240, &["translate(calc(10px,20px),30%,1px)"]);
}

// 85. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot, in either authored order: the outcome never
// depends on which slot the walk reaches first (#651).

#[test]
fn translate_decisive_invalid_outranks_opaque_argument_in_either_order() {
    assert_all_invalid(
        651250,
        &[
            "translate(calc(10px),1)",
            "translate(1,calc(20%))",
            "translate(calc(10px),20%,1px)",
        ],
    );
}

// 86. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// translate shape conclusion -- including an arity that looks decisive
// (#651).

#[test]
fn translate_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        651260,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "translate(var(--x))",
            "translate(var(--x),20%)",
            "translate(10px,var(--y))",
            "translate(10px,20%,var(--extra))",
            "matrix(1,0,0,1,0,0) translate(var(--x))",
            "translate(var(--x)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 87. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`, and
// `translate()` components mix and repeat freely, preserving exact
// authored order and repetition through the heterogeneous
// `CssTransformFunction` alternation (#418 / #645 / #647 / #649 / #651).

#[test]
fn mixed_selected_function_order_including_translate_is_preserved() {
    let result = qualify(
        651270,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) translate(10px);}",
            "b{transform:translate(10px) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) translate(10px);}",
            "d{transform:translate(10px) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) translate(10px);}",
            "f{transform:translate(10px) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) translate(10px);}",
            "h{transform:translate(10px) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(1px) translate(2px);}",
            "j{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 10);

    for index in 0..9 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 9).len(), 5);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let five_kind_sequence = qualified_functions(&result, 9);
    assert!(matches!(
        five_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        five_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        five_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        five_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        five_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));

    let repeated = qualified_functions(&result, 8);
    assert!(matches!(repeated[0], CssTransformFunction::Translate(_)));
    assert!(matches!(repeated[1], CssTransformFunction::Translate(_)));
    assert_eq!(
        translate_argument_spellings(&result, 8, 0),
        vec![(CssTransformTranslateArgumentKind::Length, "1px".to_string())]
    );
    assert_eq!(
        translate_argument_spellings(&result, 8, 1),
        vec![(CssTransformTranslateArgumentKind::Length, "2px".to_string())]
    );
}

// 88. Every `<transform-function>` other than the five selected leaves
// stays outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque
// `translate()` argument (#651).

#[test]
fn unselected_outer_function_precedence_covers_translate() {
    assert_all_unsupported(
        651280,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translate(10px) rotate(10deg)",
            "rotate(10deg) translate(10px)",
            "translate(10px) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "translateX(10px)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translate argument, identically in both authored orders.
    assert_all_unsupported(
        651290,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translate(calc(10px)) rotate(10deg)",
            "rotate(10deg) translate(calc(10px))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(651300, &["translate(1) rotate(10deg)"]);
}

// 89. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two
// `translate()` components or a `translate()` and a sibling selected
// function (#651).

#[test]
fn top_level_comma_is_invalid_around_translate() {
    assert_all_invalid(
        651310,
        &[
            "translate(10px), matrix(1,0,0,1,0,0)",
            "translate(10px), scale(2)",
            "translate(10px), translate3d(1px,2px,3px)",
            "translate(10px), rotate3d(1,0,0,90deg)",
            "translate(10px), translate(10px)",
        ],
    );
}

// 90. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `translate()` extent is qualified from
// retained interior evidence alone. EOF never fills or repairs missing
// slots, empty slots, or an invalid second argument (#651).

#[test]
fn true_stylesheet_eof_ended_translate_extent_follows_parser_authority() {
    let one_slot = qualify(651320, "a{transform:translate(10px");
    assert_eq!(
        one_slot.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(one_slot.transform_observations().len(), 1);
    assert_eq!(
        translate_argument_spellings(&one_slot, 0, 0),
        vec![(
            CssTransformTranslateArgumentKind::Length,
            "10px".to_string()
        )]
    );

    let two_slots = qualify(651321, "a{transform:translate(10px,20%");
    assert_eq!(
        two_slots.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(
        translate_argument_spellings(&two_slots, 0, 0),
        vec![
            (
                CssTransformTranslateArgumentKind::Length,
                "10px".to_string()
            ),
            (
                CssTransformTranslateArgumentKind::Percentage,
                "20%".to_string()
            ),
        ]
    );

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(651322, "a{transform:translate(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid.
    let trailing_comma = qualify(651323, "a{transform:translate(10px,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs an invalid second argument.
    let invalid_second = qualify(651324, "a{transform:translate(10px,1");
    assert_invalid(&invalid_second, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a translate".
    let trailing = qualify(651325, "a{transform:translate(10px) 7;}");
    assert_invalid(&trailing, 0);
}

// 91. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#651).

#[test]
fn trivia_and_important_never_change_translate_interpretation() {
    let result = qualify(
        651330,
        concat!(
            "a{transform:translate(10px,/**/20px);}",
            "b{transform:translate(10px/**/,20px);}",
            "c{transform:translate(10px,20px/**/);}",
            "d{transform:translate(/**/10px,20px);}",
            "e{transform:translate( 10px , 20px );}",
            "f{transform:translate(10px,20px) !important;}",
            "g{transform:translate(10px,20px)!important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 7);
    let expected = vec![
        (
            CssTransformTranslateArgumentKind::Length,
            "10px".to_string(),
        ),
        (
            CssTransformTranslateArgumentKind::Length,
            "20px".to_string(),
        ),
    ];
    for index in 0..5 {
        assert_eq!(
            translate_argument_spellings(&result, index, 0),
            expected,
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 5..7 {
        assert_eq!(translate_argument_spellings(&result, index, 0), expected);
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(
        651340,
        &["translate(10px,/**/,20px)", "translate(10px,/**/)"],
    );
}

// 92. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension/Percentage tokens
// identically across runs, exactly like matrix/scale/translate3d/rotate3d
// evidence lookup (#651).

#[test]
fn translate_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:translate(10px,20%);}",
        "b{transform:translate(calc(10px,20px));}",
        "c{transform:translate(1);}",
        "d{transform:matrix(1,0,0,1,0,0) translate(10px);}",
    );

    let first = qualify(651350, css);
    let repeated = qualify(651350, css);
    let another_source = qualify(651351, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_eq!(qualified_functions(&first, 0).len(), 1);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        translate_argument_spellings(&first, 0, 0),
        translate_argument_spellings(&repeated, 0, 0)
    );
    assert_eq!(
        translate_argument_spellings(&first, 0, 0),
        translate_argument_spellings(&another_source, 0, 0)
    );
}

// 93. `translate` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`
// boundary, and structurally malformed `translate` components -- a bare
// Function name, an unbalanced/duplicated closer, or stray leading
// material -- stay decisively `Invalid` (#651).

#[test]
fn translate_function_name_recognition_and_malformed_components() {
    let result = qualify(
        651360,
        concat!(
            "a{transform:TRANSLATE(10px);}",
            "b{transform:TrAnSlAtE(10px);}",
            "c{transform:t\\72 anslate(10px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            translate_argument_spellings(&result, index, 0),
            vec![(
                CssTransformTranslateArgumentKind::Length,
                "10px".to_string()
            )],
            "translate name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        651370,
        &[
            "translate",
            "translate(10px))",
            "translate((10px)",
            "1 translate(10px)",
            "[translate(10px)]",
        ],
    );
}

// 94. Cross-leaf isolation: `translate()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `translate()` (#418 / #606 / #645 / #647 / #649
// / #651).

#[test]
fn translate_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        651380,
        concat!(
            "a{transform:translate(10px);transform:none;}",
            "b{translate:10px;}",
            "c{scale:2;}",
            "d{rotate:45deg;}",
            "e{transform-origin:left top;}",
            "f{transform-box:border-box;}",
            "g{transform-style:flat;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(
        translate_argument_spellings(&result, 0, 0),
        vec![(
            CssTransformTranslateArgumentKind::Length,
            "10px".to_string()
        )]
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(651390, &["none translate(10px)", "translate(10px) none"]);
}
