use process_mining::EventLog;

use crate::{
    comparators::common::extraction::project_traces_on_activity,
    distance::weighted_levenshtein::postnormalized_weighted_levenshtein_distance,
    utils::attributes::attribute_error::AttributeResult,
};

use super::bootstrap_comparator::BootstrapTestComparator;

pub struct ControlFlowBootstrapComparator;

impl BootstrapTestComparator<Vec<String>> for ControlFlowBootstrapComparator {
    fn extract_representations(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
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
