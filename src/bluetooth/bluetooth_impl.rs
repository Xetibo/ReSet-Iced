use std::{
    collections::HashMap,
    sync::{atomic::AtomicU8, Arc},
    thread,
    time::Duration,
};

use iced::{
    futures::{channel::mpsc::Sender, SinkExt, StreamExt},
    widget::{column, row, text},
    Element, Length, Task,
};
use oxiced::widgets::oxi_button::ButtonVariant;
use zbus::{zvariant::OwnedObjectPath, Connection, Proxy};

use crate::{
    components::{
        easing::STANDARD,
        icons::{icon_widget, Icon},
        loading_spinner::Circular,
    },
    utils::TToError,
    PageId, ReSetMessage,
};

use super::{
    bluetooth_card::{bluetooth_adapter_view, bluetooth_device_buttons, BluetoothButtonVariant},
    dbus_interface::{BluetoothAdapter, BluetoothDbusProxy, BluetoothDevice, TPath},
};

pub struct BluetoothModel<'a> {
    proxy: Arc<BluetoothDbusProxy<'a>>,
    current_adapter: BluetoothAdapter,
    adapters: HashMap<zbus::zvariant::OwnedObjectPath, BluetoothAdapter>,
    devices: HashMap<zbus::zvariant::OwnedObjectPath, BluetoothDevice>,
    page_id: BluetoothPageId,
    is_scanning: bool,
}

#[derive(Default, Debug, Clone)]
pub enum BluetoothPageId {
    #[default]
    Devices,
    Adapter,
}

#[derive(Default, Debug, Clone)]
pub enum BluetoothMsg {
    GG,
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

// This sucks
pub async fn watch_bluetooth_dbus_signals(
    sender: &mut Sender<ReSetMessage>,
    conn: Arc<Connection>,
    current_page_id: Arc<AtomicU8>,
) -> Result<(), zbus::Error> {
    let proxy = BluetoothDbusProxy::new(&conn).await.expect("no proxy");
    let mut signals = Proxy::receive_all_signals(&proxy.into_inner()).await?;
    loop {
        if current_page_id.load(std::sync::atomic::Ordering::SeqCst) != PageId::Bluetooth.into() {
            sender.send(wrap(BluetoothMsg::GG)).await;
            break;
        } else {
            sender.send(wrap(BluetoothMsg::GG)).await;
        }
        if let Some(msg) = signals.next().await {
            match msg.header().member().unwrap().to_string().as_str() {
                "BluetoothDeviceAdded" | "BluetoothDeviceChanged" => {
                    let obj: BluetoothDevice = msg.body().deserialize()?;
                    let _res = sender
                        .send(wrap(BluetoothMsg::AddBluetoothDevice(obj)))
                        .await;
                }
                "BluetoothDeviceRemoved" => {
                    let obj: zbus::zvariant::OwnedObjectPath = msg.body().deserialize()?;
                    let _res = sender
                        .send(wrap(BluetoothMsg::RemoveBluetoothDevice(obj)))
                        .await;
                }
                _ => (),
            }
        }
    }
    Ok(())
}

fn wrap(msg: BluetoothMsg) -> ReSetMessage {
    ReSetMessage::SubMsgBluetooth(msg)
}

fn to_map<T>(elements: Vec<T>) -> HashMap<OwnedObjectPath, T>
where
    T: TPath,
{
    let mut map = HashMap::new();
    for element in elements.into_iter() {
        map.insert(element.path(), element);
    }
    map
}

impl<'a> BluetoothModel<'a> {
    pub async fn new(conn: &zbus::Connection) -> Result<Self, zbus::Error> {
        let proxy = Arc::new(BluetoothDbusProxy::new(conn).await?);
        let current_adapter = proxy.get_current_bluetooth_adapter().await?;
        let adapters = to_map(proxy.get_bluetooth_adapters().await?);
        let devices = to_map(proxy.get_bluetooth_devices().await?);
        Ok(Self {
            proxy,
            current_adapter,
            adapters,
            devices,
            page_id: Default::default(),
            is_scanning: false,
        })
    }

    pub async fn update(&mut self, msg: BluetoothMsg) -> Result<Task<ReSetMessage>, zbus::Error> {
        let task = match msg {
            BluetoothMsg::GG => {
                println!("whakasdfksdajf");
                Task::none()
            }
            BluetoothMsg::GetBluetoothAdapters => {
                let adapters = self.proxy.get_bluetooth_adapters().await?;
                self.adapters = to_map(adapters);
                Task::none()
            }
            BluetoothMsg::StartBluetoothListener => {
                self.is_scanning = true;
                // is async in order to not block
                self.proxy.start_bluetooth_listener().await?;
                let func = async || -> ReSetMessage {
                    thread::sleep(Duration::from_millis(10_000));
                    wrap(BluetoothMsg::StopBluetoothScan)
                };
                Task::future(func())
            }
            BluetoothMsg::StopBluetoothListener => {
                self.proxy.stop_bluetooth_listener().await?;
                self.is_scanning = false;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapter(adapter) => {
                self.proxy.set_bluetooth_adapter(adapter).await?;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapterEnabled(adapter, enabled) => {
                self.proxy
                    .set_bluetooth_adapter_enabled(adapter, enabled)
                    .await?;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapterDiscoverability(adapter, discoverability) => {
                self.proxy
                    .set_bluetooth_adapter_discoverability(adapter, discoverability)
                    .await?;
                Task::none()
            }
            BluetoothMsg::SetBluetoothAdapterPairability(adapter, pairability) => {
                self.proxy
                    .set_bluetooth_adapter_pairability(adapter, pairability)
                    .await?;
                Task::none()
            }
            BluetoothMsg::ConnectToBluetoothDevice(device) => {
                self.devices
                    .get_mut(&device)
                    .to_zbus_error()?
                    .conect_in_progress = true;
                self.proxy.connect_to_bluetooth_device(device).await?;
                Task::none()
            }
            BluetoothMsg::DisconnectFromBluetoothDevice(device) => {
                self.devices
                    .get_mut(&device)
                    .to_zbus_error()?
                    .conect_in_progress = true;
                self.proxy.disconnect_from_bluetooth_device(device).await?;
                Task::none()
            }
            BluetoothMsg::RemoveDevicePairing(device) => {
                self.proxy.remove_device_pairing(device).await?;
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
                self.proxy.start_bluetooth_scan().await?;
                self.is_scanning = true;
                Task::none()
            }
            BluetoothMsg::StopBluetoothScan => {
                self.proxy.stop_bluetooth_scan().await?;
                self.is_scanning = false;
                Task::none()
            }
        };
        Ok(task)
    }

    pub fn view(&self) -> Element<ReSetMessage> {
        let devices = column!(
            oxiced::widgets::oxi_button::button(
                row!(
                    text("Adapters").width(Length::Fill).size(20),
                    icon_widget(Icon::ChevronRight).width(Length::Shrink)
                )
                .width(Length::Fill),
                ButtonVariant::RowEntry
            )
            .on_press(wrap(BluetoothMsg::SetPageId(BluetoothPageId::Adapter)))
            .width(Length::Fill),
            oxiced::widgets::oxi_button::button(
                row!(
                    text(if self.is_scanning {
                        "Scanning"
                    } else {
                        "Start scan"
                    })
                    .width(Length::Fill)
                    .size(20),
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
                Some(wrap(BluetoothMsg::StartBluetoothScan))
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
                    text("Devices").width(Length::Fill).size(20),
                    icon_widget(Icon::ChevronLeft).width(Length::Shrink)
                )
                .width(Length::Fill),
                ButtonVariant::RowEntry
            )
            .on_press(wrap(BluetoothMsg::SetPageId(BluetoothPageId::Devices)))
            .width(Length::Fill),
            bluetooth_adapter_view(&self.current_adapter, &self.adapters.values().collect())
        )
        .padding(20)
        .spacing(30);
        match self.page_id {
            BluetoothPageId::Devices => devices.into(),
            BluetoothPageId::Adapter => adapter.into(),
        }
    }
}
