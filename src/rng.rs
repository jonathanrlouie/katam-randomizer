use rand::{self, Rng, SeedableRng};

pub trait RNG {
    fn get_bool(&mut self, p: f64) -> bool;
}

pub struct KatamRng {
    rng: rand::rngs::StdRng,
}

impl KatamRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
}

impl RNG for KatamRng {
    fn get_bool(&mut self, p: f64) -> bool {
        self.rng.gen_bool(p)
    }
}
