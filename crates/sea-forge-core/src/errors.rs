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
    Internal(String),
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
            Self::Internal(_) => "internal_error",
            Self::Run { .. } => "internal_error",
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
            | Self::Internal(message) => f.write_str(message),
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
