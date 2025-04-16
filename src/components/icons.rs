use enum_stringify::EnumStringify;
use oxiced::widgets::oxi_svg::SvgStyleVariant;

#[derive(EnumStringify)]
pub enum Icon {
    // Audio
    Audio,
    AudioDevices,
    AudioCards,
    MicMuted,
    Mic,
    Volume,
    VolumeMuted,
    // Wifi
    // Also is Wifi3Bar
    Wifi,
    Wifi1Bar,
    Wifi2Bar,
    Wifi3Bar,
    Wifi4Bar,
    Wifi1BarLocked,
    Wifi2BarLocked,
    Wifi3BarLocked,
    Wifi4BarLocked,
    WifiDisabled,
    WifiSettings,
    // Bluetooth
    Bluetooth,
    BluetoothConnected,
    BluetoothDisabled,
    // General
    ChevronLeft,
    ChevronRight,
    Refresh,
    SidebarOpen,
    SidebarClose,
    Exit,
}

impl From<Icon> for String {
    fn from(icon: Icon) -> Self {
        path(icon)
    }
}

fn path(icon: Icon) -> String {
    format!("./assets/{}.svg", icon)
}

pub fn icon_widget<'a>(icon: Icon) -> iced::widget::Svg<'a> {
    oxiced::widgets::oxi_svg::svg_from_path(SvgStyleVariant::Primary, path(icon))
}

pub fn icon_widget_from_plain_path<'a>(plain_path: impl Into<String>) -> iced::widget::Svg<'a> {
    oxiced::widgets::oxi_svg::svg_from_path(SvgStyleVariant::Primary, plain_path.into())
}
