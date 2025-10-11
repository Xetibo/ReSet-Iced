use iced::{
    alignment::{Horizontal, Vertical},
    widget::{column, row, text},
    Alignment, Element,
    Length::{self},
};
use oxiced::widgets::{
    oxi_button::{self, ButtonVariant},
    oxi_text_input,
};

use crate::{
    bluetooth::dbus_interface::TPath,
    network::{dbus_interface::AccessPoint, network_impl::NetworkMsg, wireless_impl::WirelessMsg},
    utils::OxiPadding,
    ReSetMessage,
};

use super::{
    icons::{icon_widget, Icon},
    rowbutton::{self, RowbuttonPosition},
    text::content_text,
};

const WIFI1: u8 = u8::MAX / 4;
const WIFI2: u8 = u8::MAX / 4 * 2;
const WIFI3: u8 = u8::MAX / 4 * 3;

pub struct WifiCard<'a, Message> {
    access_point: &'a AccessPoint,
    edit_msg: Message,
    edit_confirm_msg: Message,
    connection_msg: Message,
}

impl<'a> WifiCard<'a, ReSetMessage> {
    pub fn new(
        access_point: &'a AccessPoint,
        edit_msg: ReSetMessage,
        edit_confirm_msg: ReSetMessage,
        connection_msg: ReSetMessage,
    ) -> Self {
        Self {
            access_point,
            edit_msg,
            edit_confirm_msg,
            connection_msg,
        }
    }

    pub fn view(self) -> Option<Element<'a, ReSetMessage>> {
        let icon = match self.access_point.strength {
            0..WIFI1 => Icon::Wifi1Bar,
            WIFI1..WIFI2 => Icon::Wifi2Bar,
            WIFI2..WIFI3 => Icon::Wifi3Bar,
            WIFI3..=u8::MAX => Icon::Wifi4Bar,
        };
        let svg = icon_widget(icon);
        let name = String::from_utf8(self.access_point.ssid.clone()).ok()?;
        if name.is_empty() {
            return None;
        }
        let wifi_content: Element<'_, ReSetMessage> = if self.access_point.modal_open {
            row!(
                oxi_text_input::text_input(
                    "Password",
                    &self.access_point.current_password,
                    move |password| {
                        ReSetMessage::SubMsgNetwork(NetworkMsg::SubMsgWireless(
                            WirelessMsg::WifiEditPasswordText(self.access_point.path(), password),
                        ))
                    }
                )
                .on_submit(self.edit_confirm_msg.clone()),
                oxi_button::button(text("Confirm"), ButtonVariant::Neutral)
                    .on_press(self.edit_confirm_msg)
            )
            .spacing(OxiPadding::Medium)
            .align_y(Alignment::Center)
            .height(OxiPadding::XLarge)
            .into()
        } else if self.access_point.stored {
            row!(
                oxi_button::button(icon_widget(Icon::WifiSettings), ButtonVariant::Neutral)
                    .on_press(self.edit_msg)
            )
            .height(OxiPadding::XLarge)
            .into()
        } else {
            row!()
                .width(OxiPadding::None)
                .height(OxiPadding::XLarge)
                .into()
        };
        Some(
            oxi_button::button(
                column!(
                    row!(svg.width(Length::Shrink), content_text(name), wifi_content)
                        .padding(OxiPadding::Small)
                        .spacing(OxiPadding::Medium)
                        .width(Length::Fill)
                        .align_y(Vertical::Center)
                )
                .align_x(Horizontal::Left),
                ButtonVariant::Neutral,
            )
            .on_press(self.connection_msg)
            .style(|theme, state| {
                rowbutton::style(
                    oxiced::widgets::oxi_button::neutral_button(theme, state),
                    RowbuttonPosition::Only,
                )
            })
            .into(),
        )
    }
}
