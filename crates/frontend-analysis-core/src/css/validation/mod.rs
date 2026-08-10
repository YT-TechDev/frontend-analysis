mod candidate;
mod context_candidate;
mod context_fixtures;
mod context_gold;
mod fixtures;
mod generated;
mod gold;
mod group_context_fixtures;
mod parser_candidate;
mod parser_fixtures;
mod parser_gold;

#[cfg(test)]
mod conformance_tests;
#[cfg(test)]
mod context_conformance_tests;
#[cfg(test)]
mod context_contract_tests;
#[cfg(test)]
mod core_analysis_gate;
#[cfg(test)]
mod group_context_contract_tests;
#[cfg(test)]
mod group_lifecycle_validation_tests;
#[cfg(test)]
mod parser_conformance_tests;
#[cfg(test)]
mod parser_contract_tests;
#[cfg(test)]
mod parser_resource_tests;
#[cfg(test)]
mod resource_tests;
#[cfg(test)]
mod tests;
