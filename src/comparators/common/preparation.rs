use chrono::{DateTime, FixedOffset};
use process_mining::{
    event_log::{Attribute, AttributeValue, Trace, XESEditableAttribute},
    EventLog,
};
use std::collections::{HashMap, VecDeque};

use crate::utils::{
    attributes::{
        add_or_overwrite_attribute, attribute_error::AttributeResult, get_activity_label,
        get_complete_timestamp, get_instance_id, get_lifecycle, HasAttributes,
    },
    constants::{INSTANCE_ID_KEY, LIFECYCLE_KEY, START_TIMESTAMP_KEY},
    retain_err::retain_mut_err,
};

/// Assume that events with no lifecycle information are atomic --> fill in "complete"
/// for the lifecycle transition.
pub fn infer_lifecycle_information(trace: &mut Trace) {
    trace.events.iter_mut().for_each(|evt| {
        if evt.get_attribute_by_key(LIFECYCLE_KEY).is_err() {
            evt.attributes.add_attribute(Attribute::new(
                LIFECYCLE_KEY.to_string(),
                AttributeValue::String("complete".to_string()),
            ));
        }
    });
}

/// Assume that events with no lifecycle information are atomic --> fill in "complete"
/// for the lifecycle transition.
pub fn infer_lifecycle_information_log(log: &mut EventLog) {
    log.traces.iter_mut().for_each(infer_lifecycle_information);
}

/// Infer the event instance id based on the lifecycle information. "start" events
/// are matched to "complete" events of the same activity in a FIFO fashion.
///
/// If the trace does not have lifecycle information, see [infer_lifecycle_information].
///
/// It is possile that certain start events do not have a corresponding complete event,
/// and vice versa. In the latter case, this denotes an atomic event, and in the first
/// case, this event never completed.
///
/// Events with a lifecycle transition other than "start" or "complete" are ignored.
pub fn infer_event_instance_id(trace: &mut Trace) -> AttributeResult<()> {
    let mut pending_instance_ids: HashMap<String, VecDeque<i64>> = HashMap::default();
    // The last used id. Incremented before use --> Lowest id is 1
    // Could use uuid, in which case no counting would be necessary, but this is fine
    // for our use-case (and faster)
    let mut current_id: i64 = 0;

    for evt in trace.events.iter_mut() {
        let activity = get_activity_label(evt)?;
        match get_lifecycle(evt)?.as_ref() {
            "complete" => {
                let pending_ids = pending_instance_ids.entry(activity).or_default();
                let instance_id = pending_ids.pop_front().unwrap_or_else(|| {
                    // If there is no corresponding start event, this event will
                    // have a unique instance id and be considered atomic
                    current_id += 1;
                    current_id
                });
                add_or_overwrite_attribute(
                    evt,
                    INSTANCE_ID_KEY,
                    AttributeValue::String(instance_id.to_string()),
                );
            }
            "start" => {
                let pending_ids = pending_instance_ids.entry(activity).or_default();
                current_id += 1;
                add_or_overwrite_attribute(
                    evt,
                    INSTANCE_ID_KEY,
                    AttributeValue::String(current_id.to_string()),
                );
                pending_ids.push_back(current_id);
            }
            _ => { /* WARN: Ignoring events with different lifecycle transition */ }
        }
    }

    // It is possible that there are left over pending instance ids. This means that
    // there were start timestamps that did not get "assigned" a complete event.
    Ok(())
}

/// Infer the event instance id based on the lifecycle information. "start" events
/// are matched to "complete" events of the same activity in a FIFO fashion.
///
/// If the event log does not have lifecycle information, see [infer_lifecycle_information_log].
///
/// It is possile that certain start events do not have a corresponding complete event,
/// and vice versa. In the latter case, this denotes an atomic event, and in the first
/// case, this event never completed.
///
/// Events with a lifecycle transition other than "start" or "complete" are ignored.
pub fn infer_event_instance_id_log(log: &mut EventLog) -> AttributeResult<()> {
    log.traces.iter_mut().try_for_each(infer_event_instance_id)
}

/// Take a trace where each event has an instance id and fold correlated start
/// and end events to a single event with a start and complete timestamp.
///
/// In the end, only the events with a "complete" lifecycle transition remain, and
/// are enriched with the `start_timestamp` attribute based on the corresponding
/// "start" event, or their own completion timestamp (assumed to be atomic).
pub fn fold_instance_id_to_start_timestamps(trace: &mut Trace) -> AttributeResult<()> {
    // The general idea is as follows:
    // 1. Keep track of all start instance ids and their timestamp
    //      So: HashMap<i64, Datetime>
    // 2. Delete the start event after adding it to the hashmap
    // 3. When complete event encountered, get its timestamp from the HashMap (or
    //    default to its own timestamp) and set the start timestamp
    let mut timestamps: HashMap<String, DateTime<FixedOffset>> = HashMap::new();

    // Use retain to remove the start timestamps, and take care of the start_timestamps
    // as a side-effect.
    retain_mut_err(&mut trace.events, |event| -> AttributeResult<bool> {
        let id = get_instance_id(event)?;
        match get_lifecycle(event)?.as_str() {
            "start" => {
                let timestamp = get_complete_timestamp(event)?;
                timestamps.insert(id, timestamp);
                Ok(false)
            }
            "complete" => {
                let start_timestamp = match timestamps.remove(&id) {
                    Some(t) => t,
                    None => get_complete_timestamp(event)?,
                };

                add_or_overwrite_attribute(
                    event,
                    START_TIMESTAMP_KEY,
                    AttributeValue::Date(start_timestamp),
                );

                Ok(true)
            }
            _ => {
                // Not start or complete: Drop
                Ok(false)
            }
        }
    })
}

/// Ensure that all events in the event log have a `start_timestamp` attribute and
/// a `complete_timestamp`` attribute. For the specifics of the strategy
/// used, see [fold_instance_id_to_start_timestamps].
///
/// In the end, only the events with a "complete" lifecycle transition remain, and
/// are enriched with the `start_timestamp` attribute based on the corresponding
/// "start" event, or their own completion timestamp (assumed to be atomic).
pub fn fold_instance_id_to_start_timestamps_log(log: &mut EventLog) -> AttributeResult<()> {
    log.traces
        .iter_mut()
        .try_for_each(fold_instance_id_to_start_timestamps)
}

/// Ideally, an event log used with the comparators contains a `start_timestamp`
/// and a `timestamp` for each event. However, if this is not the case, this function
/// can be used to infer start timestamps based on lifecycle information and (optionally)
/// instance ids.
///
/// Ensure that all events in the event log have a `start_timestamp` attribute and
/// a `complete_timestamp`` attribute. This is done following the following strategy:
///
/// 1. If each event has a start timestamp, there is nothing to do.
/// 2. If not all events have a lifecycle transition, it is inferred using [infer_lifecycle_information_log].
///     So, events are assumed to be atomic, each getting a "complete" transition.
/// 3. Then, if each event has an instance id, the instance id is used to match
///     start- and complete events, creating a single event for each pair with the timestamp
///     of the first as the start timestamp and the timestamp of the second as the completion
///     timestamp.`
/// 4. Otherwise, the instance id is first inferred from the lifecycle information in a FIFO
///     fashion, so the first started execution of a particular activity is the first
///     to complete. See [infer_event_instance_id_log].
///
/// Note that using this strategy, start events with no complete event are lost, and
/// if the instance id is inferred, any existing values are overwritten.
pub fn ensure_start_timestamp_key(log: &mut EventLog) -> AttributeResult<()> {
    let all_events_have_start_timestamp = log
        .traces
        .iter()
        .flat_map(|trace| trace.events.iter())
        .all(|evt| evt.attributes.get_by_key(START_TIMESTAMP_KEY).is_some());

    if all_events_have_start_timestamp {
        return Ok(());
    }

    let all_events_have_lifecycle = log
        .traces
        .iter()
        .flat_map(|trace| trace.events.iter())
        .all(|evt| evt.attributes.get_by_key(LIFECYCLE_KEY).is_some());
    if !all_events_have_lifecycle {
        infer_lifecycle_information_log(log);
    }

    let all_events_have_instance_id = log
        .traces
        .iter()
        .flat_map(|trace| trace.events.iter())
        .all(|evt| evt.attributes.get_by_key(INSTANCE_ID_KEY).is_some());

    if !all_events_have_instance_id {
        infer_event_instance_id_log(log)?;
    }

    fold_instance_id_to_start_timestamps_log(log)
}

#[cfg(test)]
mod tests {
    use crate::utils::{attributes::get_start_timestamp, constants::ACTIVITY_KEY};

    use super::*;
    use chrono::TimeDelta;
    use process_mining_macros::{event_log, trace};

    /// Convert activities like "a_start" to activity "a", lifecycle "start"
    fn helper_activity_to_lifecycle(trace: &mut Trace) {
        trace.events.iter_mut().for_each(|evt| {
            let name = get_activity_label(evt).unwrap();
            let (activity, lifecycle) = name.split_once("_").unwrap();

            add_or_overwrite_attribute(
                evt,
                ACTIVITY_KEY,
                AttributeValue::String(activity.to_string()),
            );
            add_or_overwrite_attribute(
                evt,
                LIFECYCLE_KEY,
                AttributeValue::String(lifecycle.to_string()),
            );
        });
    }

    /// Convert activities like "a_start" to activity "a", lifecycle "start"
    fn helper_activity_to_lifecycle_instance_id(trace: &mut Trace) {
        trace.events.iter_mut().for_each(|evt| {
            let name = get_activity_label(evt).unwrap();
            let (activity, rest) = name.split_once("_").unwrap();
            let (lifecycle, instance_id) = rest.split_once("_").unwrap();

            add_or_overwrite_attribute(
                evt,
                ACTIVITY_KEY,
                AttributeValue::String(activity.to_string()),
            );
            add_or_overwrite_attribute(
                evt,
                LIFECYCLE_KEY,
                AttributeValue::String(lifecycle.to_string()),
            );
            add_or_overwrite_attribute(
                evt,
                INSTANCE_ID_KEY,
                AttributeValue::String(instance_id.to_string()),
            );
        });
    }

    #[test]
    fn test_infer_lifecycle() {
        let mut trace = trace!(a, b, c, d);
        infer_lifecycle_information(&mut trace);

        assert_eq!(
            trace
                .events
                .into_iter()
                .map(|evt| (
                    get_activity_label(&evt).unwrap(),
                    get_lifecycle(&evt).unwrap()
                ))
                .collect::<Vec<_>>(),
            vec![
                ("a".to_string(), "complete".to_string()),
                ("b".to_string(), "complete".to_string()),
                ("c".to_string(), "complete".to_string()),
                ("d".to_string(), "complete".to_string()),
            ]
        );
    }

    #[test]
    fn test_ensure_start_timestamp_from_lifecycle() {
        let mut log = event_log!(
                [a_start, b_start, a_complete, c_start, c_complete, b_complete, d_complete],
                [a_start, b_start, a_start, b_complete, a_complete, a_complete]
                ; base_timestamp=EPOCH
        );
        log.traces.iter_mut().for_each(helper_activity_to_lifecycle);

        ensure_start_timestamp_key(&mut log).unwrap();
        let base = DateTime::UNIX_EPOCH.fixed_offset();

        let expected_1 = vec![
            (
                "a".to_string(),
                "complete".to_string(),
                base,
                base + TimeDelta::hours(2),
            ),
            (
                "c".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(3),
                base + TimeDelta::hours(4),
            ),
            (
                "b".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(1),
                base + TimeDelta::hours(5),
            ),
            (
                "d".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(6),
                base + TimeDelta::hours(6),
            ),
        ];
        assert_eq!(
            log.traces[0]
                .events
                .iter()
                .map(|evt| (
                    get_activity_label(evt).unwrap(),
                    get_lifecycle(evt).unwrap(),
                    get_start_timestamp(evt).unwrap(),
                    get_complete_timestamp(evt).unwrap(),
                ))
                .collect::<Vec<_>>(),
            expected_1
        );

        let expected_2 = vec![
            (
                "b".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(1),
                base + TimeDelta::hours(3),
            ),
            (
                "a".to_string(),
                "complete".to_string(),
                base,
                base + TimeDelta::hours(4),
            ),
            (
                "a".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(2),
                base + TimeDelta::hours(5),
            ),
        ];
        assert_eq!(
            log.traces[1]
                .events
                .iter()
                .map(|evt| (
                    get_activity_label(evt).unwrap(),
                    get_lifecycle(evt).unwrap(),
                    get_start_timestamp(evt).unwrap(),
                    get_complete_timestamp(evt).unwrap(),
                ))
                .collect::<Vec<_>>(),
            expected_2
        );
    }

    #[test]
    fn test_ensure_start_timestamp_from_instance_id() {
        let mut log = event_log!(
                [a_start_1, b_start_2, a_complete_1, c_start_3, c_complete_3, b_complete_2, d_complete_4],
                [a_start_1, b_start_2, a_start_3, b_complete_2, a_complete_3, a_complete_1]
                ; base_timestamp=EPOCH
        );
        log.traces
            .iter_mut()
            .for_each(helper_activity_to_lifecycle_instance_id);

        ensure_start_timestamp_key(&mut log).unwrap();
        let base = DateTime::UNIX_EPOCH.fixed_offset();

        let expected_1 = vec![
            (
                "a".to_string(),
                "complete".to_string(),
                base,
                base + TimeDelta::hours(2),
            ),
            (
                "c".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(3),
                base + TimeDelta::hours(4),
            ),
            (
                "b".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(1),
                base + TimeDelta::hours(5),
            ),
            (
                "d".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(6),
                base + TimeDelta::hours(6),
            ),
        ];
        assert_eq!(
            log.traces[0]
                .events
                .iter()
                .map(|evt| (
                    get_activity_label(evt).unwrap(),
                    get_lifecycle(evt).unwrap(),
                    get_start_timestamp(evt).unwrap(),
                    get_complete_timestamp(evt).unwrap(),
                ))
                .collect::<Vec<_>>(),
            expected_1
        );

        let expected_2 = vec![
            (
                "b".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(1),
                base + TimeDelta::hours(3),
            ),
            (
                "a".to_string(),
                "complete".to_string(),
                base + TimeDelta::hours(2),
                base + TimeDelta::hours(4),
            ),
            (
                "a".to_string(),
                "complete".to_string(),
                base,
                base + TimeDelta::hours(5),
            ),
        ];
        assert_eq!(
            log.traces[1]
                .events
                .iter()
                .map(|evt| (
                    get_activity_label(evt).unwrap(),
                    get_lifecycle(evt).unwrap(),
                    get_start_timestamp(evt).unwrap(),
                    get_complete_timestamp(evt).unwrap(),
                ))
                .collect::<Vec<_>>(),
            expected_2
        );
    }
}
