use std::{cell::RefCell, error::Error, future::Future};

use iced::{widget::row, Element};
use oxiced::widgets::oxi_button::{button, ButtonVariant};
//pub use re_set_lib::audio::audio_structures::Sink;
use zbus::{
    proxy,
    zvariant::{DeserializeDict, DynamicDeserialize, SerializeDict, Type},
    Connection,
};

use crate::Message;

#[derive(Default)]
pub struct AudioModel<'a> {
    audio_proxy: Option<Box<AudioDbusProxy<'a>>>,
    default_sink: i32,
    default_source: i32,
    sinks: Vec<i32>,
    sources: Vec<i32>,
    input_streams: Vec<i32>,
    output_streams: Vec<i32>,
}

#[derive(Debug, Clone)]
pub enum AudioMsg {
    GetDefaultSink,
    SetSinkVolume(i32),
    SetSinkMute(i32),
    SetSinkInputStream(i32),
    SetSourceVolume(i32),
    SetSourceMute(i32),
    SetSourceOutputStream(i32),
}

//async fn create_connection<T>(create_proxy: fn(&Connection) -> T) -> Result<(), Box<dyn Error>>
//where
//    T: Future,
//{
//    let connection = Connection::session().await?;
//    let proxy = create_proxy(&connection).await?;
//    proxy
//}
//
async fn create_audio_proxy() -> Result<AudioDbusProxy<'static>, Box<dyn Error>> {
    let connection = Connection::session().await?;
    let proxy = AudioDbusProxy::new(&connection).await?;
    Ok(proxy)
}

impl<'a> AudioModel<'a> {
    pub fn new() -> Self {
        Self {
            audio_proxy: None, // TODO beforepr Some(create_audio_proxy().),
            ..Default::default()
        }
    }

    pub fn update(&mut self, msg: AudioMsg) {
        match msg {
            AudioMsg::GetDefaultSink => println!("TODO beforepr"),
            AudioMsg::SetSinkVolume(_) => println!("setsinkvolume"),
            AudioMsg::SetSinkMute(_) => println!("setsinkmute"),
            AudioMsg::SetSinkInputStream(_) => println!("setsinkinputstream"),
            AudioMsg::SetSourceVolume(_) => println!("setsourceovlume"),
            AudioMsg::SetSourceMute(_) => println!("setsourcemute"),
            AudioMsg::SetSourceOutputStream(_) => println!("setsourceoutcomestream"),
        }
    }

    pub fn view(&self) -> Element<Message> {
        println!("display audio");
        row![
            // TODO beforepr
            // button("GetDefaultSink", ButtonVariant::Primary).on_press(Set)
        ]
        .into()
    }
}

#[derive(Debug, Clone, Default, DeserializeDict, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct AudioSink {
    pub index: u32,
    pub name: String,
    pub alias: String,
    pub channels: u16,
    pub volume: Vec<u32>,
    pub muted: bool,
    pub active: i32,
}

// TODO beforepr finish this
// TODO beforepr this needs to be put into the lib as the type cant be reused
#[proxy(
    default_service = "org.Xetibo.ReSet",
    default_path = "/org/Xetibo/ReSet",
    interface = "org.Xetibo.ReSet.Audio"
)]
pub trait AudioDbus {
    #[zbus(signal)]
    fn sink_changed(&self, sink: AudioSink) -> zbus::Result<()>;

    #[zbus(signal)]
    fn sink_added(&self, sink: AudioSink) -> zbus::Result<()>;

    #[zbus(signal)]
    fn sink_removed(&self, index: u32) -> zbus::Result<()>;

    fn get_default_sink(&self) -> zbus::Result<AudioSink>;

    fn get_default_sink_name(&self) -> zbus::Result<String>;
}
