use crate::{common::{StringID, WriteData, EntranceData, EntranceShuffleType, Setting}, error, rng::RNG};
use validated::Validated::{self, Good, Fail};

pub fn randomize_game(config: impl Config, mut rng: impl RNG, mut rom: impl RomWriter) -> error::Result<()> {
    let validated_write_data: Validated<Vec<WriteData>, error::Error> = config.get_settings().into_iter().map(|setting|
        match *setting {
            Setting::EntranceShuffle{ty, entrance_data} => shuffle_entrances(rng, ty, entrance_data)
        }
    );
    let write_data = validated_write_data.to_result()?;
    rom.randomize(&write_data.bytes, &write_data.target_addresses)?;
}

fn shuffle_entrances(rng: impl RNG, ty: EntranceShuffleType, entrance_data: EntranceData) -> Validated<WriteData, error::Error> {
    Good(vec![])
}
