use crate::logs_mut;
use std::collections::HashSet;
use strum_macros::{Display, EnumIter};

#[derive(Debug)]
pub struct LogHolder {
    pub(crate) producers: HashSet<String>,
    pub(crate) max_level: LogLevel,
    pub(crate) synchronized: bool,

    pub(crate) logs: Vec<Log>,
}

impl LogHolder {
    pub const ALL: &'static str = "All";

    pub(crate) fn new() -> Self {
        let mut producers = HashSet::new();
        producers.insert(Self::ALL.to_string());

        Self {
            producers,
            max_level: LogLevel::Info,
            synchronized: false,
            logs: vec![],
        }
    }

    pub fn reset(&mut self, logs: Vec<Log>) {
        let mut producers = HashSet::new();
        producers.insert(Self::ALL.to_string());

        let mut max_level = LogLevel::Info;

        for v in &logs {
            producers.insert(v.producer.clone());
            max_level = v.level.max(max_level);
        }

        *self = LogHolder {
            producers,
            max_level,
            synchronized: false,
            logs,
        };
    }

    pub fn add(&mut self, log: Log) {
        self.producers.insert(log.producer.clone());
        self.max_level = log.level.max(self.max_level);
        self.logs.push(log);
        self.synchronized = false;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display, EnumIter)]
#[repr(u8)]
pub enum LogLevelFilter {
    Info,
    Warning,
    Error,
    All = 255,
}

#[derive(Clone, Debug)]
pub struct Log {
    pub level: LogLevel,
    pub producer: String,
    pub log: String,
}

pub struct LogHolderParams {
    pub(crate) producer_filter: String,
    pub(crate) producers: HashSet<String>,
    pub(crate) max_log_level: LogLevel,
    pub(crate) level_filter: LogLevelFilter,
}

impl Default for LogHolderParams {
    fn default() -> Self {
        LogHolderParams {
            producer_filter: LogHolder::ALL.to_string(),
            producers: logs_mut().producers.clone(),
            max_log_level: LogLevel::Info,
            level_filter: LogLevelFilter::All,
        }
    }
}

impl LogHolderParams {
    pub(crate) fn sync(&mut self) {
        let mut v = logs_mut();

        if v.synchronized {
            return;
        }

        self.producers.clone_from(&v.producers);
        self.max_log_level = v.max_level;

        v.synchronized = true;

        drop(v);

        if !self.producers.contains(&self.producer_filter) {
            self.producer_filter = LogHolder::ALL.to_string();
        }
    }
}
