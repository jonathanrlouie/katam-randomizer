#[macro_use]
extern crate rocket;

use rocket::{
    form::{Form, FromForm},
    fs::{relative, FileServer, TempFile},
    http::{ContentType, Header},
    State,
};
use std::{
    env,
    fs::{File, OpenOptions},
};
use thiserror::Error;

mod config;
mod game_graph;
mod graph;
mod katam_rng;
mod randomizer;
mod rng;
mod rom;
mod rom_file;

use config::{Config, EntranceShuffleType};

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
    KatamRando(#[from] randomizer::KatamRandoError),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for randomizer::KatamRandoError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        rocket::http::Status::InternalServerError.respond_to(req)
    }
}

#[post("/api/submit", data = "<form>")]
async fn submit<'a>(
    mut form: Form<Submit<'_>>,
    game_data_state: &State<GameData>,
) -> Result<RomResponder<'a>, Error> {
    let rom_path = format!("{}{}", relative!("/rom"), "katam_rom.gba");
    form.rom_file.persist_to(&rom_path).await?;
    let mut rom_file = OpenOptions::new().read(true).write(true).open(&rom_path)?;
    let config: Config = form.into();
    let rng = katam_rng::KatamRng::new(config.seed);
    let rom = rom_file::RomFile {
        rom_file: &mut rom_file,
    };
    let mut gd = (*game_data_state).clone();
    randomizer::randomize_katam(config, rng, rom, &mut gd.graph)?;

    let content_disposition = Header::new(
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", RANDOMIZED_ROM_NAME),
    );

    Ok(RomResponder {
        file: rom_file,
        content_disposition,
    })
}

type NodeID = String;

fn load_game_data(path: &str) -> game_graph::GameGraph {
    let file_contents = std::fs::read_to_string(path).expect("Error opening KatAM game data file.");
    let graph_data: game_graph::GraphData<NodeID> = ron::from_str(&file_contents)
        .unwrap_or_else(|e| panic!("Error deserializing KatAM game data: {}", e));
    game_graph::GameGraph::new(graph_data)
}

#[rocket::launch]
fn rocket() -> _ {
    let game_data = GameData::load_game_data(&env::var("KATAM_DATA_PATH").expect("Environment variable KATAM_DATA_PATH not set. Please set it to the path where the KatAM data file is located."));

    rocket::build()
        .mount("/", rocket::routes![submit])
        .mount("/", FileServer::from(relative!("../frontend")).rank(1))
        .manage(game_data)
}
