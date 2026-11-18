use std::fmt;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum TimerError {
    ZeroDuration,
    ExceedsMaximumDuration {
        max: Duration,
        requested: Duration,
    },
    InvalidFormat(String),
    InvalidStateTransition {
        from: String,
        action: &'static str,
    },
    ChannelDisconnected,
}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimerError::ZeroDuration => {
                write!(f, "Initial duration must be strictly greater than zero.")
            }
            TimerError::ExceedsMaximumDuration { max, requested } => {
                write!(
                    f,
                    "Requested duration ({:?}) exceeds maximum allowed duration ({:?}).",
                    requested, max
                )
            }
            TimerError::InvalidFormat(msg) => write!(f, "Invalid time format: {}", msg),
            TimerError::InvalidStateTransition { from, action } => {
                write!(f, "Cannot perform action '{}' while timer is in state '{}'.", action, from)
            }
            TimerError::ChannelDisconnected => write!(f, "Timer communication channel disconnected."),
        }
    }
}

impl std::error::Error for TimerError {}
