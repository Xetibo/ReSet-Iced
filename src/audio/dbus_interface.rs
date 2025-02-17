use std::fmt::Display;

use serde::{Deserialize, Serialize};
use zbus::{proxy, zvariant::Type};

use crate::components::{
    audio_card::{TCardUser, TStreamCardUser},
    icons::Icon,
};

use super::audio_impl::AudioMsg;

pub trait TIndex {
    fn index(&self) -> u32;
}

pub trait TAudioObject {
    fn name(&self) -> String;
    fn alias(&self) -> String;
    fn channels(&self) -> u16;
    fn index(&self) -> u32;
    fn volume(&self) -> Vec<u32>;
    fn muted(&self) -> bool;
    //fn active(&self) -> i32;
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type, PartialEq, Eq)]
pub struct AudioSink {
    pub index: u32,
    pub name: String,
    pub alias: String,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub active: i32,
}

impl TIndex for AudioSink {
    fn index(&self) -> u32 {
        self.index
    }
}

impl TAudioObject for AudioSink {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> String {
        self.alias.clone()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn volume(&self) -> Vec<u32> {
        self.volume.clone()
    }

    fn muted(&self) -> bool {
        self.muted
    }

    //fn active(&self) -> i32 {
    //    self.active
    //}

    fn index(&self) -> u32 {
        self.index
    }
}

impl TCardUser for AudioSink {
    fn volume_fn(index: u32, channels: u16, volume: u32) -> AudioMsg {
        AudioMsg::SetSinkVolume(index, channels, volume)
    }

    fn mute_fn(index: u32, muted: bool) -> AudioMsg {
        AudioMsg::SetSinkMute(index, muted)
    }

    fn default_fn(index: u32) -> AudioMsg {
        AudioMsg::SetDefaultSink(index)
    }

    fn muted_icon() -> Icon {
        Icon::VolumeMuted
    }

    fn unmuted_icon() -> Icon {
        Icon::Volume
    }

    fn title() -> String {
        "Output".to_string()
    }
}

impl Display for AudioSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.alias)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type, PartialEq, Eq)]
pub struct AudioSource {
    pub index: u32,
    pub name: String,
    pub alias: String,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub active: i32,
}

impl TAudioObject for AudioSource {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> String {
        self.alias.clone()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn volume(&self) -> Vec<u32> {
        self.volume.clone()
    }

    fn muted(&self) -> bool {
        self.muted
    }

    //fn active(&self) -> i32 {
    //    self.active
    //}

    fn index(&self) -> u32 {
        self.index
    }
}

impl TCardUser for AudioSource {
    fn volume_fn(index: u32, channels: u16, volume: u32) -> AudioMsg {
        AudioMsg::SetSourceVolume(index, channels, volume)
    }

    fn mute_fn(index: u32, muted: bool) -> AudioMsg {
        AudioMsg::SetSourceMute(index, muted)
    }

    fn default_fn(index: u32) -> AudioMsg {
        AudioMsg::SetDefaultSource(index)
    }

    fn muted_icon() -> Icon {
        Icon::MicMuted
    }

    fn unmuted_icon() -> Icon {
        Icon::Mic
    }

    fn title() -> String {
        "Input".to_string()
    }
}

impl Display for AudioSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.alias)
    }
}

impl TIndex for AudioSource {
    fn index(&self) -> u32 {
        self.index
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type, PartialEq, PartialOrd)]
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

impl TStreamCardUser<AudioSink> for InputStream {
    fn volume_fn(index: u32, channels: u16, volume: u32) -> AudioMsg {
        AudioMsg::SetInputStreamVolume(index, channels, volume)
    }

    fn mute_fn(index: u32, muted: bool) -> AudioMsg {
        AudioMsg::SetInputStreamMute(index, muted)
    }

    fn default_fn(self, sink: AudioSink) -> AudioMsg {
        AudioMsg::SetSinkOfInputStream(self, sink)
    }

    fn muted_icon() -> Icon {
        Icon::VolumeMuted
    }

    fn unmuted_icon() -> Icon {
        Icon::Volume
    }

    fn title() -> String {
        "Output".to_string()
    }

    fn obj_index(&self) -> u32 {
        self.sink_index
    }
}

impl TAudioObject for InputStream {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> String {
        self.application_name.clone()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn volume(&self) -> Vec<u32> {
        self.volume.clone()
    }

    fn muted(&self) -> bool {
        self.muted
    }

    fn index(&self) -> u32 {
        self.index
    }
}

impl TIndex for InputStream {
    fn index(&self) -> u32 {
        self.index
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type, PartialEq, PartialOrd)]
pub struct OutputStream {
    pub index: u32,
    pub name: String,
    pub application_name: String,
    pub source_index: u32,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub corked: bool,
}

impl TStreamCardUser<AudioSource> for OutputStream {
    fn volume_fn(index: u32, channels: u16, volume: u32) -> AudioMsg {
        AudioMsg::SetOutputStreamVolume(index, channels, volume)
    }

    fn mute_fn(index: u32, muted: bool) -> AudioMsg {
        AudioMsg::SetOutputStreamMute(index, muted)
    }

    fn default_fn(self, sink: AudioSource) -> AudioMsg {
        AudioMsg::SetSourceOfOutputStream(self, sink)
    }

    fn muted_icon() -> Icon {
        Icon::MicMuted
    }

    fn unmuted_icon() -> Icon {
        Icon::Mic
    }

    fn title() -> String {
        "Input".to_string()
    }

    fn obj_index(&self) -> u32 {
        self.source_index
    }
}

impl TAudioObject for OutputStream {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> String {
        self.application_name.clone()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn volume(&self) -> Vec<u32> {
        self.volume.clone()
    }

    fn muted(&self) -> bool {
        self.muted
    }

    fn index(&self) -> u32 {
        self.index
    }
}

impl TIndex for OutputStream {
    fn index(&self) -> u32 {
        self.index
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct AudioCard {
    pub index: u32,
    pub name: String,
    pub profiles: Vec<AudioCardProfile>,
    pub active_profile: String,
}

impl TIndex for AudioCard {
    fn index(&self) -> u32 {
        self.index
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct AudioCardProfile {
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
    fn source_changed(&self, source: AudioSource) -> zbus::Result<()>;
    #[zbus(signal)]
    fn source_added(&self, source: AudioSource) -> zbus::Result<()>;
    #[zbus(signal)]
    fn source_removed(&self, index: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn input_stream_changed(&self, input_stream: InputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn input_stream_added(&self, input_stream: InputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn input_stream_removed(&self, index: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn output_stream_changed(&self, output_stream: OutputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn output_stream_added(&self, output_stream: OutputStream) -> zbus::Result<()>;
    #[zbus(signal)]
    fn output_stream_removed(&self, index: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn card_changed(&self, card: AudioCard) -> zbus::Result<()>;
    #[zbus(signal)]
    fn card_added(&self, card: AudioCard) -> zbus::Result<()>;
    #[zbus(signal)]
    fn card_removed(&self, index: u32) -> zbus::Result<()>;

    fn list_sinks(&self) -> zbus::Result<Vec<AudioSink>>;
    fn get_default_sink(&self) -> zbus::Result<AudioSink>;
    fn get_default_sink_name(&self) -> zbus::Result<String>;
    fn set_sink_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_sink_mute(&self, index: u32, muted: bool) -> zbus::Result<()>;
    fn set_default_sink(&self, sink: String) -> zbus::Result<AudioSink>;

    fn list_sources(&self) -> zbus::Result<Vec<AudioSource>>;
    fn get_default_source(&self) -> zbus::Result<AudioSource>;
    fn get_default_source_name(&self) -> zbus::Result<String>;
    fn set_source_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_source_mute(&self, index: u32, muted: bool) -> zbus::Result<()>;
    fn set_default_source(&self, source: String) -> zbus::Result<AudioSource>;

    fn list_input_streams(&self) -> zbus::Result<Vec<InputStream>>;
    fn set_sink_of_input_stream(
        &self,
        input_stream: InputStream,
        sink: AudioSink,
    ) -> zbus::Result<()>;
    fn set_input_stream_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_input_stream_mute(&self, index: u32, muted: bool) -> zbus::Result<()>;

    fn list_output_streams(&self) -> zbus::Result<Vec<OutputStream>>;
    fn set_source_of_output_stream(
        &self,
        output_stream: OutputStream,
        source: AudioSource,
    ) -> zbus::Result<()>;
    fn set_output_stream_volume(&self, index: u32, channels: u16, volume: u32) -> zbus::Result<()>;
    fn set_output_stream_mute(&self, index: u32, muted: bool) -> zbus::Result<()>;

    fn list_cards(&self) -> zbus::Result<Vec<AudioCard>>;
    fn set_card_profile_of_device(
        &self,
        device_index: u32,
        profile_name: String,
    ) -> zbus::Result<()>;
}
