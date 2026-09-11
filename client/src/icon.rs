use iced::{Font, widget::text};

use crate::Text;

pub fn trash<'a>() -> Text<'a> {
    to_text('\u{E800}')
}

pub fn cancel<'a>() -> Text<'a> {
    to_text('\u{E801}')
}

pub fn plus<'a>() -> Text<'a> {
    to_text('\u{E806}')
}

pub fn play<'a>() -> Text<'a> {
    to_text('\u{E803}')
}

pub fn stop<'a>() -> Text<'a> {
    to_text('\u{E804}')
}

pub fn qrcode<'a>() -> Text<'a> {
    to_text('\u{E805}')
}

pub fn circle<'a>() -> Text<'a> {
    to_text('\u{F111}')
}

fn to_text<'a>(unicode: char) -> Text<'a> {
    text(unicode.to_string())
        .line_height(text::LineHeight::Relative(1.0))
        .font(Font::with_family("icons"))
}
