use serde::{Deserialize, Serialize};
use zbus::{proxy, zvariant::Type};

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct AudioSink {
    pub index: u32,
    pub name: String,
    pub alias: String,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub active: i32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct AudioSource {
    pub index: u32,
    pub name: String,
    pub alias: String,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub active: i32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct InputStream {
    pub index: u32,
    pub name: String,
    pub application_name: String,
    pub sink_index: u32,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub corked: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct OutputStream {
    pub index: u32,
    pub name: String,
    pub application_name: String,
    pub sink_index: u32,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub corked: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct Card {
    pub index: u32,
    pub name: String,
    pub profiles: Vec<CardProfile>,
    pub active_profile: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct CardProfile {
    pub name: String,
    pub description: String,
    pub available: bool,
}

// TODO beforepr finish this
// TODO beforepr this needs to be put into the lib as the type cant be reused
#[proxy(
    default_service = "org.Xetibo.ReSet.Daemon",
    default_path = "/org/Xetibo/ReSet/Daemon",
    interface = "org.Xetibo.ReSet.Audio"
)]
pub trait AudioDbus {
    #[zbus(signal)]
    fn sink_changed(&self, sink: AudioSink) -> zbus::Result<()>;
    #[zbus(signal)]
    fn sink_added(&self, sink: AudioSink) -> zbus::Result<()>;
    #[zbus(signal)]
    fn sink_removed(&self, index: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn source_changed(&self, sink: AudioSource) -> zbus::Result<()>;
    #[zbus(signal)]
    fn source_added(&self, sink: AudioSource) -> zbus::Result<()>;
    #[zbus(signal)]
    fn source_removed(&self, index: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn input_stream_changed(&self, sink: InputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn input_stream_added(&self, sink: InputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn input_stream_removed(&self, index: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn output_stream_changed(&self, sink: OutputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn output_stream_added(&self, sink: OutputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn output_stream_removed(&self, index: u32) -> zbus::Result<()>;

    fn list_sinks(&self) -> zbus::Result<Vec<AudioSink>>;
    fn get_default_sink(&self) -> zbus::Result<AudioSink>;
    fn get_default_sink_name(&self) -> zbus::Result<String>;
    fn set_sink_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_sink_mute(&self, index: u32, muted: bool) -> zbus::Result<()>;
    fn set_default_sink(&self, sink: AudioSink) -> zbus::Result<AudioSink>;

    fn list_sources(&self) -> zbus::Result<Vec<AudioSource>>;
    fn get_default_source(&self) -> zbus::Result<AudioSource>;
    fn get_default_source_name(&self) -> zbus::Result<String>;
    fn set_source_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_source_mute(&self, index: u32, muted: bool) -> zbus::Result<()>;
    fn set_default_source(&self, sink: AudioSource) -> zbus::Result<AudioSource>;

    fn list_input_streams(&self) -> zbus::Result<Vec<InputStream>>;
    fn set_sink_of_input_stream(
        &self,
        input_stream: InputStream,
        sink: AudioSink,
    ) -> zbus::Result<()>;
    fn set_input_stream_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_input_stream_mute(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;

    fn list_output_streams(&self) -> zbus::Result<Vec<OutputStream>>;
    fn set_source_of_output_stream(
        &self,
        output_stream: OutputStream,
        source: AudioSource,
    ) -> zbus::Result<()>;
    fn set_output_stream_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_output_stream_mute(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;

    fn list_cards(&self) -> zbus::Result<Vec<Card>>;
    fn set_card_profile_of_device(
        &self,
        device_index: u32,
        profile_name: String,
    ) -> zbus::Result<()>;
}
