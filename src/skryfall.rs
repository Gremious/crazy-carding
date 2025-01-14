use super::*;

pub async fn get_random_card() -> anyhow::Result<image::DynamicImage> {
	let bytes = CLIENT.get("https://api.scryfall.com/cards/random")
		.query(&[
			("format", "image"),
			("version", "png"),
		])
		.header(reqwest::header::USER_AGENT, "crazy-carding")
		.header(reqwest::header::ACCEPT, "*/*")
		.send().await?
		.error_for_status()?
		.bytes().await?;

	Ok(image::load_from_memory(&bytes)?)
}

pub fn resize_card_image(img: image::DynamicImage) -> anyhow::Result<image::DynamicImage> {
	let mm_width = 63.04;
	let inch_width = mm_width / 25.4;
	let mm_height = 88.0;
	let inch_height = mm_height / 25.4;

	let dpi = 300.0;
	let width: f64 = inch_width * dpi;
	let height: f64 = inch_height * dpi;

	// log::info!("{}x{}", width, height);
	Ok(img.resize(width.ceil() as u32, height.ceil() as u32, image::imageops::FilterType::CatmullRom))
}

pub fn get_ppi(img: image::DynamicImage) -> u32 {
	let mm_width = 64.04;
	let inch_width = mm_width / 25.4;

	(img.width() as f64 / inch_width).round() as u32
}

#[derive(serde::Deserialize)]
struct Symbology {
	data: Vec<CardSymbol>,
}

#[derive(serde::Deserialize)]
struct CardSymbol {
	// symbol: String,
	svg_uri: String,
	// english: String,
}

pub async fn download_svgs() -> anyhow::Result<()> {
	// https://api.scryfall.com/symbology
	let items = CLIENT.get("https://api.scryfall.com/symbology")
		.header(reqwest::header::USER_AGENT, "crazy-carding")
		.header(reqwest::header::ACCEPT, "*/*")
		.send().await?
		.error_for_status()?
		.json::<Symbology>().await?;

	for item in items.data {
		let bytes = CLIENT.get(&item.svg_uri)
			.header(reqwest::header::USER_AGENT, "crazy-carding")
			.header(reqwest::header::ACCEPT, "*/*")
			.send().await?
			.error_for_status()?
			.bytes().await?;

		std::fs::write(format!("resources/svg/mtg/{}", item.svg_uri.rsplit_once("/").unwrap().1), bytes)?;
		tokio::time::sleep(std::time::Duration::from_millis(125)).await;
	}

	Ok(())
}

