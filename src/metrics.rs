// metrics data structure
// fundamental function : inc/dec/snapshot

use anyhow::Result;
use dashmap::DashMap;
use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;
#[derive(Debug, Clone)]
pub struct Metrics {
    data: Arc<DashMap<String, i64>>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
impl Metrics {
    pub fn new() -> Self {
        Metrics {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut value = self.data.entry(key.into()).or_insert(0);
        *value += 1;
        Ok(())
    }

    pub fn dec(&self, key: impl Into<String>) -> Result<()> {
        let mut value = self.data.entry(key.into()).or_insert(0);
        *value -= 1;
        Ok(())
    }
}

impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for entry in self.data.iter() {
            writeln!(f, "{}:{}", entry.key(), entry.value())?;
        }
        Ok(())
    }
}
