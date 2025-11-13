use process_mining::EventLog;

use crate::{
    binning::{
        kmeans_binner::{KMeansArgs, KMeansBinner},
        BinnerManager,
    },
    comparators::common::extraction::{
        apply_binner_manager_on_service_time_traces, extract_service_time_traces,
    },
    distance::weighted_levenshtein::postnormalized_weighted_levenshtein_distance,
    utils::attributes::attribute_error::AttributeResult,
};

use super::permutation_test_comparator::PermutationTestComparator;

#[derive(Default, Debug)]
pub struct TimedLevenshteinPermutationComparator {
    binner_args: KMeansArgs,
}

impl TimedLevenshteinPermutationComparator {
    pub fn new(binner_args: KMeansArgs) -> Self {
        Self { binner_args }
    }
}

impl PermutationTestComparator<Vec<(String, usize)>> for TimedLevenshteinPermutationComparator {
    fn extract_representations(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
    ) -> AttributeResult<(Vec<Vec<(String, usize)>>, Vec<Vec<(String, usize)>>)> {
        let service_time_traces_1 = extract_service_time_traces(log_1)?;
        let service_time_traces_2 = extract_service_time_traces(log_2)?;

        let combined_data: Vec<(String, f64)> = service_time_traces_1
            .iter()
            .chain(service_time_traces_2.iter())
            .flatten()
            .cloned()
            .collect();
        let binner_manager = BinnerManager::<f64, KMeansBinner>::from_key_value_pairs(
            combined_data,
            self.binner_args.clone(),
        );

        Ok((
            apply_binner_manager_on_service_time_traces(service_time_traces_1, &binner_manager),
            apply_binner_manager_on_service_time_traces(service_time_traces_2, &binner_manager),
        ))
    }

    fn cost(&self, rep_1: &Vec<(String, usize)>, rep_2: &Vec<(String, usize)>) -> f64 {
        postnormalized_weighted_levenshtein_distance(rep_1, rep_2)
    }
}
