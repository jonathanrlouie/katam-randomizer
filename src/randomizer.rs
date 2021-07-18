use crate::{common::{StringID, WriteData, EntranceData, EntranceShuffleType, Setting}, config::Config, rng::RNG, rom_writer::RomWriter};
use validated::Validated::{self, Good, Fail};

pub fn randomize_game(config: impl Config, mut rng: impl RNG, mut rom: impl RomWriter) -> anyhow::Result<()> {
    let settings = config.get_settings()?;
    let validated_write_data: Validated<Vec<WriteData>, anyhow::Error> = settings.into_iter().map(|setting|
        match setting {
            Setting::EntranceShuffle{ty, data} => shuffle_entrances(rng, ty, data)
        }
    ).collect();
    let write_data = validated_write_data.to_result()?;
    rom.write_data(&write_data)?;
    Ok(())
}

fn shuffle_entrances(rng: impl RNG, ty: EntranceShuffleType, entrance_data: EntranceData) -> Validated<WriteData, anyhow::Error> {
    Good(vec![])
}
