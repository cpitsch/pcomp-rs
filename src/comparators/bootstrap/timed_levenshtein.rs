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

use super::bootstrap_comparator::BootstrapTestComparator;

/// An implementation of the [`BootstrapTestComparator`] for timed control flow
/// comparisons using K-Means++ clustering to bin service times and the postnormalized
/// weighted Levenshtein distance as a distance notion between _service-time traces_.
///
/// This is an extension to the Bootstrap Method proposed in "Statistical tests
/// and association measures for business processes" by Leemans et al. and has not
/// been evaluated anywhere.
#[derive(Default, Debug)]
pub struct TimedLevenshteinBootstrapComparator {
    binner_args: KMeansArgs,
}

impl TimedLevenshteinBootstrapComparator {
    pub fn new(binner_args: KMeansArgs) -> Self {
        Self { binner_args }
    }
}

impl BootstrapTestComparator<Vec<(String, usize)>> for TimedLevenshteinBootstrapComparator {
    fn extract_representations(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
    ) -> AttributeResult<(Vec<Vec<(String, usize)>>, Vec<Vec<(String, usize)>>)> {
        let service_time_traces_1 = extract_service_time_traces(log_1)?;
        let service_time_traces_2 = extract_service_time_traces(log_2)?;

        let combined_data: Vec<_> = service_time_traces_1
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
