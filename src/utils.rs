use std::collections::HashMap;

use crate::{bluetooth::dbus_interface::TPath, components::text::error_text};
use iced::{
    border::{self, Radius},
    widget::{column, container::Style, Column},
    Element, Length, Padding, Pixels, Task, Theme,
};
use oxiced::theme::theme_impl::OXITHEME;
use re_set_lib::utils::error::ReSetError;
use zbus::{zvariant::OwnedObjectPath, Connection};

use crate::ReSetMessage;

pub fn ignore<T>(_: T) {}

// TODO move to oxiced
pub fn rounded_card(_: &Theme) -> Style {
    Style {
        background: Some(OXITHEME.primary_bg.into()),
        border: border::rounded(10),
        ..Style::default()
    }
}

pub trait TPage<T, S, A> {
    fn enter() -> Task<ReSetMessage>;
    fn leave() -> Task<ReSetMessage>;
    async fn new(ctx: &Connection, additional_data: A) -> Result<S, ReSetError>;
    async fn update(&mut self, msg: T) -> Option<Task<ReSetMessage>>;
    fn view(&self) -> Result<Vec<Element<'_, ReSetMessage>>, ReSetError>;
}

pub fn display_view_or_error(
    view_or_error: Result<Vec<Element<ReSetMessage>>, ReSetError>,
) -> Vec<Element<ReSetMessage>> {
    match view_or_error {
        Ok(view) => view,
        Err(err) => vec![column!(error_text(err.to_string())).into()],
    }
}

pub fn to_object_map<T>(elements: Vec<T>) -> HashMap<OwnedObjectPath, T>
where
    T: TPath,
{
    let mut map = HashMap::new();
    for element in elements.into_iter() {
        map.insert(element.path(), element);
    }
    map
}

pub trait PushMany<'a, T: Into<Element<'a, T>>> {
    fn push_many(self, elems: Vec<T>) -> Self;
}

impl<'a, T> PushMany<'a, T> for Column<'a, T>
where
    T: Into<Element<'a, T>>,
{
    fn push_many(self, elems: Vec<T>) -> Self {
        let mut column = self;
        for elem in elems {
            column = column.push(elem);
        }
        column
    }
}

pub enum OxiPadding {
    None = 0,
    Small = 5,
    Medium = 10,
    Large = 20,
    XLarge = 40,
}

impl From<OxiPadding> for Padding {
    fn from(val: OxiPadding) -> Self {
        Padding::new(val as i32 as f32)
    }
}

impl From<OxiPadding> for Pixels {
    fn from(val: OxiPadding) -> Self {
        Pixels::from(val as i32 as f32)
    }
}

impl From<OxiPadding> for Length {
    fn from(val: OxiPadding) -> Self {
        Length::Fixed(val as i32 as f32)
    }
}

impl From<OxiPadding> for Radius {
    fn from(val: OxiPadding) -> Self {
        Radius::from(val as i32 as f32)
    }
}
