use std::error::Error;

use iced::{widget::row, Element};
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use zbus::Connection;

use crate::{utils::ignore, Message};

use super::dbus_interface::AudioDbusProxy;

pub struct AudioModel<'a> {
    audio_proxy: Box<AudioDbusProxy<'a>>,
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
    GetDefaultSinkName,
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
async fn create_audio_proxy(ctx: &Connection) -> Result<AudioDbusProxy<'static>, Box<dyn Error>> {
    let proxy = AudioDbusProxy::new(ctx).await?;
    Ok(proxy)
}

fn wrap(audio_msg: AudioMsg) -> Message {
    Message::SubMsgAudio(audio_msg)
}

impl<'a> AudioModel<'a> {
    pub async fn new(ctx: &Connection) -> Self {
        Self {
            audio_proxy: Box::new(create_audio_proxy(ctx).await.unwrap()),
            default_sink: Default::default(),
            default_source: Default::default(),
            sinks: Default::default(),
            sources: Default::default(),
            input_streams: Default::default(),
            output_streams: Default::default(),
        }
    }

    pub async fn update(&mut self, msg: AudioMsg) {
        match msg {
            AudioMsg::GetDefaultSink => ignore(dbg!(self.audio_proxy.get_default_sink().await)),
            AudioMsg::GetDefaultSinkName => {
                ignore(dbg!(self.audio_proxy.get_default_sink_name().await))
            }
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
            button("GetDefaultSink", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::GetDefaultSink)),
            button("GetDefaultSinkName", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::GetDefaultSinkName))
        ]
        .into()
    }
}
