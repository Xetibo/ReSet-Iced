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
    WifiDisabled,
    WifiSettings,
    // Bluetooth
    Bluetooth,
    BluetoothConnected,
    BluetoothDisabled,
    // General
    ChevronLeft,
    ChevronRight,
}

fn path(icon: Icon) -> String {
    format!("./assets/{}.svg", icon)
}

pub fn icon_widget<'a>(icon: Icon) -> iced::widget::Svg<'a> {
    oxiced::widgets::oxi_svg::svg_from_path(SvgStyleVariant::Primary, path(icon))
}
