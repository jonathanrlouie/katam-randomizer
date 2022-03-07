use crate::rng::{ChooseMultipleFill, RandomBool};
use rand::{self, prelude::IteratorRandom, Rng, SeedableRng};

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

impl RandomBool for KatamRng {
    fn get_bool(&mut self, p: f64) -> bool {
        self.rng.gen_bool(p)
    }
}

impl ChooseMultipleFill for KatamRng {
    fn choose_multiple_fill<T, I: Iterator<Item = T>>(&mut self, iter: I, buf: &mut [T]) -> usize {
        iter.choose_multiple_fill(&mut self.rng, buf)
    }
}
