use std::{collections::HashMap, error::Error, sync::Arc};

use iced::{
    futures::{channel::mpsc::Sender, SinkExt, StreamExt},
    widget::Column,
    Element, Length, Task,
};
use re_set_lib::utils::error::ReSetError;
use zbus::{proxy::SignalStream, zvariant::OwnedObjectPath, Connection};

use crate::{
    bluetooth::dbus_interface::TPath,
    components::{text::title, wifi_card::WifiCard},
    utils::{to_object_map, TPage},
    ReSetMessage,
};

use super::{
    dbus_interface::{AccessPoint, WifiDbusProxy, WifiDevice},
    network_impl::NetworkMsg,
};

#[derive(Debug)]
pub struct WirelessModel<'a> {
    proxy: Arc<WifiDbusProxy<'a>>,
    new_access_points: HashMap<OwnedObjectPath, AccessPoint>,
    known_access_points: HashMap<OwnedObjectPath, AccessPoint>,
    current_wifi_device: WifiDevice,
    wifi_devices: HashMap<OwnedObjectPath, WifiDevice>,
    enabled: bool,
}

impl<'a> TPage<WirelessMsg, WirelessModel<'a>, ()> for WirelessModel<'a> {
    fn enter() -> Task<ReSetMessage> {
        Task::done(ReSetMessage::SubMsgNetwork(NetworkMsg::SubMsgWireless(
            WirelessMsg::StartNetworkListener,
        )))
    }

    fn leave() -> Task<ReSetMessage> {
        Task::done(ReSetMessage::SubMsgNetwork(NetworkMsg::SubMsgWireless(
            WirelessMsg::StopNetworkListener,
        )))
    }

    async fn new(ctx: &Connection, _: ()) -> Result<Self, ReSetError> {
        let proxy = Arc::new(
            create_network_proxy(ctx)
                .await
                .expect("Could not create proxy for network"),
        ); // TODO beforepr expect
        let access_points = proxy.list_access_points().await?;
        let new_access_point_vec: Vec<AccessPoint> = access_points
            .clone()
            .into_iter()
            .filter(|ap| !ap.stored)
            .collect();
        let new_access_points = to_object_map(new_access_point_vec);
        let known_access_point_vec: Vec<AccessPoint> =
            access_points.into_iter().filter(|ap| ap.stored).collect();
        let known_access_points = to_object_map(known_access_point_vec);
        let current_wifi_device = proxy.get_current_wifi_device().await?;
        let wifi_devices = to_object_map(proxy.get_all_wifi_devices().await?);
        let enabled = proxy.get_wifi_status().await?;
        Ok(Self {
            proxy,
            new_access_points,
            known_access_points,
            current_wifi_device,
            wifi_devices,
            enabled,
        })
    }

    async fn update(&mut self, msg: WirelessMsg) -> Option<Task<ReSetMessage>> {
        match msg {
            WirelessMsg::ListAccessPoints => {
                let access_points = self.proxy.list_access_points().await.ok()?;
                let access_point_vec = access_points
                    .clone()
                    .into_iter()
                    .filter(|ap| !ap.stored)
                    .collect();
                self.new_access_points = to_object_map(access_point_vec);
                self.known_access_points =
                    to_object_map(access_points.into_iter().filter(|ap| ap.stored).collect());
            }
            WirelessMsg::GetWifiStatus => {
                self.enabled = self.proxy.get_wifi_status().await.ok()?;
            }
            WirelessMsg::SetWifiEnabled(enabled) => {
                let _ = self.proxy.set_wifi_enabled(enabled).await.ok()?;
            }
            WirelessMsg::GetCurrentWifiDevice => {
                self.current_wifi_device = self.proxy.get_current_wifi_device().await.ok()?;
            }
            WirelessMsg::GetAllWifiDevices => {
                self.wifi_devices = to_object_map(self.proxy.get_all_wifi_devices().await.ok()?);
            }
            WirelessMsg::SetWifiDevice(owned_object_path) => {
                let _ = self.proxy.set_wifi_device(owned_object_path).await.ok()?;
            }
            WirelessMsg::ConnectToKnownAccessPoint(access_point) => {
                let _ = self
                    .proxy
                    .connect_to_known_access_point(access_point)
                    .await
                    .ok()?;
            }
            WirelessMsg::ConnectToNewAccessPoint(access_point, password) => {
                let _ = self
                    .proxy
                    .connect_to_new_access_point(access_point, password)
                    .await
                    .ok()?;
            }
            WirelessMsg::DisconnectFromCurrentAccessPoint => {
                let _ = self
                    .proxy
                    .disconnet_from_current_access_point()
                    .await
                    .ok()?;
            }
            WirelessMsg::DeleteConnection(owned_object_path) => {
                let _ = self.proxy.delete_connection(owned_object_path).await.ok()?;
            }
            WirelessMsg::StartNetworkListener => {
                let _ = self.proxy.start_network_listener().await.ok()?;
            }
            WirelessMsg::StopNetworkListener => {
                let _ = self.proxy.stop_network_listener().await.ok()?;
            }
            // TODO finish
            WirelessMsg::EditPassword => (),
            WirelessMsg::AccessPointChangedOrAdded(access_point) => {
                let path = access_point.path();
                if access_point.stored {
                    self.new_access_points.remove(&path);
                    self.known_access_points.insert(path, access_point);
                } else {
                    self.known_access_points.remove(&path);
                    self.new_access_points.insert(path, access_point);
                }
            }
            WirelessMsg::AccessPointRemoved(owned_object_path) => {
                self.known_access_points.remove(&owned_object_path);
                self.new_access_points.remove(&owned_object_path);
            }
            WirelessMsg::WifiDeviceChanged(wifi_device) => {
                self.wifi_devices.insert(wifi_device.path(), wifi_device);
            }
            WirelessMsg::WifiEditModal(ssid, is_open) => {
                if let Some(ac) = self.known_access_points.get_mut(&ssid) {
                    ac.modal_open = is_open;
                    return None;
                }
                if let Some(ac) = self.new_access_points.get_mut(&ssid) {
                    ac.modal_open = is_open;
                    return None;
                }
            }
            WirelessMsg::WifiEditPasswordText(ssid, password) => {
                if let Some(ac) = self.known_access_points.get_mut(&ssid) {
                    ac.current_password = password.clone();
                    return None;
                }
                if let Some(ac) = self.new_access_points.get_mut(&ssid) {
                    ac.current_password = password.clone();
                    return None;
                }
            }
        };
        None
    }

    fn view(&self) -> Result<Vec<Element<'_, ReSetMessage>>, ReSetError> {
        let new_ap_cards: Vec<WifiCard<'_, ReSetMessage>> = self
            .new_access_points
            .values()
            .map(|access_point| {
                WifiCard::new(
                    access_point,
                    // TODO should not have edit
                    WirelessMsg::WifiEditModal(access_point.path(), !access_point.modal_open)
                        .into(),
                    WirelessMsg::ConnectToNewAccessPoint(
                        access_point.clone(),
                        access_point.current_password.clone(),
                    )
                    .into(),
                    WirelessMsg::WifiEditModal(access_point.path(), !access_point.modal_open)
                        .into(),
                )
            })
            .collect();
        let known_ap_cards: Vec<WifiCard<'_, ReSetMessage>> = self
            .known_access_points
            .values()
            .map(|access_point| {
                let is_connected =
                    access_point.ssid == self.current_wifi_device.active_access_point;
                WifiCard::new(
                    access_point,
                    WirelessMsg::WifiEditModal(access_point.path(), !access_point.modal_open)
                        .into(),
                    WirelessMsg::WifiEditPasswordText(
                        access_point.path(),
                        access_point.current_password.clone(),
                    )
                    .into(),
                    if is_connected {
                        WirelessMsg::DisconnectFromCurrentAccessPoint.into()
                    } else {
                        WirelessMsg::ConnectToKnownAccessPoint(access_point.clone()).into()
                    },
                )
            })
            .collect();
        let mut col = Column::new();
        col = col.push(title("Access Points"));
        for card in new_ap_cards {
            col = col.push(card.view());
        }
        col = col.push(title("Known Access Points"));
        for card in known_ap_cards {
            col = col.push(card.view());
        }
        Ok(vec![col.padding(10).spacing(10).width(Length::Fill).into()])
    }
}

#[derive(Debug, Clone)]
pub enum WirelessMsg {
    ListAccessPoints,
    GetWifiStatus,
    SetWifiEnabled(bool),
    GetCurrentWifiDevice,
    GetAllWifiDevices,
    SetWifiDevice(OwnedObjectPath),
    ConnectToKnownAccessPoint(AccessPoint),
    ConnectToNewAccessPoint(AccessPoint, String),
    DisconnectFromCurrentAccessPoint,
    DeleteConnection(OwnedObjectPath),
    StartNetworkListener,
    StopNetworkListener,
    EditPassword,
    AccessPointChangedOrAdded(AccessPoint),
    AccessPointRemoved(OwnedObjectPath),
    WifiDeviceChanged(WifiDevice),
    WifiEditModal(OwnedObjectPath, bool),
    WifiEditPasswordText(OwnedObjectPath, String),
}

impl From<WirelessMsg> for ReSetMessage {
    fn from(val: WirelessMsg) -> Self {
        ReSetMessage::SubMsgNetwork(NetworkMsg::SubMsgWireless(val))
    }
}

async fn create_network_proxy(ctx: &Connection) -> Result<WifiDbusProxy<'static>, Box<dyn Error>> {
    let proxy = WifiDbusProxy::new(ctx).await?;
    Ok(proxy)
}

pub async fn watch_wireless_dbus_signals(
    sender: &mut Sender<ReSetMessage>,
    signals: &mut SignalStream<'_>,
) -> Result<(), ReSetError> {
    if let Some(msg) = signals.next().await {
        match msg.header().member().unwrap().to_string().as_str() {
            "AccessPointChanged" | "AccessPointAdded" => {
                let obj: AccessPoint = msg.body().deserialize()?;
                let _res = sender
                    .send(WirelessMsg::AccessPointChangedOrAdded(obj).into())
                    .await;
            }
            "AccessPointRemoved" => {
                let obj: OwnedObjectPath = msg.body().deserialize()?;
                let _res = sender
                    .send(WirelessMsg::AccessPointRemoved(obj).into())
                    .await;
            }
            "WifiDeviceChanged" => {
                let obj: WifiDevice = msg.body().deserialize()?;
                let _res = sender
                    .send(WirelessMsg::WifiDeviceChanged(obj).into())
                    .await;
            }
            _ => (),
        }
    }
    Ok(())
}
