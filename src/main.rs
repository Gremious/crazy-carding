use std::sync::LazyLock;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub mod hearthstone;
pub mod skryfall;
pub mod ygoprodeck;

use hearthstone as hs;
use skryfall as mtg;
use tokio::sync::Mutex;
use ygoprodeck as ygo;

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);
pub static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::load_or_default()));

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
	pub battlenet: Battlenet,
}

impl Config {
	pub fn load() -> Config {
		toml::from_str(&std::fs::read_to_string("Config.toml").unwrap()).unwrap()
	}
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Battlenet {
	pub client_id: String,
	pub client_secret: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct State {
	hearthstone: Option<hs::State>,
}

impl State {
	pub fn save(&self) -> anyhow::Result<()> {
		Ok(std::fs::write("state.json", serde_json::to_string(self)?)?)
	}

	pub fn load_or_default() -> State {
		std::fs::read_to_string("state.json")
			.and_then(|s| Ok(serde_json::from_str(&s).unwrap_or_default()))
			.unwrap_or_default()
	}
}

#[tokio::main]
async fn main() {
	init().await.unwrap();

	// let mtg_card = mtg::get_random_card().await.unwrap();
	// print_img(&mtg_card);
	// let ygo_card = ygo::get_random_card().await.unwrap();
	// print_img(&ygo_card);
	let hs_card = hs::get_random_card().await.unwrap();
	print_img(&hs_card);
}

async fn init() -> anyhow::Result<()> {
	simple_logger::SimpleLogger::new().with_colors(true).env().init().unwrap();

	let mut state = STATE.lock().await;

	if state.hearthstone.is_none() {
		let auth = hs::fetch_oauth().await?;
		let card_res = hs::fetch_card(&auth.access_token, 1).await?;

		state.hearthstone = Some(hs::State {
			auth,
			total_cards: card_res.page_count,
		});

		state.save()?;
	}

	Ok(())
}

pub fn print_img(img: &image::DynamicImage) {
	use viuer::{print, Config};

	let conf = Config {
		height: Some(30),
		..Default::default()
	};

	print(img, &conf).expect("Image printing failed.");
}
