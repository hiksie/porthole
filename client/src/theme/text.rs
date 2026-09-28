use iced::widget::text::{Catalog, Style, StyleFn};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme| Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn secondary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.palette().text_secondary),
    }
}

pub fn green(theme: &Theme) -> Style {
    Style {
        color: Some(theme.palette().green),
    }
}
