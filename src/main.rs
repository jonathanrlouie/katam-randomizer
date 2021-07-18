#[macro_use]
extern crate rocket;

use anyhow;
use rocket::{
    form::{Form, FromForm},
    fs::{relative, FileServer, NamedFile, TempFile},
    http::ContentType,
};
use std::{
    fs::OpenOptions,
    path::Path
};

mod common;
mod config;
mod graph;
mod randomizer;
mod rng;
mod rom_writer;

use config::Config;

#[derive(Debug, FromForm)]
struct Submit<'v> {
    #[field(validate = ext(ContentType::Binary))]
    rom_file: TempFile<'v>,
}

#[post("/", data = "<form>")]
async fn submit(mut form: Form<Submit<'_>>) -> anyhow::Result<NamedFile> {
    let rom_path = format!("{}{}", relative!("/rom"), "katam_rom.gba");
    form.rom_file.persist_to(&rom_path).await?;
    let rom_file = OpenOptions::new().read(true).write(true).open(&rom_path)?;
    // TODO: Don't take a form; convert into a custom data type that we can mock first
    let config = config::KatamConfig::load_config()?;
    let rng = rng::KatamRng::new(config.get_seed());
    let rom = rom_writer::Rom::new(rom_file);
    randomizer::randomize_game(config, rng, rom)?;

    let path = Path::new(relative!("/frontend/index.html"));
    NamedFile::open(path).await
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![submit])
        .mount("/", FileServer::from(relative!("/frontend")))
}
