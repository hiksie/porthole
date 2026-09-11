use crate::theme::Theme;

pub mod app;
mod button;
pub mod font;
mod icon;
pub mod theme;
mod util;

type Element<'a, Message> = iced::Element<'a, Message, Theme, iced::Renderer>;
type Text<'a> = iced::widget::Text<'a, Theme, iced::Renderer>;
