use std::{collections::HashMap, error::Error, ops::RangeInclusive};

use iced::{
    widget::{column, row, text},
    Element,
};
use oxiced::widgets::{
    oxi_button::{button, ButtonVariant},
    oxi_picklist, oxi_slider,
};
use zbus::Connection;

use crate::{utils::ignore, Message};

use super::dbus_interface::{AudioDbusProxy, AudioSink, InputStream};

pub struct AudioModel<'a> {
    audio_proxy: Box<AudioDbusProxy<'a>>,
    default_sink: i32,
    default_source: i32,
    sinks: HashMap<u32, AudioSink>,
    sources: Vec<i32>,
    input_streams: HashMap<u32, InputStream>,
    output_streams: Vec<i32>,
}

#[derive(Debug, Clone)]
pub enum AudioMsg {
    GetDefaultSink,
    GetDefaultSinkName,
    SetSinkVolume(u32, u16, u32),
    SetSinkMute(u32, bool),
    SetSinkInputStream(i32),
    SetSourceVolume(u32, u16, u32),
    SetSourceMute(i32),
    SetSourceOutputStream(i32),
    SetInputStreamMute(u32, bool),
    SetInputStreamVolume(u32, u16, u32),
    SetSinkOfInputStream(InputStream, AudioSink),
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

fn convert_sinks(sinks: Vec<AudioSink>) -> HashMap<u32, AudioSink> {
    let mut map = HashMap::new();
    for sink in sinks.into_iter() {
        map.insert(sink.index, sink);
    }
    map
}

fn convert_input_streams(input_streams: Vec<InputStream>) -> HashMap<u32, InputStream> {
    let mut map = HashMap::new();
    for input_stream in input_streams.into_iter() {
        map.insert(input_stream.index, input_stream);
    }
    map
}

impl AudioModel<'_> {
    pub async fn new(ctx: &Connection) -> Self {
        // TODO beforepr unwrap
        let proxy = Box::new(create_audio_proxy(ctx).await.unwrap());
        let sinks = convert_sinks(proxy.list_sinks().await.unwrap());
        let input_streams = convert_input_streams(proxy.list_input_streams().await.unwrap());
        Self {
            audio_proxy: proxy,
            default_sink: Default::default(),
            default_source: Default::default(),
            sinks,
            sources: Default::default(),
            input_streams,
            output_streams: Default::default(),
        }
    }

    pub async fn update(&mut self, msg: AudioMsg) {
        match msg {
            // TODO beforepr remove the ignores
            AudioMsg::GetDefaultSink => ignore(self.audio_proxy.get_default_sink().await),
            AudioMsg::GetDefaultSinkName => ignore(self.audio_proxy.get_default_sink_name().await),
            AudioMsg::SetSinkVolume(index, channels, volume) => {
                let current_sink = self.sinks.get_mut(&index).unwrap();
                set_volume(&mut current_sink.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_sink_volume(index, channels, volume)
                        .await,
                )
            }
            AudioMsg::SetSinkMute(index, muted) => {
                // TODO beforepr handle unwrap
                self.sinks.get_mut(&index).unwrap().muted = muted;
                ignore(self.audio_proxy.set_sink_mute(index, muted).await)
            }
            AudioMsg::SetSinkInputStream(_) => println!("setsinkinputstream"),
            AudioMsg::SetSourceVolume(index, channels, volume) => {
                let current_source = self.sinks.get_mut(&index).unwrap();
                set_volume(&mut current_source.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_source_volume(index, channels, volume)
                        .await,
                )
            }
            AudioMsg::SetSourceMute(_) => println!("setsourcemute"),
            AudioMsg::SetSourceOutputStream(_) => println!("setsourceoutcomestream"),
            AudioMsg::SetInputStreamMute(index, muted) => {
                self.input_streams.get_mut(&index).unwrap().muted = muted;
                ignore(self.audio_proxy.set_input_stream_mute(index, muted).await)
            }
            AudioMsg::SetInputStreamVolume(index, channels, volume) => {
                let current_input_stream = self.input_streams.get_mut(&index).unwrap();
                set_volume(&mut current_input_stream.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_input_stream_volume(index, channels, volume)
                        .await,
                )
            }
            AudioMsg::SetSinkOfInputStream(input_stream, sink) => {
                self.input_streams
                    .get_mut(&input_stream.index)
                    .unwrap()
                    .sink_index = sink.index;
                ignore(
                    self.audio_proxy
                        .set_sink_of_input_stream(input_stream, sink)
                        .await,
                )
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let sink_cards: Vec<Element<Message>> = self.sinks.values().map(card_view).collect();
        let input_streams: Vec<Element<Message>> = self
            .input_streams
            .values()
            .map(|value| input_stream_view(value.clone(), &self.sinks))
            .collect();
        let mut col = column![];
        for elem in sink_cards {
            col = col.push(elem);
        }
        for elem in input_streams {
            col = col.push(elem);
        }
        col.into()
    }
}

fn set_volume(volume: &mut [u32], new_volume: u32) {
    for line in volume.iter_mut() {
        *line = new_volume;
    }
}

fn get_volume_level(volume: &[u32]) -> u32 {
    // TODO beforepr does this always exist?
    *volume.first().unwrap()
}

fn card_view(sink: &AudioSink) -> Element<'_, Message> {
    // TODO beforepr number?
    let current_volume = get_volume_level(&sink.volume);
    let slider = oxi_slider::slider(RangeInclusive::new(0, 100_270), current_volume, |value| {
        wrap(AudioMsg::SetSinkVolume(sink.index, sink.channels, value))
    })
    .step(2000_u32);
    row![
        row![text(sink.name.clone()), text(sink.alias.clone())],
        column![
            button("GetDefaultSinkName", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::SetSinkMute(sink.index, !sink.muted))),
            slider
        ]
    ]
    .into()
}

fn input_stream_view(
    input_stream: InputStream,
    sink_map: &HashMap<u32, AudioSink>,
) -> Element<'_, Message> {
    // TODO beforepr number?
    let current_volume = get_volume_level(&input_stream.volume);
    let slider = oxi_slider::slider(
        RangeInclusive::new(0, 100_270),
        current_volume,
        move |value| {
            wrap(AudioMsg::SetInputStreamVolume(
                input_stream.index,
                input_stream.channels,
                value,
            ))
        },
    )
    .step(2000_u32);
    let current_sink = sink_map.get(&input_stream.sink_index).unwrap();
    let sinks: Vec<AudioSink> = sink_map.clone().into_values().collect();
    let input_stream_clone = input_stream.clone();
    let pick_list = oxi_picklist::pick_list(sinks, Some(current_sink.clone()), move |sink| {
        wrap(AudioMsg::SetSinkOfInputStream(
            input_stream_clone.clone(),
            sink,
        ))
    });
    row![
        column![
            row![
                text(input_stream.application_name.clone()),
                text(input_stream.name.clone())
            ],
            pick_list
        ],
        column![
            button("Mute", ButtonVariant::Primary).on_press(wrap(AudioMsg::SetInputStreamMute(
                input_stream.index,
                !input_stream.muted
            ))),
            slider
        ]
    ]
    .into()
}
