#[macro_use]
extern crate rocket;

use rocket::{
    form::{Form, FromForm},
    fs::{relative, FileServer, NamedFile, TempFile},
    http::{ContentType, Header},
    response::content,
};
use std::{fs::{OpenOptions, File}, path::Path};
use thiserror::Error;

mod common;
mod config;
mod graph;
mod randomizer;
mod rng;
mod rom_writer;

use config::Config;

const RANDOMIZED_ROM_NAME: &str = "katam_randomized.gba";

#[derive(Debug, FromForm)]
struct Submit<'v> {
    #[field(validate = ext(ContentType::Binary))]
    rom_file: TempFile<'v>,
}

#[derive(Responder)]
#[response(content_type = "binary")]
struct RomResponder<'a> {
    file: File,
    content_disposition: Header<'a>
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

#[post("/api/submit", data = "<form>")]
async fn submit(form: Form<Submit<'_>>) -> Result<RomResponder<'_>, Error> {
    let result = submit_rom(form).await;
    result.map_err(|err| err.into())
}

async fn submit_rom(mut form: Form<Submit<'_>>) -> anyhow::Result<RomResponder<'_>> {
    let rom_path = format!("{}{}", relative!("/rom"), "katam_rom.gba");
    form.rom_file.persist_to(&rom_path).await?;
    let mut rom_file = OpenOptions::new().read(true).write(true).open(&rom_path)?;
    // TODO: Don't take a form; convert into a custom data type that we can mock first
    let config = config::KatamConfig::load_config()?;
    let rng = rng::KatamRng::new(config.get_seed());
    let rom = rom_writer::Rom::new(&mut rom_file);
    randomizer::randomize_game(config, rng, rom)?;

    let content_disposition = Header::new(
        "Content-Disposition", format!("attachment; filename=\"{}\"", RANDOMIZED_ROM_NAME));
    Ok(RomResponder {
        file: rom_file,
        content_disposition
    })
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![submit])
        .mount("/", FileServer::from(relative!("/frontend")).rank(1))
}
