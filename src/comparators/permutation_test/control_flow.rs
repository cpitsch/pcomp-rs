use crate::{
    comparators::common::extraction::project_traces_on_activity,
    distance::weighted_levenshtein::postnormalized_weighted_levenshtein_distance,
    utils::attributes::attribute_error::AttributeResult,
};

use super::permutation_test_comparator::PermutationTestComparator;

pub struct ControlFlowPermutationComparator;

impl PermutationTestComparator<Vec<String>> for ControlFlowPermutationComparator {
    fn extract_representations(
        &self,
        log_1: &process_mining::EventLog,
        log_2: &process_mining::EventLog,
    ) -> AttributeResult<(Vec<Vec<String>>, Vec<Vec<String>>)> {
        Ok((
            project_traces_on_activity(log_1)?,
            project_traces_on_activity(log_2)?,
        ))
    }

    fn cost(&self, rep_1: &Vec<String>, rep_2: &Vec<String>) -> f64 {
        postnormalized_weighted_levenshtein_distance(rep_1, rep_2)
    }
}
