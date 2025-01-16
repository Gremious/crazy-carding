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
//
// Technically, this fails e.g. "(Foo ( hehe :) smiley face ) bar)" but whatver
// It's good enough for 99% of cases
pub fn make_mtg_paragraph(text: &str) -> anyhow::Result<MtgText> {
	enum FirstPassItem {
		Text(String),
		Symbol(String),
	}

	let mut ret = Vec::new();
	let mut first_pass = Vec::new();

	let mut phrase = None;
	let mut symbol = None;
	let mut in_symbol = false;

	for c in text.chars() {
		match c {
			'{' => {
				in_symbol = true;

				let phrase = phrase.take();
				if let Some(phrase) = phrase {
					first_pass.push(FirstPassItem::Text(phrase));
				}
			},
			'}' => {
				in_symbol = false;

				let symbol = symbol.take();
				if let Some(symbol) = symbol {
					first_pass.push(FirstPassItem::Symbol(symbol));
				}
			},
			_ => {
				if in_symbol {
					if let Some(symbol) = symbol.as_mut() {
						symbol.push(c);
					} else {
						symbol = Some(String::from(c));
					}
				} else {
					if let Some(phrase) = phrase.as_mut() {
						phrase.push(c);
					} else {
						phrase = Some(String::from(c));
					}
				}
			},
		}
	}

	if let Some(phrase) = phrase.take() {
		first_pass.push(FirstPassItem::Text(phrase));
	}

	// is_italic = paren_count > 0;
	let mut paren_count = 0;
	let mut phrase: Option<String> = None;

	for item in first_pass {
		match item {
			FirstPassItem::Text(text) => {
				for c in text.chars() {
					match c {
						'(' => {
							if paren_count == 0 {
								let curr = phrase.take();
								if let Some(phrase) = curr {
									ret.push(TextItem::Regular(phrase));
								}

								phrase = Some(String::from(c));
							} else {
								if let Some(phrase) = phrase.as_mut() {
									phrase.push(c);
								}
							}

							paren_count += 1;
						},
						')' => {
							match paren_count {
								0 => {
									// Smiley face :)
									let phrase = phrase.get_or_insert(String::new());
									phrase.push(c);
								},
								1 => {
									paren_count -= 1;

									if paren_count == 0 {
										let curr = phrase.take();
										if let Some(mut phrase) = curr {
											phrase.push(c);
											ret.push(TextItem::Italic(phrase));
										}
									}
								},
								_ => {
									paren_count -= 1;

									let phrase = phrase.get_or_insert(String::new());
									phrase.push(c);
								},
							}
						},
						_ => {
							let phrase = phrase.get_or_insert(String::new());
							phrase.push(c);
						},
					}
				}
			},
			FirstPassItem::Symbol(symbol) => {
				let phrase = phrase.take();

				if let Some(p) = phrase {
					if paren_count > 0 {
						ret.push(TextItem::Italic(p));
					} else {
						ret.push(TextItem::Regular(p));
					}
				}

				ret.push(TextItem::Symbol(symbol));
			},
		}
	}

	if let Some(phrase) = phrase.take() {
		if paren_count > 0 {
			ret.push(TextItem::Italic(phrase));
		} else {
			ret.push(TextItem::Regular(phrase));
		}
	}

	Ok(ret)
}
