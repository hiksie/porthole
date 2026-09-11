use iced::{
    border,
    widget::container::{Catalog, Style, StyleFn},
};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn transparent<Theme>(_theme: &Theme) -> Style {
    Style::default()
}

pub fn light_rounded(theme: &Theme) -> Style {
    Style::default()
        .background(theme.palette().background_light)
        .border(border::rounded(2))
}
