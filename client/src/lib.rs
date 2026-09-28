use crate::theme::Theme;

pub mod app;
mod button;
mod dialog;
pub mod font;
mod icon;
mod settings;
pub mod theme;
mod util;

type Element<'a, Message> = iced::Element<'a, Message, Theme, iced::Renderer>;
type Text<'a> = iced::widget::Text<'a, Theme, iced::Renderer>;
