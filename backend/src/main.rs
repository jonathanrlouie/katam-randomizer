#[macro_use]
extern crate rocket;

use std::{
    env,
    fs::{File, OpenOptions},
};
use rocket::{
    form::{Context, Contextual, Form, FromForm},
    fs::{relative, FileServer, TempFile},
    http::{ContentType, Status, Header},
    response::{self, Responder},
    State,
    Request,
};
use rocket_dyn_templates::Template;
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
use game_graph::GameGraph;

const RANDOMIZED_ROM_NAME: &str = "katam_randomized.gba";

#[derive(Debug, FromForm)]
struct Submit<'v> {
    #[field(validate = ext(ContentType::Binary))]
    rom_file: TempFile<'v>,
    seed: u64,
    entrance_shuffle_type: EntranceShuffleType,
}

impl From<&mut Submit<'_>> for Config {
    fn from(submission: &mut Submit<'_>) -> Self {
        Config {
            seed: submission.seed,
            entrance_shuffle: submission.entrance_shuffle_type,
        }
    }
}


#[derive(Debug)]
enum FormResponse<'a> {
    Template(Status, Template),
    TemplateWithRom(Status, Template, RandomizedRom<'a>)
}

impl<'a, 'o: 'a> Responder<'a, 'o> for FormResponse<'o> {
    fn respond_to(self, request: &Request) -> response::Result<'o> {
        use FormResponse::*;
        match self {
            Template(status, template) => (status, template).respond_to(request),
            TemplateWithRom(status, template, randomized_rom) => {
                (status, template).respond_to(request)?;
                randomized_rom.respond_to(request)
            }
        }
    }
}

#[derive(Debug, Responder)]
#[response(content_type = "binary")]
struct RandomizedRom<'a> {
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

#[get("/")]
fn index() -> Template {
    Template::render("index", &Context::default())
}

#[post("/", data = "<form>")]
async fn submit<'a>(
    mut form: Form<Contextual<'a, Submit<'a>>>,
    graph: &State<GameGraph>,
) -> Result<FormResponse<'a>, Error> {
    match form.value {
        Some(ref mut submission) => {
            let randomized_rom = run_randomizer(submission, graph).await?;
            let template = Template::render("success", &form.context);
            let status = form.context.status();
            Ok(FormResponse::TemplateWithRom(status, template, randomized_rom))
        }
        None => Ok(FormResponse::Template(form.context.status(), Template::render("index", &form.context))),
    }
}

async fn run_randomizer<'a>(
    submission: &mut Submit<'_>,
    graph: &State<GameGraph>,
) -> Result<RandomizedRom<'a>, Error> {
    let rom_path = format!("{}{}", relative!("/rom"), "katam_rom.gba");
    submission.rom_file.persist_to(&rom_path).await?;
    let mut rom_file = OpenOptions::new().read(true).write(true).open(&rom_path)?;
    let config: Config = submission.into();
    let rng = katam_rng::KatamRng::new(config.seed);
    let rom = rom_file::RomFile {
        rom_file: &mut rom_file,
    };
    let mut graph_copy = (*graph).clone();
    randomizer::randomize_katam(config, rng, rom, &mut graph_copy)?;

    let content_disposition = Header::new(
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", RANDOMIZED_ROM_NAME),
    );

    Ok(RandomizedRom {
        file: rom_file,
        content_disposition,
    })
}

type NodeID = String;

fn load_game_data(path: &str) -> GameGraph {
    let file_contents = std::fs::read_to_string(path).expect("Error opening KatAM game data file.");
    let graph_data: game_graph::GraphData<NodeID> = ron::from_str(&file_contents)
        .unwrap_or_else(|e| panic!("Error deserializing KatAM game data: {}", e));
    GameGraph::new(graph_data)
}

#[rocket::launch]
fn rocket() -> _ {
    let game_data = load_game_data(&env::var("KATAM_DATA_PATH").expect("Environment variable KATAM_DATA_PATH not set. Please set it to the path where the KatAM data file is located."));

    rocket::build()
        .mount("/", rocket::routes![index, submit])
        .attach(Template::fairing())
        .mount("/", FileServer::from(relative!("/static")))
        .manage(game_data)
}
