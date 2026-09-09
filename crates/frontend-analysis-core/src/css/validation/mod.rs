mod candidate;
mod context_candidate;
mod context_fixtures;
mod context_gold;
mod descriptor_candidate;
mod descriptor_fixtures;
mod descriptor_gold;
mod fixtures;
mod generated;
mod gold;
mod group_context_fixtures;
mod keyframe_candidate;
mod keyframe_fixtures;
mod keyframe_gold;
mod page_candidate;
mod page_fixtures;
mod page_gold;
mod parser_candidate;
mod parser_fixtures;
mod parser_gold;
mod selector_gold;

#[cfg(test)]
mod anchor_name_value_qualification_tests;
#[cfg(test)]
mod animation_delay_value_qualification_tests;
#[cfg(test)]
mod animation_iteration_count_value_qualification_tests;
#[cfg(test)]
mod animation_name_value_qualification_tests;
#[cfg(test)]
mod animation_play_state_value_qualification_tests;
#[cfg(test)]
mod aspect_ratio_value_qualification_tests;
#[cfg(test)]
mod backface_visibility_value_qualification_tests;
#[cfg(test)]
mod border_collapse_value_qualification_tests;
#[cfg(test)]
mod border_spacing_value_qualification_tests;
#[cfg(test)]
mod border_top_width_value_qualification_tests;
#[cfg(test)]
mod box_decoration_break_value_qualification_tests;
#[cfg(test)]
mod box_sizing_value_qualification_tests;
#[cfg(test)]
mod clip_rule_value_qualification_tests;
#[cfg(test)]
mod color_interpolation_filters_value_qualification_tests;
#[cfg(test)]
mod color_scheme_value_qualification_tests;
#[cfg(test)]
mod column_count_value_qualification_tests;
#[cfg(test)]
mod column_fill_value_qualification_tests;
#[cfg(test)]
mod conformance_tests;
#[cfg(test)]
mod contain_value_qualification_tests;
#[cfg(test)]
mod container_name_value_qualification_tests;
#[cfg(test)]
mod context_conformance_tests;
#[cfg(test)]
mod context_contract_tests;
#[cfg(test)]
mod core_analysis_gate;
#[cfg(test)]
mod core_context_analysis_gate;
#[cfg(test)]
mod counter_increment_value_qualification_tests;
#[cfg(test)]
mod counter_reset_value_qualification_tests;
#[cfg(test)]
mod descriptor_conformance_tests;
#[cfg(test)]
mod descriptor_lifecycle_validation_tests;
#[cfg(test)]
mod direction_value_qualification_tests;
#[cfg(test)]
mod empty_cells_value_qualification_tests;
#[cfg(test)]
mod fill_opacity_value_qualification_tests;
#[cfg(test)]
mod fill_rule_value_qualification_tests;
#[cfg(test)]
mod flex_grow_value_qualification_tests;
#[cfg(test)]
mod flex_shrink_value_qualification_tests;
#[cfg(test)]
mod font_kerning_value_qualification_tests;
#[cfg(test)]
mod font_synthesis_position_value_qualification_tests;
#[cfg(test)]
mod font_synthesis_small_caps_value_qualification_tests;
#[cfg(test)]
mod font_synthesis_weight_value_qualification_tests;
#[cfg(test)]
mod font_variant_caps_value_qualification_tests;
#[cfg(test)]
mod font_variant_emoji_value_qualification_tests;
#[cfg(test)]
mod font_variant_ligatures_value_qualification_tests;
#[cfg(test)]
mod font_variant_numeric_value_qualification_tests;
#[cfg(test)]
mod font_variant_position_value_qualification_tests;
#[cfg(test)]
mod font_weight_value_qualification_tests;
#[cfg(test)]
mod forced_color_adjust_value_qualification_tests;
#[cfg(test)]
mod group_context_contract_tests;
#[cfg(test)]
mod group_lifecycle_validation_tests;
#[cfg(test)]
mod hyphenate_character_value_qualification_tests;
#[cfg(test)]
mod isolation_value_qualification_tests;
#[cfg(test)]
mod keyframe_conformance_tests;
#[cfg(test)]
mod keyframe_lifecycle_validation_tests;
#[cfg(test)]
mod line_break_value_qualification_tests;
#[cfg(test)]
mod line_height_value_qualification_tests;
#[cfg(test)]
mod mask_type_value_qualification_tests;
#[cfg(test)]
mod math_shift_value_qualification_tests;
#[cfg(test)]
mod math_style_value_qualification_tests;
#[cfg(test)]
mod offset_rotate_value_qualification_tests;
#[cfg(test)]
mod opacity_value_qualification_tests;
#[cfg(test)]
mod order_value_qualification_tests;
#[cfg(test)]
mod overflow_wrap_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_block_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_inline_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_shorthand_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_x_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_y_value_qualification_tests;
#[cfg(test)]
mod page_conformance_tests;
#[cfg(test)]
mod page_lifecycle_validation_tests;
#[cfg(test)]
mod page_value_qualification_tests;
#[cfg(test)]
mod parser_conformance_tests;
#[cfg(test)]
mod parser_contract_tests;
#[cfg(test)]
mod parser_resource_tests;
#[cfg(test)]
mod perspective_value_qualification_tests;
#[cfg(test)]
mod print_color_adjust_value_qualification_tests;
#[cfg(test)]
mod resource_tests;
#[cfg(test)]
mod ruby_align_value_qualification_tests;
#[cfg(test)]
mod ruby_merge_value_qualification_tests;
#[cfg(test)]
mod ruby_overhang_value_qualification_tests;
#[cfg(test)]
mod ruby_position_value_qualification_tests;
#[cfg(test)]
mod scroll_margin_top_value_qualification_tests;
#[cfg(test)]
mod scroll_snap_align_value_qualification_tests;
#[cfg(test)]
mod scroll_snap_stop_value_qualification_tests;
#[cfg(test)]
mod selector_conformance_tests;
#[cfg(test)]
mod selector_gold_contract_tests;
#[cfg(test)]
mod shape_image_threshold_value_qualification_tests;
#[cfg(test)]
mod shape_margin_value_qualification_tests;
#[cfg(test)]
mod shape_rendering_value_qualification_tests;
#[cfg(test)]
mod table_layout_value_qualification_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod text_align_last_value_qualification_tests;
#[cfg(test)]
mod text_anchor_value_qualification_tests;
#[cfg(test)]
mod text_decoration_line_value_qualification_tests;
#[cfg(test)]
mod text_decoration_skip_ink_value_qualification_tests;
#[cfg(test)]
mod text_decoration_style_value_qualification_tests;
#[cfg(test)]
mod text_emphasis_position_value_qualification_tests;
#[cfg(test)]
mod text_rendering_value_qualification_tests;
#[cfg(test)]
mod text_transform_value_qualification_tests;
#[cfg(test)]
mod text_underline_offset_value_qualification_tests;
#[cfg(test)]
mod transition_duration_value_qualification_tests;
#[cfg(test)]
mod transition_property_value_qualification_tests;
#[cfg(test)]
mod unicode_bidi_value_qualification_tests;
#[cfg(test)]
mod word_spacing_value_qualification_tests;
#[cfg(test)]
mod z_index_value_qualification_tests;

// Focused evidence completion for #594: unlike the true-EOF positive case in
// `counter_reset_value_qualification_tests`, this fixture has no authored `)`
// for `reversed(` and includes later declaration-shaped material. The parser's
// retained structure must therefore expose more than one inner semantic token,
// making the grammar-native `reversed(<counter-name>)` branch invalid rather
// than accepting every missing-close Function as the true-EOF one-Ident form.
#[cfg(test)]
mod counter_reset_missing_close_evidence_completion {
    use crate::css::analysis::analyze_css_source;
    use crate::css::parser::resource::CssParserLimits;
    use crate::css::parser::result::CssParserExecutionCompletion;
    use crate::css::token::{CssLexicalItem, CssTokenKind};
    use crate::css::tokenizer::resource::CssTokenizerLimits;
    use crate::css::value_qualification::{CssCounterResetQualificationOutcome, run};
    use crate::{SourceId, SourceText};

    #[test]
    fn actual_missing_reversed_close_absorbs_following_material_and_is_invalid() {
        let source = SourceText::new(
            SourceId::new(594132),
            "a{counter-reset:reversed(foo;color:red;}".to_owned(),
        );
        let tokenizer_limits =
            CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap();
        let parser_limits =
            CssParserLimits::new(100_000, 256, 256, 8192, 1024, 1024, 1024, 1024, 8192).unwrap();
        let parser_result = analyze_css_source(&source, tokenizer_limits, parser_limits).unwrap();
        let result = run(parser_result).unwrap();

        assert_eq!(
            result.execution_completion(),
            CssParserExecutionCompletion::Complete
        );
        assert_eq!(result.counter_reset_observations().len(), 1);
        assert_eq!(
            result.counter_reset_observations()[0].outcome(),
            &CssCounterResetQualificationOutcome::InvalidForSelectedValueGrammar
        );

        let lexical_items = result
            .upstream_parser_result()
            .upstream_tokenizer_result()
            .lexical_items();
        assert!(
            !lexical_items.iter().any(|item| matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::RightParenthesis)
            )),
            "fixture unexpectedly contains an authored right parenthesis"
        );
        assert!(
            lexical_items.iter().any(|item| matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Semicolon)
            )),
            "expected later semicolon material to remain in retained evidence"
        );
        assert!(
            lexical_items.iter().any(|item| matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Ident(value) if value == "color")
            )),
            "expected later declaration-shaped material to remain in retained evidence"
        );
    }
}
