use spin_sdk::http::Request;
use std::fmt;

/// Generate or extract trace ID from request
pub fn get_trace_id(req: &Request, header_name: &str) -> String {
    req.headers()
        .find(|(name, _)| name.eq_ignore_ascii_case(header_name))
        .and_then(|(_, value)| value.as_str())
        .map_or_else(
            || {
                // Generate a simple trace ID if not provided
                // Note: std::process::id() is not available in WASI
                format!(
                    "gen-{:x}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0)
                )
            },
            String::from,
        )
}

/// Structured log entry
pub struct LogEntry<'a> {
    trace_id: &'a str,
    level: LogLevel,
    message: String,
    fields: Vec<(&'static str, String)>,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    #[allow(dead_code)]
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

impl<'a> LogEntry<'a> {
    pub fn new(trace_id: &'a str, level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            trace_id,
            level,
            message: message.into(),
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, key: &'static str, value: impl fmt::Display) -> Self {
        self.fields.push((key, value.to_string()));
        self
    }

    pub fn emit(self) {
        let mut output = format!(
            "[{}] trace_id={} {}",
            self.level, self.trace_id, self.message
        );

        for (key, value) in self.fields {
            use std::fmt::Write;
            let _ = write!(&mut output, " {key}={value}");
        }

        eprintln!("{output}");
    }
}

/// Logger with trace ID context
pub struct Logger<'a> {
    trace_id: &'a str,
}

impl<'a> Logger<'a> {
    pub fn new(trace_id: &'a str) -> Self {
        Self { trace_id }
    }

    #[allow(dead_code)]
    pub fn debug(&self, message: impl Into<String>) -> LogEntry<'a> {
        LogEntry::new(self.trace_id, LogLevel::Debug, message)
    }

    pub fn info(&self, message: impl Into<String>) -> LogEntry<'a> {
        LogEntry::new(self.trace_id, LogLevel::Info, message)
    }

    pub fn warn(&self, message: impl Into<String>) -> LogEntry<'a> {
        LogEntry::new(self.trace_id, LogLevel::Warn, message)
    }

    pub fn error(&self, message: impl Into<String>) -> LogEntry<'a> {
        LogEntry::new(self.trace_id, LogLevel::Error, message)
    }
}
