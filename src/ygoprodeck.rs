use super::*;

pub async fn get_random_card() -> anyhow::Result<image::DynamicImage> {
	let res = CLIENT.get("https://db.ygoprodeck.com/api/v7/randomcard.php")
		.header(reqwest::header::USER_AGENT, "crazy-carding")
		.send().await?
		.error_for_status()?
		.json::<serde_json::Value>().await?;

	let url_value = res.pointer("/data/0/card_images/0/image_url").expect("No image URL found.").clone();
	let url = serde_json::from_value::<String>(url_value)?;

	let bytes = CLIENT.get(&url).send().await?.bytes().await?;
	Ok(image::load_from_memory(&bytes)?)
}
