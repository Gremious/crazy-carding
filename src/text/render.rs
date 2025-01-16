use std::collections::VecDeque;

use image::DynamicImage;
use itertools::Itertools;

pub fn debug_text(text: &str) -> DynamicImage {
	use imageproc::drawing::draw_text_mut;

	let mplantin = include_bytes!("../../resources/fonts/mplantin.ttf");
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
	let kw_font = super::FONTS.get("mplantin").unwrap();
	let expl_font = super::FONTS.get("mplantin_it").unwrap();

	let font_size = 38.0;
	// small
	// let font_size = 32.0;
	let max_width_px = 625;

	// (Text line, width, height)
	let mut lines: Vec<(String, u32, u32)> = Vec::new();

	let (w_kw, h_kw) = imageproc::drawing::text_size(font_size, &kw_font, keyword);
	let h_kw = h_kw;
	let (w_expl, h_expl) = imageproc::drawing::text_size(font_size, &expl_font, explanation);

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
		expl_font: &ab_glyph::FontArc,
		lines: &mut Vec<(String, u32, u32)>,
	) {

		if w_kw + w_expl > max_width_px {
			let pop = curr_line_words.pop().unwrap();
			leftover.push_front(pop);

			let new_curr_line = curr_line_words.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
			let (new_w, new_h) = imageproc::drawing::text_size(font_size, expl_font, &new_curr_line);

			split_text(w_kw, new_w, new_h, max_width_px, curr_line_words, leftover, font_size, expl_font, lines);
		} else {
			let new_curr_line = curr_line_words.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
			lines.push((new_curr_line, w_expl, h_expl));

			let leftovers = leftover.iter().map(|x| x.to_owned()).intersperse(" ").collect::<String>();
			let (rest_w, rest_h) = imageproc::drawing::text_size(font_size, expl_font, &leftovers);

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
				expl_font,
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
		let (new_w, new_h) = imageproc::drawing::text_size(font_size, &expl_font, &new_curr_line);

		split_text(w_kw, new_w, new_h, max_width_px, curr_line_words, &mut leftover, font_size, &expl_font, &mut lines);

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
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), 0, 0, font_size, &kw_font, keyword);
		let (explanation, _, _) = &lines[0];
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), w_kw as i32, 0, font_size, &expl_font, &explanation);
	} else {
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), 0, 0, font_size, &kw_font, keyword);
		let (explanation, _, h) = &lines[0];
		draw_text_mut(&mut image, image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]), w_kw as i32, 0, font_size, &expl_font, &explanation);

		let mut prev_h = h_kw.max(*h);

		for (line, _, h) in lines.iter().skip(1) {
			draw_text_mut(
				&mut image,
				image::Rgba([ 0u8, 0u8, 0u8, 255u8 ]),
				0,
				prev_h as i32,
				font_size,
				&expl_font,
				&line,
			);
			prev_h += h;
		}
	}

	let (_, height_a) = imageproc::drawing::text_size(font_size, &expl_font, ".");
	let svg = render_svg(height_a);
	image::imageops::overlay(&mut image, &svg, 0, 0);

	image
}

pub fn render_svg(size: u32) -> DynamicImage {
	let options = resvg::usvg::Options::default();
	let tree = resvg::usvg::Tree::from_str(include_str!("../../resources/mtg/svg/T.svg"), &options).unwrap();

	let scale = size as f32 / 100.0;
	let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);

	let mut pixmap_buffer = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
	resvg::render(&tree, transform, &mut pixmap_buffer.as_mut());
	image::load_from_memory_with_format(&pixmap_buffer.encode_png().unwrap(), image::ImageFormat::Png).unwrap()
}
