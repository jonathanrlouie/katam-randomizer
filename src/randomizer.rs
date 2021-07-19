use crate::{
    common::{EntranceData, EntranceShuffleType, IntoResult, Setting, StringID, WriteData},
    config::Config,
    rng::RNG,
    rom_writer::RomWriter,
};
use itertools::Itertools;
use std::fmt;
use thiserror::Error;
use validated::Validated::{self, Fail, Good};

#[derive(Error, Debug)]
enum RandomizerError {
    #[error("Entrance shuffle error: Invalid game")]
    EntranceShuffleError,
}

#[derive(Error, Debug)]
struct RandomizerErrors {
    errors: Vec<RandomizerError>,
}

impl fmt::Display for RandomizerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined: String = self.errors.iter().map(|err| err.to_string()).join("\n");
        write!(f, "{}", joined)
    }
}

impl IntoResult<Vec<WriteData>, RandomizerErrors> for Validated<Vec<WriteData>, RandomizerError> {
    fn into_result(self) -> Result<Vec<WriteData>, RandomizerErrors> {
        match self {
            Good(wd) => Ok(wd),
            Fail(errs) => Err(RandomizerErrors {
                errors: errs.into(),
            }),
        }
    }
}

pub fn randomize_game(
    config: impl Config,
    mut rng: impl RNG,
    mut rom: impl RomWriter,
) -> anyhow::Result<()> {
    use Setting::EntranceShuffle;
    let settings = config.get_settings()?;
    let validated_write_data = settings
        .into_iter()
        .map(|setting| match setting {
            EntranceShuffle { ty, data } => shuffle_entrances(&mut rng, ty, data),
        })
        .collect::<Validated<Vec<WriteData>, RandomizerError>>();
    let write_data = validated_write_data.into_result()?;
    rom.write_data(&write_data)?;
    Ok(())
}

fn shuffle_entrances(
    rng: &mut impl RNG,
    ty: EntranceShuffleType,
    entrance_data: EntranceData,
) -> Validated<WriteData, RandomizerError> {
    Good(WriteData {
        bytes: vec![],
        target_addresses: vec![],
    })
}
