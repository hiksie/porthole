use iced::{
    border,
    widget::button::{Catalog, Status, Style},
};

use crate::theme::Theme;

pub enum ButtonClass {
    Primary,
    Secondary,
    Transparent(BackgroundColor, TextColor),
}

pub enum BackgroundColor {
    Blue,
    White,
}

pub enum TextColor {
    Blue,
    White,
    Green,
}
impl Catalog for Theme {
    type Class<'a> = ButtonClass;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::Secondary
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match class {
            ButtonClass::Primary => primary(self, status),
            ButtonClass::Secondary => secondary(self, status),
            ButtonClass::Transparent(bg_color, text_color) => {
                transparent(self, status, bg_color, text_color)
            }
        }
    }
}

fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let style = Style {
        background: Some(palette.button_primary.active.into()),
        text_color: palette.text_primary.into(),
        border: border::rounded(2),
        ..Style::default()
    };

    match status {
        Status::Active => style,
        Status::Hovered => Style {
            background: Some(palette.button_primary.hovered.into()),
            ..style
        },
        Status::Pressed => Style {
            background: Some(palette.button_primary.pressed.into()),
            ..style
        },
        Status::Disabled => style,
    }
}

fn secondary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let style = Style {
        background: None,
        text_color: palette.text_secondary.into(),
        border: border::rounded(2).width(1).color(palette.border),
        ..Style::default()
    };

    match status {
        Status::Active => style,
        Status::Hovered => Style {
            background: Some(palette.button_secondary.hovered.into()),
            text_color: palette.text_secondary_hover,
            border: style.border.color(palette.border_hover),
            ..style
        },
        Status::Pressed => Style {
            background: Some(palette.button_secondary.pressed.into()),
            text_color: palette.text_secondary_pressed,
            border: style.border.color(palette.border_pressed),
            ..style
        },
        Status::Disabled => Style {
            text_color: palette.text_disabled.into(),
            ..style
        },
    }
}

fn transparent(
    theme: &Theme,
    status: Status,
    bg_color: &BackgroundColor,
    text_color: &TextColor,
) -> Style {
    let palette = theme.palette();

    let (bg_active, bg_hovered, bg_pressed) = match bg_color {
        BackgroundColor::Blue => (
            palette.button_secondary.active,
            palette.button_secondary.hovered,
            palette.button_secondary.pressed,
        ),
        BackgroundColor::White => (
            palette.button_transparent_white.active,
            palette.button_transparent_white.hovered,
            palette.button_transparent_white.pressed,
        ),
    };

    let (text_active, text_hovered, text_pressed) = match text_color {
        TextColor::Blue => (
            palette.text_secondary,
            palette.text_secondary_hover,
            palette.text_secondary_pressed,
        ),
        TextColor::White => (
            palette.text_primary,
            palette.text_primary_hover,
            palette.text_primary_pressed,
        ),
        TextColor::Green => (palette.green, palette.green_hover, palette.green_pressed),
    };

    let style = Style {
        background: Some(bg_active.into()),
        text_color: text_active.into(),
        border: border::rounded(2),
        ..Style::default()
    };

    match status {
        Status::Active => style,
        Status::Hovered => Style {
            background: Some(bg_hovered.into()),
            text_color: text_hovered,
            ..style
        },
        Status::Pressed => Style {
            background: Some(bg_pressed.into()),
            text_color: text_pressed,
            ..style
        },
        Status::Disabled => Style {
            text_color: palette.text_disabled.into(),
            ..style
        },
    }
}
