use anyhow;
use crate::common::{StringID, DoorDataMaps};

pub trait RomWriter {
    fn write_randomized_rom(&mut self, shuffled_ids: Vec<(StringID, StringID)>, door_data_maps: DoorDataMaps) -> anyhow::Result<()>;
}
