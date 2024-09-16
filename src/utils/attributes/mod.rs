use chrono::{DateTime, Utc};
use process_mining::event_log::{Attributes, Event, Trace, XESEditableAttribute};

use crate::utils::constants::{ACTIVITY_KEY, START_TIMESTAMP_KEY, TIMESTAMP_KEY};

pub trait HasAttributes {
    fn get_attributes(&self) -> &Attributes;
}

impl HasAttributes for Trace {
    fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl HasAttributes for Event {
    fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }
}

pub fn get_time_by_key(from: &impl HasAttributes, key: &str) -> Option<DateTime<Utc>> {
    from.get_attributes()
        .get_by_key(key)?
        .value
        .try_as_date()
        .cloned()
}

pub fn get_string_by_key(from: &impl HasAttributes, key: &str) -> Option<String> {
    from.get_attributes()
        .get_by_key(key)?
        .value
        .try_as_string()
        .cloned()
}
pub fn get_activity_label(event: &Event) -> Option<String> {
    get_string_by_key(event, ACTIVITY_KEY)
}

pub fn get_start_timestamp(event: &Event) -> Option<DateTime<Utc>> {
    get_time_by_key(event, START_TIMESTAMP_KEY)
}

pub fn get_complete_timestamp(event: &Event) -> Option<DateTime<Utc>> {
    get_time_by_key(event, TIMESTAMP_KEY)
}

pub fn get_service_time(event: &Event) -> Option<chrono::TimeDelta> {
    let start = get_start_timestamp(event)?;
    let end = get_complete_timestamp(event)?;
    Some(end - start)
}
