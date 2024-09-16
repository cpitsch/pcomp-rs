use std::collections::HashSet;

use process_mining::{event_log::Trace, EventLog};

use crate::{
    binning::{outer_percentile_binner::OuterPercentileBinner, BinnerManager},
    distance::weighted_levenshtein::postnormalized_weighted_levenshtein_distance,
    utils::{
        attributes::{get_activity_label, get_service_time},
        constants::{NO_ACTIVITY_LABEL_MSG, NO_START_TIMESTAMP_MSG},
    },
};

use super::permutation_test_comparator::PermutationTestComparator;

#[derive(Default)]
pub struct TimedLevenshteinPermutationComparator;

fn trace_to_service_time_trace(trace: &Trace) -> Vec<(String, f64)> {
    trace
        .events
        .iter()
        .map(|evt| {
            (
                get_activity_label(evt).expect(NO_ACTIVITY_LABEL_MSG),
                get_service_time(evt)
                    .expect(NO_START_TIMESTAMP_MSG)
                    .num_milliseconds() as f64
                    / 1000.0,
            )
        })
        .collect()
}

impl PermutationTestComparator<Vec<(String, usize)>> for TimedLevenshteinPermutationComparator {
    fn extract_representations(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
    ) -> (Vec<Vec<(String, usize)>>, Vec<Vec<(String, usize)>>) {
        let service_time_traces_1: Vec<Vec<(String, f64)>> = log_1
            .traces
            .iter()
            .map(trace_to_service_time_trace)
            .collect();
        let service_time_traces_2: Vec<Vec<(String, f64)>> = log_2
            .traces
            .iter()
            .map(trace_to_service_time_trace)
            .collect();

        let combined_data: Vec<(String, f64)> = service_time_traces_1
            .iter()
            .chain(service_time_traces_2.iter())
            .flatten()
            .cloned()
            .collect();
        let binner_manager =
            BinnerManager::<f64, OuterPercentileBinner>::from_key_value_pairs(combined_data);

        let binned_service_time_traces_1: Vec<Vec<(String, usize)>> = service_time_traces_1
            .into_iter()
            .map(|trace| {
                trace
                    .into_iter()
                    .map(|(act, time)| {
                        let binned_time = binner_manager.bin(&act, time);
                        (act, binned_time)
                    })
                    .collect()
            })
            .collect();

        let binned_service_time_traces_2: Vec<Vec<(String, usize)>> = service_time_traces_2
            .into_iter()
            .map(|trace| {
                trace
                    .into_iter()
                    .map(|(act, time)| {
                        let binned_time = binner_manager.bin(&act, time);
                        (act, binned_time)
                    })
                    .collect()
            })
            .collect();

        (binned_service_time_traces_1, binned_service_time_traces_2)
    }

    fn cost(&self, rep_1: &Vec<(String, usize)>, rep_2: &Vec<(String, usize)>) -> f64 {
        postnormalized_weighted_levenshtein_distance(rep_1, rep_2)
    }
}
