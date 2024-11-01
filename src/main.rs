use std::sync::LazyLock;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub mod skryfall;
use skryfall as mtg;

#[tokio::main]
async fn main() {
	let mtg_card = mtg::get_random_card().await.unwrap();
	print_img(&mtg_card);
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
