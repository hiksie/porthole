use iced::widget::qr_code::{Catalog, Style, StyleFn};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.palette();

    Style {
        cell: palette.qr_cell,
        background: palette.qr_background,
    }
}
