use std::collections::HashSet;
use crate::{common::Setting, error::Result};
use anyhow;

pub trait Config {
    fn get_settings(&self) -> anyhow::Result<Settings>;
}


pub struct KatamConfig;

impl KatamConfig {
    fn load_config() -> anyhow::Result<Self> {
        Ok(KatamConfig)
    }
}

impl Config for KatamConfig {

    fn get_settings(&self) -> Settings {
        Settings::new(&[Setting::Cosmetic])
    }
}
