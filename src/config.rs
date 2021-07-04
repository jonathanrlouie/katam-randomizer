pub struct Config;

pub trait LoadConfig {
    fn load_config() -> Self;
}

impl LoadConfig for Config {
    fn load_config() -> Self {
        Config
    }
}
