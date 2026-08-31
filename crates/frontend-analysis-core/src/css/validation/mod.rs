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
mod group_context_contract_tests;
#[cfg(test)]
mod group_lifecycle_validation_tests;
#[cfg(test)]
mod keyframe_conformance_tests;
#[cfg(test)]
mod keyframe_lifecycle_validation_tests;
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
mod resource_tests;
#[cfg(test)]
mod selector_conformance_tests;
#[cfg(test)]
mod selector_gold_contract_tests;
#[cfg(test)]
mod selector_semantic_handoff_gold;
#[cfg(test)]
mod selector_semantic_handoff_production_tests;
#[cfg(test)]
mod selector_semantic_handoff_reference;
#[cfg(test)]
mod selector_semantic_handoff_validation_tests;
#[cfg(test)]
mod tests;
