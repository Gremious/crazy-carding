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
