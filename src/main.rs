use std::{collections::VecDeque, sync::LazyLock};

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub mod prelude;
pub mod hearthstone;
pub mod skryfall;
pub mod ygoprodeck;
pub mod text;

use ygoprodeck as ygo;
use skryfall as mtg;
use hearthstone as hs;

use prelude::*;
use image::{DynamicImage, ExtendedColorType};
use rand::Rng;
use tokio::sync::Mutex;

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
async fn main() -> anyhow::Result<()> {

	init().await.unwrap();
	// let mut rng = rand::thread_rng();

	std::fs::create_dir_all("./images")?;

	let text = text::make_mtg_paragraph("Haste (This creature can attack and {T} as soon as it comes under your control.)");
	log::info!("{:?}", text);

	// let card_img = mtg::get_random_card().await?;

	// let len = card_img.as_bytes().len();
	// log::info!("pre-resize len: {}", len);
	// let width = card_img.width();
	// let height = card_img.height();

	// let card_img = match rng.gen_range(0..2) {
		// 0 => mtg::get_random_card().await?,
		// 1 => ygo::get_random_card().await?,
		// 2 => hs::get_random_card().await?,
		// _ => unreachable!(),
	// };

	// log::info!("original: {}x{}", card_img.width(), card_img.height());
	// let card_img = ygo::resize_card_image(card_img)?;
	// log::info!("resized: {}x{}", card_img.width(), card_img.height());

	// let dt = chrono::Local::now().format("%+");
	// let name = format!("images/{dt}.png");
	// card_img.save(&name)?;
	// print_card(&name, ygo::get_ppi(&card_img));
	// print_img(&card_img);

	// let text = text::render::text_to_png("Haste ", "(This creature can attack and {T} as soon as it comes under your control.)");
	// let text = text_to_png("Haste ", "(This creature can attack and T");
	// // let text = text_to_png("Áaaa ", "Áaaa Aaaa Áaaa Aaaa");
	// text.save("images/text.png")?;

	// let svg = text::render::render_svg(48);
	// svg.save("images/svg.png")?;
	// text_render::debug_text("Á(WT$").save("images/txt.png")?;

	Ok(())
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

pub fn display_img(img: &image::DynamicImage) {
	use viuer::{print, Config};

	let conf = Config {
		height: Some(30),
		..Default::default()
	};

	print(img, &conf).expect("Image printing failed.");
}

pub fn print_card(path: &str, ppi: u32) {
	// lp -o media=3x5,ByPassTray,Stationery -o ColorMode=Gray -o ppi=300 ./images/latest.jpg
	let cmd = std::process::Command::new("lp")
		.args(&[
			"-o",
			"media=3x5,ByPassTray,Stationery",
			"-o",
			"ColorMode=Gray",
			"-o",
			&format!("ppi={ppi}"),
			path,
		])
		.output()
		.expect("Failed to execute command.");

	log::info!("{:?}", cmd);
}

