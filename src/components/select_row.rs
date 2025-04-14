use std::borrow::Borrow;

use iced::{
    border::Radius,
    widget::{self},
    Border,
};

use crate::ReSetMessage;

use super::comborow::CustomPickList;

pub enum SelectRowPosition {
    Start,
    Between,
    End,
    Only,
}

pub fn radius(at: SelectRowPosition) -> Radius {
    match at {
        SelectRowPosition::Start => Radius::new(10).top_right(0).top_left(0),
        SelectRowPosition::Between => Radius::new(0),
        SelectRowPosition::End => Radius::new(10).bottom_right(0).bottom_left(0),
        SelectRowPosition::Only => Radius::new(10),
    }
}

pub fn select_row_style(
    style: widget::pick_list::Style,
    at: SelectRowPosition,
) -> widget::pick_list::Style {
    widget::pick_list::Style {
        border: Border {
            radius: radius(at),
            ..style.border
        },
        ..style
    }
}

pub fn picklist_to_row<'a, T, L, V>(
    picker: CustomPickList<'a, T, L, V, ReSetMessage>,
    index: usize,
    length: usize,
) -> CustomPickList<'a, T, L, V, ReSetMessage>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    picker.style(move |theme, state| {
        let row_at = if length == 1 {
            SelectRowPosition::Only
        } else if index == 0 {
            SelectRowPosition::End
        } else if index == length - 1 {
            SelectRowPosition::Start
        } else {
            SelectRowPosition::Between
        };
        select_row_style(
            oxiced::widgets::oxi_picklist::picklist_style(theme, state),
            row_at,
        )
    })
}
