use super::*;

pub async fn get_random_card() -> anyhow::Result<image::DynamicImage> {
	let res = CLIENT.get("https://db.ygoprodeck.com/api/v7/randomcard.php")
		.header(reqwest::header::USER_AGENT, "crazy-carding")
		.send().await?
		.error_for_status()?
		.json::<serde_json::Value>().await?;

	let url_value = res.pointer("/data/0/card_images/0/image_url").expect("No image URL found.").clone();
	let url = serde_json::from_value::<String>(url_value)?;
	log::info!("URL: {}", url);

	let bytes = CLIENT.get(&url).send().await?.bytes().await?;

	log::info!("Bytes: {}", bytes.len());

	Ok(image::load_from_memory(&bytes)?)
}

pub fn resize_card_image(img: image::DynamicImage) -> anyhow::Result<image::DynamicImage> {
	let mm_width = 59.0;
	let inch_width = mm_width / 25.4;
	let mm_height = 86.0;
	let inch_height = mm_height / 25.4;

	log::info!("inches: {}x{}", inch_width, inch_height);

	let dpi = 300.0;
	let width: f64 = inch_width * dpi;
	let height: f64 = inch_height * dpi;

	log::info!("{}x{}", width, height);
	Ok(img.resize(width.ceil() as u32, height.ceil() as u32, image::imageops::FilterType::CatmullRom))
}

pub fn get_ppi(img: &image::DynamicImage) -> u32 {
	let mm_width = 59.0;
	let inch_width = mm_width / 25.4;

	(img.width() as f64 / inch_width).round() as u32
}
