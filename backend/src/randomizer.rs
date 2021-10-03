use crate::{
    common::{IntoResult, StringID, WriteData},
    config::{Config, EntranceData, EntranceShuffleType},
    rng::RNG,
    rom_writer::RomWriter,
};
use itertools::Itertools;
use std::fmt;
use thiserror::Error;
use validated::Validated::{self, Fail, Good};

pub fn randomize_game(
    config: Config,
    mut rng: impl RNG,
    mut rom: impl RomWriter,
) -> anyhow::Result<()> {
    //let write_data = unimplemented!();
    //rom.write_data(&write_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const test_config: Config = Config {
        seed: 0,
        entrance_shuffle: EntranceShuffleType::Standard,
    };

    struct MockRng;

    impl RNG for MockRng {
        fn get_bool(&mut self, p: f64) -> bool {
            true
        }
    }

    struct MockRomWriter;

    impl RomWriter for MockRomWriter {
        fn write_data(&mut self, data: &[WriteData]) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_randomize_game() -> anyhow::Result<()> {
        randomize_game(test_config, MockRng, MockRomWriter)
    }
}
