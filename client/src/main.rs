use client::app::{App, Message};
use client::theme::Theme;

fn main() -> iced::Result {
    iced::application::<App, Message, Theme, iced::Renderer>(App::default, App::update, App::view)
        .title("Porthole")
        .theme(App::theme)
        .window(App::window_settings())
        .settings(App::settings())
        .run()
}
