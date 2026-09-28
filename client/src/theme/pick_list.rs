use iced::{
    Border, Color,
    widget::pick_list::{Catalog, Status, Style, StyleFn},
};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let active = Style {
        text_color: palette.text_primary,
        background: Color::TRANSPARENT.into(),
        placeholder_color: palette.text_secondary,
        handle_color: palette.text_secondary,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.border,
        },
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: palette.border_hover,
                ..active.border
            },
            ..active
        },
        Status::Opened { .. } => Style {
            border: Border {
                color: palette.border_pressed,
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            text_color: palette.text_secondary,
            ..active
        },
    }
}
