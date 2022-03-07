pub trait RandomBool {
    fn get_bool(&mut self, p: f64) -> bool;
}

pub trait ChooseMultipleFill {
    fn choose_multiple_fill<T, I: Iterator<Item = T>>(&mut self, iter: I, buf: &mut [T]) -> usize;
}
