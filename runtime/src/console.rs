use lazy_static::lazy_static;
use std::{collections::VecDeque, sync::Mutex};

#[derive(Debug, Clone)]
pub struct LuaError {
    // Allows for clickable links to the file / showing the line
    pub message: String,
    pub file: String,
    pub line: usize,
    pub repeat_count: u32,
}

pub struct RepeatableMessage {
    pub message: String,
    pub repeat_count: u32,
}

impl std::fmt::Display for RepeatableMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.repeat_count > 1 {
            write!(f, "({}x) ", self.repeat_count)?;
        }
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Display for LuaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.repeat_count > 1 {
            write!(f, "({}x) ", self.repeat_count)?;
        }
        write!(f, "{}", self.message)
    }
}

pub enum ConsoleMessage {
    Info(RepeatableMessage),
    Warning(RepeatableMessage),
    Error(RepeatableMessage),
    LuaError(LuaError),
}

impl std::fmt::Display for ConsoleMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleMessage::Info(info) => write!(f, "{}", info),
            ConsoleMessage::Warning(warning) => write!(f, "{}", warning),
            ConsoleMessage::Error(error) => write!(f, "{}", error),
            ConsoleMessage::LuaError(err) => write!(f, "{}", err),
        }
    }
}

impl RepeatableMessage {
    fn new(message: String) -> Self {
        Self {
            message,
            repeat_count: 1,
        }
    }
}

impl ConsoleMessage {
    pub fn message(&self) -> &str {
        match self {
            ConsoleMessage::Info(info) => &info.message,
            ConsoleMessage::Warning(warning) => &warning.message,
            ConsoleMessage::Error(error) => &error.message,
            ConsoleMessage::LuaError(err) => &err.message,
        }
    }
    pub fn repeat_count(&self) -> u32 {
        match self {
            ConsoleMessage::Info(info) => info.repeat_count,
            ConsoleMessage::Warning(warning) => warning.repeat_count,
            ConsoleMessage::Error(error) => error.repeat_count,
            ConsoleMessage::LuaError(err) => err.repeat_count,
        }
    }
    pub fn is_same_kind(&self, other: &ConsoleMessage) -> bool {
        matches!(
            (self, other),
            (ConsoleMessage::Info(_), ConsoleMessage::Info(_))
                | (ConsoleMessage::Warning(_), ConsoleMessage::Warning(_))
                | (ConsoleMessage::Error(_), ConsoleMessage::Error(_))
                | (ConsoleMessage::LuaError(_), ConsoleMessage::LuaError(_))
        )
    }
}

pub struct Logger {
    messages: VecDeque<ConsoleMessage>,
    frame_messages: VecDeque<String>,
}

pub const MAX_LOGS_COUNT: usize = 300;

impl Logger {
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            frame_messages: VecDeque::new(),
        }
    }

    fn add_message_without_repeat(
        &mut self,
        message: ConsoleMessage,
        repeat_candidate_index: usize, // trick to avoid double mutable borrow
    ) {
        let repeat_candidate = &mut self.messages[repeat_candidate_index];
        match (&message, repeat_candidate) {
            (ConsoleMessage::LuaError(message), ConsoleMessage::LuaError(candidate)) => {
                if message.file == candidate.file && message.line == candidate.line {
                    candidate.repeat_count += 1;
                    return;
                }
            }
            (ConsoleMessage::Info(info), ConsoleMessage::Info(candidate)) => {
                if info.message == candidate.message {
                    candidate.repeat_count += 1;
                    return;
                }
            }
            (ConsoleMessage::Warning(warning), ConsoleMessage::Warning(candidate)) => {
                if warning.message == candidate.message {
                    candidate.repeat_count += 1;
                    return;
                }
            }
            (ConsoleMessage::Error(error), ConsoleMessage::Error(candidate)) => {
                if error.message == candidate.message {
                    candidate.repeat_count += 1;
                    return;
                }
            }
            _ => {}
        }
        self.messages.push_back(message);
    }

    fn log(&mut self, message: ConsoleMessage) {
        let last_log = self
            .messages
            .iter()
            .enumerate()
            .rev()
            .find(|m| m.1.is_same_kind(&message));
        if let Some((index, _)) = last_log {
            self.add_message_without_repeat(message, index);
        } else {
            self.messages.push_back(message);
        }
    }

    fn log_frame(&mut self, msg: String) {
        self.frame_messages.push_back(msg);
        self.trim_frame();
    }

    fn log_info(&mut self, msg: String) {
        self.log(ConsoleMessage::Info(RepeatableMessage::new(msg)));
        self.trim();
    }
    fn log_warning(&mut self, msg: String) {
        self.log(ConsoleMessage::Warning(RepeatableMessage::new(msg)));
        self.trim();
    }
    fn log_error(&mut self, msg: String) {
        self.log(ConsoleMessage::Error(RepeatableMessage::new(msg)));
        self.trim();
    }
    fn log_lua_error(&mut self, message: String, file: String, line: usize) {
        self.log(ConsoleMessage::LuaError(LuaError {
            message,
            file,
            line,
            repeat_count: 1,
        }));
        self.trim();
    }

    fn trim(&mut self) {
        while self.messages.len() > MAX_LOGS_COUNT {
            self.messages.pop_front();
        }
    }
    fn trim_frame(&mut self) {
        while self.frame_messages.len() > MAX_LOGS_COUNT {
            self.frame_messages.pop_front();
        }
    }
}

lazy_static! {
    static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

/// Print an error to the editor console, or the console, or does nothing, depending on the platform and
/// the configuration.
pub fn print_err(msg: String) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log_error(msg);
    }
}

/// Print a warning to the editor console, or the console, or does nothing, depending on the platform and
/// the configuration.
pub fn print_warn(msg: String) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log_warning(msg);
    }
}

/// Print an information to the editor console, or the console, or does nothing, depending on the platform and
/// the configuration.
pub fn print_info(msg: String) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log_info(msg);
    }
}

pub fn print_lua_error(msg: String, file: String, line: usize) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log_lua_error(msg, file, line);
    }
}

pub fn print_frame(msg: String) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log_frame(msg);
    }
}

pub fn get_logs<F>(f: F)
where
    F: FnMut(&ConsoleMessage),
{
    let Ok(logger) = LOGGER.lock() else {
        return;
    };
    logger.messages.iter().for_each(f)
}

pub fn consume_logs<F>(f: F)
where
    F: FnMut(ConsoleMessage),
{
    let Ok(mut logger) = LOGGER.lock() else {
        return;
    };
    logger.messages.drain(..).for_each(f)
}

#[deprecated]
/// Use consume_frame_logs instead
pub fn get_frame_logs<F>(mut f: F)
where
    F: FnMut(&str),
{
    let Ok(logger) = LOGGER.lock() else {
        return;
    };
    logger.frame_messages.iter().for_each(|m| f(m))
}

pub fn consume_frame_logs<F>(f: F)
where
    F: FnMut(String),
{
    let Ok(mut logger) = LOGGER.lock() else {
        return;
    };
    logger.frame_messages.drain(..).for_each(f)
}

pub fn clear_all_logs() {
    let Ok(mut logger) = LOGGER.lock() else {
        return;
    };
    logger.messages.clear();
    logger.frame_messages.clear();
}
