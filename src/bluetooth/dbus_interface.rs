use serde::{Deserialize, Serialize};
use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, Type},
};

pub trait TPath {
    fn path(&self) -> zbus::zvariant::OwnedObjectPath;
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
#[zvariant(signature = "(onssobbbbbss)")]
pub struct BluetoothDevice {
    pub path: OwnedObjectPath,
    pub rssi: i16,
    pub alias: String,
    pub name: String,
    pub adapter: OwnedObjectPath,
    pub trusted: bool,
    pub bonded: bool,
    pub paired: bool,
    pub blocked: bool,
    pub connected: bool,
    pub icon: String,
    pub address: String,
    // Internal state, not sent or received
    // Always set to false
    #[zvariant(signature = "")]
    #[serde(skip_serializing, default)]
    pub conect_in_progress: bool,
}

impl TPath for BluetoothDevice {
    fn path(&self) -> zbus::zvariant::OwnedObjectPath {
        self.path.clone()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct BluetoothAdapter {
    pub path: OwnedObjectPath,
    pub alias: String,
    pub powered: bool,
    pub discoverable: bool,
    pub pairable: bool,
}

impl TPath for BluetoothAdapter {
    fn path(&self) -> zbus::zvariant::OwnedObjectPath {
        self.path.clone()
    }
}

// TODO beforepr finish and put in lib
#[proxy(
    default_service = "org.Xetibo.ReSet.Daemon",
    default_path = "/org/Xetibo/ReSet/Daemon",
    interface = "org.Xetibo.ReSet.Bluetooth"
)]
pub trait BluetoothDbus {
    fn start_bluetooth_scan(&self) -> zbus::Result<()>;
    fn stop_bluetooth_scan(&self) -> zbus::Result<()>;
    fn start_bluetooth_listener(&self) -> zbus::Result<()>;
    fn stop_bluetooth_listener(&self) -> zbus::Result<()>;
    fn get_bluetooth_adapters(&self) -> zbus::Result<Vec<BluetoothAdapter>>;
    fn get_current_bluetooth_adapter(&self) -> zbus::Result<BluetoothAdapter>;
    fn set_bluetooth_adapter(&self, obj: OwnedObjectPath) -> zbus::Result<bool>;
    fn set_bluetooth_adapter_enabled(
        &self,
        obj: OwnedObjectPath,
        pariable: bool,
    ) -> zbus::Result<bool>;
    fn set_bluetooth_adapter_discoverability(
        &self,
        obj: OwnedObjectPath,
        pariable: bool,
    ) -> zbus::Result<bool>;
    fn set_bluetooth_adapter_pairability(
        &self,
        obj: OwnedObjectPath,
        pariable: bool,
    ) -> zbus::Result<bool>;
    fn get_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDevice>>;
    fn connect_to_bluetooth_device(&self, obj: OwnedObjectPath) -> zbus::Result<bool>;
    fn disconnect_from_bluetooth_device(&self, obj: OwnedObjectPath) -> zbus::Result<bool>;
    fn remove_device_pairing(&self, obj: OwnedObjectPath) -> zbus::Result<bool>;
    fn get_connected_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDevice>>;
}
