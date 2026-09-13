use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssExponentSign, CssNumberSign, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransformFunction, CssTransformQualificationOutcome, CssTransformRotate3dAngleArgument,
    CssTransformRotate3dFunction, CssTransformRotateArgumentKind, CssTransformRotateFunction,
    CssTransformRotateXArgumentKind, CssTransformRotateXFunction, CssTransformScale3dArgumentKind,
    CssTransformScaleArgumentKind, CssTransformScaleXArgumentKind, CssTransformScaleYArgumentKind,
    CssTransformScaleZArgumentKind, CssTransformTranslate3dXyArgumentKind,
    CssTransformTranslateArgumentKind, CssTransformTranslateXArgumentKind,
    CssTransformTranslateYArgumentKind, CssTransformUnsupportedReason, CssTransformValue,
    CssValueQualificationRunResult, run,
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

/// Reconstructs one retained `Number`, `Dimension`, or `Percentage` token's
/// authored numeric structure from tokenizer-owned evidence alone, exactly
/// like `authored_translate_argument_spelling`, so `0`, `0px`, and `0%`
/// stay pairwise distinguishable in assertions and none is ever collapsed
/// into another. This is a dedicated `translateX()` test helper, distinct
/// from `authored_translate_argument_spelling` and
/// `authored_translate3d_argument_spelling`: `translateX()` argument
/// placement remains a distinct semantic role even though the underlying
/// scalar spelling logic is identical. No machine number or unit
/// conversion is ever produced.
fn authored_translatex_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        CssTokenKind::Percentage { value } => (value, "%".to_string()),
        other => panic!(
            "translateX argument evidence did not resolve to a Number/Dimension/Percentage token, got {other:?}"
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

/// Resolves one qualified `translateX()` transform component's single
/// authored argument at `function_index` within observation `index` as a
/// `(kind, spelling)` pair. Unlike `translate_argument_spellings` and
/// `scale_argument_spellings`, this never returns a vector: `translateX()`
/// has no one-vs-two cardinality to preserve, since it accepts exactly one
/// authored argument.
fn translatex_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (CssTransformTranslateXArgumentKind, String) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::TranslateX(translatex) = function else {
        panic!("expected translateX component {function_index} at {index}, got {function:?}");
    };

    let argument = translatex.argument();
    let token = result
        .transform_translatex_argument_token(argument.evidence_ref())
        .expect("translateX argument evidence did not resolve");
    (
        argument.kind(),
        authored_translatex_argument_spelling(token),
    )
}

/// Reconstructs one retained `Number`, `Dimension`, or `Percentage` token's
/// authored numeric structure from tokenizer-owned evidence alone, exactly
/// like `authored_translatex_argument_spelling`, so `0`, `0px`, and `0%`
/// stay pairwise distinguishable in assertions and none is ever collapsed
/// into another. This is a dedicated `translateY()` test helper, distinct
/// from `authored_translatex_argument_spelling`: `translateX()` and
/// `translateY()` argument placement remain distinct semantic roles even
/// though the underlying scalar spelling logic is identical. No machine
/// number or unit conversion is ever produced.
fn authored_translatey_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        CssTokenKind::Percentage { value } => (value, "%".to_string()),
        other => panic!(
            "translateY argument evidence did not resolve to a Number/Dimension/Percentage token, got {other:?}"
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

/// Resolves one qualified `translateY()` transform component's single
/// authored argument at `function_index` within observation `index` as a
/// `(kind, spelling)` pair. Unlike `translate_argument_spellings` and
/// `scale_argument_spellings`, this never returns a vector: `translateY()`
/// has no one-vs-two cardinality to preserve, since it accepts exactly one
/// authored argument.
fn translatey_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (CssTransformTranslateYArgumentKind, String) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::TranslateY(translatey) = function else {
        panic!("expected translateY component {function_index} at {index}, got {function:?}");
    };

    let argument = translatey.argument();
    let token = result
        .transform_translatey_argument_token(argument.evidence_ref())
        .expect("translateY argument evidence did not resolve");
    (
        argument.kind(),
        authored_translatey_argument_spelling(token),
    )
}

/// Reconstructs one retained `Number` or `Dimension` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_translatey_argument_spelling`, with no `Percentage` arm: a
/// qualified `translateZ()` argument can never resolve to a `Percentage`
/// token, so this dedicated `translateZ()` test helper panics on one rather
/// than silently spelling it, distinct from
/// `authored_translatex_argument_spelling`/`authored_translatey_argument_spelling`
/// (#657). No machine number or unit conversion is ever produced.
fn authored_translatez_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        other => panic!(
            "translateZ argument evidence did not resolve to a Number/Dimension token, got {other:?}"
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

/// Resolves one qualified `translateZ()` transform component's single
/// authored argument at `function_index` within observation `index` as its
/// authored spelling alone. Unlike `translatex_argument_spelling`/
/// `translatey_argument_spelling`, this returns a bare `String` rather than
/// a `(kind, spelling)` pair: a qualified `translateZ()` argument has no
/// Length/Percentage `kind` to distinguish, since it is always `Length`
/// (#657).
fn translatez_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> String {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::TranslateZ(translatez) = function else {
        panic!("expected translateZ component {function_index} at {index}, got {function:?}");
    };

    let token = result
        .transform_translatez_argument_token(translatez.argument())
        .expect("translateZ argument evidence did not resolve");
    authored_translatez_argument_spelling(token)
}

/// Reconstructs one retained `Number` or `Percentage` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_scale_argument_spelling`, with a trailing `%` marker
/// distinguishing a `Percentage` token so `2` and `2%` stay distinguishable
/// in assertions and a `Percentage` is never collapsed into a `Number`.
/// This is a dedicated `scaleX()` test helper, distinct from
/// `authored_scale_argument_spelling`: `scaleX()` argument placement and
/// `scale()` argument placement remain distinct semantic roles even though
/// the underlying scalar spelling logic is identical (#659). No machine
/// number is ever produced.
fn authored_scalex_argument_spelling(token: &CssTokenKind) -> String {
    let (value, is_percentage) = match token {
        CssTokenKind::Number { value, .. } => (value, false),
        CssTokenKind::Percentage { value } => (value, true),
        other => panic!(
            "scaleX argument evidence did not resolve to a Number/Percentage token, got {other:?}"
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

/// Resolves one qualified `scaleX()` transform component's single authored
/// argument at `function_index` within observation `index` into its
/// `(kind, spelling)` pair, mirroring `translatex_argument_spelling`.
/// Unlike `scale_argument_spellings`, this returns a single pair rather
/// than an ordered vector: `scaleX()` has no optional second argument
/// (#659).
fn scalex_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (CssTransformScaleXArgumentKind, String) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::ScaleX(scalex) = function else {
        panic!("expected scaleX component {function_index} at {index}, got {function:?}");
    };

    let argument = scalex.argument();
    let token = result
        .transform_scalex_argument_token(argument.evidence_ref())
        .expect("scaleX argument evidence did not resolve");
    (argument.kind(), authored_scalex_argument_spelling(token))
}

/// Reconstructs one retained `Number` or `Percentage` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_scalex_argument_spelling`, with a trailing `%` marker
/// distinguishing a `Percentage` token so `2` and `2%` stay distinguishable
/// in assertions and a `Percentage` is never collapsed into a `Number`.
/// This is a dedicated `scaleY()` test helper, distinct from
/// `authored_scalex_argument_spelling`: `scaleY()` argument placement,
/// `scaleX()` argument placement, and `scale()` argument placement remain
/// distinct semantic roles even though the underlying scalar spelling logic
/// is identical (#661). No machine number is ever produced.
fn authored_scaley_argument_spelling(token: &CssTokenKind) -> String {
    let (value, is_percentage) = match token {
        CssTokenKind::Number { value, .. } => (value, false),
        CssTokenKind::Percentage { value } => (value, true),
        other => panic!(
            "scaleY argument evidence did not resolve to a Number/Percentage token, got {other:?}"
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

/// Resolves one qualified `scaleY()` transform component's single authored
/// argument at `function_index` within observation `index` into its
/// `(kind, spelling)` pair, mirroring `scalex_argument_spelling`. Unlike
/// `scale_argument_spellings`, this returns a single pair rather than an
/// ordered vector: `scaleY()` has no optional second argument (#661).
fn scaley_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (CssTransformScaleYArgumentKind, String) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::ScaleY(scaley) = function else {
        panic!("expected scaleY component {function_index} at {index}, got {function:?}");
    };

    let argument = scaley.argument();
    let token = result
        .transform_scaley_argument_token(argument.evidence_ref())
        .expect("scaleY argument evidence did not resolve");
    (argument.kind(), authored_scaley_argument_spelling(token))
}

/// Reconstructs one retained `Number` or `Percentage` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_scaley_argument_spelling`, with a trailing `%` marker
/// distinguishing a `Percentage` token so `2` and `2%` stay distinguishable
/// in assertions and a `Percentage` is never collapsed into a `Number`.
/// This is a dedicated `scaleZ()` test helper, distinct from
/// `authored_scaley_argument_spelling`: `scaleZ()` argument placement,
/// `scaleY()` argument placement, `scaleX()` argument placement, and
/// `scale()` argument placement remain distinct semantic roles even though
/// the underlying scalar spelling logic is identical (#663). No machine
/// number is ever produced.
fn authored_scalez_argument_spelling(token: &CssTokenKind) -> String {
    let (value, is_percentage) = match token {
        CssTokenKind::Number { value, .. } => (value, false),
        CssTokenKind::Percentage { value } => (value, true),
        other => panic!(
            "scaleZ argument evidence did not resolve to a Number/Percentage token, got {other:?}"
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

/// Resolves one qualified `scaleZ()` transform component's single authored
/// argument at `function_index` within observation `index` into its
/// `(kind, spelling)` pair, mirroring `scaley_argument_spelling`. Unlike
/// `scale_argument_spellings`, this returns a single pair rather than an
/// ordered vector: `scaleZ()` has no optional second argument (#663).
fn scalez_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (CssTransformScaleZArgumentKind, String) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::ScaleZ(scalez) = function else {
        panic!("expected scaleZ component {function_index} at {index}, got {function:?}");
    };

    let argument = scalez.argument();
    let token = result
        .transform_scalez_argument_token(argument.evidence_ref())
        .expect("scaleZ argument evidence did not resolve");
    (argument.kind(), authored_scalez_argument_spelling(token))
}

/// Reconstructs one retained `Number` or `Percentage` token's authored
/// numeric structure from tokenizer-owned evidence alone, exactly like
/// `authored_scalez_argument_spelling`, with a trailing `%` marker
/// distinguishing a `Percentage` token so `2` and `2%` stay distinguishable
/// in assertions and a `Percentage` is never collapsed into a `Number`.
/// This is a dedicated `scale3d()` test helper, distinct from
/// `authored_scalez_argument_spelling`: `scale3d()` argument placement,
/// `scaleZ()` argument placement, `scaleY()` argument placement, `scaleX()`
/// argument placement, and `scale()` argument placement remain distinct
/// semantic roles even though the underlying scalar spelling logic is
/// identical (#665). No machine number is ever produced.
fn authored_scale3d_argument_spelling(token: &CssTokenKind) -> String {
    let (value, is_percentage) = match token {
        CssTokenKind::Number { value, .. } => (value, false),
        CssTokenKind::Percentage { value } => (value, true),
        other => panic!(
            "scale3d argument evidence did not resolve to a Number/Percentage token, got {other:?}"
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

/// Resolves one qualified `scale3d()` transform component at
/// `function_index` within observation `index` into its ordered
/// `(x_kind, x, y_kind, y, z_kind, z)` authored evidence, preserving exact
/// X/Y/Z positional order and each slot's independent Number-vs-Percentage
/// kind, mirroring `translate3d_argument_spellings` except that all three
/// slots -- not just X/Y -- carry a kind, since `scale3d()`'s Z slot admits
/// `<percentage>` unlike `translate3d()`'s Z slot (#665).
fn scale3d_argument_spellings(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (
    CssTransformScale3dArgumentKind,
    String,
    CssTransformScale3dArgumentKind,
    String,
    CssTransformScale3dArgumentKind,
    String,
) {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Scale3d(scale3d) = function else {
        panic!("expected scale3d component {function_index} at {index}, got {function:?}");
    };

    let x = scale3d.x();
    let y = scale3d.y();
    let z = scale3d.z();
    let x_token = result
        .transform_scale3d_argument_token(x.evidence_ref())
        .expect("scale3d X evidence did not resolve");
    let y_token = result
        .transform_scale3d_argument_token(y.evidence_ref())
        .expect("scale3d Y evidence did not resolve");
    let z_token = result
        .transform_scale3d_argument_token(z.evidence_ref())
        .expect("scale3d Z evidence did not resolve");

    (
        x.kind(),
        authored_scale3d_argument_spelling(x_token),
        y.kind(),
        authored_scale3d_argument_spelling(y_token),
        z.kind(),
        authored_scale3d_argument_spelling(z_token),
    )
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

/// This is a dedicated `rotate()` test helper, distinct from
/// `authored_rotate3d_argument_spelling`: `rotate()` argument placement and
/// `rotate3d()`'s fourth-slot argument placement remain distinct semantic
/// roles even though the underlying Number/Dimension spelling logic is
/// identical (#667). No machine number is ever produced.
fn authored_rotate_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        other => {
            panic!(
                "rotate argument evidence did not resolve to a Number/Dimension token, got {other:?}"
            )
        }
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

fn rotate_function(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> CssTransformRotateFunction {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::Rotate(rotate) = function else {
        panic!("expected rotate component {function_index} at {index}, got {function:?}");
    };
    *rotate
}

/// Resolves one qualified `rotate()` transform component's single authored
/// argument at `function_index` within observation `index`, returning
/// `(is_angle, spelling)` -- `is_angle` is `true` for the `Angle` branch and
/// `false` for the `Zero` branch, so `0` and `0deg` stay distinguishable by
/// authored role, never merely by spelling, mirroring
/// `rotate3d_angle_spelling`'s fourth-slot resolution.
fn rotate_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (bool, String) {
    let rotate = rotate_function(result, index, function_index);
    let argument = rotate.argument();
    let token = result
        .transform_rotate_argument_token(argument.evidence_ref())
        .expect("rotate argument evidence did not resolve");
    match argument.kind() {
        CssTransformRotateArgumentKind::Angle => (true, authored_rotate_argument_spelling(token)),
        CssTransformRotateArgumentKind::Zero => (false, authored_rotate_argument_spelling(token)),
    }
}

/// This is a dedicated `rotateX()` test helper, distinct from
/// `authored_rotate_argument_spelling` and `authored_rotate3d_argument_spelling`:
/// `rotateX()` argument placement remains a distinct semantic role from
/// `rotate()`'s single authored slot and `rotate3d()`'s fourth-slot argument
/// placement even though the underlying Number/Dimension spelling logic is
/// identical (#669). No machine number is ever produced.
fn authored_rotatex_argument_spelling(token: &CssTokenKind) -> String {
    let (value, suffix) = match token {
        CssTokenKind::Number { value, .. } => (value, String::new()),
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.clone()),
        other => {
            panic!(
                "rotateX argument evidence did not resolve to a Number/Dimension token, got {other:?}"
            )
        }
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

fn rotatex_function(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> CssTransformRotateXFunction {
    let functions = qualified_functions(result, index);
    let function = functions
        .get(function_index)
        .unwrap_or_else(|| panic!("missing transform component {function_index} at {index}"));
    let CssTransformFunction::RotateX(rotatex) = function else {
        panic!("expected rotateX component {function_index} at {index}, got {function:?}");
    };
    *rotatex
}

/// Resolves one qualified `rotateX()` transform component's single authored
/// argument at `function_index` within observation `index`, returning
/// `(is_angle, spelling)` -- `is_angle` is `true` for the `Angle` branch and
/// `false` for the `Zero` branch, so `0` and `0deg` stay distinguishable by
/// authored role, never merely by spelling, mirroring
/// `rotate_argument_spelling`'s single-slot resolution.
fn rotatex_argument_spelling(
    result: &CssValueQualificationRunResult,
    index: usize,
    function_index: usize,
) -> (bool, String) {
    let rotatex = rotatex_function(result, index, function_index);
    let argument = rotatex.argument();
    let token = result
        .transform_rotatex_argument_token(argument.evidence_ref())
        .expect("rotateX argument evidence did not resolve");
    match argument.kind() {
        CssTransformRotateXArgumentKind::Angle => (true, authored_rotatex_argument_spelling(token)),
        CssTransformRotateXArgumentKind::Zero => (false, authored_rotatex_argument_spelling(token)),
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
            "rotateZ(1deg)",
            "matrix(1,0,0,1,0,0) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1) matrix(1,0,0,1,0,0)",
            "rotateY(1deg)",
            "skewX(1deg)",
            "skew(1deg)",
            "perspective(1px)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "unknownfunction(1,0,0,1,0,0)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1) skewX(1deg)",
            "scale(2) skewX(1deg)",
            "skewX(1deg) scale(2)",
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
        "d{transform:skewX(1deg);}",
        "e{transform:matrix(var(--x),0);}",
        "f{transform:matrix(1,2);}",
        "g{transform:matrix(1,0,0,1,0,0) none;}",
        "h{transform:scale(2) matrix(1,0,0,1,0,0);}",
        "i{transform:scale(calc(1),2);}",
        "j{transform:skew(1deg);}",
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
            "skewX(1deg) matrix(1,2)",
            "matrix(1,2) skewX(1deg)",
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
            "matrix(calc(1),0,0,1,0,0) skewX(1deg)",
            "skewX(1deg) matrix(calc(1),0,0,1,0,0)",
        ],
    );

    // Deferred substitution outranks both, in every authored position.
    assert_all_unsupported(
        418330,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "var(--x) skewX(1deg) matrix(1,2)",
            "matrix(1,2) skewX(1deg) var(--x)",
            "skewX(1deg) matrix(var(--x),0) matrix(1,2)",
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
    // stays outside selected-profile coverage. `scaleX()`/`scalex()`,
    // `scaleY()`/`scaley()`, `scaleZ()`/`scalez()`, and `scale3d()` are all
    // now selected separately (#659 / #661 / #663 / #665) and are covered by
    // their own dedicated test groups, so none of them is representative
    // here anymore; an unrecognized name that merely starts with `scale`
    // preserves the surviving invariant instead.
    assert_all_unsupported(
        645290,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &["scalefoo(1)"],
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
        &["scale(calc(1)) skewX(1deg)", "skewX(1deg) scale(calc(1))"],
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
            "skewX(1deg)",
            "translate3d(1px,2px,3px) skewX(1deg)",
            "skewX(1deg) translate3d(1px,2px,3px)",
            "translate3dx(1px,2px,3px)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translate3d argument, identically in both authored orders.
    assert_all_unsupported(
        647260,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translate3d(calc(1px),2px,3px) skewX(1deg)",
            "skewX(1deg) translate3d(calc(1px),2px,3px)",
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
            "skewX(1deg)",
            "rotate3d(1,0,0,90deg) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1) rotate3d(1,0,0,90deg)",
            "rotate3dx(1,0,0,90deg)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // rotate3d argument, identically in both authored orders. `skewX()` is
    // used as the outer sentinel rather than `rotate()`, which #667 selects
    // as its own distinct semantic placement, to avoid conflating this
    // rotate3d-only invariant with the newly qualified `rotate()` leaf.
    assert_all_unsupported(
        649270,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate3d(calc(1),0,0,90deg) skewX(1deg)",
            "skewX(1deg) rotate3d(calc(1),0,0,90deg)",
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

// 88. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque
// `translate()` argument (#651). `translateX()` itself becomes a selected
// leaf under #653 and is exercised separately, not here.

#[test]
fn unselected_outer_function_precedence_covers_translate() {
    assert_all_unsupported(
        651280,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translate(10px) skewX(10deg)",
            "skewX(10deg) translate(10px)",
            "translate(10px) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translate argument, identically in both authored orders.
    assert_all_unsupported(
        651290,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translate(calc(10px)) skewX(10deg)",
            "skewX(10deg) translate(calc(10px))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(651300, &["translate(1) skewX(10deg)"]);
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

// 95. `translateX() = translateX(<length-percentage>)` (#653): a canonical
// single direct `<length>` argument qualifies, preserving the `Length`
// role and exact tokenizer-owned evidence. Unlike `translate()`,
// `translateX()` has no optional second argument.

#[test]
fn canonical_translatex_length_qualifies() {
    let result = qualify(
        653100,
        concat!(
            "a{transform:translateX(10px);}",
            "b{transform:translateX(1em);}",
            "c{transform:translateX(-2rem);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "10px".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "1em".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 2, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "-2rem".to_string()
        )
    );
}

// 96. A canonical single direct `<percentage>` argument qualifies,
// preserving the `Percentage` role and exact tokenizer-owned evidence; a
// `Percentage` is never collapsed into a `Length` (#653).

#[test]
fn canonical_translatex_percentage_qualifies() {
    let result = qualify(
        653120,
        concat!(
            "a{transform:translateX(50%);}",
            "b{transform:translateX(-20%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Percentage,
            "50%".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateXArgumentKind::Percentage,
            "-20%".to_string()
        )
    );
}

// 97. A direct exact-zero `Number` satisfies `<length>`, reusing the
// accepted `translate` (#606) / `translate3d` (#647) / `translate()`
// (#651) exact-zero-Number-as-`Length` theorem; the retained evidence
// stays a `Number` token, never converted to a `Dimension` or interpreted
// magnitude (#653).

#[test]
fn exact_zero_number_qualifies_as_length_for_translatex() {
    let result = qualify(
        653140,
        concat!(
            "a{transform:translateX(0);}",
            "b{transform:translateX(+0);}",
            "c{transform:translateX(-0);}",
            "d{transform:translateX(.0);}",
            "e{transform:translateX(0.0);}",
            "f{transform:translateX(0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (CssTransformTranslateXArgumentKind::Length, "0".to_string())
    );
    assert_eq!(
        translatex_argument_spelling(&result, 1, 0),
        (CssTransformTranslateXArgumentKind::Length, "+0".to_string())
    );
    assert_eq!(
        translatex_argument_spelling(&result, 2, 0),
        (CssTransformTranslateXArgumentKind::Length, "-0".to_string())
    );
    // Authored `.0`: the absent leading integer digit is canonicalized to
    // `0` by the tokenizer's own retained numeric contract, upstream of
    // this leaf, exactly as for `translate()`/`translate3d()` arguments.
    assert_eq!(
        translatex_argument_spelling(&result, 3, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "0.0".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 4, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "0.0".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 5, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "0e100".to_string()
        )
    );
}

// 98. `0`, `0px`, and `0%` remain three distinct authored evidences in
// `translateX()`: none is ever normalized, synthesized, or collapsed into
// another (#653).

#[test]
fn translatex_zero_number_dimension_and_percentage_identity_is_preserved() {
    let result = qualify(
        653160,
        concat!(
            "a{transform:translateX(0);}",
            "b{transform:translateX(0px);}",
            "c{transform:translateX(0%);}",
        ),
    );

    let number = translatex_argument_spelling(&result, 0, 0);
    let px = translatex_argument_spelling(&result, 1, 0);
    let percentage = translatex_argument_spelling(&result, 2, 0);

    assert_eq!(
        number,
        (CssTransformTranslateXArgumentKind::Length, "0".to_string())
    );
    assert_eq!(
        px,
        (
            CssTransformTranslateXArgumentKind::Length,
            "0px".to_string()
        )
    );
    assert_eq!(
        percentage,
        (
            CssTransformTranslateXArgumentKind::Percentage,
            "0%".to_string()
        )
    );
    assert_ne!(number, px);
    assert_ne!(number, percentage);
    assert_ne!(px, percentage);
}

// 99. Representative recognized CSS length units qualify identically to
// `translate()`/`translate3d()` arguments, including a useful ASCII-case
// unit variant, reusing the same recognized-length-unit theorem (#653).

#[test]
fn recognized_css_length_units_qualify_for_translatex() {
    let result = qualify(
        653180,
        concat!(
            "a{transform:translateX(1px);}",
            "b{transform:translateX(1em);}",
            "c{transform:translateX(1rem);}",
            "d{transform:translateX(1vh);}",
            "e{transform:translateX(1vw);}",
            "f{transform:translateX(1cm);}",
            "g{transform:translateX(1PX);}",
            "h{transform:translateX(1Px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..8 {
        let (kind, _) = translatex_argument_spelling(&result, index, 0);
        assert_eq!(
            kind,
            CssTransformTranslateXArgumentKind::Length,
            "expected Length at {index}"
        );
    }
}

// 100. Signed direct `Length`, `Percentage`, and exact-zero `Number`
// arguments preserve exact authored sign/fraction/exponent evidence
// without any machine-number conversion (#653).

#[test]
fn signed_direct_values_preserve_authored_evidence_for_translatex() {
    let result = qualify(
        653200,
        concat!(
            "a{transform:translateX(-10px);}",
            "b{transform:translateX(+10%);}",
            "c{transform:translateX(-20%);}",
            "d{transform:translateX(-0);}",
            "e{transform:translateX(+0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 5);
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "-10px".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateXArgumentKind::Percentage,
            "+10%".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 2, 0),
        (
            CssTransformTranslateXArgumentKind::Percentage,
            "-20%".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 3, 0),
        (CssTransformTranslateXArgumentKind::Length, "-0".to_string())
    );
    assert_eq!(
        translatex_argument_spelling(&result, 4, 0),
        (CssTransformTranslateXArgumentKind::Length, "+0".to_string())
    );
}

// 101. `translateX()` accepts exactly one authored argument: zero, two,
// three, or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar`, unlike `translate()`'s one-or-two
// cardinality (#653).

#[test]
fn translatex_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        653220,
        &[
            "translateX()",
            "translateX(10px,20px)",
            "translateX(10px,20%,30px)",
            "translateX(1px,2px,3px,4px)",
        ],
    );
}

// 102. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#653).

#[test]
fn translatex_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        653240,
        &[
            "translateX(,)",
            "translateX(,10px)",
            "translateX(10px,)",
            "translateX(10px,,20px)",
        ],
    );
}

// 103. A nonzero unitless `Number` never satisfies `<length>`: this leaf
// never interprets an arbitrary `Number` as `Length` (#653).

#[test]
fn nonzero_number_is_invalid_for_translatex() {
    assert_all_invalid(
        653260,
        &["translateX(1)", "translateX(-1)", "translateX(.5)"],
    );
}

// 104. `<length-percentage>` admits only a direct recognized-length
// `Dimension`, an exact-zero `Number`, or a `Percentage`: every other
// direct token category at the translateX argument position is a decisive
// direct token-category failure (#653).

#[test]
fn wrong_direct_categories_are_invalid_for_translatex() {
    assert_all_invalid(
        653280,
        &[
            "translateX(45deg)",
            "translateX(1s)",
            "translateX(1hz)",
            "translateX(1dpi)",
            "translateX(1fr)",
            "translateX(1unknownunit)",
            "translateX(foo)",
            "translateX(\"1\")",
            "translateX(#abc)",
        ],
    );
}

// 105. A complete non-deferred Function occupying the single translateX
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason `translate()`/`translate3d()`
// opaque arguments use -- this leaf never evaluates `calc()` (#653).

#[test]
fn opaque_translatex_argument_function_is_unsupported() {
    assert_all_unsupported(
        653300,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translateX(calc(10px))",
            "translateX(min(10px,20px))",
            "translateX(max(10px,20px))",
            "translateX(clamp(0px,10px,20px))",
            "matrix(1,0,0,1,0,0) translateX(calc(10px))",
        ],
    );
}

// 106. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`:
// a structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#653).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_translatex() {
    assert_all_invalid(
        653320,
        &[
            "translateX(calc(10px) 1px)",
            "translateX(calc(10px) foo)",
            "translateX(calc(10px) 20%)",
        ],
    );
}

// 107. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a translateX-level argument separator. Once the
// nesting closes, a genuine second translateX-level slot is still counted
// and makes the shell decisively Invalid, since translateX() has no
// second slot to occupy (#653).

#[test]
fn nested_commas_never_change_translatex_arity() {
    assert_all_unsupported(
        653340,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["translateX(calc(10px,20px))"],
    );

    assert_all_invalid(653341, &["translateX(calc(10px,20px),30%)"]);
}

// 108. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// translateX shape conclusion -- including an arity that looks decisive
// (#653).

#[test]
fn translatex_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        653360,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "translateX(var(--x))",
            "translateX(var(--x),20px)",
            "matrix(1,0,0,1,0,0) translateX(var(--x))",
            "translateX(var(--x)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 109. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot (#653).

#[test]
fn translatex_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(653380, &["translateX(calc(10px),1)"]);
}

// 110. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, and `translateX()` components mix and repeat freely,
// preserving exact authored order and repetition through the
// heterogeneous `CssTransformFunction` alternation, and `translate()`/
// `translateX()` evidence never drifts across each other's index in a
// mixed sequence (#418 / #645 / #647 / #649 / #651 / #653).

#[test]
fn mixed_selected_function_order_including_translatex_is_preserved() {
    let result = qualify(
        653400,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) translateX(10px);}",
            "b{transform:translateX(10px) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) translateX(10px);}",
            "d{transform:translateX(10px) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) translateX(10px);}",
            "f{transform:translateX(10px) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) translateX(10px);}",
            "h{transform:translateX(10px) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(10px) translateX(20px);}",
            "j{transform:translateX(10px) translate(20px);}",
            "k{transform:translateX(1px) translateX(2px);}",
            "l{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 12);

    for index in 0..11 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 11).len(), 6);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let six_kind_sequence = qualified_functions(&result, 11);
    assert!(matches!(
        six_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        six_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        six_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        six_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        six_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        six_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));

    // Repeated `translateX()` preserves authored order and each
    // component's own evidence.
    let repeated = qualified_functions(&result, 10);
    assert!(matches!(repeated[0], CssTransformFunction::TranslateX(_)));
    assert!(matches!(repeated[1], CssTransformFunction::TranslateX(_)));
    assert_eq!(
        translatex_argument_spelling(&result, 10, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "1px".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 10, 1),
        (
            CssTransformTranslateXArgumentKind::Length,
            "2px".to_string()
        )
    );

    // `translate()` and `translateX()` remain distinct evidence indices in
    // a mixed sequence: neither drifts into the other's slot.
    assert!(matches!(
        qualified_functions(&result, 8)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 8)[1],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[0],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[1],
        CssTransformFunction::Translate(_)
    ));
}

// 111. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque
// `translateX()` argument (#653).

#[test]
fn unselected_outer_function_precedence_covers_translatex() {
    assert_all_unsupported(
        653420,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translateX(10px) skewX(10deg)",
            "skewX(10deg) translateX(10px)",
            "translateX(10px) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translateX argument, identically in both authored orders.
    assert_all_unsupported(
        653430,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translateX(calc(10px)) skewX(10deg)",
            "skewX(10deg) translateX(calc(10px))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(653440, &["translateX(1) skewX(10deg)"]);
}

// 112. `scale3d()` was formerly the remaining scale-family sibling this test
// proved stayed unselected alongside `translateX()` coverage (#653). #665
// selects `scale3d()` too, superseding that premise: there is no remaining
// unselected scale-family sibling to retarget this sentinel to. The
// surviving invariant -- that `translateX()` coverage never conflates a
// sibling selected function's representation or evidence with its own --
// is preserved instead by proving `translateX()` and `scale3d()` now
// compose correctly and remain distinct in either authored order (#665).

#[test]
fn translatex_and_scale3d_remain_distinct_selected_components() {
    let result = qualify(
        653450,
        concat!(
            "a{transform:translateX(20px) scale3d(1,2,3);}",
            "b{transform:scale3d(1,2,3) translateX(20px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::TranslateX(_)));
    assert!(matches!(first[1], CssTransformFunction::Scale3d(_)));

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::Scale3d(_)));
    assert!(matches!(second[1], CssTransformFunction::TranslateX(_)));
}

// 113. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two
// `translateX()` components or a `translateX()` and a sibling selected
// function (#653).

#[test]
fn top_level_comma_is_invalid_around_translatex() {
    assert_all_invalid(
        653470,
        &[
            "translateX(10px), matrix(1,0,0,1,0,0)",
            "translateX(10px), scale(2)",
            "translateX(10px), translate3d(1px,2px,3px)",
            "translateX(10px), rotate3d(1,0,0,90deg)",
            "translateX(10px), translate(10px)",
            "translateX(10px), translateX(20px)",
        ],
    );
}

// 114. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `translateX()` extent is qualified from
// retained interior evidence alone. EOF never fills or repairs a missing
// argument, a wrong direct category, or a second slot translateX() has no
// room for (#653).

#[test]
fn true_stylesheet_eof_ended_translatex_extent_follows_parser_authority() {
    let one_slot = qualify(653490, "a{transform:translateX(10px");
    assert_eq!(
        one_slot.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(one_slot.transform_observations().len(), 1);
    assert_eq!(
        translatex_argument_spelling(&one_slot, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "10px".to_string()
        )
    );

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(653491, "a{transform:translateX(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid: translateX()
    // never accepts a second slot.
    let trailing_comma = qualify(653492, "a{transform:translateX(10px,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs an invalid direct category.
    let invalid_argument = qualify(653493, "a{transform:translateX(1");
    assert_invalid(&invalid_argument, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a translateX".
    let trailing = qualify(653494, "a{transform:translateX(10px) 7;}");
    assert_invalid(&trailing, 0);

    // A second slot never widens translateX() cardinality, even at true
    // EOF.
    let two_slots = qualify(653495, "a{transform:translateX(10px,20%");
    assert_invalid(&two_slots, 0);
}

// 115. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#653).

#[test]
fn trivia_and_important_never_change_translatex_interpretation() {
    let result = qualify(
        653500,
        concat!(
            "a{transform:translateX(10px/**/);}",
            "b{transform:translateX(/**/10px);}",
            "c{transform:translateX( 10px );}",
            "d{transform:translateX(10px) !important;}",
            "e{transform:translateX(10px)!important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 5);
    let expected = (
        CssTransformTranslateXArgumentKind::Length,
        "10px".to_string(),
    );
    for index in 0..3 {
        assert_eq!(
            translatex_argument_spelling(&result, index, 0),
            expected,
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(translatex_argument_spelling(&result, index, 0), expected);
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(653510, &["translateX(/**/)", "translateX(10px,/**/)"]);
}

// 116. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension/Percentage tokens
// identically across runs (#653).

#[test]
fn translatex_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:translateX(10px,20%);}",
        "b{transform:translateX(calc(10px,20px));}",
        "c{transform:translateX(1);}",
        "d{transform:matrix(1,0,0,1,0,0) translateX(10px);}",
    );

    let first = qualify(653520, css);
    let repeated = qualify(653520, css);
    let another_source = qualify(653521, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        translatex_argument_spelling(&first, 3, 1),
        translatex_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        translatex_argument_spelling(&first, 3, 1),
        translatex_argument_spelling(&another_source, 3, 1)
    );
}

// 117. `translateX` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate` boundary, and structurally malformed `translateX`
// components -- a bare Function name, an unbalanced/duplicated closer, or
// stray leading material -- stay decisively `Invalid` (#653).

#[test]
fn translatex_function_name_recognition_and_malformed_components() {
    let result = qualify(
        653530,
        concat!(
            "a{transform:TRANSLATEX(10px);}",
            "b{transform:TrAnSlAtEx(10px);}",
            "c{transform:t\\72 anslateX(10px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            translatex_argument_spelling(&result, index, 0),
            (
                CssTransformTranslateXArgumentKind::Length,
                "10px".to_string()
            ),
            "translateX name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        653540,
        &[
            "translateX",
            "translateX(10px))",
            "translateX((10px)",
            "1 translateX(10px)",
            "[translateX(10px)]",
        ],
    );
}

// 118. Cross-leaf isolation: `translateX()` recognition never leaks into
// the accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `translateX()` (#418 / #606 / #645 / #647 /
// #649 / #651 / #653).

#[test]
fn translatex_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        653550,
        concat!(
            "a{transform:translateX(10px);transform:none;}",
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
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "10px".to_string()
        )
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(653560, &["none translateX(10px)", "translateX(10px) none"]);
}

// 119. `translateY() = translateY(<length-percentage>)` (#655): a canonical
// single direct `<length>` argument qualifies, preserving the `Length`
// role and exact tokenizer-owned evidence. Like `translateX()` and unlike
// `translate()`, `translateY()` has no optional second argument.

#[test]
fn canonical_translatey_length_qualifies() {
    let result = qualify(
        655100,
        concat!(
            "a{transform:translateY(10px);}",
            "b{transform:translateY(1em);}",
            "c{transform:translateY(-2rem);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(
        translatey_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "10px".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "1em".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 2, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "-2rem".to_string()
        )
    );
}

// 120. A canonical single direct `<percentage>` argument qualifies,
// preserving the `Percentage` role and exact tokenizer-owned evidence; a
// `Percentage` is never collapsed into a `Length` (#655).

#[test]
fn canonical_translatey_percentage_qualifies() {
    let result = qualify(
        655120,
        concat!(
            "a{transform:translateY(50%);}",
            "b{transform:translateY(-20%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(
        translatey_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "50%".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "-20%".to_string()
        )
    );
}

// 121. A direct exact-zero `Number` satisfies `<length>`, reusing the
// accepted `translate` (#606) / `translate3d` (#647) / `translate()`
// (#651) / `translateX()` (#653) exact-zero-Number-as-`Length` theorem; the
// retained evidence stays a `Number` token, never converted to a
// `Dimension` or interpreted magnitude (#655).

#[test]
fn exact_zero_number_qualifies_as_length_for_translatey() {
    let result = qualify(
        655140,
        concat!(
            "a{transform:translateY(0);}",
            "b{transform:translateY(+0);}",
            "c{transform:translateY(-0);}",
            "d{transform:translateY(.0);}",
            "e{transform:translateY(0.0);}",
            "f{transform:translateY(0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    assert_eq!(
        translatey_argument_spelling(&result, 0, 0),
        (CssTransformTranslateYArgumentKind::Length, "0".to_string())
    );
    assert_eq!(
        translatey_argument_spelling(&result, 1, 0),
        (CssTransformTranslateYArgumentKind::Length, "+0".to_string())
    );
    assert_eq!(
        translatey_argument_spelling(&result, 2, 0),
        (CssTransformTranslateYArgumentKind::Length, "-0".to_string())
    );
    // Authored `.0`: the absent leading integer digit is canonicalized to
    // `0` by the tokenizer's own retained numeric contract, upstream of
    // this leaf, exactly as for `translate()`/`translate3d()`/
    // `translateX()` arguments.
    assert_eq!(
        translatey_argument_spelling(&result, 3, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "0.0".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 4, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "0.0".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 5, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "0e100".to_string()
        )
    );
}

// 122. `0`, `0px`, and `0%` remain three distinct authored evidences in
// `translateY()`: none is ever normalized, synthesized, or collapsed into
// another (#655).

#[test]
fn translatey_zero_number_dimension_and_percentage_identity_is_preserved() {
    let result = qualify(
        655160,
        concat!(
            "a{transform:translateY(0);}",
            "b{transform:translateY(0px);}",
            "c{transform:translateY(0%);}",
        ),
    );

    let number = translatey_argument_spelling(&result, 0, 0);
    let px = translatey_argument_spelling(&result, 1, 0);
    let percentage = translatey_argument_spelling(&result, 2, 0);

    assert_eq!(
        number,
        (CssTransformTranslateYArgumentKind::Length, "0".to_string())
    );
    assert_eq!(
        px,
        (
            CssTransformTranslateYArgumentKind::Length,
            "0px".to_string()
        )
    );
    assert_eq!(
        percentage,
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "0%".to_string()
        )
    );
    assert_ne!(number, px);
    assert_ne!(number, percentage);
    assert_ne!(px, percentage);
}

// 123. Representative recognized CSS length units qualify identically to
// `translate()`/`translate3d()`/`translateX()` arguments, including a
// useful ASCII-case unit variant, reusing the same recognized-length-unit
// theorem (#655).

#[test]
fn recognized_css_length_units_qualify_for_translatey() {
    let result = qualify(
        655180,
        concat!(
            "a{transform:translateY(1px);}",
            "b{transform:translateY(1em);}",
            "c{transform:translateY(1rem);}",
            "d{transform:translateY(1vh);}",
            "e{transform:translateY(1vw);}",
            "f{transform:translateY(1cm);}",
            "g{transform:translateY(1PX);}",
            "h{transform:translateY(1Px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..8 {
        let (kind, _) = translatey_argument_spelling(&result, index, 0);
        assert_eq!(
            kind,
            CssTransformTranslateYArgumentKind::Length,
            "expected Length at {index}"
        );
    }
}

// 124. Signed direct `Length`, `Percentage`, and exact-zero `Number`
// arguments preserve exact authored sign/fraction/exponent evidence
// without any machine-number conversion (#655).

#[test]
fn signed_direct_values_preserve_authored_evidence_for_translatey() {
    let result = qualify(
        655200,
        concat!(
            "a{transform:translateY(-10px);}",
            "b{transform:translateY(+10%);}",
            "c{transform:translateY(-20%);}",
            "d{transform:translateY(-0);}",
            "e{transform:translateY(+0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 5);
    assert_eq!(
        translatey_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "-10px".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "+10%".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 2, 0),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "-20%".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 3, 0),
        (CssTransformTranslateYArgumentKind::Length, "-0".to_string())
    );
    assert_eq!(
        translatey_argument_spelling(&result, 4, 0),
        (CssTransformTranslateYArgumentKind::Length, "+0".to_string())
    );
}

// 125. `translateY()` accepts exactly one authored argument: zero, two,
// three, or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar`, unlike `translate()`'s one-or-two
// cardinality (#655).

#[test]
fn translatey_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        655220,
        &[
            "translateY()",
            "translateY(10px,20px)",
            "translateY(10px,20%,30px)",
            "translateY(1px,2px,3px,4px)",
        ],
    );
}

// 126. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#655).

#[test]
fn translatey_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        655240,
        &[
            "translateY(,)",
            "translateY(,10px)",
            "translateY(10px,)",
            "translateY(10px,,20px)",
        ],
    );
}

// 127. A nonzero unitless `Number` never satisfies `<length>`: this leaf
// never interprets an arbitrary `Number` as `Length` (#655).

#[test]
fn nonzero_number_is_invalid_for_translatey() {
    assert_all_invalid(
        655260,
        &["translateY(1)", "translateY(-1)", "translateY(.5)"],
    );
}

// 128. `<length-percentage>` admits only a direct recognized-length
// `Dimension`, an exact-zero `Number`, or a `Percentage`: every other
// direct token category at the translateY argument position is a decisive
// direct token-category failure (#655).

#[test]
fn wrong_direct_categories_are_invalid_for_translatey() {
    assert_all_invalid(
        655280,
        &[
            "translateY(45deg)",
            "translateY(1s)",
            "translateY(1hz)",
            "translateY(1dpi)",
            "translateY(1fr)",
            "translateY(1unknownunit)",
            "translateY(foo)",
            "translateY(\"1\")",
            "translateY(#abc)",
        ],
    );
}

// 129. A complete non-deferred Function occupying the single translateY
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason `translate()`/`translate3d()`/
// `translateX()` opaque arguments use -- this leaf never evaluates
// `calc()` (#655).

#[test]
fn opaque_translatey_argument_function_is_unsupported() {
    assert_all_unsupported(
        655300,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translateY(calc(10px))",
            "translateY(min(10px,20px))",
            "translateY(max(10px,20px))",
            "translateY(clamp(0px,10px,20px))",
            "matrix(1,0,0,1,0,0) translateY(calc(10px))",
        ],
    );
}

// 130. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`:
// a structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#655).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_translatey() {
    assert_all_invalid(
        655320,
        &[
            "translateY(calc(10px) 1px)",
            "translateY(calc(10px) foo)",
            "translateY(calc(10px) 20%)",
        ],
    );
}

// 131. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a translateY-level argument separator. Once the
// nesting closes, a genuine second translateY-level slot is still counted
// and makes the shell decisively Invalid, since translateY() has no
// second slot to occupy (#655).

#[test]
fn nested_commas_never_change_translatey_arity() {
    assert_all_unsupported(
        655340,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["translateY(calc(10px,20px))"],
    );

    assert_all_invalid(655341, &["translateY(calc(10px,20px),30%)"]);
}

// 132. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// translateY shape conclusion -- including an arity that looks decisive
// (#655).

#[test]
fn translatey_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        655360,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "translateY(var(--y))",
            "translateY(var(--y),20px)",
            "matrix(1,0,0,1,0,0) translateY(var(--y))",
            "translateY(var(--y)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 133. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot (#655).

#[test]
fn translatey_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(655380, &["translateY(calc(10px),1)"]);
}

// 134. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, and `translateY()` components mix and
// repeat freely, preserving exact authored order and repetition through
// the heterogeneous `CssTransformFunction` alternation, and no selected
// leaf's evidence drifts across another's index in a mixed sequence
// (#418 / #645 / #647 / #649 / #651 / #653 / #655).

#[test]
fn mixed_selected_function_order_including_translatey_is_preserved() {
    let result = qualify(
        655400,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) translateY(10px);}",
            "b{transform:translateY(10px) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) translateY(10px);}",
            "d{transform:translateY(10px) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) translateY(10px);}",
            "f{transform:translateY(10px) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) translateY(10px);}",
            "h{transform:translateY(10px) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(10px) translateY(20px);}",
            "j{transform:translateY(10px) translate(20px);}",
            "k{transform:translateX(10px) translateY(20px);}",
            "l{transform:translateY(10px) translateX(20px);}",
            "m{transform:translateY(1px) translateY(2px);}",
            "n{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 14);

    for index in 0..13 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 13).len(), 7);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let seven_kind_sequence = qualified_functions(&result, 13);
    assert!(matches!(
        seven_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        seven_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        seven_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        seven_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        seven_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        seven_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        seven_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));

    // Repeated `translateY()` preserves authored order and each
    // component's own evidence.
    let repeated = qualified_functions(&result, 12);
    assert!(matches!(repeated[0], CssTransformFunction::TranslateY(_)));
    assert!(matches!(repeated[1], CssTransformFunction::TranslateY(_)));
    assert_eq!(
        translatey_argument_spelling(&result, 12, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "1px".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 12, 1),
        (
            CssTransformTranslateYArgumentKind::Length,
            "2px".to_string()
        )
    );

    // `translate()` and `translateY()` remain distinct evidence indices in
    // a mixed sequence: neither drifts into the other's slot.
    assert!(matches!(
        qualified_functions(&result, 8)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 8)[1],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[0],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[1],
        CssTransformFunction::Translate(_)
    ));
}

// 135. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque
// `translateY()` argument (#655).

#[test]
fn unselected_outer_function_precedence_covers_translatey() {
    assert_all_unsupported(
        655420,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translateY(10px) skewX(10deg)",
            "skewX(10deg) translateY(10px)",
            "translateY(10px) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translateY argument, identically in both authored orders.
    assert_all_unsupported(
        655430,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translateY(calc(10px)) skewX(10deg)",
            "skewX(10deg) translateY(calc(10px))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(655440, &["translateY(1) skewX(10deg)"]);
}

// 136. `translateX()` and `translateY()` evidence never drifts across each
// other's index in a mixed authored sequence, in either order: each
// resolves to its own argument's own evidence, and neither variant nor
// evidence reference leaks into the other's semantic placement. This is
// load-bearing for #655's scope invariant that `TranslateX` and
// `TranslateY` stay distinct representations despite identical grammar.

#[test]
fn translatex_translatey_evidence_separation_never_drifts() {
    let result = qualify(
        655450,
        concat!(
            "a{transform:translateX(10px) translateY(20%);}",
            "b{transform:translateY(30%) translateX(40px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert!(matches!(first[0], CssTransformFunction::TranslateX(_)));
    assert!(matches!(first[1], CssTransformFunction::TranslateY(_)));
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "10px".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 0, 1),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "20%".to_string()
        )
    );

    let second = qualified_functions(&result, 1);
    assert!(matches!(second[0], CssTransformFunction::TranslateY(_)));
    assert!(matches!(second[1], CssTransformFunction::TranslateX(_)));
    assert_eq!(
        translatey_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "30%".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 1, 1),
        (
            CssTransformTranslateXArgumentKind::Length,
            "40px".to_string()
        )
    );
}

// 137. `translateZ()` remains outside selected-profile coverage after
// #655: extending coverage with `translateY()` never widens the selected
// profile to a sibling `<transform-function>` still outside it -- unlike
// `translateZ()`, which #657 goes on to select, `skewX()`, `matrix3d()`,
// and `perspective()` remain outside selected-profile coverage in
// isolation, with any argument shape, or alongside a qualified
// `translateY()` in either order (#655 / #657). `scale3d()` was formerly
// listed here too; #665 selects it separately, so this generic sentinel was
// retargeted to `rotate()`. #667 selects `rotate()` too, superseding that
// retarget in turn: the sentinel is now `skewX()`, a function this task's
// scope keeps outside `<transform-function>` selected-profile coverage, to
// preserve the "sibling stays unselected" invariant without depending on
// `rotate()`.

#[test]
fn remaining_transform_siblings_stay_unselected_alongside_qualified_translatey() {
    assert_all_unsupported(
        655460,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "skewX(1deg)",
            "perspective(10px)",
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            "translateY(10px) skewX(1deg)",
            "skewX(1deg) translateY(10px)",
        ],
    );
}

// 138. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two
// `translateY()` components or a `translateY()` and a sibling selected
// function (#655).

#[test]
fn top_level_comma_is_invalid_around_translatey() {
    assert_all_invalid(
        655470,
        &[
            "translateY(10px), matrix(1,0,0,1,0,0)",
            "translateY(10px), scale(2)",
            "translateY(10px), translate3d(1px,2px,3px)",
            "translateY(10px), rotate3d(1,0,0,90deg)",
            "translateY(10px), translate(10px)",
            "translateY(10px), translateX(20px)",
            "translateY(10px), translateY(20px)",
        ],
    );
}

// 139. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `translateY()` extent is qualified from
// retained interior evidence alone. EOF never fills or repairs a missing
// argument, a wrong direct category, or a second slot translateY() has no
// room for (#655).

#[test]
fn true_stylesheet_eof_ended_translatey_extent_follows_parser_authority() {
    let one_slot = qualify(655490, "a{transform:translateY(10px");
    assert_eq!(
        one_slot.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(one_slot.transform_observations().len(), 1);
    assert_eq!(
        translatey_argument_spelling(&one_slot, 0, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "10px".to_string()
        )
    );

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(655491, "a{transform:translateY(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid: translateY()
    // never accepts a second slot.
    let trailing_comma = qualify(655492, "a{transform:translateY(10px,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs an invalid direct category.
    let invalid_argument = qualify(655493, "a{transform:translateY(1");
    assert_invalid(&invalid_argument, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a translateY".
    let trailing = qualify(655494, "a{transform:translateY(10px) 7;}");
    assert_invalid(&trailing, 0);

    // A second slot never widens translateY() cardinality, even at true
    // EOF.
    let two_slots = qualify(655495, "a{transform:translateY(10px,20%");
    assert_invalid(&two_slots, 0);
}

// 140. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#655).

#[test]
fn trivia_and_important_never_change_translatey_interpretation() {
    let result = qualify(
        655500,
        concat!(
            "a{transform:translateY(10px/**/);}",
            "b{transform:translateY(/**/10px);}",
            "c{transform:translateY( 10px );}",
            "d{transform:translateY(10px) !important;}",
            "e{transform:translateY(10px)!important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 5);
    let expected = (
        CssTransformTranslateYArgumentKind::Length,
        "10px".to_string(),
    );
    for index in 0..3 {
        assert_eq!(
            translatey_argument_spelling(&result, index, 0),
            expected,
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(translatey_argument_spelling(&result, index, 0), expected);
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(655510, &["translateY(/**/)", "translateY(10px,/**/)"]);
}

// 141. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension/Percentage tokens
// identically across runs (#655).

#[test]
fn translatey_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:translateY(10px,20%);}",
        "b{transform:translateY(calc(10px,20px));}",
        "c{transform:translateY(1);}",
        "d{transform:matrix(1,0,0,1,0,0) translateY(10px);}",
    );

    let first = qualify(655520, css);
    let repeated = qualify(655520, css);
    let another_source = qualify(655521, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        translatey_argument_spelling(&first, 3, 1),
        translatey_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        translatey_argument_spelling(&first, 3, 1),
        translatey_argument_spelling(&another_source, 3, 1)
    );
}

// 142. `translateY` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX` boundary, and structurally malformed
// `translateY` components -- a bare Function name, an
// unbalanced/duplicated closer, or stray leading material -- stay
// decisively `Invalid` (#655).

#[test]
fn translatey_function_name_recognition_and_malformed_components() {
    let result = qualify(
        655530,
        concat!(
            "a{transform:TRANSLATEY(10px);}",
            "b{transform:TrAnSlAtEy(10px);}",
            "c{transform:t\\72 anslateY(10px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            translatey_argument_spelling(&result, index, 0),
            (
                CssTransformTranslateYArgumentKind::Length,
                "10px".to_string()
            ),
            "translateY name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        655540,
        &[
            "translateY",
            "translateY(10px))",
            "translateY((10px)",
            "1 translateY(10px)",
            "[translateY(10px)]",
        ],
    );
}

// 143. Cross-leaf isolation: `translateY()` recognition never leaks into
// the accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `translateY()` (#418 / #606 / #645 / #647 /
// #649 / #651 / #653 / #655).

#[test]
fn translatey_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        655550,
        concat!(
            "a{transform:translateY(10px);transform:none;}",
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
        translatey_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateYArgumentKind::Length,
            "10px".to_string()
        )
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(655560, &["none translateY(10px)", "translateY(10px) none"]);
}

// 144. `translateZ() = translateZ(<length>)` (#657): a canonical single
// direct `<length>` argument qualifies, preserving the `Length` role and
// exact tokenizer-owned evidence. Unlike `translateX()`/`translateY()`,
// `translateZ()` accepts only `<length>`, never `<percentage>`.

#[test]
fn canonical_translatez_length_qualifies() {
    let result = qualify(
        657100,
        concat!(
            "a{transform:translateZ(10px);}",
            "b{transform:translateZ(1em);}",
            "c{transform:translateZ(-2rem);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(translatez_argument_spelling(&result, 0, 0), "10px");
    assert_eq!(translatez_argument_spelling(&result, 1, 0), "1em");
    assert_eq!(translatez_argument_spelling(&result, 2, 0), "-2rem");
}

// 145. A direct exact-zero `Number` satisfies `<length>`, reusing the
// accepted `translate` (#606) / `translate3d` (#647) / `translate()` (#651)
// / `translateX()` (#653) / `translateY()` (#655) exact-zero-Number-as-
// `Length` theorem; the retained evidence stays a `Number` token, never
// converted to a `Dimension` or interpreted magnitude (#657).

#[test]
fn exact_zero_number_qualifies_as_length_for_translatez() {
    let result = qualify(
        657140,
        concat!(
            "a{transform:translateZ(0);}",
            "b{transform:translateZ(+0);}",
            "c{transform:translateZ(-0);}",
            "d{transform:translateZ(.0);}",
            "e{transform:translateZ(0.0);}",
            "f{transform:translateZ(0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    assert_eq!(translatez_argument_spelling(&result, 0, 0), "0");
    assert_eq!(translatez_argument_spelling(&result, 1, 0), "+0");
    assert_eq!(translatez_argument_spelling(&result, 2, 0), "-0");
    // Authored `.0`: the absent leading integer digit is canonicalized to
    // `0` by the tokenizer's own retained numeric contract, upstream of
    // this leaf, exactly as for `translate()`/`translate3d()`/
    // `translateX()`/`translateY()` arguments.
    assert_eq!(translatez_argument_spelling(&result, 3, 0), "0.0");
    assert_eq!(translatez_argument_spelling(&result, 4, 0), "0.0");
    assert_eq!(translatez_argument_spelling(&result, 5, 0), "0e100");
}

// 146. `0`, `0px`, and `0%` remain three distinct authored evidences at the
// `translateZ()` boundary: `0` and `0px` both qualify as `Length` through
// distinct Number/Dimension evidence, while `0%` is decisively `Invalid`
// because `translateZ()` admits only `<length>` -- a `Percentage` is never
// treated as zero `Length` merely because its numeric magnitude is zero.
// This boundary is load-bearing for #657.

#[test]
fn translatez_zero_number_dimension_and_percentage_boundary_is_preserved() {
    let result = qualify(
        657160,
        concat!(
            "a{transform:translateZ(0);}",
            "b{transform:translateZ(0px);}",
        ),
    );

    let number = translatez_argument_spelling(&result, 0, 0);
    let px = translatez_argument_spelling(&result, 1, 0);
    assert_eq!(number, "0");
    assert_eq!(px, "0px");
    assert_ne!(number, px);

    assert_all_invalid(657165, &["translateZ(0%)"]);
}

// 147. Direct `Percentage` is decisively `Invalid` for `translateZ()`,
// including `0%`, `50%`, `-20%`, and `+10%`: this leaf never applies the
// `translateX()`/`translateY()` `<length-percentage>` classifier, and never
// resolves a percentage basis (#657).

#[test]
fn percentage_is_invalid_for_translatez_including_zero_percent() {
    assert_all_invalid(
        657180,
        &[
            "translateZ(0%)",
            "translateZ(50%)",
            "translateZ(-20%)",
            "translateZ(+10%)",
        ],
    );
}

// 148. `translateX()`/`translateY()` `<length-percentage>` acceptance is
// unaffected by adding `translateZ()`: representative `Percentage`
// arguments remain `Qualified` for `translateX()`/`translateY()` while the
// identical spelling is decisively `Invalid` for `translateZ()`, proving
// this leaf never accidentally routed `translateZ()` through the X/Y
// classifier, nor `translateX()`/`translateY()` through a Length-only one
// (#657).

#[test]
fn translatex_translatey_percentage_regression_against_translatez() {
    let result = qualify(
        657200,
        concat!(
            "a{transform:translateX(50%);}",
            "b{transform:translateY(50%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Percentage,
            "50%".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 1, 0),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "50%".to_string()
        )
    );

    assert_all_invalid(657210, &["translateZ(50%)"]);
}

// 149. Representative recognized CSS length units qualify identically to
// `translate()`/`translate3d()`/`translateX()`/`translateY()` arguments,
// including a useful ASCII-case unit variant, reusing the same
// recognized-length-unit theorem (#657).

#[test]
fn recognized_css_length_units_qualify_for_translatez() {
    let result = qualify(
        657220,
        concat!(
            "a{transform:translateZ(1px);}",
            "b{transform:translateZ(1em);}",
            "c{transform:translateZ(1rem);}",
            "d{transform:translateZ(1vh);}",
            "e{transform:translateZ(1vw);}",
            "f{transform:translateZ(1cm);}",
            "g{transform:translateZ(1PX);}",
            "h{transform:translateZ(1Px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for (index, expected_unit) in ["px", "em", "rem", "vh", "vw", "cm", "PX", "Px"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            translatez_argument_spelling(&result, index, 0),
            format!("1{expected_unit}"),
            "expected Length at {index}"
        );
    }
}

// 150. Signed direct `Length` and exact-zero `Number` arguments preserve
// exact authored sign/fraction/exponent evidence without any
// machine-number conversion (#657).

#[test]
fn signed_direct_length_values_preserve_authored_evidence_for_translatez() {
    let result = qualify(
        657240,
        concat!(
            "a{transform:translateZ(-10px);}",
            "b{transform:translateZ(-0);}",
            "c{transform:translateZ(+0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    assert_eq!(translatez_argument_spelling(&result, 0, 0), "-10px");
    assert_eq!(translatez_argument_spelling(&result, 1, 0), "-0");
    assert_eq!(translatez_argument_spelling(&result, 2, 0), "+0");
}

// 151. `translateZ()` accepts exactly one authored argument: zero, two,
// three, or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` (#657).

#[test]
fn translatez_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        657260,
        &[
            "translateZ()",
            "translateZ(10px,20px)",
            "translateZ(10px,20%,30px)",
            "translateZ(1px,2px,3px,4px)",
        ],
    );
}

// 152. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#657).

#[test]
fn translatez_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        657280,
        &[
            "translateZ(,)",
            "translateZ(,10px)",
            "translateZ(10px,)",
            "translateZ(10px,,20px)",
        ],
    );
}

// 153. A nonzero unitless `Number` never satisfies `<length>`: this leaf
// never interprets an arbitrary `Number` as `Length` (#657).

#[test]
fn nonzero_number_is_invalid_for_translatez() {
    assert_all_invalid(
        657300,
        &["translateZ(1)", "translateZ(-1)", "translateZ(.5)"],
    );
}

// 154. `<length>` admits only a direct recognized-length `Dimension` or an
// exact-zero `Number`: every other direct token category at the
// `translateZ()` argument position -- including `Percentage`, covered
// separately above -- is a decisive direct token-category failure (#657).

#[test]
fn wrong_direct_categories_are_invalid_for_translatez() {
    assert_all_invalid(
        657320,
        &[
            "translateZ(45deg)",
            "translateZ(1s)",
            "translateZ(1hz)",
            "translateZ(1dpi)",
            "translateZ(1fr)",
            "translateZ(1unknownunit)",
            "translateZ(foo)",
            "translateZ(\"1\")",
            "translateZ(#abc)",
        ],
    );
}

// 155. A complete non-deferred Function occupying the single `translateZ()`
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason `translate()`/`translate3d()`/
// `translateX()`/`translateY()` opaque arguments use -- this leaf never
// evaluates `calc()` (#657).

#[test]
fn opaque_translatez_argument_function_is_unsupported() {
    assert_all_unsupported(
        657340,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "translateZ(calc(10px))",
            "translateZ(min(10px,20px))",
            "translateZ(max(10px,20px))",
            "translateZ(clamp(0px,10px,20px))",
            "matrix(1,0,0,1,0,0) translateZ(calc(10px))",
        ],
    );
}

// 156. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#657).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_translatez() {
    assert_all_invalid(
        657360,
        &[
            "translateZ(calc(10px) 1px)",
            "translateZ(calc(10px) foo)",
            "translateZ(calc(10px) 20%)",
        ],
    );
}

// 157. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a translateZ-level argument separator. Once the
// nesting closes, a genuine second translateZ-level slot is still counted
// and makes the shell decisively Invalid, since translateZ() has no second
// slot to occupy (#657).

#[test]
fn nested_commas_never_change_translatez_arity() {
    assert_all_unsupported(
        657380,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["translateZ(calc(10px,20px))"],
    );

    assert_all_invalid(657381, &["translateZ(calc(10px,20px),30px)"]);
}

// 158. An opaque Function whose inner content looks like a `Percentage` is
// not directly classified by its unevaluated inner result type: this leaf
// does not own CSS math type/value resolution, so `calc(0%)`/`calc(50%)`
// occupying the single `translateZ()` slot stay selected-profile
// Unsupported via the shared opaque-Function boundary, exactly like any
// other complete non-deferred Function -- never a direct `Percentage`
// `Invalid` (#657).

#[test]
fn opaque_percentage_looking_function_remains_unsupported_for_translatez() {
    assert_all_unsupported(
        657400,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["translateZ(calc(0%))", "translateZ(calc(50%))"],
    );
}

// 159. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// translateZ shape conclusion -- including an arity that looks decisive
// (#657).

#[test]
fn translatez_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        657420,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "translateZ(var(--z))",
            "translateZ(var(--z),20px)",
            "matrix(1,0,0,1,0,0) translateZ(var(--z))",
            "translateZ(var(--z)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 160. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot; a direct Percentage failure remains decisive
// even alongside an outer unselected sibling, in either authored order
// (#657).

#[test]
fn translatez_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(657440, &["translateZ(calc(10px),1)"]);

    assert_all_invalid(
        657441,
        &["translateZ(0%) skewX(10deg)", "skewX(10deg) translateZ(0%)"],
    );
}

// 161. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, and `translateZ()`
// components mix and repeat freely, preserving exact authored order and
// repetition through the heterogeneous `CssTransformFunction` alternation,
// and no selected leaf's evidence drifts across another's index in a mixed
// sequence (#418 / #645 / #647 / #649 / #651 / #653 / #655 / #657).

#[test]
fn mixed_selected_function_order_including_translatez_is_preserved() {
    let result = qualify(
        657460,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) translateZ(10px);}",
            "b{transform:translateZ(10px) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) translateZ(10px);}",
            "d{transform:translateZ(10px) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) translateZ(10px);}",
            "f{transform:translateZ(10px) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) translateZ(10px);}",
            "h{transform:translateZ(10px) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(10px) translateZ(20px);}",
            "j{transform:translateZ(10px) translate(20px);}",
            "k{transform:translateY(10px) translateZ(20px);}",
            "l{transform:translateZ(10px) translateY(20px);}",
            "m{transform:translateZ(1px) translateZ(2px);}",
            "n{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 14);

    for index in 0..13 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 13).len(), 8);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let eight_kind_sequence = qualified_functions(&result, 13);
    assert!(matches!(
        eight_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        eight_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        eight_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        eight_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        eight_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        eight_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        eight_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        eight_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));

    // Repeated `translateZ()` preserves authored order and each
    // component's own evidence.
    let repeated = qualified_functions(&result, 12);
    assert!(matches!(repeated[0], CssTransformFunction::TranslateZ(_)));
    assert!(matches!(repeated[1], CssTransformFunction::TranslateZ(_)));
    assert_eq!(translatez_argument_spelling(&result, 12, 0), "1px");
    assert_eq!(translatez_argument_spelling(&result, 12, 1), "2px");

    // `translate()` and `translateZ()` remain distinct evidence indices in
    // a mixed sequence: neither drifts into the other's slot.
    assert!(matches!(
        qualified_functions(&result, 8)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 8)[1],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[0],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[1],
        CssTransformFunction::Translate(_)
    ));
}

// 162. `translateX()`, `translateY()`, and `translateZ()` evidence never
// drifts across each other's index in a mixed authored sequence, in any
// order: each resolves to its own argument's own evidence, and neither
// variant nor evidence reference leaks into another's semantic placement.
// This is load-bearing for #657's scope invariant that `TranslateX`,
// `TranslateY`, and `TranslateZ` stay distinct representations despite
// `TranslateX`/`TranslateY` sharing an identical argument grammar.

#[test]
fn translatex_translatey_translatez_evidence_separation_never_drifts() {
    let result = qualify(
        657480,
        concat!(
            "a{transform:translateX(10px) translateY(20%) translateZ(30px);}",
            "b{transform:translateZ(40px) translateY(50%) translateX(60px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert!(matches!(first[0], CssTransformFunction::TranslateX(_)));
    assert!(matches!(first[1], CssTransformFunction::TranslateY(_)));
    assert!(matches!(first[2], CssTransformFunction::TranslateZ(_)));
    assert_eq!(
        translatex_argument_spelling(&result, 0, 0),
        (
            CssTransformTranslateXArgumentKind::Length,
            "10px".to_string()
        )
    );
    assert_eq!(
        translatey_argument_spelling(&result, 0, 1),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "20%".to_string()
        )
    );
    assert_eq!(translatez_argument_spelling(&result, 0, 2), "30px");

    let second = qualified_functions(&result, 1);
    assert!(matches!(second[0], CssTransformFunction::TranslateZ(_)));
    assert!(matches!(second[1], CssTransformFunction::TranslateY(_)));
    assert!(matches!(second[2], CssTransformFunction::TranslateX(_)));
    assert_eq!(translatez_argument_spelling(&result, 1, 0), "40px");
    assert_eq!(
        translatey_argument_spelling(&result, 1, 1),
        (
            CssTransformTranslateYArgumentKind::Percentage,
            "50%".to_string()
        )
    );
    assert_eq!(
        translatex_argument_spelling(&result, 1, 2),
        (
            CssTransformTranslateXArgumentKind::Length,
            "60px".to_string()
        )
    );
}

// 163. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque
// `translateZ()` argument (#657).

#[test]
fn unselected_outer_function_precedence_covers_translatez() {
    assert_all_unsupported(
        657500,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translateZ(10px) skewX(10deg)",
            "skewX(10deg) translateZ(10px)",
            "translateZ(10px) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // translateZ argument, identically in both authored orders.
    assert_all_unsupported(
        657510,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "translateZ(calc(10px)) skewX(10deg)",
            "skewX(10deg) translateZ(calc(10px))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(657520, &["translateZ(1) skewX(10deg)"]);
}

// 164. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two
// `translateZ()` components or a `translateZ()` and a sibling selected
// function (#657).

#[test]
fn top_level_comma_is_invalid_around_translatez() {
    assert_all_invalid(
        657540,
        &[
            "translateZ(10px), matrix(1,0,0,1,0,0)",
            "translateZ(10px), scale(2)",
            "translateZ(10px), translate3d(1px,2px,3px)",
            "translateZ(10px), rotate3d(1,0,0,90deg)",
            "translateZ(10px), translate(10px)",
            "translateZ(10px), translateX(20px)",
            "translateZ(10px), translateY(20px)",
            "translateZ(10px), translateZ(20px)",
        ],
    );
}

// 165. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `translateZ()` extent is qualified from
// retained interior evidence alone. EOF never fills or repairs a missing
// argument, a wrong direct category, a direct `Percentage`, or a second
// slot `translateZ()` has no room for (#657).

#[test]
fn true_stylesheet_eof_ended_translatez_extent_follows_parser_authority() {
    let one_slot = qualify(657560, "a{transform:translateZ(10px");
    assert_eq!(
        one_slot.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(one_slot.transform_observations().len(), 1);
    assert_eq!(translatez_argument_spelling(&one_slot, 0, 0), "10px");

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(657561, "a{transform:translateZ(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid: translateZ()
    // never accepts a second slot.
    let trailing_comma = qualify(657562, "a{transform:translateZ(10px,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs a nonzero-Number direct category.
    let invalid_argument = qualify(657563, "a{transform:translateZ(1");
    assert_invalid(&invalid_argument, 0);

    // EOF never repairs a direct Percentage into a Length.
    let percentage_eof = qualify(657564, "a{transform:translateZ(0%");
    assert_invalid(&percentage_eof, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a translateZ".
    let trailing = qualify(657565, "a{transform:translateZ(10px) 7;}");
    assert_invalid(&trailing, 0);

    // A second slot never widens translateZ() cardinality, even at true
    // EOF.
    let two_slots = qualify(657566, "a{transform:translateZ(10px,20px");
    assert_invalid(&two_slots, 0);
}

// 166. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#657).

#[test]
fn trivia_and_important_never_change_translatez_interpretation() {
    let result = qualify(
        657580,
        concat!(
            "a{transform:translateZ(10px/**/);}",
            "b{transform:translateZ(/**/10px);}",
            "c{transform:translateZ( 10px );}",
            "d{transform:translateZ(10px) !important;}",
            "e{transform:translateZ(10px)!important;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 5);
    for index in 0..3 {
        assert_eq!(
            translatez_argument_spelling(&result, index, 0),
            "10px",
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(translatez_argument_spelling(&result, index, 0), "10px");
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(657590, &["translateZ(/**/)", "translateZ(10px,/**/)"]);
}

// 167. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension tokens identically
// across runs (#657).

#[test]
fn translatez_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:translateZ(10px,20px);}",
        "b{transform:translateZ(calc(10px,20px));}",
        "c{transform:translateZ(1);}",
        "d{transform:matrix(1,0,0,1,0,0) translateZ(10px);}",
    );

    let first = qualify(657600, css);
    let repeated = qualify(657600, css);
    let another_source = qualify(657601, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        translatez_argument_spelling(&first, 3, 1),
        translatez_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        translatez_argument_spelling(&first, 3, 1),
        translatez_argument_spelling(&another_source, 3, 1)
    );
}

// 168. `translateZ` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY` boundary, and structurally
// malformed `translateZ` components -- a bare Function name, an
// unbalanced/duplicated closer, or stray leading material -- stay
// decisively `Invalid` (#657).

#[test]
fn translatez_function_name_recognition_and_malformed_components() {
    let result = qualify(
        657620,
        concat!(
            "a{transform:TRANSLATEZ(10px);}",
            "b{transform:TrAnSlAtEz(10px);}",
            "c{transform:t\\72 anslateZ(10px);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            translatez_argument_spelling(&result, index, 0),
            "10px",
            "translateZ name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        657630,
        &[
            "translateZ",
            "translateZ(10px))",
            "translateZ((10px)",
            "1 translateZ(10px)",
            "[translateZ(10px)]",
        ],
    );
}

// 169. Cross-leaf isolation: `translateZ()` recognition never leaks into
// the accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `translateZ()` (#418 / #606 / #645 / #647 / #649
// / #651 / #653 / #655 / #657).

#[test]
fn translatez_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        657640,
        concat!(
            "a{transform:translateZ(10px);transform:none;}",
            "b{translate:10px;}",
            "c{scale:2;}",
            "d{rotate:45deg;}",
            "e{transform-origin:left top;}",
            "f{transform-box:border-box;}",
            "g{transform-style:flat;}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);
    assert_eq!(translatez_argument_spelling(&result, 0, 0), "10px");
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(657650, &["none translateZ(10px)", "translateZ(10px) none"]);
}

// 170. `scaleX() = scaleX([<number> | <percentage>])` under current CSS
// Transforms Level 2 authority (#659): a direct `<number>` argument
// qualifies, preserving exact authored sign/fraction/exponent evidence
// without any machine-number conversion, reusing the accepted `scale()`
// (#645) Number theorem. Unlike `translateX()`/`translateY()`/`translateZ()`,
// `scaleX()` never restricts a `Number` to exact-zero: a nonzero `Number`
// is directly valid here.

#[test]
fn canonical_scalex_number_qualifies() {
    let result = qualify(
        659100,
        concat!(
            "a{transform:scaleX(2);}",
            "b{transform:scaleX(-1);}",
            "c{transform:scaleX(0);}",
            "d{transform:scaleX(+0);}",
            "e{transform:scaleX(-0);}",
            "f{transform:scaleX(.5);}",
            "g{transform:scaleX(1.5);}",
            "h{transform:scaleX(1e2);}",
            "i{transform:scaleX(-1e-2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 9);
    let expected = [
        "2", "-1", "0", "+0", "-0",
        // Authored `.5`: the absent leading integer digit is canonicalized
        // to `0` by the tokenizer's own retained numeric contract, upstream
        // of this leaf, exactly as for `scale()` arguments.
        "0.5", "1.5", "1e2", "-1e-2",
    ];
    for (index, spelling) in expected.into_iter().enumerate() {
        assert_eq!(
            scalex_argument_spelling(&result, index, 0),
            (CssTransformScaleXArgumentKind::Number, spelling.to_string()),
            "expected Number at {index}"
        );
    }
}

// 171. A direct `<percentage>` argument qualifies identically to `<number>`
// under current CSS Transforms Level 2 authority, retaining current Level 2
// Percentage support rather than regressing to the older Level 1
// `<number>`-only grammar (#659).

#[test]
fn canonical_scalex_percentage_qualifies() {
    let result = qualify(
        659120,
        concat!(
            "a{transform:scaleX(250%);}",
            "b{transform:scaleX(0%);}",
            "c{transform:scaleX(-20%);}",
            "d{transform:scaleX(+10%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    for (index, spelling) in ["250%", "0%", "-20%", "+10%"].into_iter().enumerate() {
        assert_eq!(
            scalex_argument_spelling(&result, index, 0),
            (
                CssTransformScaleXArgumentKind::Percentage,
                spelling.to_string()
            ),
            "expected Percentage at {index}"
        );
    }
}

// 172. `scaleX(0)` and `scaleX(0%)` remain pairwise distinguishable: `0`
// qualifies through `Number` evidence and `0%` qualifies through
// `Percentage` evidence, never collapsed into each other merely because
// their downstream transform semantics may be numerically equivalent. This
// boundary is load-bearing for #659.

#[test]
fn scalex_zero_number_vs_zero_percent_identity() {
    let result = qualify(
        659140,
        concat!("a{transform:scaleX(0);}", "b{transform:scaleX(0%);}",),
    );

    let number = scalex_argument_spelling(&result, 0, 0);
    let percentage = scalex_argument_spelling(&result, 1, 0);
    assert_eq!(
        number,
        (CssTransformScaleXArgumentKind::Number, "0".to_string())
    );
    assert_eq!(
        percentage,
        (CssTransformScaleXArgumentKind::Percentage, "0%".to_string())
    );
    assert_ne!(number.0, percentage.0);
    assert_ne!(number.1, percentage.1);
}

// 173. `scaleX()` accepts exactly one authored argument: zero, two, three,
// or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` (#659).

#[test]
fn scalex_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        659160,
        &[
            "scaleX()",
            "scaleX(1,2)",
            "scaleX(1,2,3)",
            "scaleX(2,3,4,5)",
        ],
    );
}

// 174. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#659).

#[test]
fn scalex_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        659180,
        &["scaleX(,)", "scaleX(,1)", "scaleX(1,)", "scaleX(1,,2)"],
    );
}

// 175. `ScaleXArgument := DirectNumber | DirectPercentage` admits no other
// direct token category: a `Dimension` -- including a recognized length,
// angle, or unrecognized unit -- and every other direct token category are
// decisive direct token-category failures (#659). No direct `Dimension` is
// ever accepted, unlike a proposed or historical scale-length theorem this
// leaf does not implement.

#[test]
fn wrong_direct_categories_are_invalid_for_scalex() {
    assert_all_invalid(
        659200,
        &[
            "scaleX(1px)",
            "scaleX(1em)",
            "scaleX(45deg)",
            "scaleX(1s)",
            "scaleX(1fr)",
            "scaleX(1unknownunit)",
            "scaleX(foo)",
            "scaleX(\"1\")",
            "scaleX(#abc)",
        ],
    );
}

// 176. A complete non-deferred Function occupying the single `scaleX()`
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason `scale()`/`translate()`/
// `translate3d()`/`translateX()`/`translateY()`/`translateZ()` opaque
// arguments use -- this leaf never evaluates `calc()` and never decides an
// opaque Function's inner result type, so a `calc()` whose inner content
// looks like a `Percentage` stays Unsupported via the same shared boundary
// (#659).

#[test]
fn opaque_scalex_argument_function_is_unsupported() {
    assert_all_unsupported(
        659220,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "scaleX(calc(1))",
            "scaleX(calc(100%))",
            "scaleX(min(1,2))",
            "scaleX(max(1,2))",
            "scaleX(clamp(0,1,2))",
            "scaleX(calc(0%))",
            "scaleX(calc(50%))",
            "matrix(1,0,0,1,0,0) scaleX(calc(1))",
        ],
    );
}

// 177. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#659).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_scalex() {
    assert_all_invalid(
        659240,
        &[
            "scaleX(calc(1) 2)",
            "scaleX(calc(1) foo)",
            "scaleX(calc(1) 20%)",
        ],
    );
}

// 178. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a scaleX-level argument separator. Once the
// nesting closes, a genuine second scaleX-level slot is still counted and
// makes the shell decisively Invalid, since `scaleX()` has no second slot
// to occupy (#659).

#[test]
fn nested_commas_never_change_scalex_arity() {
    assert_all_unsupported(
        659260,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["scaleX(calc(1,2))"],
    );

    assert_all_invalid(659261, &["scaleX(calc(1,2),3)"]);
}

// 179. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// scaleX shape conclusion -- including an arity that looks decisive (#659).

#[test]
fn scalex_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        659280,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "scaleX(var(--x))",
            "scaleX(var(--x),2)",
            "matrix(1,0,0,1,0,0) scaleX(var(--x))",
            "scaleX(var(--x)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 180. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot; a direct wrong-category failure remains
// decisive even alongside an outer unselected sibling, in either authored
// order (#659).

#[test]
fn scalex_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(659300, &["scaleX(calc(1),1)"]);

    assert_all_invalid(
        659301,
        &["scaleX(1px) skewX(10deg)", "skewX(10deg) scaleX(1px)"],
    );
}

// 181. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, `translateZ()`, and
// `scaleX()` components mix and repeat freely, preserving exact authored
// order and repetition through the heterogeneous `CssTransformFunction`
// alternation, and no selected leaf's evidence drifts across another's
// index in a mixed sequence (#418 / #645 / #647 / #649 / #651 / #653 /
// #655 / #657 / #659).

#[test]
fn mixed_selected_function_order_including_scalex_is_preserved() {
    let result = qualify(
        659320,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) scaleX(2);}",
            "b{transform:scaleX(2) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) scaleX(3);}",
            "d{transform:scaleX(3) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) scaleX(2);}",
            "f{transform:scaleX(2) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) scaleX(2);}",
            "h{transform:scaleX(2) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(10px) scaleX(2);}",
            "j{transform:scaleX(2) translate(10px);}",
            "k{transform:translateX(10px) scaleX(2);}",
            "l{transform:scaleX(2) translateX(10px);}",
            "m{transform:translateY(10px) scaleX(2);}",
            "n{transform:scaleX(2) translateY(10px);}",
            "o{transform:translateZ(10px) scaleX(2);}",
            "p{transform:scaleX(2) translateZ(10px);}",
            "q{transform:scaleX(1) scaleX(2);}",
            "r{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px) scaleX(50);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 18);

    for index in 0..17 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 17).len(), 9);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let nine_kind_sequence = qualified_functions(&result, 17);
    assert!(matches!(
        nine_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        nine_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        nine_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        nine_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        nine_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        nine_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        nine_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        nine_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        nine_kind_sequence[8],
        CssTransformFunction::ScaleX(_)
    ));

    // Repeated `scaleX()` preserves authored order and each component's own
    // evidence.
    let repeated = qualified_functions(&result, 16);
    assert!(matches!(repeated[0], CssTransformFunction::ScaleX(_)));
    assert!(matches!(repeated[1], CssTransformFunction::ScaleX(_)));
    assert_eq!(
        scalex_argument_spelling(&result, 16, 0),
        (CssTransformScaleXArgumentKind::Number, "1".to_string())
    );
    assert_eq!(
        scalex_argument_spelling(&result, 16, 1),
        (CssTransformScaleXArgumentKind::Number, "2".to_string())
    );

    // `translate()` and `scaleX()` remain distinct evidence indices in a
    // mixed sequence: neither drifts into the other's slot.
    assert!(matches!(
        qualified_functions(&result, 8)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 8)[1],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[0],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[1],
        CssTransformFunction::Translate(_)
    ));
}

// 182. `scale()` and `scaleX()` remain distinct semantic placements: neither
// their representation, cardinality, nor evidence leaks into the other in a
// mixed sequence, in either authored order, and `scale()` retains its
// accepted one-or-two authored cardinality unaffected by `scaleX()`'s
// exact-one cardinality (#645 / #659).

#[test]
fn scale_scalex_separation_preserves_distinct_representation_and_evidence() {
    let result = qualify(
        659340,
        concat!(
            "a{transform:scale(1,200%) scaleX(300%);}",
            "b{transform:scaleX(400%) scale(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::Scale(_)));
    assert!(matches!(first[1], CssTransformFunction::ScaleX(_)));
    assert_eq!(
        scale_argument_spellings(&result, 0, 0),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
    assert_eq!(
        scalex_argument_spelling(&result, 0, 1),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "300%".to_string()
        )
    );

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::ScaleX(_)));
    assert!(matches!(second[1], CssTransformFunction::Scale(_)));
    assert_eq!(
        scalex_argument_spelling(&result, 1, 0),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "400%".to_string()
        )
    );
    assert_eq!(
        scale_argument_spellings(&result, 1, 1),
        vec![(CssTransformScaleArgumentKind::Number, "2".to_string())]
    );
}

// 183. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque `scaleX()`
// argument (#659).

#[test]
fn unselected_outer_function_precedence_covers_scalex() {
    assert_all_unsupported(
        659360,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scaleX(2) skewX(10deg)",
            "skewX(10deg) scaleX(2)",
            "scaleX(2) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // scaleX argument, identically in both authored orders.
    assert_all_unsupported(
        659380,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scaleX(calc(1)) skewX(10deg)",
            "skewX(10deg) scaleX(calc(1))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(659400, &["scaleX(1px) skewX(10deg)"]);
}

// 184. `scale3d()` was formerly the remaining scale-family sibling this
// test proved stayed unselected alongside `scaleX()` coverage (#659). #665
// selects `scale3d()` too, superseding that premise: there is no remaining
// unselected scale-family sibling to retarget this sentinel to. The
// surviving invariant -- that `scaleX()` coverage never conflates a sibling
// selected function's representation or evidence with its own -- is
// preserved instead by proving `scaleX()` and `scale3d()` now compose
// correctly and remain distinct in either authored order (#665).

#[test]
fn scalex_and_scale3d_remain_distinct_selected_components() {
    let result = qualify(
        659420,
        concat!(
            "a{transform:scaleX(2) scale3d(1,2,3);}",
            "b{transform:scale3d(1,2,3) scaleX(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::ScaleX(_)));
    assert!(matches!(first[1], CssTransformFunction::Scale3d(_)));

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::Scale3d(_)));
    assert!(matches!(second[1], CssTransformFunction::ScaleX(_)));
}

// 185. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two `scaleX()`
// components or a `scaleX()` and a sibling selected function (#659).

#[test]
fn top_level_comma_is_invalid_around_scalex() {
    assert_all_invalid(
        659440,
        &[
            "scaleX(2), matrix(1,0,0,1,0,0)",
            "scaleX(2), scale(2)",
            "scaleX(2), translate3d(1px,2px,3px)",
            "scaleX(2), rotate3d(1,0,0,90deg)",
            "scaleX(2), translate(10px)",
            "scaleX(2), translateX(20px)",
            "scaleX(2), translateY(20px)",
            "scaleX(2), translateZ(20px)",
            "scaleX(2), scaleX(3)",
        ],
    );
}

// 186. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `scaleX()` extent is qualified from retained
// interior evidence alone. EOF never fills or repairs a missing argument, a
// wrong direct category, or a second slot `scaleX()` has no room for
// (#659).

#[test]
fn true_stylesheet_eof_ended_scalex_extent_follows_parser_authority() {
    let number_eof = qualify(659460, "a{transform:scaleX(2");
    assert_eq!(
        number_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(number_eof.transform_observations().len(), 1);
    assert_eq!(
        scalex_argument_spelling(&number_eof, 0, 0),
        (CssTransformScaleXArgumentKind::Number, "2".to_string())
    );

    let percentage_eof = qualify(659461, "a{transform:scaleX(50%");
    assert_eq!(
        percentage_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(percentage_eof.transform_observations().len(), 1);
    assert_eq!(
        scalex_argument_spelling(&percentage_eof, 0, 0),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "50%".to_string()
        )
    );

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(659462, "a{transform:scaleX(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid: scaleX()
    // never accepts a second slot.
    let trailing_comma = qualify(659463, "a{transform:scaleX(2,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs a direct Dimension category.
    let wrong_category_eof = qualify(659464, "a{transform:scaleX(1px");
    assert_invalid(&wrong_category_eof, 0);

    // A second slot never widens scaleX() cardinality, even at true EOF.
    let two_slots_eof = qualify(659465, "a{transform:scaleX(1,2");
    assert_invalid(&two_slots_eof, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a scaleX".
    let trailing = qualify(659466, "a{transform:scaleX(2) 7;}");
    assert_invalid(&trailing, 0);
}

// 187. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#659).

#[test]
fn trivia_and_important_never_change_scalex_interpretation() {
    let result = qualify(
        659480,
        concat!(
            "a{transform:scaleX(2/**/);}",
            "b{transform:scaleX(/**/2);}",
            "c{transform:scaleX( 2 );}",
            "d{transform:scaleX(2) !important;}",
            "e{transform:scaleX(2)!important;}",
            "f{transform:scaleX(/**/50%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..3 {
        assert_eq!(
            scalex_argument_spelling(&result, index, 0),
            (CssTransformScaleXArgumentKind::Number, "2".to_string()),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(
            scalex_argument_spelling(&result, index, 0),
            (CssTransformScaleXArgumentKind::Number, "2".to_string())
        );
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
    assert_eq!(
        scalex_argument_spelling(&result, 5, 0),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "50%".to_string()
        ),
        "trivia changed Percentage slot interpretation"
    );

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(659500, &["scaleX(/**/)", "scaleX(2,/**/)"]);
}

// 188. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Percentage tokens
// identically across runs (#659).

#[test]
fn scalex_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:scaleX(1,2);}",
        "b{transform:scaleX(calc(1,2));}",
        "c{transform:scaleX(1px);}",
        "d{transform:matrix(1,0,0,1,0,0) scaleX(2);}",
        "e{transform:scaleX(50%);}",
    );

    let first = qualify(659520, css);
    let repeated = qualify(659520, css);
    let another_source = qualify(659521, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        scalex_argument_spelling(&first, 3, 1),
        scalex_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        scalex_argument_spelling(&first, 3, 1),
        scalex_argument_spelling(&another_source, 3, 1)
    );
    assert_eq!(
        scalex_argument_spelling(&first, 4, 0),
        scalex_argument_spelling(&repeated, 4, 0)
    );
    assert_eq!(
        scalex_argument_spelling(&first, 4, 0),
        scalex_argument_spelling(&another_source, 4, 0)
    );
}

// 189. `scaleX` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY`/`translateZ` boundary, and
// structurally malformed `scaleX` components -- a bare Function name, an
// unbalanced/duplicated closer, or stray leading material -- stay
// decisively `Invalid` (#659).

#[test]
fn scalex_function_name_recognition_and_malformed_components() {
    let result = qualify(
        659540,
        concat!(
            "a{transform:SCALEX(2);}",
            "b{transform:ScAlEx(2);}",
            "c{transform:s\\63 alex(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            scalex_argument_spelling(&result, index, 0),
            (CssTransformScaleXArgumentKind::Number, "2".to_string()),
            "scaleX name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        659560,
        &[
            "scaleX",
            "scaleX(2))",
            "scaleX((2)",
            "1 scaleX(2)",
            "[scaleX(2)]",
        ],
    );
}

// 190. Cross-leaf isolation: `scaleX()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `scaleX()` (#418 / #606 / #645 / #647 / #649 /
// #651 / #653 / #655 / #657 / #659).

#[test]
fn scalex_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        659580,
        concat!(
            "a{transform:scaleX(2);transform:none;}",
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
        scalex_argument_spelling(&result, 0, 0),
        (CssTransformScaleXArgumentKind::Number, "2".to_string())
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(659600, &["none scaleX(2)", "scaleX(2) none"]);
}

// 191. `scaleY() = scaleY([<number> | <percentage>])` under current CSS
// Transforms Level 2 authority (#661): a direct `<number>` argument
// qualifies, preserving exact authored sign/fraction/exponent evidence
// without any machine-number conversion, reusing the accepted `scale()`
// (#645) / `scaleX()` (#659) Number theorem. Like `scaleX()`, `scaleY()`
// never restricts a `Number` to exact-zero: a nonzero `Number` is directly
// valid here.

#[test]
fn canonical_scaley_number_qualifies() {
    let result = qualify(
        661100,
        concat!(
            "a{transform:scaleY(2);}",
            "b{transform:scaleY(-1);}",
            "c{transform:scaleY(0);}",
            "d{transform:scaleY(+0);}",
            "e{transform:scaleY(-0);}",
            "f{transform:scaleY(.5);}",
            "g{transform:scaleY(1.5);}",
            "h{transform:scaleY(1e2);}",
            "i{transform:scaleY(-1e-2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 9);
    let expected = [
        "2", "-1", "0", "+0", "-0",
        // Authored `.5`: the absent leading integer digit is canonicalized
        // to `0` by the tokenizer's own retained numeric contract, upstream
        // of this leaf, exactly as for `scale()`/`scaleX()` arguments.
        "0.5", "1.5", "1e2", "-1e-2",
    ];
    for (index, spelling) in expected.into_iter().enumerate() {
        assert_eq!(
            scaley_argument_spelling(&result, index, 0),
            (CssTransformScaleYArgumentKind::Number, spelling.to_string()),
            "expected Number at {index}"
        );
    }
}

// 192. A direct `<percentage>` argument qualifies identically to `<number>`
// under current CSS Transforms Level 2 authority, retaining current Level 2
// Percentage support rather than regressing to the older Level 1
// `<number>`-only grammar (#661).

#[test]
fn canonical_scaley_percentage_qualifies() {
    let result = qualify(
        661120,
        concat!(
            "a{transform:scaleY(250%);}",
            "b{transform:scaleY(0%);}",
            "c{transform:scaleY(-20%);}",
            "d{transform:scaleY(+10%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    for (index, spelling) in ["250%", "0%", "-20%", "+10%"].into_iter().enumerate() {
        assert_eq!(
            scaley_argument_spelling(&result, index, 0),
            (
                CssTransformScaleYArgumentKind::Percentage,
                spelling.to_string()
            ),
            "expected Percentage at {index}"
        );
    }
}

// 193. `scaleY(0)` and `scaleY(0%)` remain pairwise distinguishable: `0`
// qualifies through `Number` evidence and `0%` qualifies through
// `Percentage` evidence, never collapsed into each other merely because
// their downstream transform semantics may be numerically equivalent. This
// boundary is load-bearing for #661.

#[test]
fn scaley_zero_number_vs_zero_percent_identity() {
    let result = qualify(
        661140,
        concat!("a{transform:scaleY(0);}", "b{transform:scaleY(0%);}",),
    );

    let number = scaley_argument_spelling(&result, 0, 0);
    let percentage = scaley_argument_spelling(&result, 1, 0);
    assert_eq!(
        number,
        (CssTransformScaleYArgumentKind::Number, "0".to_string())
    );
    assert_eq!(
        percentage,
        (CssTransformScaleYArgumentKind::Percentage, "0%".to_string())
    );
    assert_ne!(number.0, percentage.0);
    assert_ne!(number.1, percentage.1);
}

// 194. `scaleY()` accepts exactly one authored argument: zero, two, three,
// or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` (#661).

#[test]
fn scaley_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        661160,
        &[
            "scaleY()",
            "scaleY(1,2)",
            "scaleY(1,2,3)",
            "scaleY(2,3,4,5)",
        ],
    );
}

// 195. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#661).

#[test]
fn scaley_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        661180,
        &["scaleY(,)", "scaleY(,1)", "scaleY(1,)", "scaleY(1,,2)"],
    );
}

// 196. `ScaleYArgument := DirectNumber | DirectPercentage` admits no other
// direct token category: a `Dimension` -- including a recognized length,
// angle, or unrecognized unit -- and every other direct token category are
// decisive direct token-category failures (#661). No direct `Dimension` is
// ever accepted, unlike a proposed or historical scale-length theorem this
// leaf does not implement.

#[test]
fn wrong_direct_categories_are_invalid_for_scaley() {
    assert_all_invalid(
        661200,
        &[
            "scaleY(1px)",
            "scaleY(1em)",
            "scaleY(45deg)",
            "scaleY(1s)",
            "scaleY(1fr)",
            "scaleY(1unknownunit)",
            "scaleY(foo)",
            "scaleY(\"1\")",
            "scaleY(#abc)",
        ],
    );
}

// 197. A complete non-deferred Function occupying the single `scaleY()`
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason `scale()`/`scaleX()`/
// `translate()`/`translate3d()`/`translateX()`/`translateY()`/
// `translateZ()` opaque arguments use -- this leaf never evaluates `calc()`
// and never decides an opaque Function's inner result type, so a `calc()`
// whose inner content looks like a `Percentage` stays Unsupported via the
// same shared boundary (#661).

#[test]
fn opaque_scaley_argument_function_is_unsupported() {
    assert_all_unsupported(
        661220,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "scaleY(calc(1))",
            "scaleY(calc(100%))",
            "scaleY(min(1,2))",
            "scaleY(max(1,2))",
            "scaleY(clamp(0,1,2))",
            "scaleY(calc(0%))",
            "scaleY(calc(50%))",
            "matrix(1,0,0,1,0,0) scaleY(calc(1))",
        ],
    );
}

// 198. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#661).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_scaley() {
    assert_all_invalid(
        661240,
        &[
            "scaleY(calc(1) 2)",
            "scaleY(calc(1) foo)",
            "scaleY(calc(1) 20%)",
        ],
    );
}

// 199. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a scaleY-level argument separator. Once the
// nesting closes, a genuine second scaleY-level slot is still counted and
// makes the shell decisively Invalid, since `scaleY()` has no second slot
// to occupy (#661).

#[test]
fn nested_commas_never_change_scaley_arity() {
    assert_all_unsupported(
        661260,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["scaleY(calc(1,2))"],
    );

    assert_all_invalid(661261, &["scaleY(calc(1,2),3)"]);
}

// 200. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// scaleY shape conclusion -- including an arity that looks decisive (#661).

#[test]
fn scaley_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        661280,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "scaleY(var(--y))",
            "scaleY(var(--y),2)",
            "matrix(1,0,0,1,0,0) scaleY(var(--y))",
            "scaleY(var(--y)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 201. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot; a direct wrong-category failure remains
// decisive even alongside an outer unselected sibling, in either authored
// order (#661).

#[test]
fn scaley_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(661300, &["scaleY(calc(1),1)"]);

    assert_all_invalid(
        661301,
        &["scaleY(1px) skewX(10deg)", "skewX(10deg) scaleY(1px)"],
    );
}

// 202. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, `translateZ()`,
// `scaleX()`, and `scaleY()` components mix and repeat freely, preserving
// exact authored order and repetition through the heterogeneous
// `CssTransformFunction` alternation, and no selected leaf's evidence
// drifts across another's index in a mixed sequence (#418 / #645 / #647 /
// #649 / #651 / #653 / #655 / #657 / #659 / #661).

#[test]
fn mixed_selected_function_order_including_scaley_is_preserved() {
    let result = qualify(
        661320,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) scaleY(2);}",
            "b{transform:scaleY(2) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) scaleY(3);}",
            "d{transform:scaleY(3) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) scaleY(2);}",
            "f{transform:scaleY(2) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) scaleY(2);}",
            "h{transform:scaleY(2) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(10px) scaleY(2);}",
            "j{transform:scaleY(2) translate(10px);}",
            "k{transform:translateX(10px) scaleY(2);}",
            "l{transform:scaleY(2) translateX(10px);}",
            "m{transform:translateY(10px) scaleY(2);}",
            "n{transform:scaleY(2) translateY(10px);}",
            "o{transform:translateZ(10px) scaleY(2);}",
            "p{transform:scaleY(2) translateZ(10px);}",
            "q{transform:scaleX(20%) scaleY(30%);}",
            "r{transform:scaleY(40%) scaleX(50%);}",
            "s{transform:scaleY(1) scaleY(2);}",
            "t{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px) scaleX(50) scaleY(60);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 20);

    for index in 0..19 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 19).len(), 10);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let ten_kind_sequence = qualified_functions(&result, 19);
    assert!(matches!(
        ten_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        ten_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        ten_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        ten_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        ten_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        ten_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        ten_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        ten_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        ten_kind_sequence[8],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        ten_kind_sequence[9],
        CssTransformFunction::ScaleY(_)
    ));

    // Repeated `scaleY()` preserves authored order and each component's own
    // evidence.
    let repeated = qualified_functions(&result, 18);
    assert!(matches!(repeated[0], CssTransformFunction::ScaleY(_)));
    assert!(matches!(repeated[1], CssTransformFunction::ScaleY(_)));
    assert_eq!(
        scaley_argument_spelling(&result, 18, 0),
        (CssTransformScaleYArgumentKind::Number, "1".to_string())
    );
    assert_eq!(
        scaley_argument_spelling(&result, 18, 1),
        (CssTransformScaleYArgumentKind::Number, "2".to_string())
    );

    // `translate()` and `scaleY()` remain distinct evidence indices in a
    // mixed sequence: neither drifts into the other's slot.
    assert!(matches!(
        qualified_functions(&result, 8)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 8)[1],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[0],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[1],
        CssTransformFunction::Translate(_)
    ));

    // `scaleX()` and `scaleY()` remain distinct semantic placements in
    // either authored order: neither representation nor evidence leaks into
    // the other, and evidence indices resolve independently.
    assert!(matches!(
        qualified_functions(&result, 16)[0],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 16)[1],
        CssTransformFunction::ScaleY(_)
    ));
    assert_eq!(
        scalex_argument_spelling(&result, 16, 0),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "20%".to_string()
        )
    );
    assert_eq!(
        scaley_argument_spelling(&result, 16, 1),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "30%".to_string()
        )
    );
    assert!(matches!(
        qualified_functions(&result, 17)[0],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 17)[1],
        CssTransformFunction::ScaleX(_)
    ));
    assert_eq!(
        scaley_argument_spelling(&result, 17, 0),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "40%".to_string()
        )
    );
    assert_eq!(
        scalex_argument_spelling(&result, 17, 1),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "50%".to_string()
        )
    );
}

// 203. `scale()`, `scaleX()`, and `scaleY()` remain distinct semantic
// placements: neither their representation, cardinality, nor evidence
// leaks into one another in a mixed sequence, in any authored order, and
// `scale()` retains its accepted one-or-two authored cardinality
// unaffected by `scaleX()`'s and `scaleY()`'s exact-one cardinality
// (#645 / #659 / #661).

#[test]
fn scale_scalex_scaley_separation_preserves_distinct_representation_and_evidence() {
    let result = qualify(
        661340,
        concat!(
            "a{transform:scale(1,200%) scaleX(300%) scaleY(400%);}",
            "b{transform:scaleY(400%) scaleX(300%) scale(1,200%);}",
            "c{transform:scaleX(300%) scaleY(400%) scale(1,200%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 3);
    assert!(matches!(first[0], CssTransformFunction::Scale(_)));
    assert!(matches!(first[1], CssTransformFunction::ScaleX(_)));
    assert!(matches!(first[2], CssTransformFunction::ScaleY(_)));
    assert_eq!(
        scale_argument_spellings(&result, 0, 0),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
    assert_eq!(
        scalex_argument_spelling(&result, 0, 1),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "300%".to_string()
        )
    );
    assert_eq!(
        scaley_argument_spelling(&result, 0, 2),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "400%".to_string()
        )
    );

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 3);
    assert!(matches!(second[0], CssTransformFunction::ScaleY(_)));
    assert!(matches!(second[1], CssTransformFunction::ScaleX(_)));
    assert!(matches!(second[2], CssTransformFunction::Scale(_)));
    assert_eq!(
        scaley_argument_spelling(&result, 1, 0),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "400%".to_string()
        )
    );
    assert_eq!(
        scalex_argument_spelling(&result, 1, 1),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "300%".to_string()
        )
    );
    assert_eq!(
        scale_argument_spellings(&result, 1, 2),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );

    let third = qualified_functions(&result, 2);
    assert_eq!(third.len(), 3);
    assert!(matches!(third[0], CssTransformFunction::ScaleX(_)));
    assert!(matches!(third[1], CssTransformFunction::ScaleY(_)));
    assert!(matches!(third[2], CssTransformFunction::Scale(_)));
    assert_eq!(
        scalex_argument_spelling(&result, 2, 0),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "300%".to_string()
        )
    );
    assert_eq!(
        scaley_argument_spelling(&result, 2, 1),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "400%".to_string()
        )
    );
    assert_eq!(
        scale_argument_spellings(&result, 2, 2),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
}

// 204. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque `scaleY()`
// argument (#661).

#[test]
fn unselected_outer_function_precedence_covers_scaley() {
    assert_all_unsupported(
        661360,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scaleY(2) skewX(10deg)",
            "skewX(10deg) scaleY(2)",
            "scaleY(2) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // scaleY argument, identically in both authored orders.
    assert_all_unsupported(
        661380,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scaleY(calc(1)) skewX(10deg)",
            "skewX(10deg) scaleY(calc(1))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(661400, &["scaleY(1px) skewX(10deg)"]);
}

// 205. `scale3d()` was formerly the remaining scale-family sibling this
// test proved stayed unselected alongside `scaleY()` coverage (#661). #665
// selects `scale3d()` too, superseding that premise: there is no remaining
// unselected scale-family sibling to retarget this sentinel to. The
// surviving invariant -- that `scaleY()` coverage never conflates a sibling
// selected function's representation or evidence with its own -- is
// preserved instead by proving `scaleY()` and `scale3d()` now compose
// correctly and remain distinct in either authored order (#665).

#[test]
fn scaley_and_scale3d_remain_distinct_selected_components() {
    let result = qualify(
        661420,
        concat!(
            "a{transform:scaleY(2) scale3d(1,2,3);}",
            "b{transform:scale3d(1,2,3) scaleY(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::ScaleY(_)));
    assert!(matches!(first[1], CssTransformFunction::Scale3d(_)));

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::Scale3d(_)));
    assert!(matches!(second[1], CssTransformFunction::ScaleY(_)));
}

// 206. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two `scaleY()`
// components or a `scaleY()` and a sibling selected function (#661).

#[test]
fn top_level_comma_is_invalid_around_scaley() {
    assert_all_invalid(
        661440,
        &[
            "scaleY(2), matrix(1,0,0,1,0,0)",
            "scaleY(2), scale(2)",
            "scaleY(2), translate3d(1px,2px,3px)",
            "scaleY(2), rotate3d(1,0,0,90deg)",
            "scaleY(2), translate(10px)",
            "scaleY(2), translateX(20px)",
            "scaleY(2), translateY(20px)",
            "scaleY(2), translateZ(20px)",
            "scaleY(2), scaleX(3)",
            "scaleY(2), scaleY(3)",
        ],
    );
}

// 207. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `scaleY()` extent is qualified from retained
// interior evidence alone. EOF never fills or repairs a missing argument, a
// wrong direct category, or a second slot `scaleY()` has no room for
// (#661).

#[test]
fn true_stylesheet_eof_ended_scaley_extent_follows_parser_authority() {
    let number_eof = qualify(661460, "a{transform:scaleY(2");
    assert_eq!(
        number_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(number_eof.transform_observations().len(), 1);
    assert_eq!(
        scaley_argument_spelling(&number_eof, 0, 0),
        (CssTransformScaleYArgumentKind::Number, "2".to_string())
    );

    let percentage_eof = qualify(661461, "a{transform:scaleY(50%");
    assert_eq!(
        percentage_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(percentage_eof.transform_observations().len(), 1);
    assert_eq!(
        scaley_argument_spelling(&percentage_eof, 0, 0),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "50%".to_string()
        )
    );

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(661462, "a{transform:scaleY(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid: scaleY()
    // never accepts a second slot.
    let trailing_comma = qualify(661463, "a{transform:scaleY(2,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs a direct Dimension category.
    let wrong_category_eof = qualify(661464, "a{transform:scaleY(1px");
    assert_invalid(&wrong_category_eof, 0);

    // A second slot never widens scaleY() cardinality, even at true EOF.
    let two_slots_eof = qualify(661465, "a{transform:scaleY(1,2");
    assert_invalid(&two_slots_eof, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a scaleY".
    let trailing = qualify(661466, "a{transform:scaleY(2) 7;}");
    assert_invalid(&trailing, 0);
}

// 208. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#661).

#[test]
fn trivia_and_important_never_change_scaley_interpretation() {
    let result = qualify(
        661480,
        concat!(
            "a{transform:scaleY(2/**/);}",
            "b{transform:scaleY(/**/2);}",
            "c{transform:scaleY( 2 );}",
            "d{transform:scaleY(2) !important;}",
            "e{transform:scaleY(2)!important;}",
            "f{transform:scaleY(/**/50%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..3 {
        assert_eq!(
            scaley_argument_spelling(&result, index, 0),
            (CssTransformScaleYArgumentKind::Number, "2".to_string()),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(
            scaley_argument_spelling(&result, index, 0),
            (CssTransformScaleYArgumentKind::Number, "2".to_string())
        );
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
    assert_eq!(
        scaley_argument_spelling(&result, 5, 0),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "50%".to_string()
        ),
        "trivia changed Percentage slot interpretation"
    );

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(661500, &["scaleY(/**/)", "scaleY(2,/**/)"]);
}

// 209. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Percentage tokens
// identically across runs (#661).

#[test]
fn scaley_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:scaleY(1,2);}",
        "b{transform:scaleY(calc(1,2));}",
        "c{transform:scaleY(1px);}",
        "d{transform:matrix(1,0,0,1,0,0) scaleY(2);}",
        "e{transform:scaleY(50%);}",
    );

    let first = qualify(661520, css);
    let repeated = qualify(661520, css);
    let another_source = qualify(661521, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        scaley_argument_spelling(&first, 3, 1),
        scaley_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        scaley_argument_spelling(&first, 3, 1),
        scaley_argument_spelling(&another_source, 3, 1)
    );
    assert_eq!(
        scaley_argument_spelling(&first, 4, 0),
        scaley_argument_spelling(&repeated, 4, 0)
    );
    assert_eq!(
        scaley_argument_spelling(&first, 4, 0),
        scaley_argument_spelling(&another_source, 4, 0)
    );
}

// 210. `scaleY` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY`/`translateZ`/`scaleX` boundary, and
// structurally malformed `scaleY` components -- a bare Function name, an
// unbalanced/duplicated closer, or stray leading material -- stay
// decisively `Invalid` (#661).

#[test]
fn scaley_function_name_recognition_and_malformed_components() {
    let result = qualify(
        661540,
        concat!(
            "a{transform:SCALEY(2);}",
            "b{transform:ScAlEy(2);}",
            "c{transform:s\\63 aley(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            scaley_argument_spelling(&result, index, 0),
            (CssTransformScaleYArgumentKind::Number, "2".to_string()),
            "scaleY name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        661560,
        &[
            "scaleY",
            "scaleY(2))",
            "scaleY((2)",
            "1 scaleY(2)",
            "[scaleY(2)]",
        ],
    );
}

// 211. Cross-leaf isolation: `scaleY()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `scaleY()` (#418 / #606 / #645 / #647 / #649 /
// #651 / #653 / #655 / #657 / #659 / #661).

#[test]
fn scaley_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        661580,
        concat!(
            "a{transform:scaleY(2);transform:none;}",
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
        scaley_argument_spelling(&result, 0, 0),
        (CssTransformScaleYArgumentKind::Number, "2".to_string())
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(661600, &["none scaleY(2)", "scaleY(2) none"]);
}

// 212. `scaleZ() = scaleZ([<number> | <percentage>])` under current CSS
// Transforms Level 2 authority (#663): a direct `<number>` argument
// qualifies, preserving exact authored sign/fraction/exponent evidence
// without any machine-number conversion, reusing the accepted `scale()`
// (#645) / `scaleX()` (#659) / `scaleY()` (#661) Number theorem. Like
// `scaleX()`/`scaleY()`, `scaleZ()` never restricts a `Number` to
// exact-zero: a nonzero `Number` is directly valid here. This is a
// distinct grammar from `translateZ()`'s `<length>`-only theorem despite
// the shared `Z` suffix (#657) -- see the dedicated ScaleZ/TranslateZ
// separation test below.

#[test]
fn canonical_scalez_number_qualifies() {
    let result = qualify(
        663100,
        concat!(
            "a{transform:scaleZ(2);}",
            "b{transform:scaleZ(-1);}",
            "c{transform:scaleZ(0);}",
            "d{transform:scaleZ(+0);}",
            "e{transform:scaleZ(-0);}",
            "f{transform:scaleZ(.5);}",
            "g{transform:scaleZ(1.5);}",
            "h{transform:scaleZ(1e2);}",
            "i{transform:scaleZ(-1e-2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 9);
    let expected = [
        "2", "-1", "0", "+0", "-0",
        // Authored `.5`: the absent leading integer digit is canonicalized
        // to `0` by the tokenizer's own retained numeric contract, upstream
        // of this leaf, exactly as for `scale()`/`scaleX()`/`scaleY()`
        // arguments.
        "0.5", "1.5", "1e2", "-1e-2",
    ];
    for (index, spelling) in expected.into_iter().enumerate() {
        assert_eq!(
            scalez_argument_spelling(&result, index, 0),
            (CssTransformScaleZArgumentKind::Number, spelling.to_string()),
            "expected Number at {index}"
        );
    }
}

// 213. A direct `<percentage>` argument qualifies identically to `<number>`
// under current CSS Transforms Level 2 authority, retaining current Level 2
// Percentage support rather than regressing to the older Level 1
// `<number>`-only grammar (#663).

#[test]
fn canonical_scalez_percentage_qualifies() {
    let result = qualify(
        663120,
        concat!(
            "a{transform:scaleZ(250%);}",
            "b{transform:scaleZ(0%);}",
            "c{transform:scaleZ(-20%);}",
            "d{transform:scaleZ(+10%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);
    for (index, spelling) in ["250%", "0%", "-20%", "+10%"].into_iter().enumerate() {
        assert_eq!(
            scalez_argument_spelling(&result, index, 0),
            (
                CssTransformScaleZArgumentKind::Percentage,
                spelling.to_string()
            ),
            "expected Percentage at {index}"
        );
    }
}

// 214. `scaleZ(0)` and `scaleZ(0%)` remain pairwise distinguishable: `0`
// qualifies through `Number` evidence and `0%` qualifies through
// `Percentage` evidence, never collapsed into each other merely because
// their downstream transform semantics may be numerically equivalent. This
// boundary is load-bearing for #663.

#[test]
fn scalez_zero_number_vs_zero_percent_identity() {
    let result = qualify(
        663140,
        concat!("a{transform:scaleZ(0);}", "b{transform:scaleZ(0%);}",),
    );

    let number = scalez_argument_spelling(&result, 0, 0);
    let percentage = scalez_argument_spelling(&result, 1, 0);
    assert_eq!(
        number,
        (CssTransformScaleZArgumentKind::Number, "0".to_string())
    );
    assert_eq!(
        percentage,
        (CssTransformScaleZArgumentKind::Percentage, "0%".to_string())
    );
    assert_ne!(number.0, percentage.0);
    assert_ne!(number.1, percentage.1);
}

// 215. `scaleZ()` accepts exactly one authored argument: zero, two, three,
// or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` (#663).

#[test]
fn scalez_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        663160,
        &[
            "scaleZ()",
            "scaleZ(1,2)",
            "scaleZ(1,2,3)",
            "scaleZ(2,3,4,5)",
        ],
    );
}

// 216. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#663).

#[test]
fn scalez_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        663180,
        &["scaleZ(,)", "scaleZ(,1)", "scaleZ(1,)", "scaleZ(1,,2)"],
    );
}

// 217. `ScaleZArgument := DirectNumber | DirectPercentage` admits no other
// direct token category: a `Dimension` -- including a recognized length,
// angle, or unrecognized unit -- and every other direct token category are
// decisive direct token-category failures (#663). No direct `Dimension` is
// ever accepted, unlike `translateZ()`'s `<length>` theorem this leaf does
// not implement: `scaleZ(1px)` is decisively `Invalid` even though
// `translateZ(1px)` qualifies.

#[test]
fn wrong_direct_categories_are_invalid_for_scalez() {
    assert_all_invalid(
        663200,
        &[
            "scaleZ(1px)",
            "scaleZ(1em)",
            "scaleZ(45deg)",
            "scaleZ(1s)",
            "scaleZ(1fr)",
            "scaleZ(1unknownunit)",
            "scaleZ(foo)",
            "scaleZ(\"1\")",
            "scaleZ(#abc)",
        ],
    );
}

// 218. A complete non-deferred Function occupying the single `scaleZ()`
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason `scale()`/`scaleX()`/`scaleY()`/
// `translate()`/`translate3d()`/`translateX()`/`translateY()`/
// `translateZ()` opaque arguments use -- this leaf never evaluates `calc()`
// and never decides an opaque Function's inner result type, so a `calc()`
// whose inner content looks like a `Percentage` stays Unsupported via the
// same shared boundary (#663).

#[test]
fn opaque_scalez_argument_function_is_unsupported() {
    assert_all_unsupported(
        663220,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "scaleZ(calc(1))",
            "scaleZ(calc(100%))",
            "scaleZ(min(1,2))",
            "scaleZ(max(1,2))",
            "scaleZ(clamp(0,1,2))",
            "scaleZ(calc(0%))",
            "scaleZ(calc(50%))",
            "matrix(1,0,0,1,0,0) scaleZ(calc(1))",
        ],
    );
}

// 219. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#663).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_scalez() {
    assert_all_invalid(
        663240,
        &[
            "scaleZ(calc(1) 2)",
            "scaleZ(calc(1) foo)",
            "scaleZ(calc(1) 20%)",
        ],
    );
}

// 220. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a scaleZ-level argument separator. Once the
// nesting closes, a genuine second scaleZ-level slot is still counted and
// makes the shell decisively Invalid, since `scaleZ()` has no second slot
// to occupy (#663).

#[test]
fn nested_commas_never_change_scalez_arity() {
    assert_all_unsupported(
        663260,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["scaleZ(calc(1,2))"],
    );

    assert_all_invalid(663261, &["scaleZ(calc(1,2),3)"]);
}

// 221. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// scaleZ shape conclusion -- including an arity that looks decisive (#663).

#[test]
fn scalez_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        663280,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "scaleZ(var(--z))",
            "scaleZ(var(--z),2)",
            "matrix(1,0,0,1,0,0) scaleZ(var(--z))",
            "scaleZ(var(--z)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 222. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot; a direct wrong-category failure remains
// decisive even alongside an outer unselected sibling, in either authored
// order (#663).

#[test]
fn scalez_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(663300, &["scaleZ(calc(1),1)"]);

    assert_all_invalid(
        663301,
        &["scaleZ(1px) skewX(10deg)", "skewX(10deg) scaleZ(1px)"],
    );
}

// 223. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, `translateZ()`,
// `scaleX()`, `scaleY()`, and `scaleZ()` components mix and repeat freely,
// preserving exact authored order and repetition through the heterogeneous
// `CssTransformFunction` alternation, and no selected leaf's evidence
// drifts across another's index in a mixed sequence (#418 / #645 / #647 /
// #649 / #651 / #653 / #655 / #657 / #659 / #661 / #663).

#[test]
fn mixed_selected_function_order_including_scalez_is_preserved() {
    let result = qualify(
        663320,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) scaleZ(2);}",
            "b{transform:scaleZ(2) matrix(1,0,0,1,0,0);}",
            "c{transform:scale(2) scaleZ(3);}",
            "d{transform:scaleZ(3) scale(2);}",
            "e{transform:translate3d(1px,2px,3px) scaleZ(2);}",
            "f{transform:scaleZ(2) translate3d(1px,2px,3px);}",
            "g{transform:rotate3d(1,0,0,90deg) scaleZ(2);}",
            "h{transform:scaleZ(2) rotate3d(1,0,0,90deg);}",
            "i{transform:translate(10px) scaleZ(2);}",
            "j{transform:scaleZ(2) translate(10px);}",
            "k{transform:translateX(10px) scaleZ(2);}",
            "l{transform:scaleZ(2) translateX(10px);}",
            "m{transform:translateY(10px) scaleZ(2);}",
            "n{transform:scaleZ(2) translateY(10px);}",
            "o{transform:translateZ(10px) scaleZ(2);}",
            "p{transform:scaleZ(2) translateZ(10px);}",
            "q{transform:scaleX(20%) scaleZ(30%);}",
            "r{transform:scaleZ(40%) scaleX(50%);}",
            "s{transform:scaleY(60%) scaleZ(70%);}",
            "t{transform:scaleZ(80%) scaleY(90%);}",
            "u{transform:scaleZ(1) scaleZ(2);}",
            "v{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px) scaleX(50) scaleY(60) scaleZ(70);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 22);

    for index in 0..21 {
        assert_eq!(
            qualified_functions(&result, index).len(),
            2,
            "expected two components at {index}"
        );
    }
    assert_eq!(qualified_functions(&result, 21).len(), 11);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    let eleven_kind_sequence = qualified_functions(&result, 21);
    assert!(matches!(
        eleven_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[8],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[9],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        eleven_kind_sequence[10],
        CssTransformFunction::ScaleZ(_)
    ));

    // Repeated `scaleZ()` preserves authored order and each component's own
    // evidence.
    let repeated = qualified_functions(&result, 20);
    assert!(matches!(repeated[0], CssTransformFunction::ScaleZ(_)));
    assert!(matches!(repeated[1], CssTransformFunction::ScaleZ(_)));
    assert_eq!(
        scalez_argument_spelling(&result, 20, 0),
        (CssTransformScaleZArgumentKind::Number, "1".to_string())
    );
    assert_eq!(
        scalez_argument_spelling(&result, 20, 1),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );

    // `translate()` and `scaleZ()` remain distinct evidence indices in a
    // mixed sequence: neither drifts into the other's slot.
    assert!(matches!(
        qualified_functions(&result, 8)[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 8)[1],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[0],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 9)[1],
        CssTransformFunction::Translate(_)
    ));

    // `scaleX()` and `scaleZ()` remain distinct semantic placements in
    // either authored order: neither representation nor evidence leaks into
    // the other, and evidence indices resolve independently.
    assert!(matches!(
        qualified_functions(&result, 16)[0],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 16)[1],
        CssTransformFunction::ScaleZ(_)
    ));
    assert_eq!(
        scalex_argument_spelling(&result, 16, 0),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "20%".to_string()
        )
    );
    assert_eq!(
        scalez_argument_spelling(&result, 16, 1),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "30%".to_string()
        )
    );
    assert!(matches!(
        qualified_functions(&result, 17)[0],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 17)[1],
        CssTransformFunction::ScaleX(_)
    ));
    assert_eq!(
        scalez_argument_spelling(&result, 17, 0),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "40%".to_string()
        )
    );
    assert_eq!(
        scalex_argument_spelling(&result, 17, 1),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "50%".to_string()
        )
    );

    // `scaleY()` and `scaleZ()` remain distinct semantic placements in
    // either authored order, identically to `scaleX()`/`scaleZ()` above.
    assert!(matches!(
        qualified_functions(&result, 18)[0],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 18)[1],
        CssTransformFunction::ScaleZ(_)
    ));
    assert_eq!(
        scaley_argument_spelling(&result, 18, 0),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "60%".to_string()
        )
    );
    assert_eq!(
        scalez_argument_spelling(&result, 18, 1),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "70%".to_string()
        )
    );
    assert!(matches!(
        qualified_functions(&result, 19)[0],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 19)[1],
        CssTransformFunction::ScaleY(_)
    ));
    assert_eq!(
        scalez_argument_spelling(&result, 19, 0),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "80%".to_string()
        )
    );
    assert_eq!(
        scaley_argument_spelling(&result, 19, 1),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "90%".to_string()
        )
    );
}

// 224. `scale()`, `scaleX()`, `scaleY()`, and `scaleZ()` remain distinct
// semantic placements: neither their representation, cardinality, nor
// evidence leaks into one another in a mixed sequence, in any authored
// order, and `scale()` retains its accepted one-or-two authored cardinality
// unaffected by `scaleX()`'s, `scaleY()`'s, and `scaleZ()`'s exact-one
// cardinality (#645 / #659 / #661 / #663).

#[test]
fn scale_scalex_scaley_scalez_separation_preserves_distinct_representation_and_evidence() {
    let result = qualify(
        663340,
        concat!(
            "a{transform:scale(1,200%) scaleX(300%) scaleY(400%) scaleZ(500%);}",
            "b{transform:scaleZ(500%) scaleY(400%) scaleX(300%) scale(1,200%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 4);
    assert!(matches!(first[0], CssTransformFunction::Scale(_)));
    assert!(matches!(first[1], CssTransformFunction::ScaleX(_)));
    assert!(matches!(first[2], CssTransformFunction::ScaleY(_)));
    assert!(matches!(first[3], CssTransformFunction::ScaleZ(_)));
    assert_eq!(
        scale_argument_spellings(&result, 0, 0),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
    assert_eq!(
        scalex_argument_spelling(&result, 0, 1),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "300%".to_string()
        )
    );
    assert_eq!(
        scaley_argument_spelling(&result, 0, 2),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "400%".to_string()
        )
    );
    assert_eq!(
        scalez_argument_spelling(&result, 0, 3),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "500%".to_string()
        )
    );

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 4);
    assert!(matches!(second[0], CssTransformFunction::ScaleZ(_)));
    assert!(matches!(second[1], CssTransformFunction::ScaleY(_)));
    assert!(matches!(second[2], CssTransformFunction::ScaleX(_)));
    assert!(matches!(second[3], CssTransformFunction::Scale(_)));
    assert_eq!(
        scalez_argument_spelling(&result, 1, 0),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "500%".to_string()
        )
    );
    assert_eq!(
        scaley_argument_spelling(&result, 1, 1),
        (
            CssTransformScaleYArgumentKind::Percentage,
            "400%".to_string()
        )
    );
    assert_eq!(
        scalex_argument_spelling(&result, 1, 2),
        (
            CssTransformScaleXArgumentKind::Percentage,
            "300%".to_string()
        )
    );
    assert_eq!(
        scale_argument_spellings(&result, 1, 3),
        vec![
            (CssTransformScaleArgumentKind::Number, "1".to_string()),
            (
                CssTransformScaleArgumentKind::Percentage,
                "200%".to_string()
            ),
        ]
    );
}

// 225. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque `scaleZ()`
// argument (#663).

#[test]
fn unselected_outer_function_precedence_covers_scalez() {
    assert_all_unsupported(
        663360,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scaleZ(2) skewX(10deg)",
            "skewX(10deg) scaleZ(2)",
            "scaleZ(2) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // scaleZ argument, identically in both authored orders.
    assert_all_unsupported(
        663380,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scaleZ(calc(1)) skewX(10deg)",
            "skewX(10deg) scaleZ(calc(1))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(663400, &["scaleZ(1px) skewX(10deg)"]);
}

// 226. `scale3d()` was formerly the sole remaining scale-family sibling
// this test proved stayed unselected alongside `scaleZ()` coverage (#663).
// #665 selects `scale3d()` too, superseding that premise: there is no
// remaining unselected scale-family sibling at all after #665. The
// surviving invariant -- that `scaleZ()` coverage never conflates a sibling
// selected function's representation or evidence with its own -- is
// preserved instead by proving `scaleZ()` and `scale3d()` now compose
// correctly and remain distinct in either authored order, including their
// independent Number/Percentage evidence (#665).

#[test]
fn scalez_and_scale3d_remain_distinct_selected_components() {
    let result = qualify(
        663420,
        concat!(
            "a{transform:scaleZ(2) scale3d(1,2,3);}",
            "b{transform:scale3d(1,2,3) scaleZ(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::ScaleZ(_)));
    assert!(matches!(first[1], CssTransformFunction::Scale3d(_)));
    assert_eq!(
        scalez_argument_spelling(&result, 0, 0),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::Scale3d(_)));
    assert!(matches!(second[1], CssTransformFunction::ScaleZ(_)));
    assert_eq!(
        scalez_argument_spelling(&result, 1, 1),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );
}

// 227. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two `scaleZ()`
// components or a `scaleZ()` and a sibling selected function (#663).

#[test]
fn top_level_comma_is_invalid_around_scalez() {
    assert_all_invalid(
        663440,
        &[
            "scaleZ(2), matrix(1,0,0,1,0,0)",
            "scaleZ(2), scale(2)",
            "scaleZ(2), translate3d(1px,2px,3px)",
            "scaleZ(2), rotate3d(1,0,0,90deg)",
            "scaleZ(2), translate(10px)",
            "scaleZ(2), translateX(20px)",
            "scaleZ(2), translateY(20px)",
            "scaleZ(2), translateZ(20px)",
            "scaleZ(2), scaleX(3)",
            "scaleZ(2), scaleY(3)",
            "scaleZ(2), scaleZ(3)",
        ],
    );
}

// 228. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `scaleZ()` extent is qualified from retained
// interior evidence alone. EOF never fills or repairs a missing argument, a
// wrong direct category, or a second slot `scaleZ()` has no room for
// (#663).

#[test]
fn true_stylesheet_eof_ended_scalez_extent_follows_parser_authority() {
    let number_eof = qualify(663460, "a{transform:scaleZ(2");
    assert_eq!(
        number_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(number_eof.transform_observations().len(), 1);
    assert_eq!(
        scalez_argument_spelling(&number_eof, 0, 0),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );

    let percentage_eof = qualify(663461, "a{transform:scaleZ(50%");
    assert_eq!(
        percentage_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(percentage_eof.transform_observations().len(), 1);
    assert_eq!(
        scalez_argument_spelling(&percentage_eof, 0, 0),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "50%".to_string()
        )
    );

    // Zero-slot EOF stays decisively Invalid.
    let zero_slot = qualify(663462, "a{transform:scaleZ(");
    assert_invalid(&zero_slot, 0);

    // A trailing comma at true EOF stays decisively Invalid: scaleZ()
    // never accepts a second slot.
    let trailing_comma = qualify(663463, "a{transform:scaleZ(2,");
    assert_invalid(&trailing_comma, 0);

    // EOF never repairs a direct Dimension category.
    let wrong_category_eof = qualify(663464, "a{transform:scaleZ(1px");
    assert_invalid(&wrong_category_eof, 0);

    // A second slot never widens scaleZ() cardinality, even at true EOF.
    let two_slots_eof = qualify(663465, "a{transform:scaleZ(1,2");
    assert_invalid(&two_slots_eof, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a scaleZ".
    let trailing = qualify(663466, "a{transform:scaleZ(2) 7;}");
    assert_invalid(&trailing, 0);
}

// 229. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#663).

#[test]
fn trivia_and_important_never_change_scalez_interpretation() {
    let result = qualify(
        663480,
        concat!(
            "a{transform:scaleZ(2/**/);}",
            "b{transform:scaleZ(/**/2);}",
            "c{transform:scaleZ( 2 );}",
            "d{transform:scaleZ(2) !important;}",
            "e{transform:scaleZ(2)!important;}",
            "f{transform:scaleZ(/**/50%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..3 {
        assert_eq!(
            scalez_argument_spelling(&result, index, 0),
            (CssTransformScaleZArgumentKind::Number, "2".to_string()),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(
            scalez_argument_spelling(&result, index, 0),
            (CssTransformScaleZArgumentKind::Number, "2".to_string())
        );
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
    assert_eq!(
        scalez_argument_spelling(&result, 5, 0),
        (
            CssTransformScaleZArgumentKind::Percentage,
            "50%".to_string()
        ),
        "trivia changed Percentage slot interpretation"
    );

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(663500, &["scaleZ(/**/)", "scaleZ(2,/**/)"]);
}

// 230. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Percentage tokens
// identically across runs (#663).

#[test]
fn scalez_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:scaleZ(1,2);}",
        "b{transform:scaleZ(calc(1,2));}",
        "c{transform:scaleZ(1px);}",
        "d{transform:matrix(1,0,0,1,0,0) scaleZ(2);}",
        "e{transform:scaleZ(50%);}",
    );

    let first = qualify(663520, css);
    let repeated = qualify(663520, css);
    let another_source = qualify(663521, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        scalez_argument_spelling(&first, 3, 1),
        scalez_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        scalez_argument_spelling(&first, 3, 1),
        scalez_argument_spelling(&another_source, 3, 1)
    );
    assert_eq!(
        scalez_argument_spelling(&first, 4, 0),
        scalez_argument_spelling(&repeated, 4, 0)
    );
    assert_eq!(
        scalez_argument_spelling(&first, 4, 0),
        scalez_argument_spelling(&another_source, 4, 0)
    );
}

// 231. `scaleZ` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY`/`translateZ`/`scaleX`/`scaleY`
// boundary, and structurally malformed `scaleZ` components -- a bare
// Function name, an unbalanced/duplicated closer, or stray leading
// material -- stay decisively `Invalid` (#663).

#[test]
fn scalez_function_name_recognition_and_malformed_components() {
    let result = qualify(
        663540,
        concat!(
            "a{transform:SCALEZ(2);}",
            "b{transform:ScAlEz(2);}",
            "c{transform:s\\63 alez(2);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            scalez_argument_spelling(&result, index, 0),
            (CssTransformScaleZArgumentKind::Number, "2".to_string()),
            "scaleZ name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        663560,
        &[
            "scaleZ",
            "scaleZ(2))",
            "scaleZ((2)",
            "1 scaleZ(2)",
            "[scaleZ(2)]",
        ],
    );
}

// 232. Cross-leaf isolation: `scaleZ()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `scaleZ()` (#418 / #606 / #645 / #647 / #649 /
// #651 / #653 / #655 / #657 / #659 / #661 / #663).

#[test]
fn scalez_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        663580,
        concat!(
            "a{transform:scaleZ(2);transform:none;}",
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
        scalez_argument_spelling(&result, 0, 0),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(663600, &["none scaleZ(2)", "scaleZ(2) none"]);
}

// 233. `scaleZ()` and `translateZ()` share their `Z`-axis semantic
// placement and their `Z` name suffix, but their authored argument
// grammars are entirely different and must never be conflated:
// `translateZ()` admits only a direct exact-zero `Number` or a
// recognized-length `Dimension` (#657), while `scaleZ()` admits any direct
// `Number` (no exact-zero restriction) or a direct `Percentage` (#663).
// This is a high-value regression boundary: `scaleZ(10px)` is `Invalid`
// where `translateZ(10px)` qualifies, `scaleZ(2)` qualifies where
// `translateZ(2)` is `Invalid`, `scaleZ(0)` and `translateZ(0)` both
// qualify for unrelated grammar reasons, and `scaleZ(0%)` qualifies as a
// `Percentage` where `translateZ(0%)` is decisively `Invalid`. Evidence and
// representation for the two functions must never leak into each other,
// even when mixed together in the same declaration.

#[test]
fn scalez_translatez_separation_preserves_distinct_grammar_and_evidence() {
    // `translateZ()` accepts a Length; `scaleZ()` rejects the same Dimension.
    let px_result = qualify(
        663620,
        concat!(
            "a{transform:translateZ(10px);}",
            "b{transform:scaleZ(10px);}",
        ),
    );
    assert_eq!(
        translatez_argument_spelling(&px_result, 0, 0),
        "10px".to_string()
    );
    assert_invalid(&px_result, 1);

    // `scaleZ()` accepts a nonzero Number; `translateZ()` rejects the same
    // Number, since it admits only an exact-zero Number or a Dimension.
    let number_result = qualify(
        663621,
        concat!("a{transform:translateZ(2);}", "b{transform:scaleZ(2);}",),
    );
    assert_invalid(&number_result, 0);
    assert_eq!(
        scalez_argument_spelling(&number_result, 1, 0),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );

    // `0` qualifies for both, but through unrelated grammar branches: an
    // exact-zero Length-role Number for `translateZ()`, and an ordinary
    // Number for `scaleZ()`.
    let zero_result = qualify(
        663622,
        concat!("a{transform:translateZ(0);}", "b{transform:scaleZ(0);}",),
    );
    assert_eq!(
        translatez_argument_spelling(&zero_result, 0, 0),
        "0".to_string()
    );
    assert_eq!(
        scalez_argument_spelling(&zero_result, 1, 0),
        (CssTransformScaleZArgumentKind::Number, "0".to_string())
    );

    // `0%` is decisively Invalid for `translateZ()` (Percentage is not
    // `<length>`), but qualifies as a Percentage for `scaleZ()`.
    let zero_percent_result = qualify(
        663623,
        concat!("a{transform:translateZ(0%);}", "b{transform:scaleZ(0%);}",),
    );
    assert_invalid(&zero_percent_result, 0);
    assert_eq!(
        scalez_argument_spelling(&zero_percent_result, 1, 0),
        (CssTransformScaleZArgumentKind::Percentage, "0%".to_string())
    );

    // Representation and evidence remain separate when both functions
    // appear together in one declaration, in either authored order, with
    // no evidence-index drift between them.
    let mixed = qualify(
        663624,
        concat!(
            "a{transform:translateZ(10px) scaleZ(2);}",
            "b{transform:scaleZ(2) translateZ(10px);}",
        ),
    );
    assert!(matches!(
        qualified_functions(&mixed, 0)[0],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        qualified_functions(&mixed, 0)[1],
        CssTransformFunction::ScaleZ(_)
    ));
    assert_eq!(
        translatez_argument_spelling(&mixed, 0, 0),
        "10px".to_string()
    );
    assert_eq!(
        scalez_argument_spelling(&mixed, 0, 1),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );
    assert!(matches!(
        qualified_functions(&mixed, 1)[0],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&mixed, 1)[1],
        CssTransformFunction::TranslateZ(_)
    ));
    assert_eq!(
        scalez_argument_spelling(&mixed, 1, 0),
        (CssTransformScaleZArgumentKind::Number, "2".to_string())
    );
    assert_eq!(
        translatez_argument_spelling(&mixed, 1, 1),
        "10px".to_string()
    );
}

// 234. `scale3d() = scale3d([<number> | <percentage>]#{3})` under current
// CSS Transforms Level 2 authority (#665): composing `translate3d()`'s
// (#647) exact-three ordered function-local slot structure with
// `scale()`/`scaleX()`/`scaleY()`/`scaleZ()`'s direct `Number` theorem. A
// direct `<number>` argument in every slot qualifies, preserving exact
// authored sign/fraction/exponent evidence without any machine-number
// conversion. Like `scaleX()`/`scaleY()`/`scaleZ()`, `scale3d()` never
// restricts a `Number` to exact-zero in any slot.

#[test]
fn canonical_scale3d_all_number_triple_qualifies() {
    let result = qualify(
        665100,
        concat!(
            "a{transform:scale3d(1,2,3);}",
            "b{transform:scale3d(-1,2,3);}",
            "c{transform:scale3d(1,-2,3);}",
            "d{transform:scale3d(1,2,-3);}",
            "e{transform:scale3d(+1,+2,+3);}",
            "f{transform:scale3d(-0,+0,0);}",
            "g{transform:scale3d(.5,1.5,1e2);}",
            "h{transform:scale3d(-1e-2,+2,-3);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);

    let expected = [
        ("1", "2", "3"),
        ("-1", "2", "3"),
        ("1", "-2", "3"),
        ("1", "2", "-3"),
        ("+1", "+2", "+3"),
        ("-0", "+0", "0"),
        // Authored `.5`: the absent leading integer digit is canonicalized
        // to `0` by the tokenizer's own retained numeric contract, upstream
        // of this leaf, exactly as for `scale()`/`scaleX()`/`scaleY()`/
        // `scaleZ()` arguments.
        ("0.5", "1.5", "1e2"),
        ("-1e-2", "+2", "-3"),
    ];
    for (index, (x, y, z)) in expected.into_iter().enumerate() {
        assert_eq!(
            scale3d_argument_spellings(&result, index, 0),
            (
                CssTransformScale3dArgumentKind::Number,
                x.to_string(),
                CssTransformScale3dArgumentKind::Number,
                y.to_string(),
                CssTransformScale3dArgumentKind::Number,
                z.to_string(),
            ),
            "expected Number/Number/Number at {index}"
        );
    }
}

// 235. A direct `<percentage>` argument qualifies identically to `<number>`
// in every slot under current CSS Transforms Level 2 authority, retaining
// current Level 2 Percentage support rather than regressing to the older
// Level 1 `<number>`-only grammar (#665).

#[test]
fn canonical_scale3d_all_percentage_triple_qualifies() {
    let result = qualify(
        665120,
        concat!(
            "a{transform:scale3d(100%,200%,300%);}",
            "b{transform:scale3d(0%,0%,0%);}",
            "c{transform:scale3d(-20%,+10%,0%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);

    let expected = [
        ("100%", "200%", "300%"),
        ("0%", "0%", "0%"),
        ("-20%", "+10%", "0%"),
    ];
    for (index, (x, y, z)) in expected.into_iter().enumerate() {
        assert_eq!(
            scale3d_argument_spellings(&result, index, 0),
            (
                CssTransformScale3dArgumentKind::Percentage,
                x.to_string(),
                CssTransformScale3dArgumentKind::Percentage,
                y.to_string(),
                CssTransformScale3dArgumentKind::Percentage,
                z.to_string(),
            ),
            "expected Percentage/Percentage/Percentage at {index}"
        );
    }
}

// 236. Each of `scale3d()`'s three slots independently admits `<number>` or
// `<percentage>`: a mixed triple qualifies with each slot's own kind and
// evidence resolved independently, and `0` versus `0%` remain pairwise
// distinguishable in every position -- this positional Number/Percentage
// identity is load-bearing for #665.

#[test]
fn scale3d_mixed_number_percentage_per_slot_identity() {
    let result = qualify(
        665140,
        concat!(
            "a{transform:scale3d(100%,2,300%);}",
            "b{transform:scale3d(1,200%,3);}",
            "c{transform:scale3d(1,2,300%);}",
            "d{transform:scale3d(0%,0,0);}",
            "e{transform:scale3d(0,0%,0);}",
            "f{transform:scale3d(0,0,0%);}",
            "g{transform:scale3d(0%,0,0%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 7);

    use CssTransformScale3dArgumentKind::{Number, Percentage};
    let expected = [
        (Percentage, "100%", Number, "2", Percentage, "300%"),
        (Number, "1", Percentage, "200%", Number, "3"),
        (Number, "1", Number, "2", Percentage, "300%"),
        (Percentage, "0%", Number, "0", Number, "0"),
        (Number, "0", Percentage, "0%", Number, "0"),
        (Number, "0", Number, "0", Percentage, "0%"),
        (Percentage, "0%", Number, "0", Percentage, "0%"),
    ];
    for (index, (x_kind, x, y_kind, y, z_kind, z)) in expected.into_iter().enumerate() {
        assert_eq!(
            scale3d_argument_spellings(&result, index, 0),
            (
                x_kind,
                x.to_string(),
                y_kind,
                y.to_string(),
                z_kind,
                z.to_string()
            ),
            "expected mixed per-slot kind/evidence at {index}"
        );
    }

    // `0` and `0%` never collapse into each other in any position.
    let (x_kind, x, _, _, _, _) = scale3d_argument_spellings(&result, 3, 0);
    assert_eq!((x_kind, x.as_str()), (Percentage, "0%"));
    let (_, _, y_kind, y, _, _) = scale3d_argument_spellings(&result, 4, 0);
    assert_eq!((y_kind, y.as_str()), (Percentage, "0%"));
    let (_, _, _, _, z_kind, z) = scale3d_argument_spellings(&result, 5, 0);
    assert_eq!((z_kind, z.as_str()), (Percentage, "0%"));
}

// 237. `scale3d()` accepts exactly three authored arguments: zero, one, two,
// four, or more directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` -- wrong cardinality is never
// synthesized into a missing Y/Z, truncated, or reinterpreted as `scale()`/
// `scaleX()`/`scaleY()`/`scaleZ()` (#665).

#[test]
fn scale3d_argument_cardinality_other_than_three_is_invalid() {
    assert_all_invalid(
        665160,
        &[
            "scale3d()",
            "scale3d(1)",
            "scale3d(1,2)",
            "scale3d(1,2,3,4)",
            "scale3d(1,2,3,4,5)",
        ],
    );
}

// 238. A comma is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away, in any of the three positions (#665).

#[test]
fn scale3d_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(
        665180,
        &[
            "scale3d(,,)",
            "scale3d(,2,3)",
            "scale3d(1,,3)",
            "scale3d(1,2,)",
            "scale3d(1,,)",
            "scale3d(,2,)",
            "scale3d(,,3)",
            "scale3d(1,,,3)",
        ],
    );
}

// 239. `Scale3dArgument := DirectNumber | DirectPercentage` admits no other
// direct token category in any of the three slots: a `Dimension` --
// including a recognized length, angle, or unrecognized unit -- and every
// other direct token category are decisive direct token-category failures,
// independently in the X, Y, and Z positions. No direct `Dimension` is ever
// accepted, unlike `translate3d()`'s X/Y `<length-percentage>` theorem this
// leaf does not implement: `scale3d(1px,2,3)` is decisively `Invalid` even
// though `translate3d(1px,2px,3px)` qualifies (#665).

#[test]
fn wrong_direct_categories_are_invalid_for_scale3d() {
    assert_all_invalid(
        665200,
        &[
            // Wrong category in X.
            "scale3d(1px,2,3)",
            "scale3d(45deg,2,3)",
            "scale3d(1s,2,3)",
            "scale3d(1unknownunit,2,3)",
            "scale3d(foo,2,3)",
            "scale3d(\"1\",2,3)",
            "scale3d(#abc,2,3)",
            // Wrong category in Y.
            "scale3d(1,1px,3)",
            "scale3d(1,45deg,3)",
            "scale3d(1,foo,3)",
            "scale3d(1,\"1\",3)",
            "scale3d(1,#abc,3)",
            // Wrong category in Z.
            "scale3d(1,2,1px)",
            "scale3d(1,2,45deg)",
            "scale3d(1,2,foo)",
            "scale3d(1,2,\"1\")",
            "scale3d(1,2,#abc)",
        ],
    );
}

// 240. A complete non-deferred Function occupying any one of the three
// `scale3d()` argument slots is selected-profile Unsupported, mirroring the
// shared `FunctionValuedTransformArgument` reason `scale()`/`scaleX()`/
// `scaleY()`/`scaleZ()`/`translate3d()` opaque arguments use -- sealed
// independently for X, Y, and Z, proving no positional classifier
// accidentally skips one axis. This leaf never evaluates `calc()` and never
// decides an opaque Function's inner result type, so a `calc()` whose inner
// content looks like a `Percentage` stays Unsupported via the same shared
// boundary (#665).

#[test]
fn opaque_scale3d_argument_function_is_unsupported() {
    assert_all_unsupported(
        665220,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            // Opaque X.
            "scale3d(calc(1),2,3)",
            "scale3d(max(1,2),2,3)",
            // Opaque Y.
            "scale3d(1,calc(1),3)",
            "scale3d(1,calc(100%),3)",
            "scale3d(1,clamp(0,1,2),3)",
            // Opaque Z.
            "scale3d(1,2,calc(1))",
            "scale3d(1,2,min(1,2))",
            // Alongside a sibling selected component.
            "matrix(1,0,0,1,0,0) scale3d(calc(1),2,3)",
        ],
    );
}

// 241. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot, in any of the
// three positions (#665).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_scale3d() {
    assert_all_invalid(
        665240,
        &[
            "scale3d(calc(1) 2,3,4)",
            "scale3d(1,calc(2) foo,3)",
            "scale3d(1,2,calc(3) 20%)",
        ],
    );
}

// 242. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a scale3d-level argument separator. Once the
// nesting closes, a genuine fourth scale3d-level slot is still counted and
// makes the shell decisively Invalid, since `scale3d()` has no fourth slot
// to occupy (#665).

#[test]
fn nested_commas_never_change_scale3d_arity() {
    assert_all_unsupported(
        665260,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["scale3d(calc(1,2),2,3)", "scale3d(1,min(1,2),3)"],
    );

    assert_all_invalid(665261, &["scale3d(calc(1,2),2,3,4)"]);
}

// 243. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// scale3d shape conclusion -- including an arity that looks decisive,
// exercised in each of the three positions (#665).

#[test]
fn scale3d_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        665280,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "scale3d(var(--x),2,3)",
            "scale3d(1,var(--y),3)",
            "scale3d(1,2,var(--z))",
            "scale3d(var(--all))",
            "scale3d(var(--x),2)",
            "scale3d(var(--x),2,3,4)",
            "matrix(1,0,0,1,0,0) scale3d(var(--x),2,3)",
            "scale3d(var(--x),2,3) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 244. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot, regardless of which slot holds which; a direct
// wrong-category failure remains decisive even alongside an outer
// unselected sibling, in either authored order (#665).

#[test]
fn scale3d_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(
        665300,
        &[
            "scale3d(calc(1),1px,3)",
            "scale3d(1px,calc(1),3)",
            "scale3d(calc(1),2,1px)",
            "scale3d(1,1px,calc(1))",
            "scale3d(1px,2,calc(1))",
            "scale3d(calc(1),1px,1px)",
        ],
    );

    assert_all_invalid(
        665301,
        &[
            "scale3d(1px,2,3) skewX(10deg)",
            "skewX(10deg) scale3d(1px,2,3)",
        ],
    );
}

// 245. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, `translateZ()`,
// `scaleX()`, `scaleY()`, `scaleZ()`, and `scale3d()` components mix and
// repeat freely, preserving exact authored order and repetition through the
// heterogeneous `CssTransformFunction` alternation, and no selected leaf's
// evidence drifts across another's index in a mixed sequence (#418 / #645 /
// #647 / #649 / #651 / #653 / #655 / #657 / #659 / #661 / #663 / #665).

#[test]
fn mixed_selected_function_order_including_scale3d_is_preserved() {
    let result = qualify(
        665320,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) scale3d(2,3,4);}",
            "b{transform:scale3d(2,3,4) matrix(1,0,0,1,0,0);}",
            "c{transform:scaleZ(5) scale3d(2,3,4);}",
            "d{transform:scale3d(2,3,4) scaleZ(5);}",
            "e{transform:scale3d(1,1,1) scale3d(2,2,2);}",
            "f{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px) scaleX(50) scaleY(60) scaleZ(70) scale3d(80,90,100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 2)[0],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 2)[1],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 3)[0],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 3)[1],
        CssTransformFunction::ScaleZ(_)
    ));

    // Repeated `scale3d()` preserves authored order and each component's
    // own evidence.
    let repeated = qualified_functions(&result, 4);
    assert_eq!(repeated.len(), 2);
    assert!(matches!(repeated[0], CssTransformFunction::Scale3d(_)));
    assert!(matches!(repeated[1], CssTransformFunction::Scale3d(_)));
    assert_eq!(
        scale3d_argument_spellings(&result, 4, 0),
        (
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
        )
    );
    assert_eq!(
        scale3d_argument_spellings(&result, 4, 1),
        (
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
        )
    );

    let twelve_kind_sequence = qualified_functions(&result, 5);
    assert_eq!(twelve_kind_sequence.len(), 12);
    assert!(matches!(
        twelve_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[8],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[9],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[10],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        twelve_kind_sequence[11],
        CssTransformFunction::Scale3d(_)
    ));
    assert_eq!(
        scale3d_argument_spellings(&result, 5, 11),
        (
            CssTransformScale3dArgumentKind::Number,
            "80".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "90".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "100".to_string(),
        )
    );
}

// 246. `scale()`, `scaleX()`, `scaleY()`, `scaleZ()`, and `scale3d()` remain
// distinct semantic placements: neither their representation, cardinality,
// nor evidence leaks into one another in a mixed sequence, in any authored
// order, and `scale()` retains its accepted one-or-two authored cardinality
// unaffected by the other four's exact-one/exact-three cardinality (#645 /
// #659 / #661 / #663 / #665).

#[test]
fn scale_family_five_way_separation_preserves_distinct_representation_and_evidence() {
    let result = qualify(
        665340,
        concat!(
            "a{transform:scale(1,200%) scaleX(300%) scaleY(400%) scaleZ(500%) scale3d(1,2,3);}",
            "b{transform:scale3d(1,2,3) scaleZ(500%) scaleY(400%) scaleX(300%) scale(1,200%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 2);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 5);
    assert!(matches!(first[0], CssTransformFunction::Scale(_)));
    assert!(matches!(first[1], CssTransformFunction::ScaleX(_)));
    assert!(matches!(first[2], CssTransformFunction::ScaleY(_)));
    assert!(matches!(first[3], CssTransformFunction::ScaleZ(_)));
    assert!(matches!(first[4], CssTransformFunction::Scale3d(_)));
    assert_eq!(
        scale3d_argument_spellings(&result, 0, 4),
        (
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "3".to_string(),
        )
    );

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 5);
    assert!(matches!(second[0], CssTransformFunction::Scale3d(_)));
    assert!(matches!(second[1], CssTransformFunction::ScaleZ(_)));
    assert!(matches!(second[2], CssTransformFunction::ScaleY(_)));
    assert!(matches!(second[3], CssTransformFunction::ScaleX(_)));
    assert!(matches!(second[4], CssTransformFunction::Scale(_)));
    assert_eq!(
        scale3d_argument_spellings(&result, 1, 0),
        (
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "3".to_string(),
        )
    );
}

// 247. `scale3d()`'s three positions are fixed and semantically meaningful
// (X/Y/Z): reversing the authored values proves index resolution is
// position-driven, not value-driven, and no evidence-index swap occurs
// between the X, Y, and Z slots (#665).

#[test]
fn scale3d_x_y_z_positional_order_and_evidence_are_independent() {
    let result = qualify(
        665360,
        concat!(
            "a{transform:scale3d(10%,20,30%);}",
            "b{transform:scale3d(30%,20,10%);}",
            "c{transform:scale3d(1,2,3);}",
            "d{transform:scale3d(3,2,1);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);

    use CssTransformScale3dArgumentKind::{Number, Percentage};
    assert_eq!(
        scale3d_argument_spellings(&result, 0, 0),
        (
            Percentage,
            "10%".to_string(),
            Number,
            "20".to_string(),
            Percentage,
            "30%".to_string()
        )
    );
    assert_eq!(
        scale3d_argument_spellings(&result, 1, 0),
        (
            Percentage,
            "30%".to_string(),
            Number,
            "20".to_string(),
            Percentage,
            "10%".to_string()
        )
    );
    assert_eq!(
        scale3d_argument_spellings(&result, 2, 0),
        (
            Number,
            "1".to_string(),
            Number,
            "2".to_string(),
            Number,
            "3".to_string()
        )
    );
    assert_eq!(
        scale3d_argument_spellings(&result, 3, 0),
        (
            Number,
            "3".to_string(),
            Number,
            "2".to_string(),
            Number,
            "1".to_string()
        )
    );
}

// 248. `scale3d()` and `translate3d()` share the same exact-three ordered
// function-local slot structure but their authored argument grammars are
// entirely different and must never be conflated: `translate3d()`'s X/Y
// admit `<length-percentage>` and Z admits `<length>` only (#647), while
// `scale3d()` admits `<number> | <percentage>` in all three slots (#665).
// This is a high-value regression boundary: `translate3d(1px,2px,3px)`
// qualifies where `scale3d(1px,2px,3px)` is `Invalid`, `scale3d(1,2,3)`
// qualifies where `translate3d(1,2,3)` is `Invalid` (a nonzero unitless
// Number is not a `<length>`), and `scale3d(0%,0%,0%)` qualifies as three
// Percentages where `translate3d(0%,0%,0%)` is `Invalid` because its Z slot
// admits `<length>` only. Evidence and representation for the two functions
// must never leak into each other, even mixed in the same declaration.

#[test]
fn scale3d_translate3d_separation_preserves_distinct_grammar_and_evidence() {
    // `translate3d()` accepts three Lengths; `scale3d()` rejects the same
    // Dimensions.
    let px_result = qualify(
        665380,
        concat!(
            "a{transform:translate3d(1px,2px,3px);}",
            "b{transform:scale3d(1px,2px,3px);}",
        ),
    );
    assert_eq!(
        translate3d_argument_spellings(&px_result, 0, 0),
        (
            CssTransformTranslate3dXyArgumentKind::Length,
            "1px".to_string(),
            CssTransformTranslate3dXyArgumentKind::Length,
            "2px".to_string(),
            "3px".to_string(),
        )
    );
    assert_invalid(&px_result, 1);

    // `scale3d()` accepts three nonzero Numbers; `translate3d()` rejects
    // the same Numbers, since a nonzero unitless Number is not a `<length>`.
    let number_result = qualify(
        665381,
        concat!(
            "a{transform:translate3d(1,2,3);}",
            "b{transform:scale3d(1,2,3);}",
        ),
    );
    assert_invalid(&number_result, 0);
    assert_eq!(
        scale3d_argument_spellings(&number_result, 1, 0),
        (
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "3".to_string(),
        )
    );

    // `0%` in every position is decisively Invalid for `translate3d()`
    // (its Z slot is `<length>` only), but qualifies as three Percentages
    // for `scale3d()`.
    let zero_percent_result = qualify(
        665382,
        concat!(
            "a{transform:translate3d(0%,0%,0%);}",
            "b{transform:scale3d(0%,0%,0%);}",
        ),
    );
    assert_invalid(&zero_percent_result, 0);
    assert_eq!(
        scale3d_argument_spellings(&zero_percent_result, 1, 0),
        (
            CssTransformScale3dArgumentKind::Percentage,
            "0%".to_string(),
            CssTransformScale3dArgumentKind::Percentage,
            "0%".to_string(),
            CssTransformScale3dArgumentKind::Percentage,
            "0%".to_string(),
        )
    );

    // Representation and evidence remain separate when both functions
    // appear together in one declaration, in either authored order, with
    // no evidence-index drift between them.
    let mixed = qualify(
        665383,
        concat!(
            "a{transform:translate3d(1px,2px,3px) scale3d(4,5,6);}",
            "b{transform:scale3d(4,5,6) translate3d(1px,2px,3px);}",
        ),
    );
    assert!(matches!(
        qualified_functions(&mixed, 0)[0],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        qualified_functions(&mixed, 0)[1],
        CssTransformFunction::Scale3d(_)
    ));
    assert_eq!(
        scale3d_argument_spellings(&mixed, 0, 1),
        (
            CssTransformScale3dArgumentKind::Number,
            "4".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "5".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "6".to_string(),
        )
    );
    assert!(matches!(
        qualified_functions(&mixed, 1)[0],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        qualified_functions(&mixed, 1)[1],
        CssTransformFunction::Translate3d(_)
    ));
    assert_eq!(
        scale3d_argument_spellings(&mixed, 1, 0),
        (
            CssTransformScale3dArgumentKind::Number,
            "4".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "5".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "6".to_string(),
        )
    );
}

// 249. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified selected function, and the coarser
// outer unselected-function coverage outranks an inner opaque `scale3d()`
// argument, independently of which of the three slots holds it (#665).

#[test]
fn unselected_outer_function_precedence_covers_scale3d() {
    assert_all_unsupported(
        665400,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scale3d(2,3,4) skewX(10deg)",
            "skewX(10deg) scale3d(2,3,4)",
            "scale3d(2,3,4) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // scale3d argument, identically in both authored orders and
    // independently of which slot is opaque.
    assert_all_unsupported(
        665410,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "scale3d(calc(1),2,3) skewX(10deg)",
            "skewX(10deg) scale3d(calc(1),2,3)",
            "scale3d(1,calc(1),3) skewX(10deg)",
            "skewX(10deg) scale3d(1,2,calc(1))",
        ],
    );
}

// 250. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two `scale3d()`
// components or a `scale3d()` and a sibling selected function (#665).

#[test]
fn top_level_comma_is_invalid_around_scale3d() {
    assert_all_invalid(
        665440,
        &[
            "scale3d(2,3,4), matrix(1,0,0,1,0,0)",
            "scale3d(2,3,4), scale(2)",
            "scale3d(2,3,4), translate3d(1px,2px,3px)",
            "scale3d(2,3,4), rotate3d(1,0,0,90deg)",
            "scale3d(2,3,4), translate(10px)",
            "scale3d(2,3,4), translateX(20px)",
            "scale3d(2,3,4), translateY(20px)",
            "scale3d(2,3,4), translateZ(20px)",
            "scale3d(2,3,4), scaleX(3)",
            "scale3d(2,3,4), scaleY(3)",
            "scale3d(2,3,4), scaleZ(3)",
            "scale3d(2,3,4), scale3d(5,6,7)",
        ],
    );
}

// 251. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `scale3d()` extent is qualified from
// retained interior evidence alone once all three slots are complete. EOF
// never fills or repairs a missing argument, a wrong direct category, or a
// fourth slot `scale3d()` has no room for (#665).

#[test]
fn true_stylesheet_eof_ended_scale3d_extent_follows_parser_authority() {
    let number_eof = qualify(665460, "a{transform:scale3d(1,2,3");
    assert_eq!(
        number_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(number_eof.transform_observations().len(), 1);
    assert_eq!(
        scale3d_argument_spellings(&number_eof, 0, 0),
        (
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "3".to_string(),
        )
    );

    let mixed_eof = qualify(665461, "a{transform:scale3d(100%,2,300%");
    assert_eq!(
        mixed_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(mixed_eof.transform_observations().len(), 1);
    assert_eq!(
        scale3d_argument_spellings(&mixed_eof, 0, 0),
        (
            CssTransformScale3dArgumentKind::Percentage,
            "100%".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Percentage,
            "300%".to_string(),
        )
    );

    // Zero/one/two-slot EOF all stay decisively Invalid.
    let zero_slot = qualify(665462, "a{transform:scale3d(");
    assert_invalid(&zero_slot, 0);
    let one_slot = qualify(665463, "a{transform:scale3d(1");
    assert_invalid(&one_slot, 0);
    let two_slots = qualify(665464, "a{transform:scale3d(1,2");
    assert_invalid(&two_slots, 0);

    // A trailing empty third slot at true EOF stays decisively Invalid.
    let trailing_empty = qualify(665465, "a{transform:scale3d(1,2,");
    assert_invalid(&trailing_empty, 0);

    // A fourth slot never widens scale3d() cardinality, even at true EOF.
    let four_slots_eof = qualify(665466, "a{transform:scale3d(1,2,3,4");
    assert_invalid(&four_slots_eof, 0);

    // EOF never repairs a direct Dimension category.
    let wrong_category_eof = qualify(665467, "a{transform:scale3d(1px,2,3");
    assert_invalid(&wrong_category_eof, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a scale3d".
    let trailing = qualify(665468, "a{transform:scale3d(1,2,3) 7;}");
    assert_invalid(&trailing, 0);
}

// 252. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation in any of the
// three positions, a comment never fills an authored-empty slot, and
// `!important` remains outside the semantic value window (#665).

#[test]
fn trivia_and_important_never_change_scale3d_interpretation() {
    let result = qualify(
        665480,
        concat!(
            "a{transform:scale3d(1/**/,2,3);}",
            "b{transform:scale3d(1,/**/2,3);}",
            "c{transform:scale3d( 1 , 2 , 3 );}",
            "d{transform:scale3d(1,2,3) !important;}",
            "e{transform:scale3d(1,2,3)!important;}",
            "f{transform:scale3d(/**/50%,2,300%);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..3 {
        assert_eq!(
            scale3d_argument_spellings(&result, index, 0),
            (
                CssTransformScale3dArgumentKind::Number,
                "1".to_string(),
                CssTransformScale3dArgumentKind::Number,
                "2".to_string(),
                CssTransformScale3dArgumentKind::Number,
                "3".to_string(),
            ),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(
            scale3d_argument_spellings(&result, index, 0),
            (
                CssTransformScale3dArgumentKind::Number,
                "1".to_string(),
                CssTransformScale3dArgumentKind::Number,
                "2".to_string(),
                CssTransformScale3dArgumentKind::Number,
                "3".to_string(),
            )
        );
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
    assert_eq!(
        scale3d_argument_spellings(&result, 5, 0),
        (
            CssTransformScale3dArgumentKind::Percentage,
            "50%".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Percentage,
            "300%".to_string(),
        ),
        "trivia changed Percentage slot interpretation"
    );

    // A comment is trivia, never an argument: it can neither fill an
    // authored-empty slot nor stand in for a missing one.
    assert_all_invalid(665500, &["scale3d(/**/,/**/,/**/)", "scale3d(1,2,/**/)"]);
}

// 253. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Percentage tokens
// identically across runs, independently in all three positions (#665).

#[test]
fn scale3d_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:scale3d(1,2,3);}",
        "b{transform:scale3d(calc(1,2),2,3);}",
        "c{transform:scale3d(1px,2,3);}",
        "d{transform:matrix(1,0,0,1,0,0) scale3d(2,3,4);}",
        "e{transform:scale3d(50%,60%,70%);}",
    );

    let first = qualify(665520, css);
    let repeated = qualify(665520, css);
    let another_source = qualify(665521, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_eq!(
        scale3d_argument_spellings(&first, 0, 0),
        scale3d_argument_spellings(&repeated, 0, 0)
    );
    assert_eq!(
        scale3d_argument_spellings(&first, 0, 0),
        scale3d_argument_spellings(&another_source, 0, 0)
    );
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);
    assert_eq!(
        scale3d_argument_spellings(&first, 3, 1),
        scale3d_argument_spellings(&repeated, 3, 1)
    );
    assert_eq!(
        scale3d_argument_spellings(&first, 3, 1),
        scale3d_argument_spellings(&another_source, 3, 1)
    );
    assert_eq!(
        scale3d_argument_spellings(&first, 4, 0),
        scale3d_argument_spellings(&repeated, 4, 0)
    );
    assert_eq!(
        scale3d_argument_spellings(&first, 4, 0),
        scale3d_argument_spellings(&another_source, 4, 0)
    );
}

// 254. `scale3d` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY`/`translateZ`/`scaleX`/`scaleY`/
// `scaleZ` boundary, and structurally malformed `scale3d` components -- a
// bare Function name, an unbalanced/duplicated closer, or stray leading
// material -- stay decisively `Invalid` (#665).

#[test]
fn scale3d_function_name_recognition_and_malformed_components() {
    let result = qualify(
        665540,
        concat!(
            "a{transform:SCALE3D(1,2,3);}",
            "b{transform:ScAlE3d(1,2,3);}",
            "c{transform:s\\63 ale3d(1,2,3);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            scale3d_argument_spellings(&result, index, 0),
            (
                CssTransformScale3dArgumentKind::Number,
                "1".to_string(),
                CssTransformScale3dArgumentKind::Number,
                "2".to_string(),
                CssTransformScale3dArgumentKind::Number,
                "3".to_string(),
            ),
            "scale3d name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        665560,
        &[
            "scale3d",
            "scale3d(1,2,3))",
            "scale3d((1,2,3)",
            "1 scale3d(1,2,3)",
            "[scale3d(1,2,3)]",
        ],
    );
}

// 255. Cross-leaf isolation: `scale3d()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `scale3d()` (#418 / #606 / #645 / #647 / #649 /
// #651 / #653 / #655 / #657 / #659 / #661 / #663 / #665).

#[test]
fn scale3d_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        665580,
        concat!(
            "a{transform:scale3d(1,2,3);transform:none;}",
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
        scale3d_argument_spellings(&result, 0, 0),
        (
            CssTransformScale3dArgumentKind::Number,
            "1".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "2".to_string(),
            CssTransformScale3dArgumentKind::Number,
            "3".to_string(),
        )
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(665600, &["none scale3d(1,2,3)", "scale3d(1,2,3) none"]);
}

// 256. `rotate() = rotate([<angle> | <zero>])` under current CSS Transforms
// Level 2 / CSS Values authority (#667): a direct `<angle>` argument
// qualifies in every recognized unit, ASCII-case-insensitively, and with
// signed spelling preserved, reusing the accepted `rotate3d()` (#649)
// fourth-slot `is_css_angle_unit` theorem without a second independent
// unit table.

#[test]
fn canonical_rotate_angle_qualifies() {
    let result = qualify(
        667100,
        concat!(
            "a{transform:rotate(90deg);}",
            "b{transform:rotate(100grad);}",
            "c{transform:rotate(1rad);}",
            "d{transform:rotate(0.25turn);}",
            "e{transform:rotate(90DEG);}",
            "f{transform:rotate(1RAD);}",
            "g{transform:rotate(-45deg);}",
            "h{transform:rotate(+30deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..8 {
        let (is_angle, _) = rotate_argument_spelling(&result, index, 0);
        assert!(is_angle, "expected Angle role at {index}");
    }
    assert_eq!(rotate_argument_spelling(&result, 0, 0).1, "90deg");
    assert_eq!(rotate_argument_spelling(&result, 1, 0).1, "100grad");
    assert_eq!(rotate_argument_spelling(&result, 2, 0).1, "1rad");
    assert_eq!(rotate_argument_spelling(&result, 3, 0).1, "0.25turn");
    assert_eq!(rotate_argument_spelling(&result, 4, 0).1, "90DEG");
    assert_eq!(rotate_argument_spelling(&result, 5, 0).1, "1RAD");
    assert_eq!(rotate_argument_spelling(&result, 6, 0).1, "-45deg");
    assert_eq!(rotate_argument_spelling(&result, 7, 0).1, "+30deg");
}

// 257. Representative exact-zero `Number` spellings qualify the single
// `rotate()` slot through the `<zero>` branch, reusing the accepted
// `is_direct_zero_numeric_value` theorem; the retained evidence stays a
// `Number` token (#667).

#[test]
fn canonical_rotate_zero_qualifies() {
    let result = qualify(
        667120,
        concat!(
            "a{transform:rotate(0);}",
            "b{transform:rotate(+0);}",
            "c{transform:rotate(-0);}",
            "d{transform:rotate(.0);}",
            "e{transform:rotate(0.0);}",
            "f{transform:rotate(0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        let (is_angle, _) = rotate_argument_spelling(&result, index, 0);
        assert!(!is_angle, "expected Zero role at {index}");
    }
    assert_eq!(rotate_argument_spelling(&result, 0, 0).1, "0");
    assert_eq!(rotate_argument_spelling(&result, 1, 0).1, "+0");
    assert_eq!(rotate_argument_spelling(&result, 2, 0).1, "-0");
    assert_eq!(rotate_argument_spelling(&result, 3, 0).1, "0.0");
    assert_eq!(rotate_argument_spelling(&result, 4, 0).1, "0.0");
    assert_eq!(rotate_argument_spelling(&result, 5, 0).1, "0e100");
}

// 258. `0` and `0deg` remain distinct authored roles -- `<zero>` and
// `<angle>` respectively -- never collapsed into one generic scalar, never
// normalized into each other, and a `<zero>` never gains a synthesized
// `deg` unit. This is load-bearing (#667).

#[test]
fn rotate_zero_vs_angle_authored_identity_is_preserved() {
    let result = qualify(
        667140,
        concat!("a{transform:rotate(0);}", "b{transform:rotate(0deg);}",),
    );

    assert_eq!(result.transform_observations().len(), 2);
    let (zero_is_angle, zero_spelling) = rotate_argument_spelling(&result, 0, 0);
    let (angle_is_angle, angle_spelling) = rotate_argument_spelling(&result, 1, 0);
    assert!(!zero_is_angle, "expected authored `0` to qualify as Zero");
    assert!(
        angle_is_angle,
        "expected authored `0deg` to qualify as Angle"
    );
    assert_eq!(zero_spelling, "0");
    assert_eq!(angle_spelling, "0deg");
}

// 259. `rotate()` accepts exactly one authored argument: zero, two, or
// three directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` before any argument content is
// consulted (#667).

#[test]
fn rotate_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        667160,
        &[
            "rotate()",
            "rotate(0,90deg)",
            "rotate(90deg,0)",
            "rotate(0,0)",
            "rotate(90deg,180deg)",
            "rotate(90deg,180deg,270deg)",
        ],
    );
}

// 260. `,` is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#667).

#[test]
fn rotate_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(667180, &["rotate(,)", "rotate(,0)", "rotate(0,)"]);
}

// 261. `RotateArgument := DirectAngle | DirectZero` admits no other direct
// token category: a nonzero unitless `Number`, a `Percentage`, a
// non-angle-unit `Dimension`, an `Ident` -- including a longhand-axis
// keyword, which this transform-function grammar never imports -- a
// `String`, and a `Hash` are all decisive direct token-category failures
// (#667).

#[test]
fn wrong_direct_categories_are_invalid_for_rotate() {
    assert_all_invalid(
        667200,
        &[
            "rotate(1)",
            "rotate(-1)",
            "rotate(.5)",
            "rotate(1e0)",
            "rotate(1px)",
            "rotate(1em)",
            "rotate(1s)",
            "rotate(1fr)",
            "rotate(1unknownunit)",
            "rotate(0%)",
            "rotate(100%)",
            "rotate(foo)",
            "rotate(x)",
            "rotate(y)",
            "rotate(z)",
            "rotate(\"0\")",
            "rotate(#abc)",
        ],
    );
}

// 262. A complete non-deferred Function occupying the single `rotate()`
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason the other selected functions use
// -- this leaf never evaluates `calc()`, so `calc(0)` is never direct
// `<zero>` and `calc(90deg)` is never direct `<angle>` (#667).

#[test]
fn opaque_rotate_argument_function_is_unsupported() {
    assert_all_unsupported(
        667220,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "rotate(calc(0))",
            "rotate(calc(90deg))",
            "rotate(min(0deg,90deg))",
            "rotate(max(0deg,90deg))",
            "rotate(clamp(0deg,45deg,90deg))",
            "matrix(1,0,0,1,0,0) rotate(calc(0))",
        ],
    );
}

// 263. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#667).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_rotate() {
    assert_all_invalid(
        667240,
        &[
            "rotate(calc(0) 0)",
            "rotate(calc(90deg) foo)",
            "rotate(calc(90deg) 1deg)",
        ],
    );
}

// 264. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a rotate-level argument separator. Once the
// nesting closes, a genuinely different rotate-level slot count is still
// counted and makes the shell decisively Invalid, since `rotate()` has no
// second slot to occupy (#667).

#[test]
fn nested_commas_never_change_rotate_arity() {
    assert_all_unsupported(
        667260,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["rotate(calc(1,2))"],
    );

    assert_all_invalid(667261, &["rotate(calc(1,2),0)"]);
}

// 265. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// rotate shape conclusion -- including an arity that looks decisive (#667).

#[test]
fn rotate_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        667280,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "rotate(var(--r))",
            "rotate(var(--r),90deg)",
            "matrix(1,0,0,1,0,0) rotate(var(--r))",
            "rotate(var(--r)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 266. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot; a direct wrong-category failure remains
// decisive even alongside an outer unselected sibling, in either authored
// order (#667). `skewX()` is used as the still-unselected outer sentinel
// here, rather than `rotateX()`, because #669 selects `rotateX()` and this
// sentinel choice avoids repeated churn as the rotate family is completed.

#[test]
fn rotate_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(667300, &["rotate(calc(0),1)"]);

    assert_all_invalid(667301, &["rotate(1) skewX(1deg)", "skewX(1deg) rotate(1)"]);
}

// 267. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified `rotate()`, and the coarser outer
// unselected-function coverage outranks an inner opaque `rotate()`
// argument. `skewX()`/`matrix3d()` are used as the still-unselected outer
// sentinels here, rather than `rotateX()`, because #669 selects `rotateX()`
// and this sentinel choice avoids repeated churn as the rotate family is
// completed (#667 / #669).

#[test]
fn unselected_outer_function_precedence_covers_rotate() {
    assert_all_unsupported(
        667310,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate(90deg) skewX(1deg)",
            "skewX(1deg) rotate(90deg)",
            "rotate(90deg) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // rotate argument, identically in both authored orders.
    assert_all_unsupported(
        667320,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotate(calc(90deg)) skewX(1deg)",
            "skewX(1deg) rotate(calc(90deg))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(667330, &["rotate(1) skewX(1deg)"]);
}

// 268. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, `translateZ()`,
// `scaleX()`, `scaleY()`, `scaleZ()`, `scale3d()`, and `rotate()`
// components mix and repeat freely, preserving exact authored order and
// repetition through the heterogeneous `CssTransformFunction` alternation,
// and no selected leaf's evidence drifts across another's index in a mixed
// sequence (#418 / #645 / #647 / #649 / #651 / #653 / #655 / #657 / #659 /
// #661 / #663 / #665 / #667).

#[test]
fn mixed_selected_function_order_including_rotate_is_preserved() {
    let result = qualify(
        667340,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) rotate(90deg);}",
            "b{transform:rotate(90deg) matrix(1,0,0,1,0,0);}",
            "c{transform:rotate3d(1,0,0,90deg) rotate(90deg);}",
            "d{transform:rotate(90deg) rotate3d(1,0,0,90deg);}",
            "e{transform:rotate(1deg) rotate(2deg);}",
            "f{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px) scaleX(50) scaleY(60) scaleZ(70) scale3d(80,90,100) rotate(45deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::Rotate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::Rotate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 2)[0],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 2)[1],
        CssTransformFunction::Rotate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 3)[0],
        CssTransformFunction::Rotate(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 3)[1],
        CssTransformFunction::Rotate3d(_)
    ));

    // Repeated `rotate()` preserves authored order and each component's own
    // evidence.
    let repeated = qualified_functions(&result, 4);
    assert_eq!(repeated.len(), 2);
    assert!(matches!(repeated[0], CssTransformFunction::Rotate(_)));
    assert!(matches!(repeated[1], CssTransformFunction::Rotate(_)));
    assert_eq!(
        rotate_argument_spelling(&result, 4, 0),
        (true, "1deg".to_string())
    );
    assert_eq!(
        rotate_argument_spelling(&result, 4, 1),
        (true, "2deg".to_string())
    );

    let thirteen_kind_sequence = qualified_functions(&result, 5);
    assert_eq!(thirteen_kind_sequence.len(), 13);
    assert!(matches!(
        thirteen_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[8],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[9],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[10],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[11],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        thirteen_kind_sequence[12],
        CssTransformFunction::Rotate(_)
    ));
    assert_eq!(
        rotate_argument_spelling(&result, 5, 12),
        (true, "45deg".to_string())
    );
}

// 269. `rotate()` and `rotate3d()` remain distinct selected components:
// neither their representation, cardinality, nor evidence leaks into one
// another in a mixed sequence, in any authored order, even though
// `rotate()` composes `rotate3d()`'s fourth-slot `Angle | Zero` scalar
// theorem (#649 / #667).

#[test]
fn rotate_and_rotate3d_remain_distinct_selected_components() {
    let result = qualify(
        667360,
        concat!(
            "a{transform:rotate(90deg) rotate3d(0,0,1,90deg);}",
            "b{transform:rotate3d(0,0,1,90deg) rotate(90deg);}",
            "c{transform:rotate(0) rotate3d(0,0,1,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::Rotate(_)));
    assert!(matches!(first[1], CssTransformFunction::Rotate3d(_)));

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::Rotate3d(_)));
    assert!(matches!(second[1], CssTransformFunction::Rotate(_)));

    let third = qualified_functions(&result, 2);
    assert_eq!(third.len(), 2);
    assert!(matches!(third[0], CssTransformFunction::Rotate(_)));
    assert!(matches!(third[1], CssTransformFunction::Rotate3d(_)));
    assert_eq!(
        rotate_argument_spelling(&result, 2, 0),
        (false, "0".to_string())
    );
    let CssTransformFunction::Rotate3d(rotate3d) = third[1] else {
        panic!("expected rotate3d component");
    };
    let CssTransformRotate3dAngleArgument::Zero(_) = rotate3d.angle() else {
        panic!(
            "expected rotate3d fourth slot to remain Zero independently of rotate()'s own Zero role"
        );
    };
}

// 270. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two `rotate()`
// components or a `rotate()` and a sibling selected function (#667).

#[test]
fn top_level_comma_is_invalid_around_rotate() {
    assert_all_invalid(
        667380,
        &[
            "rotate(90deg), matrix(1,0,0,1,0,0)",
            "rotate(90deg), scale(2)",
            "rotate(90deg), translate3d(1px,2px,3px)",
            "rotate(90deg), rotate3d(1,0,0,90deg)",
            "rotate(90deg), translate(10px)",
            "rotate(90deg), translateX(20px)",
            "rotate(90deg), translateY(20px)",
            "rotate(90deg), translateZ(20px)",
            "rotate(90deg), scaleX(3)",
            "rotate(90deg), scaleY(3)",
            "rotate(90deg), scaleZ(3)",
            "rotate(90deg), scale3d(2,3,4)",
            "rotate(90deg), rotate(90deg)",
        ],
    );
}

// 271. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `rotate()` extent is qualified from retained
// interior evidence alone once the slot is complete. EOF never fills or
// repairs a missing argument, a wrong direct category, or a second slot
// `rotate()` has no room for (#667).

#[test]
fn true_stylesheet_eof_ended_rotate_extent_follows_parser_authority() {
    let angle_eof = qualify(667400, "a{transform:rotate(90deg");
    assert_eq!(
        angle_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(angle_eof.transform_observations().len(), 1);
    assert_eq!(
        rotate_argument_spelling(&angle_eof, 0, 0),
        (true, "90deg".to_string())
    );

    let zero_eof = qualify(667401, "a{transform:rotate(0");
    assert_eq!(
        zero_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(zero_eof.transform_observations().len(), 1);
    assert_eq!(
        rotate_argument_spelling(&zero_eof, 0, 0),
        (false, "0".to_string())
    );

    // Zero-slot EOF stays decisively Invalid.
    let empty = qualify(667402, "a{transform:rotate(");
    assert_invalid(&empty, 0);

    // EOF never repairs a decisive direct category.
    let wrong_category_eof = qualify(667403, "a{transform:rotate(1");
    assert_invalid(&wrong_category_eof, 0);

    // A trailing comma at true EOF stays decisively Invalid: rotate() never
    // accepts a second slot.
    let trailing_comma = qualify(667404, "a{transform:rotate(0,");
    assert_invalid(&trailing_comma, 0);

    // A second slot never widens rotate() cardinality, even at true EOF.
    let two_slots_eof = qualify(667405, "a{transform:rotate(90deg,0");
    assert_invalid(&two_slots_eof, 0);

    // A closed component followed by stray material is invalid, proving the
    // accepted EOF case is not "accept whatever trails a rotate".
    let trailing = qualify(667406, "a{transform:rotate(90deg) 7;}");
    assert_invalid(&trailing, 0);
}

// 272. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#667).

#[test]
fn trivia_and_important_never_change_rotate_interpretation() {
    let result = qualify(
        667420,
        concat!(
            "a{transform:rotate(90deg/**/);}",
            "b{transform:rotate(/**/90deg);}",
            "c{transform:rotate( 90deg );}",
            "d{transform:rotate(90deg) !important;}",
            "e{transform:rotate(90deg)!important;}",
            "f{transform:rotate(/**/0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..3 {
        assert_eq!(
            rotate_argument_spelling(&result, index, 0),
            (true, "90deg".to_string()),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(
            rotate_argument_spelling(&result, index, 0),
            (true, "90deg".to_string())
        );
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
    assert_eq!(
        rotate_argument_spelling(&result, 5, 0),
        (false, "0".to_string()),
        "trivia changed Zero slot interpretation"
    );

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(667440, &["rotate(/**/)", "rotate(90deg,/**/)"]);
}

// 273. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension tokens
// identically across runs (#667).

#[test]
fn rotate_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:rotate(0,90deg);}",
        "b{transform:rotate(calc(0));}",
        "c{transform:rotate(1px);}",
        "d{transform:matrix(1,0,0,1,0,0) rotate(90deg);}",
        "e{transform:rotate(0);}",
    );

    let first = qualify(667460, css);
    let repeated = qualify(667460, css);
    let another_source = qualify(667461, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        rotate_argument_spelling(&first, 3, 1),
        rotate_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        rotate_argument_spelling(&first, 3, 1),
        rotate_argument_spelling(&another_source, 3, 1)
    );
    assert_eq!(
        rotate_argument_spelling(&first, 4, 0),
        rotate_argument_spelling(&repeated, 4, 0)
    );
    assert_eq!(
        rotate_argument_spelling(&first, 4, 0),
        rotate_argument_spelling(&another_source, 4, 0)
    );
}

// 274. `rotate` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY`/`translateZ`/`scaleX`/`scaleY`/
// `scaleZ`/`scale3d` boundary, and structurally malformed `rotate`
// components -- a bare Function name, an unbalanced/duplicated closer, or
// stray leading material -- stay decisively `Invalid` (#667).

#[test]
fn rotate_function_name_recognition_and_malformed_components() {
    let result = qualify(
        667480,
        concat!(
            "a{transform:ROTATE(90deg);}",
            "b{transform:RoTaTe(90deg);}",
            "c{transform:r\\6f tate(90deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            rotate_argument_spelling(&result, index, 0),
            (true, "90deg".to_string()),
            "rotate name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        667500,
        &[
            "rotate",
            "rotate(90deg))",
            "rotate((90deg)",
            "1 rotate(90deg)",
            "[rotate(90deg)]",
        ],
    );
}

// 275. Cross-leaf isolation: `rotate()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `rotate()`. In particular, the longhand `rotate`
// property qualifier (#604) stays entirely unaffected by this leaf: it
// remains its own separate value grammar and representation, never gaining
// the transform-function legacy `<zero>` branch (#418 / #604 / #606 / #645
// / #647 / #649 / #651 / #653 / #655 / #657 / #659 / #661 / #663 / #665 /
// #667).

#[test]
fn rotate_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        667520,
        concat!(
            "a{transform:rotate(90deg);transform:none;}",
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
        rotate_argument_spelling(&result, 0, 0),
        (true, "90deg".to_string())
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(667540, &["none rotate(90deg)", "rotate(90deg) none"]);
}

// 276. Selecting `rotate()` and `rotateX()` does not select `rotateY()` or
// `rotateZ()`: each remains outside selected-profile coverage in isolation
// and alongside a qualified `rotate()`/`rotateX()` in either authored order.
// This theorem is intentionally superseded in part by #669, which selects
// `rotateX()`: the surviving assertion is that `rotateY()`/`rotateZ()`
// remain unselected, not that `rotateX()` does (#667 / #669).

#[test]
fn rotatey_rotatez_remain_unselected_after_rotatex_selection() {
    assert_all_unsupported(
        667560,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotateY(90deg)",
            "rotateZ(90deg)",
            "rotateY(90deg) rotate(90deg)",
            "rotate(90deg) rotateZ(90deg)",
            "rotateY(90deg) rotateX(90deg)",
            "rotateX(90deg) rotateZ(90deg)",
        ],
    );
}

// 277. `rotateX() = rotateX([<angle> | <zero>])` under current CSS
// Transforms Level 2 / CSS Values authority (#669): a direct `<angle>`
// argument qualifies in every recognized unit, ASCII-case-insensitively,
// and with signed spelling preserved, reusing the accepted rotate-family
// (#667) `is_css_angle_unit` theorem without a second independent unit
// table.

#[test]
fn canonical_rotatex_angle_qualifies() {
    let result = qualify(
        669100,
        concat!(
            "a{transform:rotateX(90deg);}",
            "b{transform:rotateX(100grad);}",
            "c{transform:rotateX(1rad);}",
            "d{transform:rotateX(0.25turn);}",
            "e{transform:rotateX(90DEG);}",
            "f{transform:rotateX(1RAD);}",
            "g{transform:rotateX(-45deg);}",
            "h{transform:rotateX(+30deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 8);
    for index in 0..8 {
        let (is_angle, _) = rotatex_argument_spelling(&result, index, 0);
        assert!(is_angle, "expected Angle role at {index}");
    }
    assert_eq!(rotatex_argument_spelling(&result, 0, 0).1, "90deg");
    assert_eq!(rotatex_argument_spelling(&result, 1, 0).1, "100grad");
    assert_eq!(rotatex_argument_spelling(&result, 2, 0).1, "1rad");
    assert_eq!(rotatex_argument_spelling(&result, 3, 0).1, "0.25turn");
    assert_eq!(rotatex_argument_spelling(&result, 4, 0).1, "90DEG");
    assert_eq!(rotatex_argument_spelling(&result, 5, 0).1, "1RAD");
    assert_eq!(rotatex_argument_spelling(&result, 6, 0).1, "-45deg");
    assert_eq!(rotatex_argument_spelling(&result, 7, 0).1, "+30deg");
}

// 278. Representative exact-zero `Number` spellings qualify the single
// `rotateX()` slot through the `<zero>` branch, reusing the accepted
// `is_direct_zero_numeric_value` theorem; the retained evidence stays a
// `Number` token (#669).

#[test]
fn canonical_rotatex_zero_qualifies() {
    let result = qualify(
        669120,
        concat!(
            "a{transform:rotateX(0);}",
            "b{transform:rotateX(+0);}",
            "c{transform:rotateX(-0);}",
            "d{transform:rotateX(.0);}",
            "e{transform:rotateX(0.0);}",
            "f{transform:rotateX(0e100);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..6 {
        let (is_angle, _) = rotatex_argument_spelling(&result, index, 0);
        assert!(!is_angle, "expected Zero role at {index}");
    }
    assert_eq!(rotatex_argument_spelling(&result, 0, 0).1, "0");
    assert_eq!(rotatex_argument_spelling(&result, 1, 0).1, "+0");
    assert_eq!(rotatex_argument_spelling(&result, 2, 0).1, "-0");
    assert_eq!(rotatex_argument_spelling(&result, 3, 0).1, "0.0");
    assert_eq!(rotatex_argument_spelling(&result, 4, 0).1, "0.0");
    assert_eq!(rotatex_argument_spelling(&result, 5, 0).1, "0e100");
}

// 279. `0` and `0deg` remain distinct authored roles -- `<zero>` and
// `<angle>` respectively -- never collapsed into one generic scalar, never
// normalized into each other, and a `<zero>` never gains a synthesized
// `deg` unit. This is load-bearing (#669).

#[test]
fn rotatex_zero_vs_angle_authored_identity_is_preserved() {
    let result = qualify(
        669140,
        concat!("a{transform:rotateX(0);}", "b{transform:rotateX(0deg);}",),
    );

    assert_eq!(result.transform_observations().len(), 2);
    let (zero_is_angle, zero_spelling) = rotatex_argument_spelling(&result, 0, 0);
    let (angle_is_angle, angle_spelling) = rotatex_argument_spelling(&result, 1, 0);
    assert!(!zero_is_angle, "expected authored `0` to qualify as Zero");
    assert!(
        angle_is_angle,
        "expected authored `0deg` to qualify as Angle"
    );
    assert_eq!(zero_spelling, "0");
    assert_eq!(angle_spelling, "0deg");
}

// 280. `rotateX()` accepts exactly one authored argument: zero, two, or
// three directly visible arguments are decisive
// `InvalidForSelectedValueGrammar` before any argument content is
// consulted (#669).

#[test]
fn rotatex_argument_cardinality_other_than_one_is_invalid() {
    assert_all_invalid(
        669160,
        &[
            "rotateX()",
            "rotateX(0,90deg)",
            "rotateX(90deg,0)",
            "rotateX(0,0)",
            "rotateX(90deg,180deg)",
            "rotateX(90deg,180deg,270deg)",
        ],
    );
}

// 281. `,` is the only accepted inner separator, and an authored-empty
// position is preserved as its own ordered slot and rejected -- never
// collapsed away (#669).

#[test]
fn rotatex_argument_delimiter_failures_are_invalid() {
    assert_all_invalid(669180, &["rotateX(,)", "rotateX(,0)", "rotateX(0,)"]);
}

// 282. `RotateXArgument := DirectAngle | DirectZero` admits no other direct
// token category: a nonzero unitless `Number`, a `Percentage`, a
// non-angle-unit `Dimension`, an `Ident` -- including a longhand-axis
// keyword, which this transform-function grammar never imports -- a
// `String`, and a `Hash` are all decisive direct token-category failures
// (#669).

#[test]
fn wrong_direct_categories_are_invalid_for_rotatex() {
    assert_all_invalid(
        669200,
        &[
            "rotateX(1)",
            "rotateX(-1)",
            "rotateX(.5)",
            "rotateX(1e0)",
            "rotateX(1px)",
            "rotateX(1em)",
            "rotateX(1s)",
            "rotateX(1fr)",
            "rotateX(1unknownunit)",
            "rotateX(0%)",
            "rotateX(100%)",
            "rotateX(foo)",
            "rotateX(x)",
            "rotateX(y)",
            "rotateX(z)",
            "rotateX(\"0\")",
            "rotateX(#abc)",
        ],
    );
}

// 283. A complete non-deferred Function occupying the single `rotateX()`
// argument slot is selected-profile Unsupported, mirroring the shared
// `FunctionValuedTransformArgument` reason the other selected functions use
// -- this leaf never evaluates `calc()`, so `calc(0)` is never direct
// `<zero>` and `calc(90deg)` is never direct `<angle>` (#669).

#[test]
fn opaque_rotatex_argument_function_is_unsupported() {
    assert_all_unsupported(
        669220,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &[
            "rotateX(calc(0))",
            "rotateX(calc(90deg))",
            "rotateX(min(0deg,90deg))",
            "rotateX(max(0deg,90deg))",
            "rotateX(clamp(0deg,45deg,90deg))",
            "matrix(1,0,0,1,0,0) rotateX(calc(0))",
        ],
    );
}

// 284. A Function followed by additional direct material in the same slot
// is directly visible structural failure and stays decisively `Invalid`: a
// structurally feasible complete opaque Function is never softened to
// Unsupported by an unevaluated sibling in the same slot (#669).

#[test]
fn function_plus_junk_in_same_slot_is_invalid_for_rotatex() {
    assert_all_invalid(
        669240,
        &[
            "rotateX(calc(0) 0)",
            "rotateX(calc(90deg) foo)",
            "rotateX(calc(90deg) 1deg)",
        ],
    );
}

// 285. A comma nested inside a Function argument is at a deeper relative
// depth and never becomes a rotateX-level argument separator. Once the
// nesting closes, a genuinely different rotateX-level slot count is still
// counted and makes the shell decisively Invalid, since `rotateX()` has no
// second slot to occupy (#669).

#[test]
fn nested_commas_never_change_rotatex_arity() {
    assert_all_unsupported(
        669260,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        &["rotateX(calc(1,2))"],
    );

    assert_all_invalid(669261, &["rotateX(calc(1,2),0)"]);
}

// 286. Deferred substitution can alter the enclosing token sequence,
// separators, and cardinality, so it is resolved before any surrounding
// rotateX shape conclusion -- including an arity that looks decisive
// (#669).

#[test]
fn rotatex_deferred_substitution_outranks_surrounding_shape_conclusions() {
    assert_all_unsupported(
        669280,
        CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        &[
            "rotateX(var(--r))",
            "rotateX(var(--r),90deg)",
            "matrix(1,0,0,1,0,0) rotateX(var(--r))",
            "rotateX(var(--r)) matrix(1,0,0,1,0,0)",
        ],
    );
}

// 287. Directly visible decisive invalidity outranks an opaque Function
// found in a sibling slot; a direct wrong-category failure remains
// decisive even alongside an outer unselected sibling, in either authored
// order. `rotateY()` is used as the still-unselected outer sentinel here,
// since it remains outside selected-profile coverage after this leaf
// (#669).

#[test]
fn rotatex_decisive_invalid_outranks_opaque_argument() {
    assert_all_invalid(669300, &["rotateX(calc(0),1)"]);

    assert_all_invalid(
        669301,
        &["rotateX(1) rotateY(90deg)", "rotateY(90deg) rotateX(1)"],
    );
}

// 288. Every `<transform-function>` other than the selected leaves stays
// outside selected-profile coverage, in either authored order and
// regardless of a sibling qualified `rotateX()`, and the coarser outer
// unselected-function coverage outranks an inner opaque `rotateX()`
// argument. `rotateY()`/`matrix3d()` are used as the still-unselected outer
// sentinels here, since they remain outside selected-profile coverage
// after this leaf (#669).

#[test]
fn unselected_outer_function_precedence_covers_rotatex() {
    assert_all_unsupported(
        669310,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotateX(90deg) rotateY(90deg)",
            "rotateY(90deg) rotateX(90deg)",
            "rotateX(90deg) matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
        ],
    );

    // Coarser outer unselected-function coverage outranks an inner opaque
    // rotateX argument, identically in both authored orders.
    assert_all_unsupported(
        669320,
        CssTransformUnsupportedReason::UnselectedTransformFunction,
        &[
            "rotateX(calc(90deg)) rotateY(90deg)",
            "rotateY(90deg) rotateX(calc(90deg))",
        ],
    );

    // Decisive direct Invalid still wins over the outer unselected sibling.
    assert_all_invalid(669330, &["rotateX(1) rotateY(90deg)"]);
}

// 289. Selected `matrix()`, `scale()`, `translate3d()`, `rotate3d()`,
// `translate()`, `translateX()`, `translateY()`, `translateZ()`,
// `scaleX()`, `scaleY()`, `scaleZ()`, `scale3d()`, `rotate()`, and
// `rotateX()` components mix and repeat freely, preserving exact authored
// order and repetition through the heterogeneous `CssTransformFunction`
// alternation, and no selected leaf's evidence drifts across another's
// index in a mixed sequence (#418 / #645 / #647 / #649 / #651 / #653 /
// #655 / #657 / #659 / #661 / #663 / #665 / #667 / #669).

#[test]
fn mixed_selected_function_order_including_rotatex_is_preserved() {
    let result = qualify(
        669340,
        concat!(
            "a{transform:matrix(1,0,0,1,0,0) rotateX(90deg);}",
            "b{transform:rotateX(90deg) matrix(1,0,0,1,0,0);}",
            "c{transform:rotateX(1deg) rotateX(2deg);}",
            "d{transform:matrix(1,0,0,1,0,0) scale(2) translate3d(1px,2px,3px) rotate3d(1,0,0,90deg) translate(10px) translateX(20px) translateY(30px) translateZ(40px) scaleX(50) scaleY(60) scaleZ(70) scale3d(80,90,100) rotate(45deg) rotateX(45deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 4);

    assert!(matches!(
        qualified_functions(&result, 0)[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 0)[1],
        CssTransformFunction::RotateX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[0],
        CssTransformFunction::RotateX(_)
    ));
    assert!(matches!(
        qualified_functions(&result, 1)[1],
        CssTransformFunction::Matrix(_)
    ));

    // Repeated `rotateX()` preserves authored order and each component's
    // own evidence.
    let repeated = qualified_functions(&result, 2);
    assert_eq!(repeated.len(), 2);
    assert!(matches!(repeated[0], CssTransformFunction::RotateX(_)));
    assert!(matches!(repeated[1], CssTransformFunction::RotateX(_)));
    assert_eq!(
        rotatex_argument_spelling(&result, 2, 0),
        (true, "1deg".to_string())
    );
    assert_eq!(
        rotatex_argument_spelling(&result, 2, 1),
        (true, "2deg".to_string())
    );

    let fourteen_kind_sequence = qualified_functions(&result, 3);
    assert_eq!(fourteen_kind_sequence.len(), 14);
    assert!(matches!(
        fourteen_kind_sequence[0],
        CssTransformFunction::Matrix(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[1],
        CssTransformFunction::Scale(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[2],
        CssTransformFunction::Translate3d(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[3],
        CssTransformFunction::Rotate3d(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[4],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[5],
        CssTransformFunction::TranslateX(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[6],
        CssTransformFunction::TranslateY(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[7],
        CssTransformFunction::TranslateZ(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[8],
        CssTransformFunction::ScaleX(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[9],
        CssTransformFunction::ScaleY(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[10],
        CssTransformFunction::ScaleZ(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[11],
        CssTransformFunction::Scale3d(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[12],
        CssTransformFunction::Rotate(_)
    ));
    assert!(matches!(
        fourteen_kind_sequence[13],
        CssTransformFunction::RotateX(_)
    ));
    assert_eq!(
        rotate_argument_spelling(&result, 3, 12),
        (true, "45deg".to_string())
    );
    assert_eq!(
        rotatex_argument_spelling(&result, 3, 13),
        (true, "45deg".to_string())
    );
}

// 290. `rotate()` and `rotateX()` remain distinct selected components:
// neither their representation, cardinality, nor evidence leaks into one
// another in a mixed sequence, in any authored order, even though they
// share the same direct `Angle | Zero` scalar theorem (#667 / #669).

#[test]
fn rotate_and_rotatex_remain_distinct_selected_components() {
    let result = qualify(
        669360,
        concat!(
            "a{transform:rotate(90deg) rotateX(90deg);}",
            "b{transform:rotateX(90deg) rotate(90deg);}",
            "c{transform:rotate(0) rotateX(0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::Rotate(_)));
    assert!(matches!(first[1], CssTransformFunction::RotateX(_)));

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::RotateX(_)));
    assert!(matches!(second[1], CssTransformFunction::Rotate(_)));

    let third = qualified_functions(&result, 2);
    assert_eq!(third.len(), 2);
    assert!(matches!(third[0], CssTransformFunction::Rotate(_)));
    assert!(matches!(third[1], CssTransformFunction::RotateX(_)));
    assert_eq!(
        rotate_argument_spelling(&result, 2, 0),
        (false, "0".to_string())
    );
    assert_eq!(
        rotatex_argument_spelling(&result, 2, 1),
        (false, "0".to_string())
    );
}

// 291. `rotateX()` and `rotate3d()` remain distinct selected components:
// neither their representation, cardinality, nor evidence leaks into one
// another in a mixed sequence, in any authored order, even though
// `rotateX()` composes `rotate3d()`'s fourth-slot `Angle | Zero` scalar
// theorem, and `rotateX()` is never synthesized as `rotate3d(1,0,0,...)`
// (#649 / #669).

#[test]
fn rotatex_and_rotate3d_remain_distinct_selected_components() {
    let result = qualify(
        669380,
        concat!(
            "a{transform:rotateX(90deg) rotate3d(1,0,0,90deg);}",
            "b{transform:rotate3d(1,0,0,90deg) rotateX(90deg);}",
            "c{transform:rotateX(0) rotate3d(1,0,0,0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);

    let first = qualified_functions(&result, 0);
    assert_eq!(first.len(), 2);
    assert!(matches!(first[0], CssTransformFunction::RotateX(_)));
    assert!(matches!(first[1], CssTransformFunction::Rotate3d(_)));

    let second = qualified_functions(&result, 1);
    assert_eq!(second.len(), 2);
    assert!(matches!(second[0], CssTransformFunction::Rotate3d(_)));
    assert!(matches!(second[1], CssTransformFunction::RotateX(_)));

    let third = qualified_functions(&result, 2);
    assert_eq!(third.len(), 2);
    assert!(matches!(third[0], CssTransformFunction::RotateX(_)));
    assert!(matches!(third[1], CssTransformFunction::Rotate3d(_)));
    assert_eq!(
        rotatex_argument_spelling(&result, 2, 0),
        (false, "0".to_string())
    );
    let CssTransformFunction::Rotate3d(rotate3d) = third[1] else {
        panic!("expected rotate3d component");
    };
    let CssTransformRotate3dAngleArgument::Zero(_) = rotate3d.angle() else {
        panic!(
            "expected rotate3d fourth slot to remain Zero independently of rotateX()'s own Zero role"
        );
    };
}

// 292. `rotate(0)`, `rotateX(0)`, and `rotate3d(1,0,0,0)` are three
// distinct semantic placements that each independently preserve the
// authored `Zero` role: no placement's Zero evidence is aliased with
// another's, and none is normalized into a shared generic rotate-axis
// representation (#667 / #649 / #669).

#[test]
fn rotate_placements_preserve_zero_role_independently() {
    let result = qualify(
        669400,
        "a{transform:rotate(0) rotateX(0) rotate3d(1,0,0,0);}",
    );

    assert_eq!(result.transform_observations().len(), 1);
    let functions = qualified_functions(&result, 0);
    assert_eq!(functions.len(), 3);
    assert!(matches!(functions[0], CssTransformFunction::Rotate(_)));
    assert!(matches!(functions[1], CssTransformFunction::RotateX(_)));
    assert!(matches!(functions[2], CssTransformFunction::Rotate3d(_)));

    assert_eq!(
        rotate_argument_spelling(&result, 0, 0),
        (false, "0".to_string())
    );
    assert_eq!(
        rotatex_argument_spelling(&result, 0, 1),
        (false, "0".to_string())
    );
    let CssTransformFunction::Rotate3d(rotate3d) = functions[2] else {
        panic!("expected rotate3d component");
    };
    let CssTransformRotate3dAngleArgument::Zero(_) = rotate3d.angle() else {
        panic!("expected rotate3d fourth slot to preserve its own independent Zero role");
    };
}

// 293. `<transform-list>` is whitespace-separated repetition: a top-level
// comma is never a permitted separator, including between two `rotateX()`
// components or a `rotateX()` and a sibling selected function (#669).

#[test]
fn top_level_comma_is_invalid_around_rotatex() {
    assert_all_invalid(
        669420,
        &[
            "rotateX(90deg), matrix(1,0,0,1,0,0)",
            "rotateX(90deg), scale(2)",
            "rotateX(90deg), translate3d(1px,2px,3px)",
            "rotateX(90deg), rotate3d(1,0,0,90deg)",
            "rotateX(90deg), translate(10px)",
            "rotateX(90deg), translateX(20px)",
            "rotateX(90deg), translateY(20px)",
            "rotateX(90deg), translateZ(20px)",
            "rotateX(90deg), scaleX(3)",
            "rotateX(90deg), scaleY(3)",
            "rotateX(90deg), scaleZ(3)",
            "rotateX(90deg), scale3d(2,3,4)",
            "rotateX(90deg), rotate(90deg)",
            "rotateX(90deg), rotateX(90deg)",
        ],
    );
}

// 294. CSS Syntax function consumption may end at a true stylesheet EOF, so
// a parser-committed EOF-ended `rotateX()` extent is qualified from
// retained interior evidence alone once the slot is complete. EOF never
// fills or repairs a missing argument, a wrong direct category, or a
// second slot `rotateX()` has no room for (#669).

#[test]
fn true_stylesheet_eof_ended_rotatex_extent_follows_parser_authority() {
    let angle_eof = qualify(669450, "a{transform:rotateX(90deg");
    assert_eq!(
        angle_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(angle_eof.transform_observations().len(), 1);
    assert_eq!(
        rotatex_argument_spelling(&angle_eof, 0, 0),
        (true, "90deg".to_string())
    );

    let zero_eof = qualify(669451, "a{transform:rotateX(0");
    assert_eq!(
        zero_eof.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(zero_eof.transform_observations().len(), 1);
    assert_eq!(
        rotatex_argument_spelling(&zero_eof, 0, 0),
        (false, "0".to_string())
    );

    // Zero-slot EOF stays decisively Invalid.
    let empty = qualify(669452, "a{transform:rotateX(");
    assert_invalid(&empty, 0);

    // EOF never repairs a decisive direct category.
    let wrong_category_eof = qualify(669453, "a{transform:rotateX(1");
    assert_invalid(&wrong_category_eof, 0);

    // A trailing comma at true EOF stays decisively Invalid: rotateX()
    // never accepts a second slot.
    let trailing_comma = qualify(669454, "a{transform:rotateX(0,");
    assert_invalid(&trailing_comma, 0);

    // A second slot never widens rotateX() cardinality, even at true EOF.
    let two_slots_eof = qualify(669455, "a{transform:rotateX(90deg,0");
    assert_invalid(&two_slots_eof, 0);

    // A closed component followed by stray material is invalid, proving
    // the accepted EOF case is not "accept whatever trails a rotateX".
    let trailing = qualify(669456, "a{transform:rotateX(90deg) 7;}");
    assert_invalid(&trailing, 0);
}

// 295. Lower-layer lifecycle evidence stays owned by the tokenizer and
// parser: comments/trivia never change slot interpretation, a comment
// never fills an authored-empty slot, and `!important` remains outside the
// semantic value window (#669).

#[test]
fn trivia_and_important_never_change_rotatex_interpretation() {
    let result = qualify(
        669470,
        concat!(
            "a{transform:rotateX(90deg/**/);}",
            "b{transform:rotateX(/**/90deg);}",
            "c{transform:rotateX( 90deg );}",
            "d{transform:rotateX(90deg) !important;}",
            "e{transform:rotateX(90deg)!important;}",
            "f{transform:rotateX(/**/0);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 6);
    for index in 0..3 {
        assert_eq!(
            rotatex_argument_spelling(&result, index, 0),
            (true, "90deg".to_string()),
            "trivia changed slot interpretation at {index}"
        );
    }
    for index in 3..5 {
        assert_eq!(
            rotatex_argument_spelling(&result, index, 0),
            (true, "90deg".to_string())
        );
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some(),
            "expected retained priority evidence at {index}"
        );
    }
    assert_eq!(
        rotatex_argument_spelling(&result, 5, 0),
        (false, "0".to_string()),
        "trivia changed Zero slot interpretation"
    );

    // A comment is trivia, never an argument: it can neither fill the
    // single authored slot nor stand in for a missing one.
    assert_all_invalid(669480, &["rotateX(/**/)", "rotateX(90deg,/**/)"]);
}

// 296. Repeated and cross-source runs remain deterministic, and evidence
// lookup resolves to the exact retained Number/Dimension tokens
// identically across runs (#669).

#[test]
fn rotatex_repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform:rotateX(0,90deg);}",
        "b{transform:rotateX(calc(0));}",
        "c{transform:rotateX(1px);}",
        "d{transform:matrix(1,0,0,1,0,0) rotateX(90deg);}",
        "e{transform:rotateX(0);}",
    );

    let first = qualify(669500, css);
    let repeated = qualify(669500, css);
    let another_source = qualify(669501, css);

    assert_eq!(
        first.transform_observations(),
        repeated.transform_observations()
    );
    assert_eq!(
        first.transform_observations(),
        another_source.transform_observations()
    );

    assert_invalid(&first, 0);
    assert_unsupported(
        &first,
        1,
        CssTransformUnsupportedReason::FunctionValuedTransformArgument,
    );
    assert_invalid(&first, 2);
    assert_eq!(qualified_functions(&first, 3).len(), 2);

    assert_eq!(
        rotatex_argument_spelling(&first, 3, 1),
        rotatex_argument_spelling(&repeated, 3, 1)
    );
    assert_eq!(
        rotatex_argument_spelling(&first, 3, 1),
        rotatex_argument_spelling(&another_source, 3, 1)
    );
    assert_eq!(
        rotatex_argument_spelling(&first, 4, 0),
        rotatex_argument_spelling(&repeated, 4, 0)
    );
    assert_eq!(
        rotatex_argument_spelling(&first, 4, 0),
        rotatex_argument_spelling(&another_source, 4, 0)
    );
}

// 297. `rotateX` function-name recognition is ASCII-case-insensitive,
// matching the accepted `matrix`/`scale`/`translate3d`/`rotate3d`/
// `translate`/`translateX`/`translateY`/`translateZ`/`scaleX`/`scaleY`/
// `scaleZ`/`scale3d`/`rotate` boundary, and structurally malformed
// `rotateX` components -- a bare Function name, an unbalanced/duplicated
// closer, or stray leading material -- stay decisively `Invalid` (#669).

#[test]
fn rotatex_function_name_recognition_and_malformed_components() {
    let result = qualify(
        669520,
        concat!(
            "a{transform:ROTATEX(90deg);}",
            "b{transform:RoTaTeX(90deg);}",
            "c{transform:r\\6f tateX(90deg);}",
        ),
    );

    assert_eq!(result.transform_observations().len(), 3);
    for index in 0..3 {
        assert_eq!(
            rotatex_argument_spelling(&result, index, 0),
            (true, "90deg".to_string()),
            "rotateX name recognition failed at {index}"
        );
    }

    assert_all_invalid(
        669530,
        &[
            "rotateX",
            "rotateX(90deg))",
            "rotateX((90deg)",
            "1 rotateX(90deg)",
            "[rotateX(90deg)]",
        ],
    );
}

// 298. Cross-leaf isolation: `rotateX()` recognition never leaks into the
// accepted longhand `translate`/`scale`/`rotate` leaves, the other
// `transform-*` single-value leaves, or the other selected `transform`
// function branches, and `none` remains an exclusive whole-value branch
// even when combined with `rotateX()`. In particular, the longhand
// `rotate` property qualifier (#604) stays entirely unaffected by this
// leaf: it remains its own separate value grammar and representation,
// never gaining the transform-function legacy `<zero>` branch (#418 /
// #604 / #606 / #645 / #647 / #649 / #651 / #653 / #655 / #657 / #659 /
// #661 / #663 / #665 / #667 / #669).

#[test]
fn rotatex_cross_leaf_isolation_is_preserved() {
    let result = qualify(
        669550,
        concat!(
            "a{transform:rotateX(90deg);transform:none;}",
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
        rotatex_argument_spelling(&result, 0, 0),
        (true, "90deg".to_string())
    );
    assert_whole_none(&result, 1);

    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);

    assert_all_invalid(669570, &["none rotateX(90deg)", "rotateX(90deg) none"]);
}
