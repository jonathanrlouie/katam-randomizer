mod rom_writer;
mod randomizer;
mod graph;
mod algorithm;
mod config;
mod common;
mod error;

fn main() {
    use config::{Config, LoadConfig};
    let config: Config = Config::load_config();

    println!("Hello, world!");
}
