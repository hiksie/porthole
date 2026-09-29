use iced::{
    Border, Shadow,
    overlay::menu::{Catalog, Style, StyleFn},
};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.palette();

    Style {
        background: palette.background_light.into(),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: palette.border,
        },
        text_color: palette.text_primary,
        selected_text_color: palette.text_primary,
        selected_background: palette.button_secondary.hovered.into(),
        shadow: Shadow::default(),
    }
}
