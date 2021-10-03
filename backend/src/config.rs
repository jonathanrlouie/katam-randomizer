use rocket::form::FromFormField;

#[derive(Debug)]
pub struct EntranceData;

#[derive(Copy, Clone, Debug, FromFormField)]
pub enum EntranceShuffleType {
    // two-way doors are truly two-way; one-way doors lead to one-way exits
    Standard,
    // two-way doors behave like one-way doors; every door can lead to any exit
    Chaos,
}

// Represents a user's input configuration
pub struct Config {
    pub seed: u64,
    pub entrance_shuffle: EntranceShuffleType,
}

