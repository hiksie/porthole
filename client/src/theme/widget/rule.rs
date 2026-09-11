use iced::widget::rule::{Catalog, FillMode, Style, StyleFn};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> iced::widget::rule::Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.palette();

    Style {
        color: palette.border,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
        snap: true,
    }
}
