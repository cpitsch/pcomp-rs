use process_mining::{event_log::Trace, EventLog};

use crate::{
    binning::{Binner, BinnerManager},
    utils::attributes::{attribute_error::AttributeResult, get_activity_label, get_service_time},
};

/// Extract a sequence of activities from a [`Trace`].
///
/// Returns an [`AttributeError`] if the trace is missing the `concept:name` attribute
/// or the attribute is not a String.
///
/// [`AttributeError`]: crate::utils::attributes::attribute_error::AttributeError
pub fn project_trace_on_activity(trace: &Trace) -> AttributeResult<Vec<String>> {
    trace.events.iter().map(get_activity_label).collect()
}

/// Extract a sequence of activities for each [`Trace`] in the event log.
///
/// Returns an [`AttributeError`] if any trace is missing the `concept:name` attribute
/// or the attribute is not a String.
///
/// [`AttributeError`]: crate::utils::attributes::attribute_error::AttributeError
pub fn project_traces_on_activity(log: &EventLog) -> AttributeResult<Vec<Vec<String>>> {
    log.traces.iter().map(project_trace_on_activity).collect()
}

/// Extract a _service time trace_ from a [`Trace`], i.e., a sequence of tuples of
/// activity and service time.
///
/// Returns an [`AttributeError`] if:
///
/// - The trace is missing the `concept:name` attribute or it is not a String.
/// - The trace is missing the `time:timestamp` attribute or it is not a [`DateTime`].
/// - The trace is missing the `start_timestamp` attribute or it is not a [`DateTime`].
///
/// [`AttributeError`]: crate::utils::attributes::attribute_error::AttributeError
/// [`DateTime`]: chrono::DateTime
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

/// Extract a _service time trace_ for each [`Trace`] in the event log. I.e., a
/// sequence of tuples of activity and service time (in seconds).
///
/// Returns an [`AttributeError`] if for any trace:
///
/// - The `concept:name` attribute is missing or it is not a String.
/// - The `time:timestamp` attribute is missing or it is not a [`DateTime`].
/// - The `start_timestamp` attribute is missing or it is not a [`DateTime`].
///
/// [`AttributeError`]: crate::utils::attributes::attribute_error::AttributeError
/// [`DateTime`]: chrono::DateTime
pub fn extract_service_time_traces(log: &EventLog) -> AttributeResult<Vec<Vec<(String, f64)>>> {
    log.traces.iter().map(trace_to_service_time_trace).collect()
}

/// Apply binning to a service time trace.
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

/// Apply binning to service time traces.
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
