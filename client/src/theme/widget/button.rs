use iced::{
    border,
    widget::button::{Catalog, Status, Style},
};

use crate::theme::Theme;

pub enum ButtonClass {
    Primary,
    Secondary,
    TransparentBlue,
    TransparentWhite,
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
            ButtonClass::TransparentBlue => transparent_blue(self, status),
            ButtonClass::TransparentWhite => transparent_white(self, status),
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

fn transparent_blue(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let style = Style {
        background: Some(palette.button_secondary.active.into()),
        text_color: palette.text_secondary.into(),
        border: border::rounded(2),
        ..Style::default()
    };

    match status {
        Status::Active => style,
        Status::Hovered => Style {
            background: Some(palette.button_secondary.hovered.into()),
            text_color: palette.text_secondary_hover,
            ..style
        },
        Status::Pressed => Style {
            background: Some(palette.button_secondary.pressed.into()),
            text_color: palette.text_secondary_pressed,
            ..style
        },
        Status::Disabled => Style {
            text_color: palette.text_disabled.into(),
            ..style
        },
    }
}

fn transparent_white(theme: &Theme, status: Status) -> Style{
    let palette = theme.palette();

    let style = Style {
        background: Some(palette.button_transparent_white.active.into()),
        text_color: palette.text_primary.into(),
        border: border::rounded(2),
        ..Style::default()
    };

    match status {
        Status::Active => style,
        Status::Hovered => Style {
            background: Some(palette.button_transparent_white.hovered.into()),
            text_color: palette.text_primary_hover,
            ..style
        },
        Status::Pressed => Style {
            background: Some(palette.button_transparent_white.pressed.into()),
            text_color: palette.text_primary_pressed,
            ..style
        },
        Status::Disabled => Style {
            text_color: palette.text_disabled.into(),
            ..style
        },
    }
}
