use iced::{border, widget::container::Style, Theme};

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

pub trait TToError<T> {
    fn to_zbus_error(self) -> Result<T, zbus::Error>;
}

impl<T> TToError<T> for Option<T> {
    fn to_zbus_error(self) -> Result<T, zbus::Error> {
        if let Some(value) = self {
            Ok(value)
        } else {
            Err(zbus::Error::Failure("Error".to_string()))
        }
    }
}
