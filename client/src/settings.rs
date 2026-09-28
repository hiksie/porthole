use std::net::IpAddr;

use iced::{
    Length, padding,
    widget::{column, container, pick_list, right, row, text, text_input},
};

use crate::{Element, button::ButtonLabel, dialog::dialog, theme};

#[derive(Debug, Default)]
pub enum Settings {
    #[default]
    Closed,
    Open(Data),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub port: Option<u16>,
    pub port_error: bool,
    pub ip: IpAddr,
    pub ips: Vec<IpAddr>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(Data),
    Close,
    Form(FormMessage),
}

#[derive(Debug, Clone)]
pub enum FormMessage {
    ChangePort(String),
    ChangeAddr(IpAddr),
    Save,
}

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    None,
    Save(IpAddr, u16),
}

impl Settings {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Open(data) => {
                *self = Settings::Open(data);
                Action::None
            }
            Message::Close => {
                *self = Settings::Closed;
                Action::None
            }
            Message::Form(message) => match self {
                Settings::Closed => Action::None,
                Settings::Open(data) => match message {
                    FormMessage::ChangePort(value) => {
                        let value = value.trim();

                        if value.is_empty() {
                            data.port = None;
                        } else {
                            if let Ok(value) = value.parse::<u16>() {
                                data.port = Some(value)
                            }
                        }

                        Action::None
                    }
                    FormMessage::ChangeAddr(addr) => {
                        data.ip = addr;
                        Action::None
                    }
                    FormMessage::Save => match data.port {
                        Some(port) => Action::Save(data.ip, port),
                        None => {
                            data.port_error = true;
                            Action::None
                        }
                    },
                },
            },
        }
    }

    pub fn view(&self) -> Option<Element<'_, Message>> {
        let Settings::Open(data) = self else {
            return None;
        };

        let col = column![
            row![
                text("ADDRESS")
                    .style(theme::text::secondary)
                    .width(Length::Fill),
                text("PORT").style(theme::text::secondary).width(100),
            ]
            .spacing(15),
            row![
                pick_list(Some(data.ip), data.ips.clone(), |ip| ip.to_string())
                    .padding([9, 10])
                    .width(Length::Fill)
                    .on_select(|addr| Message::Form(FormMessage::ChangeAddr(addr))),
                text_input(
                    "",
                    data.port.map(|port| port.to_string()).unwrap_or_default()
                )
                .on_input(|v| Message::Form(FormMessage::ChangePort(v)))
                .width(100)
                .padding([9, 10])
                .style(if data.port_error {
                    theme::text_input::error
                } else {
                    theme::text_input::default
                })
            ]
            .spacing(15),
            right(
                ButtonLabel::Text("SAVE".into())
                    .into_button()
                    .on_press(Message::Form(FormMessage::Save))
            )
            .padding(padding::top(20))
        ]
        .spacing(15);

        let content = container(col)
            .width(360)
            .padding(30)
            .style(theme::container::light_rounded)
            .into();

        Some(dialog(content, Message::Close))
    }
}
