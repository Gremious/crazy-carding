pub mod render;

use super::mtg;
use std::{collections::HashMap, sync::LazyLock};
use sugars::hmap;

static FONTS: LazyLock<HashMap<&'static str, ab_glyph::FontArc>> = LazyLock::new(|| {
	let mplantin = include_bytes!("../resources/fonts/mplantin.ttf");
	let mplantin = ab_glyph::FontArc::try_from_slice(mplantin).unwrap();

	let mplantin_it = include_bytes!("../resources/fonts/mplantin-italic.ttf");
	let mplantin_it = ab_glyph::FontArc::try_from_slice(mplantin_it).unwrap();

	let mplantin_bold = include_bytes!("../resources/fonts/mplantin-bold.ttf");
	let mplantin_bold = ab_glyph::FontArc::try_from_slice(mplantin_bold).unwrap();

	hmap! {
		"mplantin" => mplantin,
		"mplantin_it" => mplantin_it,
		"mplantin_bold" => mplantin_bold,
	}
});


type MtgText = Vec<TextItem>;

#[derive(Debug)]
pub enum TextItem {
	// Bold(String),
	Regular(String),
	Italic(String),
	Symbol(String),
}

impl TextItem {
	fn width(&self, font_size: f32) -> u32 {
		match self {
			// TextItem::Bold(x) => todo!(),
			TextItem::Regular(x) => todo!(),
			TextItem::Italic(x) => todo!(),
			TextItem::Symbol(x) => todo!(),
		}
	}
}

// Things that are italic:
// Everthing in parentheses
// flavor-words (Mark of Chaos Ascendant)
// ability words (Delirium)
// but notably not keyword-abilities (Cumulative Upkeep)
//
// Maybe worth looking into
// https://docs.rs/parse-hyperlinks/latest/src/parse_hyperlinks/lib.rs.html#41
// Or nom parsing in general
// It seems be quite nice for stuff like this
pub fn make_mtg_paragraph(text: &str) -> anyhow::Result<MtgText> {
	let mut ret = Vec::new();

	let mut maybe_italic = None;

	let open_paren = text.find(|c| c == '(');
	let closed_paren = text.rfind(|c| c == ')');
	if let (Some(start), Some(end)) = (open_paren, closed_paren) {
		maybe_italic = Some((start, end));
	}
	log::debug!("maybe_italic: {maybe_italic:?}");

	let mut maybe_phrase = None;

	for (i, c) in text.chars().enumerate() {
		log::debug!("pphrase: {maybe_phrase:?}");
		let in_italics = maybe_italic.is_some_and(|(it_start, it_end)| (it_start..it_end).contains(&i));
		log::debug!("in it: {in_italics}");

		let Some(ref mut phrase) = maybe_phrase else {
			match c {
				'{' => {
					maybe_phrase = Some(TextItem::Symbol(String::new()));
				},
				_ => {
					maybe_phrase = if in_italics {
						Some(TextItem::Italic(c.to_string()))
					} else {
						Some(TextItem::Regular(c.to_string()))
					};
				},
			};
			continue;
		};

		if c == '{' {
			match phrase {
				TextItem::Regular(_) | TextItem::Italic(_) => {
					ret.push(maybe_phrase.take().unwrap());
					maybe_phrase = Some(TextItem::Symbol(String::new()));
					continue;
				},
				// Too funky, just trust
				TextItem::Symbol(s) => s.push(c),
			}
		}

		match (phrase, in_italics) {
			(TextItem::Regular(_), true) => {
				ret.push(maybe_phrase.take().unwrap());
				maybe_phrase = Some(TextItem::Italic(c.to_string()));
			},
			(TextItem::Regular(s), false) => s.push(c),
			(TextItem::Italic(s), true) => s.push(c),
			(TextItem::Italic(_), false) => {
				ret.push(maybe_phrase.take().unwrap());
				maybe_phrase = Some(TextItem::Regular(c.to_string()));
			},
			(TextItem::Symbol(s), _) => {
				if c == '}' {
					ret.push(maybe_phrase.take().unwrap());
				} else {
					s.push(c);
				}
			},
		}
	}

	Ok(ret)
}

