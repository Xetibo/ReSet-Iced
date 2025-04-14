use iced::{Color, Element, Length};

pub fn title<'a, Message>(
    title_str: impl iced::widget::text::IntoFragment<'a>,
) -> Element<'a, Message> {
    iced::widget::text(title_str)
        .width(Length::Fill)
        .size(24)
        .into()
}

pub fn subtitle<'a, Message>(
    subtitle_str: impl iced::widget::text::IntoFragment<'a>,
) -> Element<'a, Message> {
    iced::widget::text(subtitle_str)
        .width(Length::Fill)
        .size(20)
        .into()
}

pub fn content_text<'a, Message>(
    content_str: impl iced::widget::text::IntoFragment<'a>,
) -> Element<'a, Message> {
    iced::widget::text(content_str)
        .width(Length::Fill)
        .size(16)
        .into()
}

pub fn error_text<'a, Message>(
    content_str: impl iced::widget::text::IntoFragment<'a>,
) -> Element<'a, Message> {
    iced::widget::text(content_str)
        .width(Length::Fill)
        .size(18)
        .color(Color::from_rgb(1.0, 0.0, 0.0))
        .into()
}
