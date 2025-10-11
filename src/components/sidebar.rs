use iced::{
    border::Radius,
    color,
    widget::{
        button::{Status, Style},
        container, row,
    },
    Border, Element, Length, Padding, Shadow, Theme, Vector,
};
use oxiced::{
    theme::theme_impl::OXITHEME,
    utils::color::{darken_color, lighten_color},
};
use re_set_lib::utils::iced_sidebar::{EntryButton, EntryButtonLevel, EntryCategory};

use crate::ReSetMessage;

use super::icons::icon_widget_from_plain_path;

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
        background: Some(iced::Background::Color(OXITHEME.mantle)),
        text_color: theme.palette().text,
        border: Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(10),
        },
        shadow: Shadow {
            color: darken_color(&color!(0x181825), 10.0),
            offset: Vector { x: 0.2, y: 0.2 },
            blur_radius: 2.0,
        },
        snap: true,
    };

    match status {
        Status::Active => base,
        Status::Pressed => Style {
            background: Some(iced::Background::Color(lighten_color(
                &lighten_color(&OXITHEME.mantle, OXITHEME.tint_amount),
                OXITHEME.tint_amount,
            ))),
            ..base
        },
        Status::Hovered => Style {
            background: Some(iced::Background::Color(lighten_color(
                &OXITHEME.mantle,
                OXITHEME.tint_amount,
            ))),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn icon_and_text<'a>(text: &'static str, icon_opt: Option<String>) -> Element<'a, ReSetMessage> {
    let icon: Vec<Element<'_, ReSetMessage>> = icon_opt
        .into_iter()
        .map(icon_widget_from_plain_path)
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
            .on_press(entry.msg.downcast_ref::<ReSetMessage>().unwrap().clone())
            .style(side_bar_button_style)
            .padding(Padding::new(10.0).top(10).bottom(10))
            .width(Length::Fill)
            .into(),
        EntryButtonLevel::SubLevel => {
            row!(iced::widget::button(icon_and_text(entry.title, entry.icon))
                .on_press(entry.msg.downcast_ref::<ReSetMessage>().unwrap().clone())
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
    sub_buttons.push(iced::widget::rule::horizontal(2).into());
    sub_buttons
}

pub fn sidebar<'a>(entries: Vec<EntryCategory>) -> Element<'a, ReSetMessage> {
    // TODO beforepr
    // only show with responsive size
    //let size: Size = iced::window::get_size(Id::unique());
    //match size {}
    //
    let category_buttons: Vec<Element<'a, ReSetMessage>> =
        entries.into_iter().flat_map(create_category).collect();
    let col = iced::widget::Column::with_children(category_buttons)
        .padding(10)
        .spacing(5);
    container(col)
        .style(container::bordered_box)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .into()
}
