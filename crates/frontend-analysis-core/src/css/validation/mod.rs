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
mod backface_visibility_value_qualification_tests;
#[cfg(test)]
mod border_collapse_value_qualification_tests;
#[cfg(test)]
mod border_top_width_value_qualification_tests;
#[cfg(test)]
mod box_decoration_break_value_qualification_tests;
#[cfg(test)]
mod box_sizing_value_qualification_tests;
#[cfg(test)]
mod column_count_value_qualification_tests;
#[cfg(test)]
mod conformance_tests;
#[cfg(test)]
mod context_conformance_tests;
#[cfg(test)]
mod context_contract_tests;
#[cfg(test)]
mod core_analysis_gate;
#[cfg(test)]
mod core_context_analysis_gate;
#[cfg(test)]
mod descriptor_conformance_tests;
#[cfg(test)]
mod descriptor_lifecycle_validation_tests;
#[cfg(test)]
mod direction_value_qualification_tests;
#[cfg(test)]
mod empty_cells_value_qualification_tests;
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
mod font_variant_position_value_qualification_tests;
#[cfg(test)]
mod group_context_contract_tests;
#[cfg(test)]
mod group_lifecycle_validation_tests;
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
mod opacity_value_qualification_tests;
#[cfg(test)]
mod order_value_qualification_tests;
#[cfg(test)]
mod overflow_wrap_value_qualification_tests;
#[cfg(test)]
mod page_conformance_tests;
#[cfg(test)]
mod page_lifecycle_validation_tests;
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
mod table_layout_value_qualification_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod text_decoration_style_value_qualification_tests;
#[cfg(test)]
mod text_underline_offset_value_qualification_tests;
#[cfg(test)]
mod unicode_bidi_value_qualification_tests;
#[cfg(test)]
mod word_spacing_value_qualification_tests;
#[cfg(test)]
mod z_index_value_qualification_tests;
