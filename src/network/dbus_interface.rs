use serde::{Deserialize, Serialize};
use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, Type},
};

use crate::bluetooth::dbus_interface::TPath;

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct WifiDevice {
    pub path: OwnedObjectPath,
    pub name: String,
    pub active_access_point: Vec<u8>,
}

impl TPath for WifiDevice {
    fn path(&self) -> zbus::zvariant::OwnedObjectPath {
        self.path.clone()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type, PartialEq, Eq, Hash)]
#[zvariant(signature = "(ayyoob)")]
pub struct AccessPoint {
    pub ssid: Vec<u8>,
    pub strength: u8,
    pub associated_connection: OwnedObjectPath,
    pub path: OwnedObjectPath,
    pub stored: bool,
    // Internal state, not sent or received
    // Always set to false
    #[zvariant(signature = "")]
    #[serde(skip_serializing, default)]
    pub modal_open: bool,
    #[zvariant(signature = "")]
    #[serde(skip_serializing, default)]
    pub current_password: String,
}

impl TPath for AccessPoint {
    fn path(&self) -> zbus::zvariant::OwnedObjectPath {
        self.path.clone()
    }
}

// TODO beforepr finish and put in lib
#[proxy(
    default_service = "org.Xetibo.ReSet.Daemon",
    default_path = "/org/Xetibo/ReSet/Daemon",
    interface = "org.Xetibo.ReSet.Network"
)]
pub trait WifiDbus {
    fn list_access_points(&self) -> zbus::Result<Vec<AccessPoint>>;
    fn get_wifi_status(&self) -> zbus::Result<bool>;
    fn set_wifi_enabled(&self, enabled: bool) -> zbus::Result<bool>;
    fn get_current_wifi_device(&self) -> zbus::Result<WifiDevice>;
    fn get_all_wifi_devices(&self) -> zbus::Result<Vec<WifiDevice>>;
    fn set_wifi_device(&self, path: OwnedObjectPath) -> zbus::Result<bool>;
    fn connect_to_known_access_point(&self, access_point: AccessPoint) -> zbus::Result<bool>;
    fn connect_to_new_access_point(
        &self,
        access_point: AccessPoint,
        password: String,
    ) -> zbus::Result<bool>;
    fn disconnet_from_current_access_point(&self) -> zbus::Result<bool>;
    //fn get_connection_settings(path: OwnedObjectPath) -> zbus::Result<bool>; // Hashmap<String,
    //Refarg?????>
    //fn get_connection_settings(path: OwnedObjectPath) -> zbus::Result<bool>; // Hashmap<String,
    fn delete_connection(&self, path: OwnedObjectPath) -> zbus::Result<bool>;
    fn start_network_listener(&self) -> zbus::Result<bool>;
    fn stop_network_listener(&self) -> zbus::Result<bool>;
}
