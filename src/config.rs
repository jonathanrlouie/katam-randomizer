use crate::common::{NonUniqueSettingsError, Settings};

pub trait Config {
    fn get_settings(&self) -> Result<Settings, NonUniqueSettingsError>;
    fn get_seed(&self) -> u64;
}

pub struct KatamConfig;

impl KatamConfig {
    pub fn load_config() -> anyhow::Result<Self> {
        Ok(KatamConfig)
    }
}

impl Config for KatamConfig {
    fn get_settings(&self) -> Result<Settings, NonUniqueSettingsError> {
        Settings::new(Vec::new())
    }

    fn get_seed(&self) -> u64 {
        0
    }
}
