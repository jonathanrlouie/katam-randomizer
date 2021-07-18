use std::collections::HashSet;
use crate::randomizer::Randomizer;
use anyhow;

pub struct KatamConfig;

pub trait Config {
    fn load_config() -> anyhow::Result<Self> where Self: Sized;
    fn get_randomizer(&self) -> Randomizer;
}

impl Config for KatamConfig {
    fn load_config() -> anyhow::Result<KatamConfig> {
        Ok(KatamConfig)
    }

    // Return multiple randomizers so they can be composed
    fn get_randomizers(&self) -> HashSet<Randomizer> {
        HashSet::new(Randomizer::Entrance)
    }
}
