use std::error::Error;

pub struct TelemetryDb;

impl TelemetryDb {
    pub fn new(_path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self)
    }
}
