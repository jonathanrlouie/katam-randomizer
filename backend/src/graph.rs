use crate::{
    error::EdgeSwapError,
    rng::{ChooseMultipleFill, RandomBool},
};
use std::fmt::Debug;

pub trait Graph<N: Debug, E> {
    fn swap_edges(&mut self, edge1: E, edge2: E) -> std::result::Result<(E, E), EdgeSwapError>;
    fn pick_random_edges<R>(&self, rng: &mut R) -> Option<(E, E)>
    where
        R: RandomBool + ChooseMultipleFill;
    // TODO: Change this to return Vec<(N, N)> once string IDs are moved to descriptions
    fn get_edges(&self) -> Vec<(String, String)>;
    fn get_unreachable_regions(&self) -> Vec<Vec<N>>;
}
