use client::app::{App, Message};
use client::font;
use client::theme::Theme;

fn main() -> iced::Result {
    iced::application::<App, Message, Theme, iced::Renderer>(App::default, App::update, App::view)
        .title("Porthole")
        .theme(App::theme)
        .window_size(iced::Size::new(500.0, 768.0))
        .settings(iced::Settings {
            id: Some(common::APP_ID.into()),
            default_font: iced::Font::new("JetBrains Mono"),
            default_text_size: 14.into(),
            fonts: font::load(),
            ..iced::Settings::default()
        })
        .run()
}
