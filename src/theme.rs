use iced::{
    Color, color,
    theme::{Mode, Style, palette::Seed},
};

pub mod button;
pub mod container;
pub mod menu;
pub mod pick_list;
pub mod qr_code;
pub mod rule;
pub mod scrollable;
pub mod text;
pub mod text_input;

#[derive(Debug, Default)]
pub struct Theme;

impl iced::theme::Base for Theme {
    fn default(_: Mode) -> Self {
        Default::default()
    }

    fn mode(&self) -> Mode {
        Mode::None
    }

    fn base(&self) -> Style {
        let palette = self.palette();

        Style {
            background_color: palette.background_dark,
            text_color: palette.text_primary,
        }
    }

    fn seed(&self) -> Option<Seed> {
        None
    }

    fn name(&self) -> &str {
        "Dark"
    }
}

impl Theme {
    pub fn palette(&self) -> Palette {
        Palette::DARK
    }
}

pub struct Palette {
    background_dark: Color,
    background_light: Color,
    border: Color,
    border_hover: Color,
    border_pressed: Color,
    text_primary: Color,
    text_primary_hover: Color,
    text_primary_pressed: Color,
    text_secondary: Color,
    text_secondary_hover: Color,
    text_secondary_pressed: Color,
    text_disabled: Color,
    green: Color,
    green_hover: Color,
    green_pressed: Color,
    red: Color,
    backdrop: Color,
    scroller: ScrollerColors,
    button_primary: ButtonColors,
    button_secondary: ButtonColors,
    button_transparent_white: ButtonColors,
    qr_background: Color,
    qr_cell: Color,
}

pub struct ScrollerColors {
    active: Color,
    hovered: Color,
    draggred: Color,
}

pub struct ButtonColors {
    active: Color,
    hovered: Color,
    pressed: Color,
}

impl Palette {
    pub const DARK: Self = Self {
        background_dark: color!(0x0d1621),
        background_light: color!(0x172331),
        border: color!(0x3c4b5d),
        border_hover: color!(0x50697c),
        border_pressed: color!(0x587589),
        text_primary: color!(0xd9e2ec),
        text_primary_hover: color!(0xeef2f7),
        text_primary_pressed: color!(0xfcfcfd),
        text_secondary: color!(0x728caa),
        text_secondary_hover: color!(0x8ba3bb),
        text_secondary_pressed: color!(0x9eb6c7),
        text_disabled: color!(0x525860),
        green: color!(0x7fae6e),
        green_hover: color!(0x8cb67c),
        green_pressed: color!(0x95bc85),
        red: color!(0xbf4f5a),
        backdrop: color!(0x0d1621, 0.7),
        scroller: ScrollerColors {
            active: color!(0x283343),
            hovered: color!(0x303e4f),
            draggred: color!(0x3c4c62),
        },
        button_primary: ButtonColors {
            active: color!(0x415a7c),
            hovered: color!(0x476085),
            pressed: color!(0x4d668e),
        },
        button_secondary: ButtonColors {
            active: Color::TRANSPARENT,
            hovered: color!(0x6a83a8, 0.15),
            pressed: color!(0x6a83a8, 0.20),
        },
        button_transparent_white: ButtonColors {
            active: Color::TRANSPARENT,
            hovered: color!(0xffffff, 0.15),
            pressed: color!(0xffffff, 0.20),
        },
        qr_background: color!(0xffffff),
        qr_cell: color!(0x000000),
    };
}
