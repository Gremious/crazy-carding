use std::sync::LazyLock;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub mod skryfall;
pub mod ygoprodeck;
use skryfall as mtg;
use ygoprodeck as ygo;

#[tokio::main]
async fn main() {
	// simple_logger::SimpleLogger::new().env().init().unwrap();
	// let mtg_card = mtg::get_random_card().await.unwrap();
	// print_img(&mtg_card);
	let ygo_card = ygo::get_random_card().await.unwrap();
	print_img(&ygo_card);
}

pub fn print_img(img: &image::DynamicImage) {
	use viuer::{print, Config};

	let conf = Config {
		// Start from row 4 and column 20.
		height: Some(30),
		..Default::default()
	};

	print(img, &conf).expect("Image printing failed.");
}
