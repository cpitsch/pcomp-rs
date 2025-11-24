pub mod attribute_error;

use attribute_error::{AttributeError, AttributeErrorKind, AttributeLevel, AttributeResult};
use chrono::{DateTime, FixedOffset};
use process_mining::event_log::{
    Attribute, AttributeValue, Attributes, Event, Trace, XESEditableAttribute,
};

use crate::utils::constants::{
    ACTIVITY_KEY, INSTANCE_ID_KEY, LIFECYCLE_KEY, START_TIMESTAMP_KEY, TIMESTAMP_KEY,
};

/// Helper-trait for a unified interface to attributes.
pub trait HasAttributes {
    const ATTRIBUTE_LEVEL: AttributeLevel;

    fn get_attributes(&self) -> &Attributes;

    fn get_attributes_mut(&mut self) -> &mut Attributes;

    fn get_attribute_by_key(&self, key: &str) -> AttributeResult<&Attribute> {
        self.get_attributes()
            .get_by_key(key)
            .ok_or(AttributeError::new(
                Self::ATTRIBUTE_LEVEL,
                key,
                AttributeErrorKind::MissingAttribute,
            ))
    }

    fn get_string_by_key(&self, key: &str) -> AttributeResult<String> {
        let attribute = self.get_attribute_by_key(key)?;
        attribute.value.try_as_string().cloned().ok_or_else(|| {
            AttributeError::new(
                Self::ATTRIBUTE_LEVEL,
                key,
                AttributeErrorKind::TypeMismatch("String".to_string(), attribute.value.clone()),
            )
        })
    }

    fn get_time_by_key(&self, key: &str) -> AttributeResult<DateTime<FixedOffset>> {
        let attribute = self.get_attribute_by_key(key)?;
        attribute.value.try_as_date().copied().ok_or_else(|| {
            AttributeError::new(
                Self::ATTRIBUTE_LEVEL,
                key,
                AttributeErrorKind::TypeMismatch("Date".to_string(), attribute.value.clone()),
            )
        })
    }

    fn get_int_by_key(&self, key: &str) -> AttributeResult<i64> {
        let attribute = self.get_attribute_by_key(key)?;
        attribute.value.try_as_int().copied().ok_or_else(|| {
            AttributeError::new(
                Self::ATTRIBUTE_LEVEL,
                key,
                AttributeErrorKind::TypeMismatch("Int".to_string(), attribute.value.clone()),
            )
        })
    }

    fn get_float_by_key(&self, key: &str) -> AttributeResult<f64> {
        let attribute = self.get_attribute_by_key(key)?;
        attribute.value.try_as_float().copied().ok_or_else(|| {
            AttributeError::new(
                Self::ATTRIBUTE_LEVEL,
                key,
                AttributeErrorKind::TypeMismatch("Float".to_string(), attribute.value.clone()),
            )
        })
    }
}

impl HasAttributes for Trace {
    const ATTRIBUTE_LEVEL: AttributeLevel = AttributeLevel::Trace;

    fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn get_attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }
}

impl HasAttributes for Event {
    const ATTRIBUTE_LEVEL: AttributeLevel = AttributeLevel::Event;

    fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn get_attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }
}

/// Add an attribute, or overwrite it if it already exists.
pub fn add_or_overwrite_attribute(on: &mut impl HasAttributes, key: &str, value: AttributeValue) {
    if let Some(attr) = on.get_attributes_mut().get_by_key_mut(key) {
        attr.value = value
    } else {
        on.get_attributes_mut()
            .add_attribute(Attribute::new(key.to_string(), value));
    }
}

/// Get the activity of an event.
///
/// Returns an `Err` if the `concept:name` attribute does not exist or is not a string.
pub fn get_activity_label(event: &Event) -> AttributeResult<String> {
    event.get_string_by_key(ACTIVITY_KEY)
}

/// Get the start timestamp of an event.
///
/// Returns an `Err` if the `start_timestamp` attribute does not exist or is not a [`DateTime`].
///
pub fn get_start_timestamp(event: &Event) -> AttributeResult<DateTime<FixedOffset>> {
    event.get_time_by_key(START_TIMESTAMP_KEY)
}

/// Get the complete timestamp of an event.
///
/// Returns an `Err` if the `time:timestamp` attribute does not exist or is not a [`DateTime`].
pub fn get_complete_timestamp(event: &Event) -> AttributeResult<DateTime<FixedOffset>> {
    event.get_time_by_key(TIMESTAMP_KEY)
}

/// Get the service timestamp of an event.
///
/// Returns an `Err` if:
///
/// - The `time:timestamp` attribute does not exist or is not a [`DateTime`].
/// - The `start_timestamp` attribute does not exist or is not a [`DateTime`].
pub fn get_service_time(event: &Event) -> AttributeResult<chrono::TimeDelta> {
    let start = get_start_timestamp(event)?;
    let end = get_complete_timestamp(event)?;
    Ok(end - start)
}

/// Get the lifecycle transition of an event.
///
/// Returns an `Err` if the `lifecycle:transition` attribute does not exist or is
/// not a string.
pub fn get_lifecycle(event: &Event) -> AttributeResult<String> {
    event.get_string_by_key(LIFECYCLE_KEY)
}

/// Get the instance ID of the event.
///
/// Returns an `Err` if the `concept:instance` attribute does not exist or is not a string.
pub fn get_instance_id(event: &Event) -> AttributeResult<String> {
    event.get_string_by_key(INSTANCE_ID_KEY)
}
