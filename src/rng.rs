pub trait RNG {
    fn get_bool(&mut self) -> bool;
    fn get_number(&mut self) -> u32;
}

pub struct KatamRng {
    rng: rand::StdRng;
}

impl KatamRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::StdRng::new(seed)
        }
    }
}

impl RNG for KatamRng {
    fn get_bool(&mut self) -> bool {
        self.rng.get_bool()
    }

    fn get_number(&mut self) -> u32 {
        self.rng.get_number()
    }
}
