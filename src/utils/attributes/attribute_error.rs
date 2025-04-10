use process_mining::event_log::AttributeValue;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum AttributeLevel {
    Event,
    Trace,
    Log,
}

impl std::fmt::Display for AttributeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_str = match self {
            AttributeLevel::Event => "Event",
            AttributeLevel::Trace => "Trace",
            AttributeLevel::Log => "Log",
        };
        write!(f, "{}", level_str)
    }
}

#[derive(Debug, Clone, Error)]
pub enum AttributeErrorKind {
    #[error("not found")]
    MissingAttribute,
    #[error("has unexpected type. Expected {0}, found {1:?}")]
    TypeMismatch(String, AttributeValue),
}

#[derive(Debug, Clone, Error)]
#[error("{level}-level attribute \"{key}\" {kind}.")]
pub struct AttributeError {
    pub level: AttributeLevel,
    pub key: String,
    pub kind: AttributeErrorKind,
}

impl AttributeError {
    pub fn new(level: AttributeLevel, key: impl Into<String>, kind: AttributeErrorKind) -> Self {
        Self {
            level,
            kind,
            key: key.into(),
        }
    }
}

pub type AttributeResult<T> = Result<T, AttributeError>;
