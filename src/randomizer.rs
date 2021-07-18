use crate::{common::StringID, error};
use validated::Validated::{self, Good, Fail};

pub struct EntranceData;
pub enum EntranceShuffleType {
    // two-way doors are truly two-way; one-way doors lead to one-way exits
    Standard,
    // two-way doors behave like one-way doors; every door can lead to any exit
    Chaos,
}

pub struct EnemyData;

pub enum Setting {
    EntranceShuffle{ty: EntranceShuffleType, data: EntranceData},
    Enemizer(EnemyData),
    Cosmetic,
}

struct WriteData {
    bytes: Vec<u8>,
    target_addresses: Vec<usize>
}

pub fn randomize_game(impl Config, mut rom: impl RomWriter) -> error::Result<()> {
    let validated_write_data: Validated<WriteData, error::Error> = config.get_settings().into_iter().map(|setting|
        match *setting {
            Setting::EntranceShuffle{ty, entrance_data} => shuffle_entrances(ty, entrance_data),
            Setting::Enemizer(enemy_data) => unimplemented!(),
            Setting::Cosmetic => unimplemented!(),
        }
    );
    let write_data = validated_write_data.to_result()?;
    rom.randomize(&write_data.bytes, &write_data.target_addresses)?;
}

fn shuffle_entrances(ty: EntranceShuffleType, entrance_data: EntranceData) -> Validated<WriteData, error::Error> {
    Good(vec![])
}
