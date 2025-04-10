pub mod attribute_error;

use attribute_error::{AttributeError, AttributeErrorKind, AttributeLevel, AttributeResult};
use chrono::{DateTime, FixedOffset};
use process_mining::event_log::{
    Attribute, AttributeValue, Attributes, Event, Trace, XESEditableAttribute,
};

use crate::utils::constants::{ACTIVITY_KEY, LIFECYCLE_KEY, START_TIMESTAMP_KEY, TIMESTAMP_KEY};

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

    fn attribute_level(&self) -> AttributeLevel {
        Self::ATTRIBUTE_LEVEL
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

pub fn add_or_overwrite_attribute(on: &mut impl HasAttributes, key: &str, value: AttributeValue) {
    if let Some(attr) = on.get_attributes_mut().get_by_key_mut(key) {
        attr.value = value
    } else {
        on.get_attributes_mut()
            .add_attribute(Attribute::new(key.to_string(), value));
    }
}

pub fn get_time_by_key(
    from: &impl HasAttributes,
    key: &str,
) -> AttributeResult<DateTime<FixedOffset>> {
    let attribute = from.get_attribute_by_key(key)?;
    attribute
        .value
        .try_as_date()
        .ok_or(AttributeError::new(
            from.attribute_level(),
            key,
            AttributeErrorKind::TypeMismatch("DateTime".to_string(), attribute.value.clone()),
        ))
        .copied()
}

pub fn get_string_by_key(from: &impl HasAttributes, key: &str) -> AttributeResult<String> {
    let attribute = from.get_attribute_by_key(key)?;
    attribute
        .value
        .try_as_string()
        .ok_or(AttributeError::new(
            from.attribute_level(),
            key,
            AttributeErrorKind::TypeMismatch("String".to_string(), attribute.value.clone()),
        ))
        .cloned()
}

pub fn get_int_by_key(from: &impl HasAttributes, key: &str) -> AttributeResult<i64> {
    let attribute = from.get_attribute_by_key(key)?;
    attribute
        .value
        .try_as_int()
        .ok_or(AttributeError::new(
            from.attribute_level(),
            key,
            AttributeErrorKind::TypeMismatch("Int".to_string(), attribute.value.clone()),
        ))
        .copied()
}

pub fn get_activity_label(event: &Event) -> AttributeResult<String> {
    get_string_by_key(event, ACTIVITY_KEY)
}

pub fn get_start_timestamp(event: &Event) -> AttributeResult<DateTime<FixedOffset>> {
    get_time_by_key(event, START_TIMESTAMP_KEY)
}

pub fn get_complete_timestamp(event: &Event) -> AttributeResult<DateTime<FixedOffset>> {
    get_time_by_key(event, TIMESTAMP_KEY)
}

pub fn get_service_time(event: &Event) -> AttributeResult<chrono::TimeDelta> {
    let start = get_start_timestamp(event)?;
    let end = get_complete_timestamp(event)?;
    Ok(end - start)
}

pub fn get_lifecycle(event: &Event) -> AttributeResult<String> {
    get_string_by_key(event, LIFECYCLE_KEY)
}
