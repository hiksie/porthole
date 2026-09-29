use iced::{
    Background, Border, Color,
    widget::text_input::{Catalog, Status, Style, StyleFn},
};

use crate::theme::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

enum Variant {
    Normal,
    Error,
}

pub fn default(theme: &Theme, status: Status) -> Style {
    style(theme, status, Variant::Normal)
}

pub fn error(theme: &Theme, status: Status) -> Style {
    style(theme, status, Variant::Error)
}

fn style(theme: &Theme, status: Status, variant: Variant) -> Style {
    let palette = theme.palette();

    let active = Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: match variant {
                Variant::Normal => palette.border,
                Variant::Error => palette.red,
            },
        },
        placeholder: palette.text_secondary,
        value: palette.text_primary,
        selection: palette.border,
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: match variant {
                    Variant::Normal => palette.border_hover,
                    Variant::Error => palette.red,
                },
                ..active.border
            },
            ..active
        },
        Status::Focused { .. } => Style {
            border: Border {
                color: match variant {
                    Variant::Normal => palette.border_pressed,
                    Variant::Error => palette.red,
                },
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            value: active.placeholder,
            ..active
        },
    }
}
