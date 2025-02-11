use std::collections::HashMap;

use iced::{
    border::Radius,
    widget::{self, column, container, row, text},
    Border, Element, Length,
};
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use zbus::zvariant::OwnedObjectPath;

use crate::{
    components::icons::{icon_widget, Icon},
    utils::rounded_card,
    ReSetMessage,
};

use super::{
    bluetooth_impl::BluetoothMsg,
    dbus_interface::{BluetoothAdapter, BluetoothDevice},
};

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
) -> Option<Element<'a, ReSetMessage>> {
    let (msg, icon) = match variant {
        BluetoothButtonVariant::Connect => (
            BluetoothMsg::ConnectToBluetoothDevice(value.path.clone()),
            Icon::Bluetooth,
        ),
        BluetoothButtonVariant::Disconnect => (
            BluetoothMsg::RemoveBluetoothDevice(value.path.clone()),
            Icon::BluetoothConnected,
        ),
    };
    // TODO beforepr should the empty entries be removed??
    if value.alias == "" {
        None
    } else {
        Some(
            button(
                row!(
                    icon_widget(icon).width(Length::Shrink),
                    text(value.alias.clone()),
                )
                .spacing(10),
                ButtonVariant::Primary,
            )
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
                row_button_style(oxiced::widgets::oxi_button::row_entry(theme, state), at)
            })
            .width(Length::Fill)
            .into(),
        )
    }
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
        .filter_map(|(index, value)| create_button(index, length, value, variant))
        .collect();
    column!(
        text(title).size(25),
        iced::widget::Column::with_children(views).width(Length::Fill)
    )
    .spacing(20)
    .into()
}

fn wrap(msg: BluetoothMsg) -> ReSetMessage {
    ReSetMessage::SubMsgBluetooth(msg)
}

fn card_view<'a>(
    index: usize,
    is_default: bool,
    adapter: &BluetoothAdapter,
) -> Element<'a, ReSetMessage> {
    let default_index = if is_default { Some(index) } else { None };
    let path = adapter.path.clone();
    let path1 = adapter.path.clone();
    let path2 = adapter.path.clone();
    let path3 = adapter.path.clone();
    let col = column!(
        row!(
            text(adapter.alias.clone()).width(Length::Fill).size(25),
            oxiced::widgets::oxi_radio::radio("", index, default_index, move |_| wrap(
                BluetoothMsg::SetBluetoothAdapter(path.clone())
            ))
        ),
        row!(
            text("Powered").width(Length::Fill),
            oxiced::widgets::oxi_toggler::toggler(adapter.powered).on_toggle(move |value| wrap(
                BluetoothMsg::SetBluetoothAdapterDiscoverability(path1.clone(), value)
            ))
        ),
        row!(
            text("Discoverable").width(Length::Fill),
            oxiced::widgets::oxi_toggler::toggler(adapter.discoverable).on_toggle(move |value| {
                wrap(BluetoothMsg::SetBluetoothAdapterDiscoverability(
                    path2.clone(),
                    value,
                ))
            })
        ),
        row!(
            text("Pairable").width(Length::Fill),
            oxiced::widgets::oxi_toggler::toggler(adapter.pairable).on_toggle(move |value| wrap(
                BluetoothMsg::SetBluetoothAdapterDiscoverability(path3.clone(), value)
            ))
        )
    )
    .spacing(10)
    .padding(10)
    .width(Length::Fill);
    container(col).style(rounded_card).into()
}

pub fn bluetooth_adapter_view<'a>(
    default_adapter: &BluetoothAdapter,
    adapters: &Vec<&BluetoothAdapter>,
) -> Element<'a, ReSetMessage> {
    // TODO add adapter picker
    let views: Vec<Element<'a, ReSetMessage>> = adapters
        .iter()
        .enumerate()
        .map(|(index, adapter)| {
            let is_default = adapter.path == default_adapter.path;
            card_view(index, is_default, adapter)
        })
        .collect();
    iced::widget::Column::with_children(views).into()
}
