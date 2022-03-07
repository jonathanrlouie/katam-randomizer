#[macro_use]
extern crate rocket;

use rocket::{
    form::{Form, FromForm},
    fs::{relative, FileServer, NamedFile, TempFile},
    http::{ContentType, Header},
    response::content,
    State
};
use std::{
    env,
    fs::{File, OpenOptions},
    path::Path,
};
use thiserror::Error;

mod config;
mod error;
mod game_data;
mod graph;
mod randomizer;
mod rng;
mod rom;
mod types;

use config::{Config, EntranceData, EntranceShuffleType};

const RANDOMIZED_ROM_NAME: &str = "katam_randomized.gba";

#[derive(Debug, FromForm)]
struct Submit<'v> {
    #[field(validate = ext(ContentType::Binary))]
    rom_file: TempFile<'v>,
    seed: u64,
    entrance_shuffle_type: EntranceShuffleType,
}

impl From<Form<Submit<'_>>> for Config {
    fn from(form: Form<Submit<'_>>) -> Self {
        Config {
            seed: form.seed,
            entrance_shuffle: form.entrance_shuffle_type,
        }
    }
}

#[derive(Responder)]
#[response(content_type = "binary")]
struct RomResponder<'a> {
    file: File,
    content_disposition: Header<'a>,
}

#[derive(Responder, Debug, Error)]
enum Error {
    #[error("IO Error {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Randomizer Error {0:?}")]
    KatamRando(#[from] error::KatamRandoError)
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for error::KatamRandoError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            _ => rocket::http::Status::InternalServerError.respond_to(req)
        }
    }
}

#[post("/api/submit", data = "<form>")]
async fn submit<'a>(mut form: Form<Submit<'a>>, data: &State<game_data::GameData>) -> Result<RomResponder<'a>, Error> {
    let rom_path = format!("{}{}", relative!("/rom"), "katam_rom.gba");
    form.rom_file.persist_to(&rom_path).await?;
    let mut rom_file = OpenOptions::new().read(true).write(true).open(&rom_path)?;
    let config: Config = form.into();
    let rng = rng::KatamRng::new(config.seed);
    let rom = rom::RomFile { rom_file: &mut rom_file };
    let mut gd = (*data).clone();
    randomizer::randomize_katam(
        config,
        rng,
        rom,
        &gd.rom_data_maps,
        &mut gd.graph)?;

    let content_disposition = Header::new(
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", RANDOMIZED_ROM_NAME),
    );

    Ok(RomResponder {
        file: rom_file,
        content_disposition,
    })
}

#[rocket::launch]
fn rocket() -> _ {
    let game_data = game_data::load_game_data(&env::var("KATAM_DATA_PATH").expect("Environment variable KATAM_DATA_PATH not set. Please set it to the path where the KatAM data file is located."));

    rocket::build()
        .mount("/", rocket::routes![submit])
        .mount("/", FileServer::from(relative!("../frontend")).rank(1))
        .manage(game_data)
}
