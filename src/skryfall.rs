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
