use crate::config;
use crate::error;

trait RandomizerAlgorithm<T> {
    fn randomize_game(config: config::Config) -> error::Result<T>;
}
