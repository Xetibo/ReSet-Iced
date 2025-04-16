use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use iced::{
    futures::{channel::mpsc::Sender, SinkExt, StreamExt},
    widget::{column, row},
    Element, Length, Task,
};
use oxiced::widgets::oxi_button::ButtonVariant;
use re_set_lib::utils::error::ReSetError;
use zbus::proxy::SignalStream;

use crate::{
    components::{
        easing::STANDARD,
        icons::{icon_widget, Icon},
        loading_spinner::Circular,
        text::{subtitle, title},
    },
    utils::{to_object_map, TPage},
    ReSetMessage,
};

use super::{
    bluetooth_card::{bluetooth_adapter_view, bluetooth_device_buttons, BluetoothButtonVariant},
    dbus_interface::{BluetoothAdapter, BluetoothDbusProxy, BluetoothDevice, TPath},
};

#[derive(Debug)]
pub struct BluetoothModel<'a> {
    proxy: Arc<BluetoothDbusProxy<'a>>,
    current_adapter: BluetoothAdapter,
    adapters: HashMap<zbus::zvariant::OwnedObjectPath, BluetoothAdapter>,
    devices: HashMap<zbus::zvariant::OwnedObjectPath, BluetoothDevice>,
    page_id: BluetoothPageId,
    is_scanning: bool,
}

impl<'a> TPage<BluetoothMsg, BluetoothModel<'a>, ()> for BluetoothModel<'a> {
    fn enter() -> Task<ReSetMessage> {
        Task::done(ReSetMessage::SubMsgBluetooth(
            BluetoothMsg::StartBluetoothListener,
        ))
    }

    fn leave() -> Task<ReSetMessage> {
        Task::done(ReSetMessage::SubMsgBluetooth(
            BluetoothMsg::StopBluetoothListener,
        ))
    }

    async fn update(&mut self, msg: BluetoothMsg) -> Option<Task<ReSetMessage>> {
        let task = match msg {
            BluetoothMsg::GetBluetoothAdapters => {
                let adapters = self.proxy.get_bluetooth_adapters().await.ok()?;
                self.adapters = to_object_map(adapters);
                Task::none()
            }
            BluetoothMsg::StartBluetoothListener => {
                self.is_scanning = true;
                // is async in order to not block
                self.proxy.start_bluetooth_listener().await.ok()?;
                let func = async || -> ReSetMessage {
                    thread::sleep(Duration::from_millis(10_000));
                    BluetoothMsg::StopBluetoothScan.into()
                };
                Task::future(func())
            }
            BluetoothMsg::StopBluetoothListener => {
                self.proxy.stop_bluetooth_listener().await.ok()?;
                self.is_scanning = false;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapter(adapter) => {
                self.proxy.set_bluetooth_adapter(adapter).await.ok()?;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapterEnabled(adapter, enabled) => {
                self.proxy
                    .set_bluetooth_adapter_enabled(adapter, enabled)
                    .await
                    .ok()?;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapterDiscoverability(adapter, discoverability) => {
                self.proxy
                    .set_bluetooth_adapter_discoverability(adapter, discoverability)
                    .await
                    .ok()?;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapterPairability(adapter, pairability) => {
                self.proxy
                    .set_bluetooth_adapter_pairability(adapter, pairability)
                    .await
                    .ok()?;
                Task::none()
            }
            BluetoothMsg::ConnectToBluetoothDevice(device) => {
                self.devices.get_mut(&device)?.conect_in_progress = true;
                self.proxy.connect_to_bluetooth_device(device).await.ok()?;
                Task::none()
            }
            BluetoothMsg::DisconnectFromBluetoothDevice(device) => {
                self.devices.get_mut(&device)?.conect_in_progress = true;
                self.proxy
                    .disconnect_from_bluetooth_device(device)
                    .await
                    .ok()?;
                Task::none()
            }
            BluetoothMsg::RemoveDevicePairing(device) => {
                self.proxy.remove_device_pairing(device).await.ok()?;
                Task::none()
            }
            BluetoothMsg::AddBluetoothDevice(bluetooth_device) => {
                self.devices
                    .insert(bluetooth_device.path(), bluetooth_device);
                Task::none()
            }
            BluetoothMsg::RemoveBluetoothDevice(device_path) => {
                self.devices.remove(&device_path);
                Task::none()
            }
            BluetoothMsg::SetPageId(page_id) => {
                self.page_id = page_id;
                Task::none()
            }
            BluetoothMsg::StartBluetoothScan => {
                self.proxy.start_bluetooth_scan().await.ok()?;
                self.is_scanning = true;
                Task::none()
            }
            BluetoothMsg::StopBluetoothScan => {
                self.proxy.stop_bluetooth_scan().await.ok()?;
                self.is_scanning = false;
                Task::none()
            }
        };
        Some(task)
    }

    async fn new(conn: &zbus::Connection, _: ()) -> Result<Self, ReSetError> {
        let proxy = Arc::new(BluetoothDbusProxy::new(conn).await?);
        let current_adapter = proxy.get_current_bluetooth_adapter().await?;
        let adapters = to_object_map(proxy.get_bluetooth_adapters().await?);
        let devices = to_object_map(proxy.get_bluetooth_devices().await?);
        Ok(Self {
            proxy,
            current_adapter,
            adapters,
            devices,
            page_id: Default::default(),
            is_scanning: false,
        })
    }

    fn view(&self) -> Result<Element<ReSetMessage>, ReSetError> {
        let devices = column!(
            oxiced::widgets::oxi_button::button(
                row!(
                    title("Adapters"),
                    icon_widget(Icon::ChevronRight).width(Length::Shrink)
                )
                .width(Length::Fill),
                ButtonVariant::RowEntry
            )
            .on_press(BluetoothMsg::SetPageId(BluetoothPageId::Adapter).into())
            .width(Length::Fill),
            oxiced::widgets::oxi_button::button(
                row!(
                    subtitle(if self.is_scanning {
                        "Scanning"
                    } else {
                        "Start scan"
                    }),
                    if self.is_scanning {
                        row!(Circular::new()
                            .easing(&STANDARD)
                            .cycle_duration(Duration::from_millis(3000)))
                    } else {
                        row!(icon_widget(Icon::Refresh).width(Length::Shrink))
                    }
                )
                .width(Length::Fill),
                ButtonVariant::RowEntry
            )
            .on_press_maybe(if self.is_scanning {
                None
            } else {
                Some(BluetoothMsg::StartBluetoothScan.into())
            })
            .width(Length::Fill),
            bluetooth_device_buttons(
                &self
                    .devices
                    .values()
                    .filter(|value| !value.connected)
                    .collect(),
                BluetoothButtonVariant::Connect
            ),
            bluetooth_device_buttons(
                &self
                    .devices
                    .values()
                    .filter(|value| value.connected)
                    .collect(),
                BluetoothButtonVariant::Disconnect
            ),
        )
        .padding(20)
        .spacing(30);
        let adapter = column!(
            oxiced::widgets::oxi_button::button(
                row!(
                    title("Devices"),
                    icon_widget(Icon::ChevronLeft).width(Length::Shrink)
                )
                .width(Length::Fill),
                ButtonVariant::RowEntry
            )
            .on_press(BluetoothMsg::SetPageId(BluetoothPageId::Devices).into())
            .width(Length::Fill),
            bluetooth_adapter_view(&self.current_adapter, &self.adapters.values().collect())
        )
        .padding(20)
        .spacing(30);
        Ok(match self.page_id {
            BluetoothPageId::Devices => devices.into(),
            BluetoothPageId::Adapter => adapter.into(),
        })
    }
}

#[derive(Default, Debug, Clone)]
pub enum BluetoothPageId {
    #[default]
    Devices,
    Adapter,
}

#[derive(Default, Debug, Clone)]
pub enum BluetoothMsg {
    #[default]
    GetBluetoothAdapters,
    StartBluetoothListener,
    StopBluetoothListener, // TODO use when moving away from this page
    StartBluetoothScan,
    StopBluetoothScan,
    SetBluetoothAdapter(zbus::zvariant::OwnedObjectPath),
    SetBluetoothAdapterEnabled(zbus::zvariant::OwnedObjectPath, bool),
    SetBluetoothAdapterDiscoverability(zbus::zvariant::OwnedObjectPath, bool),
    SetBluetoothAdapterPairability(zbus::zvariant::OwnedObjectPath, bool),
    ConnectToBluetoothDevice(zbus::zvariant::OwnedObjectPath),
    DisconnectFromBluetoothDevice(zbus::zvariant::OwnedObjectPath),
    RemoveDevicePairing(zbus::zvariant::OwnedObjectPath), // TODO should this even be done?
    AddBluetoothDevice(BluetoothDevice),
    RemoveBluetoothDevice(zbus::zvariant::OwnedObjectPath),
    SetPageId(BluetoothPageId),
}

impl From<BluetoothMsg> for ReSetMessage {
    fn from(val: BluetoothMsg) -> Self {
        ReSetMessage::SubMsgBluetooth(val)
    }
}

// This sucks
pub async fn watch_bluetooth_dbus_signals(
    sender: &mut Sender<ReSetMessage>,
    signals: &mut SignalStream<'_>,
) -> Result<(), ReSetError> {
    if let Some(msg) = signals.next().await {
        match msg.header().member().unwrap().to_string().as_str() {
            "BluetoothDeviceAdded" | "BluetoothDeviceChanged" => {
                let obj: BluetoothDevice = msg.body().deserialize()?;
                let _res = sender
                    .send(BluetoothMsg::AddBluetoothDevice(obj).into())
                    .await;
            }
            "BluetoothDeviceRemoved" => {
                let obj: zbus::zvariant::OwnedObjectPath = msg.body().deserialize()?;
                let _res = sender
                    .send(BluetoothMsg::RemoveBluetoothDevice(obj).into())
                    .await;
            }
            _ => (),
        }
    }
    Ok(())
}
