use std::marker::PhantomData;

use iced::{
    border::Radius,
    color,
    widget::{
        button::{Status, Style},
        container, row, text,
    },
    window::Id,
    Alignment, Border, Element, Length, Padding, Shadow, Size, Task, Theme, Vector,
};
use oxiced::widgets::common::{darken_color, lighten_color};

use crate::{PageId, ReSetMessage};

use super::icons::{icon_widget, Icon};

pub enum EntryButtonLevel {
    TopLevel,
    SubLevel,
}

pub struct EntryButton {
    pub title: &'static str,
    pub icon: Option<Icon>,
    pub msg: ReSetMessage,
    pub level: EntryButtonLevel,
}

pub struct EntryCategory {
    pub main_entry: EntryButton,
    pub sub_entries: Vec<EntryButton>,
}

// TODO beforepr deduplicate from oxiced
fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}

fn side_bar_button_style(theme: &Theme, status: Status) -> Style {
    let base = Style {
        background: Some(iced::Background::Color(color!(0x181825))),
        text_color: theme.palette().text,
        border: Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(10),
        },
        shadow: Shadow {
            color: darken_color(color!(0x181825)),
            offset: Vector { x: 0.2, y: 0.2 },
            blur_radius: 2.0,
        },
    };

    match status {
        Status::Active => base,
        Status::Pressed => Style {
            background: Some(iced::Background::Color(lighten_color(lighten_color(
                color!(0x181825),
            )))),
            ..base
        },
        Status::Hovered => Style {
            background: Some(iced::Background::Color(lighten_color(color!(0x181825)))),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn icon_and_text<'a>(text: &'static str, icon_opt: Option<Icon>) -> Element<'a, ReSetMessage> {
    let icon: Vec<Element<'_, ReSetMessage>> = icon_opt
        .into_iter()
        .map(icon_widget)
        .map(|value| value.width(Length::Shrink).into())
        .collect();
    iced::widget::Row::with_children(icon)
        .push(iced::widget::text(text).width(Length::Fill))
        .spacing(10)
        .into()
}

fn create_button<'a>(entry: EntryButton) -> Element<'a, ReSetMessage> {
    match entry.level {
        EntryButtonLevel::TopLevel => iced::widget::button(icon_and_text(entry.title, entry.icon))
            .on_press(entry.msg)
            .style(side_bar_button_style)
            .padding(Padding::new(10.0).top(10).bottom(10))
            .width(Length::Fill)
            .into(),
        EntryButtonLevel::SubLevel => {
            row!(iced::widget::button(icon_and_text(entry.title, entry.icon))
                .on_press(entry.msg)
                .style(side_bar_button_style)
                .padding(Padding::new(20.0).top(10).bottom(10))
                .width(Length::Fill))
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
    sub_buttons.push(iced::widget::Rule::horizontal(2).into());
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
        .spacing(5);
    container(col)
        .style(container::bordered_box)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .into()
}
