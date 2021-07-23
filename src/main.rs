#[macro_use]
extern crate rocket;

use rocket::{
    form::{Form, FromForm},
    fs::{relative, FileServer, NamedFile, TempFile},
    http::ContentType,
};
use std::{fs::OpenOptions, path::Path};
use thiserror::Error;

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

#[derive(Responder, Debug, Error)]
#[error("Internal server error")]
enum Error {
    InternalServerError(String),
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::InternalServerError("internal server error".to_string())
    }
}

#[post("/", data = "<form>")]
async fn submit(form: Form<Submit<'_>>) -> Result<NamedFile, Error> {
    let result = submit_rom(form).await;
    result.map_err(|err| err.into())
}

async fn submit_rom(mut form: Form<Submit<'_>>) -> anyhow::Result<NamedFile> {
    let rom_path = format!("{}{}", relative!("/rom"), "katam_rom.gba");
    form.rom_file.persist_to(&rom_path).await?;
    let rom_file = OpenOptions::new().read(true).write(true).open(&rom_path)?;
    // TODO: Don't take a form; convert into a custom data type that we can mock first
    let config = config::KatamConfig::load_config()?;
    let rng = rng::KatamRng::new(config.get_seed());
    let rom = rom_writer::Rom::new(rom_file);
    randomizer::randomize_game(config, rng, rom)?;

    let path = Path::new(relative!("/frontend/index.html"));
    let index_file = NamedFile::open(path).await?;
    Ok(index_file)
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![submit])
        .mount("/", FileServer::from(relative!("/frontend")))
}
