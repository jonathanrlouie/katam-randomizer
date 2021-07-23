use crate::common::{NonUniqueSettingsError, Settings};

pub trait Config {
    fn get_settings(&self) -> Result<Settings, NonUniqueSettingsError>;
    fn get_seed(&self) -> u64;
}

pub struct KatamConfig;

impl KatamConfig {
    pub fn load_config() -> anyhow::Result<Self> {
        unimplemented!()
    }
}

impl Config for KatamConfig {
    fn get_settings(&self) -> Result<Settings, NonUniqueSettingsError> {
        unimplemented!()
    }

    fn get_seed(&self) -> u64 {
        unimplemented!()
    }
}
