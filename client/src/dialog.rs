use iced::{
    Length,
    widget::{container, opaque, stack},
};

use crate::{
    Element,
    button::ButtonLabel,
    icon,
    theme::{self, button::ButtonClass},
};

pub fn dialog<'a, Message: Clone + 'a>(
    content: Element<'a, Message>,
    close: Message,
) -> Element<'a, Message> {
    let overlay = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(theme::container::backdrop);

    let close_btn = container(
        ButtonLabel::Icon(icon::cancel())
            .into_button()
            .padding([0, 14])
            .on_press(close)
            .class(ButtonClass::Transparent(
                theme::button::BackgroundColor::Blue,
                theme::button::TextColor::Blue,
            )),
    )
    .align_right(Length::Fill)
    .padding(15);

    opaque(stack![overlay, close_btn]).into()
}
