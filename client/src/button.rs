use iced::widget::text::{LineHeight, Wrapping};
use iced::{
    Alignment, Length,
    widget::{center, row, text},
};

use crate::{Element, Text, theme::Theme};

pub enum ButtonLabel<'a> {
    Icon(Text<'a>),
    Text(&'a str),
    IconWithText(Text<'a>, &'a str),
}

impl<'a> ButtonLabel<'a> {
    pub fn into_button<Message: 'a>(
        self,
    ) -> iced::widget::Button<'a, Message, Theme, iced::Renderer> {
        match self {
            ButtonLabel::Icon(icon) => Self::button(icon).padding([0, 12]),
            ButtonLabel::Text(text) => Self::button(Self::text(text)).padding([0, 18]),
            ButtonLabel::IconWithText(icon, text) => Self::button(
                row![icon, Self::text(text)]
                    .align_y(Alignment::Center)
                    .spacing(8),
            )
            .padding([0, 18]),
        }
    }

    fn text(label_text: &'a str) -> Text<'a> {
        text(label_text)
            .wrapping(Wrapping::None)
            .line_height(LineHeight::Relative(1.5))
    }

    fn button<Message: 'a>(
        content: impl Into<Element<'a, Message>>,
    ) -> iced::widget::Button<'a, Message, Theme, iced::Renderer> {
        iced::widget::button(center(content))
            .width(Length::Shrink)
            .height(36)
            .clip(true)
    }
}
