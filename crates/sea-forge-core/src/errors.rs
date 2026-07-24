use std::{error::Error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum ForgeError {
    Input(String),
    UnknownIntent(String),
    Config {
        class: &'static str,
        path: PathBuf,
        message: String,
    },
    UnsafePath(String),
    Io {
        context: String,
        source: io::Error,
    },
    Serialization(String),
    Plan {
        class: &'static str,
        message: String,
    },
    Internal(String),
    /// M9 (E11) self-model integrity failure: bundled model missing/drifted,
    /// DomainForge validation failure, or snapshot-source drift. Blast radius
    /// is self-model consumers and `ask`; runs and all other work are unaffected.
    SelfModel(String),
    Run {
        run_id: String,
        source: Box<ForgeError>,
    },
}

impl ForgeError {
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    pub fn class(&self) -> &'static str {
        match self {
            Self::Input(_) | Self::UnknownIntent(_) => "input_error",
            Self::Config { class, .. } => class,
            Self::UnsafePath(_) => "unsafe_path_error",
            Self::Io { .. } => "io_error",
            Self::Serialization(_) => "serialization_error",
            Self::Plan { class, .. } => class,
            Self::Internal(_) | Self::Run { .. } => "internal_error",
            Self::SelfModel(_) => "self_model_error",
        }
    }

    pub fn run(run_id: impl Into<String>, source: Self) -> Self {
        Self::Run {
            run_id: run_id.into(),
            source: Box::new(source),
        }
    }

    pub fn run_id(&self) -> Option<&str> {
        match self {
            Self::Run { run_id, .. } => Some(run_id),
            _ => None,
        }
    }
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message)
            | Self::UnknownIntent(message)
            | Self::UnsafePath(message)
            | Self::Serialization(message)
            | Self::Internal(message)
            | Self::SelfModel(message) => f.write_str(message),
            Self::Plan { class, message } => write!(f, "{class}: {message}"),
            Self::Config {
                class,
                path,
                message,
            } => {
                write!(f, "{class}: {}: {message}", path.display())
            }
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::Run { source, .. } => source.fmt(f),
        }
    }
}

impl Error for ForgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Run { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ForgeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}
