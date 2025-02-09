use std::marker::PhantomData;

use iced::{
    widget::{container, text},
    window::Id,
    Alignment, Element, Length, Size, Task,
};
use oxiced::widgets::oxi_button::{button, ButtonVariant};

use crate::{PageId, ReSetMessage};

pub enum EntryButtonLevel {
    TopLevel,
    SubLevel,
}

pub struct EntryButton {
    pub title: &'static str,
    pub msg: ReSetMessage,
    pub level: EntryButtonLevel,
}

pub struct EntryCategory {
    pub main_entry: EntryButton,
    pub sub_entries: Vec<EntryButton>,
}

fn create_button<'a>(entry: EntryButton) -> Element<'a, ReSetMessage> {
    match entry.level {
        EntryButtonLevel::TopLevel => button(entry.title, ButtonVariant::LeftMenuEntry)
            .on_press(entry.msg)
            .width(Length::Fill)
            .into(),
        EntryButtonLevel::SubLevel => {
            let mid_text = text(entry.title)
                .align_x(Alignment::Center)
                .center()
                .width(Length::Fill);
            button(mid_text, ButtonVariant::LeftMenuEntry)
                .on_press(entry.msg)
                .width(Length::Fill)
                .into()
        }
    }
}

fn create_category<'a>(category: EntryCategory) -> Vec<Element<'a, ReSetMessage>> {
    let mut sub_buttons: Vec<Element<'a, ReSetMessage>> = category
        .sub_entries
        .into_iter()
        .map(create_button)
        .collect();
    sub_buttons.insert(0, create_button(category.main_entry));
    sub_buttons
}

pub fn sidebar<'a>(entries: Vec<EntryCategory>) -> Element<'a, ReSetMessage> {
    // TODO beforepr
    // only show with responsive size
    //let size: Size = iced::window::get_size(Id::unique());
    //match size {}
    //
    let category_buttons: Vec<Element<'a, ReSetMessage>> =
        entries.into_iter().map(create_category).flatten().collect();
    let col = iced::widget::Column::with_children(category_buttons)
        .padding(10)
        .spacing(10);
    container(col)
        .style(container::bordered_box)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .into()
}
