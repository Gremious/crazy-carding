use rand::Rng;
use tap::TapOptional;

use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
	pub auth: hs::Oauth,
	pub total_cards: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Oauth {
	pub access_token: String,
	// Seconds, usually 86399 e.g. 23.99 hours
	pub expires_in: u64,
	#[serde(skip_deserializing)]
	#[serde(default = "now")]
	pub created_at: chrono::DateTime<chrono::Utc>,
}

fn now() -> chrono::DateTime<chrono::Utc> { chrono::Utc::now() }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardsResponse {
	pub cards: Vec<Card>,
	pub page_count: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
	pub image: String,
}

pub async fn get_random_card() -> anyhow::Result<image::DynamicImage> {
	let Some(total_cards) = STATE.lock().await.hearthstone.as_ref().map(|hs| hs.total_cards) else {
		return Err(anyhow::anyhow!("No Hearthstone state found"));
	};
	let mut rng = rand::thread_rng();

	let number = rng.gen_range(1..=total_cards);
	let auth = get_fresh_auth().await?;
	let mut card_res = fetch_card(&auth, number).await?;

	if total_cards != card_res.page_count {
		let mut state = STATE.lock().await;
		state.hearthstone.as_mut().tap_some_mut(|hs| hs.total_cards = card_res.page_count);
		state.save()?;
		drop(state);

		let new_number = rng.gen_range(1..=card_res.page_count);
		let auth = get_fresh_auth().await?;
		card_res = fetch_card(&auth, new_number).await?;
	}

	let image = CLIENT.get(&card_res.cards[0].image).send().await?.bytes().await?;
	return Ok(image::load_from_memory(&image)?);
}

pub async fn fetch_card(auth: &String, number: u64) -> anyhow::Result<CardsResponse> {
	Ok(CLIENT.get("https://eu.api.blizzard.com/hearthstone/cards")
		.bearer_auth(auth)
		.query(&[
			("region", "eu"),
			("locale", "en_US"),
			("page", &number.to_string()),
			("pageSize", "1"),
		])
		.send().await?
		.error_for_status()?
		.json::<CardsResponse>().await?)
}

pub async fn get_fresh_auth() -> anyhow::Result<String> {
	let Some(oauth) = STATE.lock().await.hearthstone.as_ref().map(|hs| hs.auth.clone()) else {
		return Err(anyhow::anyhow!("No Hearthstone state found"));
	};

	if chrono::Utc::now() < oauth.created_at + chrono::Duration::seconds(oauth.expires_in as i64) {
		return Ok(oauth.access_token);
	}

	let fresh = fetch_oauth().await?;
	let token = fresh.access_token.clone();

	let mut state = STATE.lock().await;
	state.hearthstone.as_mut().tap_some_mut(|hs| hs.auth = fresh);
	state.save()?;

	Ok(token)
}

pub async fn fetch_oauth() -> anyhow::Result<Oauth> {
	log::info!("Fetching fresh Hearthstone OAuth token");

	Ok(CLIENT.post("https://oauth.battle.net/token")
		.basic_auth(&CONFIG.battlenet.client_id, Some(&CONFIG.battlenet.client_secret))
		.query(&[("grant_type", "client_credentials")])
		.send()
		.await?
		.error_for_status()?
		.json::<Oauth>().await?)
}
