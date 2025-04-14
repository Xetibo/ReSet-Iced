use iced::{
    widget::{row, Column},
    Element, Length,
};

use crate::ReSetMessage;

use crate::components::text::title;

pub fn row_with_title<'a>(
    title_str: impl iced::widget::text::IntoFragment<'a>,
    content: impl Into<Element<'a, ReSetMessage>>,
) -> Element<'a, ReSetMessage> {
    row!(title(title_str), content.into(),)
        .width(Length::Fill)
        .padding(20)
        .width(20)
        .into()
}

pub fn col_with_title<'a>(
    title_str: impl iced::widget::text::IntoFragment<'a>,
    mut elements: Vec<Element<'a, ReSetMessage>>,
) -> Element<'a, ReSetMessage> {
    elements.insert(0, title(title_str));
    Column::from_vec(elements)
        .width(Length::Fill)
        .padding(20)
        .width(20)
        .into()
}
