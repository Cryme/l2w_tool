use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum StringCow {
    Owned(String),
    Borrowed(Arc<String>),
}

impl PartialEq for StringCow {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Display for StringCow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl StringCow {
    pub fn as_mut_string(&mut self) -> &mut String {
        match self {
            StringCow::Owned(v) => v,
            StringCow::Borrowed(v) => {
                *self = Self::Owned(v.to_string());

                self.as_mut_string()
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            StringCow::Owned(v) => v,
            StringCow::Borrowed(v) => v,
        }
    }
}

impl Default for StringCow {
    fn default() -> Self {
        Self::Borrowed(Arc::new(String::default()))
    }
}

impl From<&str> for StringCow {
    fn from(value: &str) -> Self {
        StringCow::Owned(value.to_string())
    }
}

impl From<&String> for StringCow {
    fn from(value: &String) -> Self {
        StringCow::Owned(value.clone())
    }
}

impl From<String> for StringCow {
    fn from(value: String) -> Self {
        StringCow::Owned(value)
    }
}

impl From<&Arc<String>> for StringCow {
    fn from(value: &Arc<String>) -> Self {
        StringCow::Borrowed(value.clone())
    }
}

impl From<Arc<String>> for StringCow {
    fn from(value: Arc<String>) -> Self {
        StringCow::Borrowed(value)
    }
}

/**
range_str: &str in form u32-32 or u32 (`11-34` or `11`)
*/
pub fn is_in_range(range_str: &str, val: u32) -> bool {
    let range: Vec<_> = range_str.split('-').collect();

    match range.len() {
        2 => {
            let Ok(min) = u32::from_str(range[0]) else {
                return false;
            };

            let Ok(max) = u32::from_str(range[1]) else {
                return false;
            };

            min <= val && val <= max
        }
        1 => {
            let Ok(min) = u32::from_str(range[0]) else {
                return false;
            };

            min <= val
        }
        _ => false,
    }
}
