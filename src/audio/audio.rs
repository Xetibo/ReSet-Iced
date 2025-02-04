use std::{
    collections::HashMap,
    error::Error,
    ops::RangeInclusive,
    rc::Rc,
    sync::Arc,
    thread::{self, Thread},
};

use iced::{
    futures::{
        channel::mpsc::{self, Receiver, Sender},
        executor::block_on,
        SinkExt, StreamExt,
    },
    widget::{column, row, text},
    Element,
};
use oxiced::widgets::{
    oxi_button::{button, ButtonVariant},
    oxi_picklist, oxi_slider,
};
use zbus::Connection;

use crate::{utils::ignore, Message};

use super::dbus_interface::{
    AudioDbusProxy, AudioSink, AudioSource, InputStream, OutputStream, TIndex,
};

#[derive(Debug, Clone, Default)]
pub enum AudioVariant {
    Input,
    #[default]
    Output,
    Both,
}

pub struct AudioModel<'a> {
    audio_proxy: Arc<AudioDbusProxy<'a>>,
    default_sink: i32,
    default_source: i32,
    receiver: Receiver<AudioMsg>,
    audio_variant: AudioVariant,
    sinks: HashMap<u32, AudioSink>,
    sources: HashMap<u32, AudioSource>,
    input_streams: HashMap<u32, InputStream>,
    output_streams: HashMap<u32, OutputStream>,
}

#[derive(Debug, Clone)]
pub enum AudioMsg {
    SetAudioVariant(AudioVariant),
    SetSinkVolume(u32, u16, u32),
    SetSinkMute(u32, bool),
    AddSink(AudioSink),
    RemoveSink(u32),
    SetSourceVolume(u32, u16, u32),
    SetSourceMute(u32, bool),
    AddSource(AudioSource),
    RemoveSource(u32),
    SetOutputStreamMute(u32, bool),
    SetOutputStreamVolume(u32, u16, u32),
    SetSourceOfOutputStream(OutputStream, AudioSource),
    AddOutputStream(OutputStream),
    RemoveOutputStream(u32),
    SetInputStreamMute(u32, bool),
    SetInputStreamVolume(u32, u16, u32),
    SetSinkOfInputStream(InputStream, AudioSink),
    AddInputStream(InputStream),
    RemoveInputStream(u32),
    Test,
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

fn to_map<T>(elements: Vec<T>) -> HashMap<u32, T>
where
    T: TIndex,
{
    let mut map = HashMap::new();
    for element in elements.into_iter() {
        map.insert(element.index(), element);
    }
    map
}

pub async fn watch_audio_dbus_signals(mut sender: &mut Sender<Message>, conn: Arc<Connection>) {
    let proxy = AudioDbusProxy::new(&conn).await.expect("no proxy");
    let mut new_jobs_stream = proxy
        .receive_output_stream_added()
        .await
        .expect("no output stream");

    while let Some(msg) = new_jobs_stream.next().await {
        sender.send(wrap(AudioMsg::Test)).await; // TODO beforepr this worked :)

        //let what = msg.args().expect("");
        //what.input_stream
        //let args: JobNewArgs = msg.args().expect("Error parsing message");
    }
}

impl AudioModel<'_> {
    pub async fn new(ctx: &Connection) -> Self {
        // TODO beforepr unwrap
        let proxy = Arc::new(create_audio_proxy(ctx).await.unwrap());
        let sinks = to_map(proxy.list_sinks().await.unwrap());
        let input_streams = to_map(proxy.list_input_streams().await.unwrap());
        let sources = to_map(proxy.list_sources().await.unwrap());
        let output_streams = to_map(proxy.list_output_streams().await.unwrap());
        let (sender, receiver) = mpsc::channel::<AudioMsg>(30);
        let thread_proxy = proxy.clone();
        // TODO beforerpr
        //thread::spawn(move || {
        //    block_on(watch_audio_dbus_signals(sender, thread_proxy));
        //});
        Self {
            audio_proxy: proxy,
            default_sink: Default::default(),
            default_source: Default::default(),
            sinks,
            sources,
            input_streams,
            output_streams,
            audio_variant: Default::default(),
            receiver,
        }
    }

    pub async fn update(&mut self, msg: AudioMsg) {
        match msg {
            //TODO beforepr remove
            AudioMsg::Test => println!("test"),

            AudioMsg::SetAudioVariant(audio_variant) => self.audio_variant = audio_variant,
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
            AudioMsg::AddSink(sink) => ignore(self.sinks.insert(sink.index, sink)),
            AudioMsg::RemoveSink(index) => ignore(self.sinks.remove(&index)),
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
            AudioMsg::AddInputStream(input_stream) => {
                ignore(self.input_streams.insert(input_stream.index, input_stream))
            }
            AudioMsg::RemoveInputStream(input_stream) => {
                ignore(self.input_streams.remove(&input_stream))
            }
            AudioMsg::SetSourceVolume(index, channels, volume) => {
                let current_source = self.sinks.get_mut(&index).unwrap();
                set_volume(&mut current_source.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_source_volume(index, channels, volume)
                        .await,
                )
            }
            AudioMsg::SetSourceMute(index, muted) => {
                self.sources.get_mut(&index).unwrap().muted = muted;
                ignore(self.audio_proxy.set_source_mute(index, muted).await)
            }
            AudioMsg::AddSource(source) => ignore(self.sources.insert(source.index, source)),
            AudioMsg::RemoveSource(sources) => ignore(self.sources.remove(&sources)),
            AudioMsg::SetOutputStreamMute(index, muted) => {
                self.output_streams.get_mut(&index).unwrap().muted = muted;
                ignore(self.audio_proxy.set_output_stream_mute(index, muted).await)
            }
            AudioMsg::SetOutputStreamVolume(index, channels, volume) => {
                let current_output_stream = self.output_streams.get_mut(&index).unwrap();
                set_volume(&mut current_output_stream.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_output_stream_volume(index, channels, volume)
                        .await,
                )
            }
            AudioMsg::SetSourceOfOutputStream(output_stream, source) => {
                self.output_streams
                    .get_mut(&output_stream.index)
                    .unwrap()
                    .source_index = source.index;
                ignore(
                    self.audio_proxy
                        .set_source_of_output_stream(output_stream, source)
                        .await,
                )
            }
            AudioMsg::AddOutputStream(output_stream) => ignore(
                self.output_streams
                    .insert(output_stream.index, output_stream),
            ),
            AudioMsg::RemoveOutputStream(output_stream) => {
                ignore(self.output_streams.remove(&output_stream))
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let output: Element<Message> = {
            let sink_cards: Vec<Element<Message>> =
                self.sinks.values().map(sink_card_view).collect();
            let input_streams_cards: Vec<Element<Message>> = self
                .input_streams
                .values()
                .map(|value| input_stream_card_view(value.clone(), &self.sinks))
                .collect();
            let mut col = column![];
            for elem in sink_cards {
                col = col.push(elem);
            }
            for elem in input_streams_cards {
                col = col.push(elem);
            }
            col.into()
        };
        let input = {
            let source_cards: Vec<Element<Message>> =
                self.sources.values().map(source_card_view).collect();
            let output_stream_cards: Vec<Element<Message>> = self
                .output_streams
                .values()
                .map(|value| output_stream_card_view(value.clone(), &self.sources))
                .collect();
            let mut col = column![];
            for elem in source_cards {
                col = col.push(elem);
            }
            for elem in output_stream_cards {
                col = col.push(elem);
            }
            col.into()
        };
        let base = match self.audio_variant {
            AudioVariant::Input => input,
            AudioVariant::Output => output,
            AudioVariant::Both => row![output, input].into(),
        };
        // Make an enum to buttons function
        column![
            row![
                button("Both", ButtonVariant::Primary)
                    .on_press(wrap(AudioMsg::SetAudioVariant(AudioVariant::Both))),
                button("Input", ButtonVariant::Primary)
                    .on_press(wrap(AudioMsg::SetAudioVariant(AudioVariant::Input))),
                button("Output", ButtonVariant::Primary)
                    .on_press(wrap(AudioMsg::SetAudioVariant(AudioVariant::Output)))
            ],
            base
        ]
        .into()
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

// TODO beforepr deduplicate
fn sink_card_view(sink: &AudioSink) -> Element<'_, Message> {
    // TODO beforepr number?
    let current_volume = get_volume_level(&sink.volume);
    let slider = oxi_slider::slider(RangeInclusive::new(0, 100_270), current_volume, |value| {
        wrap(AudioMsg::SetSinkVolume(sink.index, sink.channels, value))
    })
    .step(2000_u32);
    row![
        row![text(sink.name.clone()), text(sink.alias.clone())],
        column![
            button("Mute", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::SetSinkMute(sink.index, !sink.muted))),
            slider
        ]
    ]
    .into()
}

fn source_card_view(sink: &AudioSource) -> Element<'_, Message> {
    // TODO beforepr number?
    let current_volume = get_volume_level(&sink.volume);
    let slider = oxi_slider::slider(RangeInclusive::new(0, 100_270), current_volume, |value| {
        wrap(AudioMsg::SetSourceVolume(sink.index, sink.channels, value))
    })
    .step(2000_u32);
    row![
        row![text(sink.name.clone()), text(sink.alias.clone())],
        column![
            button("Mute", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::SetSourceMute(sink.index, !sink.muted))),
            slider
        ]
    ]
    .into()
}

// TODO beforepr deduplicate
fn input_stream_card_view(
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

fn output_stream_card_view(
    output_stream: OutputStream,
    source_map: &HashMap<u32, AudioSource>,
) -> Element<'_, Message> {
    // TODO beforepr number?
    let current_volume = get_volume_level(&output_stream.volume);
    let slider = oxi_slider::slider(
        RangeInclusive::new(0, 100_270),
        current_volume,
        move |value| {
            wrap(AudioMsg::SetOutputStreamVolume(
                output_stream.index,
                output_stream.channels,
                value,
            ))
        },
    )
    .step(2000_u32);
    let current_source = source_map.get(&output_stream.source_index).unwrap();
    let sinks: Vec<AudioSource> = source_map.clone().into_values().collect();
    let output_stream_clone = output_stream.clone();
    let pick_list = oxi_picklist::pick_list(sinks, Some(current_source.clone()), move |source| {
        wrap(AudioMsg::SetSourceOfOutputStream(
            output_stream_clone.clone(),
            source,
        ))
    });
    row![
        column![
            row![
                text(output_stream.application_name.clone()),
                text(output_stream.name.clone())
            ],
            pick_list
        ],
        column![
            button("Mute", ButtonVariant::Primary).on_press(wrap(AudioMsg::SetOutputStreamMute(
                output_stream.index,
                !output_stream.muted
            ))),
            slider
        ]
    ]
    .into()
}
