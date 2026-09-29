mod app;
mod button;
mod dialog;
mod font;
mod icon;
mod settings;
mod theme;
mod util;

use crate::theme::Theme;
use crate::app::{App, Message};

type Element<'a, Message> = iced::Element<'a, Message, Theme, iced::Renderer>;
type Text<'a> = iced::widget::Text<'a, Theme, iced::Renderer>;

fn main() -> iced::Result {
    iced::application::<App, Message, Theme, iced::Renderer>(App::default, App::update, App::view)
        .title("Porthole")
        .theme(App::theme)
        .window(App::window_settings())
        .settings(App::settings())
        .run()
}


