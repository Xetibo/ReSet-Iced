use std::{
    collections::{HashMap, HashSet},
    fmt::{self},
};

use crate::{bluetooth::dbus_interface::TPath, components::text::error_text};
use iced::{
    border,
    widget::{column, container::Style},
    Element, Task, Theme,
};
use zbus::{zvariant::OwnedObjectPath, Connection};

use crate::ReSetMessage;

pub fn ignore<T>(_: T) {}

// TODO move to oxiced
pub fn rounded_card(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Some(palette.background.weak.color.into()),
        border: border::rounded(10),
        ..Style::default()
    }
}

pub trait TPage<T, S, A> {
    fn enter() -> Task<ReSetMessage>;
    fn leave() -> Task<ReSetMessage>;
    async fn new(ctx: &Connection, additional_data: A) -> Result<S, ReSetError>;
    async fn update(&mut self, msg: T) -> Option<Task<ReSetMessage>>;
    fn view(&self) -> Result<Element<ReSetMessage>, ReSetError>;
}

pub type ReSetError = Box<dyn TReSetError>;

pub trait TReSetError: fmt::Debug + fmt::Display + Send + Sync + 'static {}

pub fn create_error(err: impl TReSetError) -> ReSetError {
    Box::new(err)
}

impl TReSetError for zbus::Error {}
impl From<zbus::Error> for ReSetError {
    fn from(value: zbus::Error) -> Self {
        create_error(value)
    }
}

impl TReSetError for String {}
impl From<String> for ReSetError {
    fn from(value: String) -> Self {
        create_error(value)
    }
}

pub fn display_view_or_error(
    view_or_error: Result<Element<ReSetMessage>, ReSetError>,
) -> Element<ReSetMessage> {
    match view_or_error {
        Ok(view) => view,
        Err(err) => column!(error_text(err.to_string())).into(),
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
