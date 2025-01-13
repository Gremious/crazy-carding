use std::{collections::VecDeque, sync::LazyLock};
use itertools::Itertools;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub mod hearthstone;
pub mod skryfall;
pub mod ygoprodeck;

use hearthstone as hs;
use image::{DynamicImage, ExtendedColorType};
use rand::Rng;
use skryfall as mtg;
use tap::{Pipe, Tap};
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
async fn main() -> anyhow::Result<()> {
	init().await.unwrap();
	// let mut rng = rand::thread_rng();

	std::fs::create_dir_all("./images")?;

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

	let text = text_to_png("Haste ", "(This creature can attack and W as soon as it comes under your control.)");
	// let text = text_to_png("Haste ", "(This creature can attack and T");
	// // let text = text_to_png("Áaaa ", "Áaaa Aaaa Áaaa Aaaa");
	text.save("images/text.png")?;

	let svg = render_svg(48);
	svg.save("images/svg.png")?;
	// debug_text("Á(WT$").save("images/txt.png")?;

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

pub fn debug_text(text: &str) -> DynamicImage {
	use imageproc::drawing::draw_text_mut;

	let mplantin = include_bytes!("../resources/fonts/mplantin.ttf");
	let mplantin = ab_glyph::FontArc::try_from_slice(mplantin).unwrap();

	let font_size = 128.0;
	let text = "ÁÄg(aaa";

	let (w, h) = imageproc::drawing::text_size(font_size, &mplantin, text);
	let mut image: DynamicImage = DynamicImage::new_rgba8(w, h);

	draw_text_mut(&mut image, image::Rgba([ 100u8, 100u8, 100u8, 255u8 ]), 0, 0, font_size, &mplantin, text);

	image
}

// We're doing all this to print with italic and bold text and maybe even symbols (wow)
// on the thermal printer
pub fn text_to_png(keyword: &str, explanation: &str) -> DynamicImage {
	use imageproc::drawing::draw_text_mut;

	let mplantin = include_bytes!("../resources/fonts/mplantin.ttf");
	let mplantin = ab_glyph::FontArc::try_from_slice(mplantin).unwrap();

	let mplantin_it = include_bytes!("../resources/fonts/mplantin-italic.ttf");
	let mplantin_it = ab_glyph::FontArc::try_from_slice(mplantin_it).unwrap();

	let mplantin_bold = include_bytes!("../resources/fonts/mplantin-bold.ttf");
	let mplantin_bold = ab_glyph::FontArc::try_from_slice(mplantin_bold).unwrap();


	let font_size = 38.0;
	// small
	// let font_size = 32.0;
	let max_width_px = 625;

	// (Text line, width, height)
	let mut lines: Vec<(String, u32, u32)> = Vec::new();

	let (w_kw, h_kw) = imageproc::drawing::text_size(font_size, &mplantin_bold, keyword);
	let h_kw = h_kw;
	let (w_expl, h_expl) = imageproc::drawing::text_size(font_size, &mplantin_it, explanation);

	// Splits words to fit, like this:
	//
	// (This creature can attack and {T} as soon as it comes under your control.)
	// ->
	// (This creature can attack  |
	// and {T} as soon as it comes under your control.)
	// ->
	// (This creature can attack  |
	// and {T} as soon as it comes|
	// under your control.)       |
	fn split_text<'a>(
		w_kw: u32,
		w_expl: u32,
		h_expl: u32,
		max_width_px: u32,
		mut curr_line_words: Vec<&'a str>,
		leftover: &mut VecDeque<&'a str>,
		font_size: f32,
		mplantin_it: &ab_glyph::FontArc,
		lines: &mut Vec<(String, u32, u32)>,
	) {

		if w_kw + w_expl > max_width_px {
			let pop = curr_line_words.pop().unwrap();
			leftover.push_front(pop);

			let new_curr_line = curr_line_words.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
			let (new_w, new_h) = imageproc::drawing::text_size(font_size, mplantin_it, &new_curr_line);

			split_text(w_kw, new_w, new_h, max_width_px, curr_line_words, leftover, font_size, mplantin_it, lines);
		} else {
			let new_curr_line = curr_line_words.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
			lines.push((new_curr_line, w_expl, h_expl));

			let leftovers = leftover.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
			let (rest_w, rest_h) = imageproc::drawing::text_size(font_size, mplantin_it, &leftovers);

			if leftover.is_empty() {
				return;
			}

			split_text(
				0,
				rest_w,
				rest_h,
				max_width_px,
				leftover.iter().map(|x| *x).collect::<Vec<_>>(),
				&mut VecDeque::new(),
				font_size,
				mplantin_it,
				lines,
			);

		}
	}

	if w_kw + w_expl > max_width_px {
		let mut curr_line_words = explanation.split_whitespace().collect::<Vec<_>>();
		let mut leftover = VecDeque::new();

		let pop = curr_line_words.pop().unwrap();
		leftover.push_front(pop);

		let new_curr_line = curr_line_words.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
		let (new_w, new_h) = imageproc::drawing::text_size(font_size, &mplantin_it, &new_curr_line);

		split_text(w_kw, new_w, new_h, max_width_px, curr_line_words, &mut leftover, font_size, &mplantin_it, &mut lines);

	} else {
		lines.push((explanation.to_owned(), w_expl, h_expl));
	}


	log::info!("{}x{}", w_kw + w_expl, h_kw);

	let height = if lines.len() == 1 {
		h_kw.max(h_expl)
	} else {
		let (_, _, h_first_line) = lines.first().unwrap();
		let first_height = h_kw.max(*h_first_line);
		let rest = lines.iter().skip(1).map(|(_, _, h)| h).sum::<u32>();
		first_height + rest
	};

	let mut image: DynamicImage = DynamicImage::new_rgba8(max_width_px, height);

	if lines.len() == 1 {
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), 0, 0, font_size, &mplantin_bold, keyword);
		let (explanation, _, _) = &lines[0];
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), w_kw as i32, 0, font_size, &mplantin_it, &explanation);
	} else {
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), 0, 0, font_size, &mplantin_bold, keyword);
		let (explanation, _, h) = &lines[0];
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), w_kw as i32, 0, font_size, &mplantin_it, &explanation);

		let mut prev_h = h_kw.max(*h);

		for (line, _, h) in lines.iter().skip(1) {
			draw_text_mut(
				&mut image,
				image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]),
				0,
				prev_h as i32,
				font_size,
				&mplantin_it,
				&line,
			);
			prev_h += h;
		}
	}

	let (_, height_a) = imageproc::drawing::text_size(font_size, &mplantin_it, ".");
	let svg = render_svg(height_a);
	image::imageops::overlay(&mut image, &svg, 0, 0);

	image
}

pub fn render_svg(size: u32) -> DynamicImage {

	let options = resvg::usvg::Options::default();
	let tree = resvg::usvg::Tree::from_str(include_str!("../resources/svg/tap.svg"), &options).unwrap();

	let scale = size as f32 / 100.0;
	let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);

	let mut pixmap_buffer = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
	resvg::render(&tree, transform, &mut pixmap_buffer.as_mut());
	image::load_from_memory_with_format(&pixmap_buffer.encode_png().unwrap(), image::ImageFormat::Png).unwrap()
}
