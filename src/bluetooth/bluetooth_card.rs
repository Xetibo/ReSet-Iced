use std::collections::HashMap;

use iced::{
    border::Radius,
    widget::{self, column, text},
    Border, Element, Length,
};
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use zbus::zvariant::OwnedObjectPath;

use crate::ReSetMessage;

use super::{bluetooth_impl::BluetoothMsg, dbus_interface::BluetoothDevice};

#[derive(Clone, Copy)]
pub enum BluetoothButtonVariant {
    Connect,
    Disconnect,
}

enum RowAt {
    Start,
    Between,
    End,
    Only,
}

fn radius(at: RowAt) -> Radius {
    match at {
        RowAt::Start => Radius::new(10).top_right(0).top_left(0),
        RowAt::Between => Radius::new(0),
        RowAt::End => Radius::new(10).bottom_right(0).bottom_left(0),
        RowAt::Only => Radius::new(10),
    }
}

fn row_button_style(style: widget::button::Style, at: RowAt) -> widget::button::Style {
    widget::button::Style {
        border: Border {
            radius: radius(at),
            ..style.border
        },
        ..style
    }
}

fn create_button<'a>(
    index: usize,
    length: usize,
    value: &BluetoothDevice,
    variant: BluetoothButtonVariant,
) -> Element<'a, ReSetMessage> {
    let msg = match variant {
        BluetoothButtonVariant::Connect => {
            BluetoothMsg::ConnectToBluetoothDevice(value.path.clone())
        }
        BluetoothButtonVariant::Disconnect => {
            BluetoothMsg::RemoveBluetoothDevice(value.path.clone())
        }
    };
    button(text(value.alias.clone()), ButtonVariant::Primary)
        .on_press(ReSetMessage::SubMsgBluetooth(msg))
        .style(move |theme, state| {
            let at = if length == 1 {
                RowAt::Only
            } else if index == 0 {
                RowAt::End
            } else if index == length - 1 {
                RowAt::Start
            } else {
                RowAt::Between
            };
            row_button_style(
                oxiced::widgets::oxi_button::primary_button(theme, state),
                at,
            )
        })
        .width(Length::Fill)
        .into()
}

pub fn bluetooth_device_buttons<'a>(
    devices: &Vec<&BluetoothDevice>,
    variant: BluetoothButtonVariant,
) -> Element<'a, ReSetMessage> {
    let length = devices.len();
    let title = match variant {
        BluetoothButtonVariant::Connect => "Devices",
        BluetoothButtonVariant::Disconnect => "Connected Devices",
    };
    let views: Vec<Element<'_, ReSetMessage>> = devices
        .iter()
        .enumerate()
        .map(|(index, value)| create_button(index, length, value, variant))
        .collect();
    column!(
        text(title),
        iced::widget::Column::with_children(views)
            .width(Length::Fill)
            .padding(20)
    )
    .into()
}
