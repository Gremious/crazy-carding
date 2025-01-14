pub mod render;
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

	fn push_char(&mut self, c: char) {
		match self {
			// TextItem::Bold(x) => x.push(c),
			TextItem::Regular(x) => x.push(c),
			TextItem::Italic(x) => x.push(c),
			TextItem::Symbol(_) => {},
		}
	}
}

// Things that are italic:
// Everthing in parentheses
// flavor-words (Mark of Chaos Ascendant)
// ability words (Delirium)
// but notably not keyword-abilities (Cumulative Upkeep)
pub fn make_mtg_paragraph(text: &str) -> MtgText {
	let mut ret = Vec::new();
	let mut text_items = Vec::new();
	let chars = text.chars();
	let mut phrase: Option<TextItem> = None;

	// WIP, for now, just parentheses
	// todo: if text beings with special phrase, push to italics, then skip to the rest..l.

	for c in chars {
		match c {
			'(' => {
				match phrase {
					Some(_) => {
						text_items.push(phrase.take().unwrap());
						phrase = Some(TextItem::Italic(String::from(c)));
					},
					None => {
						phrase = Some(TextItem::Italic(String::from(c)));
					},
				}
			},
			')' => {
				match &mut phrase {
					Some(ref mut p) => {
						p.push_char(c);
						text_items.push(phrase.take().unwrap());
					},
					None => {
						// Shouldn't happen but who knows
						phrase = Some(TextItem::Regular(String::from(c)));
					},
				}
			},
			_ => {
				match &mut phrase {
					Some(ref mut p) => {
						p.push_char(c);
					},
					None => {
						phrase = Some(TextItem::Regular(String::from(c)));
					},
				}
			},
		}
	}

	// Parsing '{X}' like symbols
	// I should load the svgs in a lazy static map
	// there aren't that many, so I think it's fine to render the tree as well?
	// if it's slow for some reason, I can be lazy per symbol or something
	for item in text_items.iter_mut() {
		match item {
			// TextItem::Bold(_) => todo!(),
			TextItem::Regular(text) => {
				if let Some(i) = text.find("{") {

				}
			},
			TextItem::Italic(text) => todo!(),
			TextItem::Symbol(_) => unreachable!(),
		}
	}

	ret
}
