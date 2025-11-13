use process_mining::{event_log::Trace, EventLog};

use crate::{
    binning::{Binner, BinnerManager},
    utils::attributes::{attribute_error::AttributeResult, get_activity_label, get_service_time},
};

pub fn project_trace_on_activity(trace: &Trace) -> AttributeResult<Vec<String>> {
    trace.events.iter().map(get_activity_label).collect()
}

pub fn project_traces_on_activity(log: &EventLog) -> AttributeResult<Vec<Vec<String>>> {
    log.traces.iter().map(project_trace_on_activity).collect()
}

pub fn trace_to_service_time_trace(trace: &Trace) -> AttributeResult<Vec<(String, f64)>> {
    trace
        .events
        .iter()
        .map(|evt| {
            Ok((
                get_activity_label(evt)?,
                get_service_time(evt)?.num_milliseconds() as f64 / 1000.0,
            ))
        })
        .collect()
}

pub fn extract_service_time_traces(log: &EventLog) -> AttributeResult<Vec<Vec<(String, f64)>>> {
    log.traces.iter().map(trace_to_service_time_trace).collect()
}

pub fn apply_binner_manager_on_service_time_trace<T: Binner<f64>>(
    service_time_trace: Vec<(String, f64)>,
    binner_manager: &BinnerManager<f64, T>,
) -> Vec<(String, usize)> {
    service_time_trace
        .into_iter()
        .map(|(activity, service_time)| {
            let binned_time = binner_manager.bin(&activity, service_time);
            (activity, binned_time)
        })
        .collect()
}

pub fn apply_binner_manager_on_service_time_traces<T: Binner<f64>>(
    service_time_traces: Vec<Vec<(String, f64)>>,
    binner_manager: &BinnerManager<f64, T>,
) -> Vec<Vec<(String, usize)>> {
    service_time_traces
        .into_iter()
        .map(|service_time_trace| {
            apply_binner_manager_on_service_time_trace(service_time_trace, binner_manager)
        })
        .collect()
}
