use crate::{
    distance::weighted_levenshtein::postnormalized_weighted_levenshtein_distance,
    utils::{attributes::get_activity_label, constants::NO_ACTIVITY_LABEL_MSG},
};

use super::permutation_test_comparator::PermutationTestComparator;

pub struct ControlFlowPermutationComparator;

impl PermutationTestComparator<Vec<String>> for ControlFlowPermutationComparator {
    fn extract_representations(
        &self,
        log_1: &process_mining::EventLog,
        log_2: &process_mining::EventLog,
    ) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        (
            log_1
                .traces
                .iter()
                .map(|trace| {
                    trace
                        .events
                        .iter()
                        .map(|event| get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG))
                        .collect()
                })
                .collect(),
            log_2
                .traces
                .iter()
                .map(|trace| {
                    trace
                        .events
                        .iter()
                        .map(|event| get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG))
                        .collect()
                })
                .collect(),
        )
    }

    fn cost(&self, rep_1: &Vec<String>, rep_2: &Vec<String>) -> f64 {
        postnormalized_weighted_levenshtein_distance(rep_1, rep_2)
    }
}
