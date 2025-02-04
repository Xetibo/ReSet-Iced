use std::{collections::HashMap, error::Error, ops::RangeInclusive, sync::Arc};

use iced::{
    futures::{channel::mpsc::Sender, SinkExt, StreamExt},
    widget::{column, row, text},
    Element,
};
use oxiced::widgets::{
    oxi_button::{button, ButtonVariant},
    oxi_picklist, oxi_slider,
};
use zbus::{Connection, Proxy};

use crate::{utils::ignore, Message};

use super::dbus_interface::{
    AudioCard, AudioDbusProxy, AudioSink, AudioSource, InputStream, OutputStream, TIndex,
};

#[derive(Debug, Clone, Default)]
pub enum AudioVariant {
    Input,
    #[default]
    Output,
    Cards,
    Both,
}

pub struct AudioModel<'a> {
    audio_proxy: Arc<AudioDbusProxy<'a>>,
    default_sink: u32,
    default_source: u32,
    audio_variant: AudioVariant,
    sinks: HashMap<u32, AudioSink>,
    sources: HashMap<u32, AudioSource>,
    input_streams: HashMap<u32, InputStream>,
    output_streams: HashMap<u32, OutputStream>,
    cards: HashMap<u32, AudioCard>,
}

#[derive(Debug, Clone)]
pub enum AudioMsg {
    SetAudioVariant(AudioVariant),
    SetSinkVolume(u32, u16, u32),
    SetSinkMute(u32, bool),
    AddSink(AudioSink),
    RemoveSink(u32),
    SetDefaultSink(u32),
    SetSourceVolume(u32, u16, u32),
    SetSourceMute(u32, bool),
    AddSource(AudioSource),
    RemoveSource(u32),
    SetDefaultSource(u32),
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
    SetProfileOfCard(u32, String),
}

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

// This sucks
pub async fn watch_audio_dbus_signals(sender: &mut Sender<Message>, conn: Arc<Connection>) {
    let proxy = AudioDbusProxy::new(&conn).await.expect("no proxy");
    let mut signals = Proxy::receive_all_signals(&proxy.into_inner())
        .await
        .unwrap();
    while let Some(msg) = signals.next().await {
        match msg.header().member().unwrap().to_string().as_str() {
            "OutputStreamAdded" | "OutputStreamChanged" => {
                let obj: OutputStream = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::AddOutputStream(obj))).await;
            }
            "OutputStreamRemoved" => {
                let obj: u32 = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::RemoveOutputStream(obj))).await;
            }
            "InputStreamAdded" | "InputStreamChanged" => {
                let obj: InputStream = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::AddInputStream(obj))).await;
            }
            "InputStreamRemoved" => {
                let obj: u32 = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::RemoveInputStream(obj))).await;
            }
            "SinkAdded" | "SinkChanged" => {
                let obj: AudioSink = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::AddSink(obj))).await;
            }
            "SinkRemoved" => {
                let obj: u32 = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::RemoveSink(obj))).await;
            }
            "SourceAdded" | "SourceChanged" => {
                let obj: AudioSource = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::AddSource(obj))).await;
            }
            "SourceRemoved" => {
                let obj: u32 = msg.body().deserialize().unwrap();
                let _res = sender.send(wrap(AudioMsg::RemoveSource(obj))).await;
            }
            _ => (),
        }
    }
}

impl AudioModel<'_> {
    pub async fn new(ctx: &Connection) -> Self {
        // TODO beforepr unwrap
        let proxy = Arc::new(create_audio_proxy(ctx).await.unwrap());
        let sinks = to_map(proxy.list_sinks().await.unwrap());
        let default_sink = proxy.get_default_sink().await.unwrap();
        let input_streams = to_map(proxy.list_input_streams().await.unwrap());
        let sources = to_map(proxy.list_sources().await.unwrap());
        let default_source = proxy.get_default_source().await.unwrap();
        let output_streams = to_map(proxy.list_output_streams().await.unwrap());
        let cards = to_map(proxy.list_cards().await.unwrap());
        Self {
            audio_proxy: proxy,
            default_sink: default_sink.index,
            default_source: default_source.index,
            sinks,
            sources,
            input_streams,
            output_streams,
            audio_variant: Default::default(),
            cards,
        }
    }

    pub async fn update(&mut self, msg: AudioMsg) {
        match msg {
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
            AudioMsg::RemoveInputStream(index) => ignore(self.input_streams.remove(&index)),
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
            AudioMsg::RemoveSource(index) => ignore(self.sources.remove(&index)),
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
            AudioMsg::RemoveOutputStream(index) => ignore(self.output_streams.remove(&index)),
            // TODO beforepr handle these properly when sink or source changes
            AudioMsg::SetDefaultSink(index) => {
                self.default_sink = index;
                let sink = self.sinks.get(&index).unwrap().clone();
                ignore(self.audio_proxy.set_default_sink(sink).await)
            }
            AudioMsg::SetDefaultSource(index) => {
                self.default_source = index;
                let source = self.sources.get(&index).unwrap().clone();
                ignore(self.audio_proxy.set_default_source(source).await)
            }
            AudioMsg::SetProfileOfCard(index, profile) => {
                self.cards.get_mut(&index).unwrap().active_profile = profile.clone();
                ignore(
                    self.audio_proxy
                        .set_card_profile_of_device(index, profile)
                        .await,
                )
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let cards = {
            let card_elements: Vec<Element<Message>> =
                self.cards.values().map(audio_cards).collect();
            let mut col = column![];
            for elem in card_elements {
                col = col.push(elem);
            }
            col.into()
        };
        let output: Element<Message> = {
            let sink_card = sink_card_view(self.default_sink, &self.sinks);
            let input_streams_cards: Vec<Element<Message>> = self
                .input_streams
                .values()
                .map(|value| input_stream_card_view(value.clone(), &self.sinks))
                .collect();
            let mut col = column![];
            col = col.push(sink_card);
            for elem in input_streams_cards {
                col = col.push(elem);
            }
            col.into()
        };
        let input = {
            let source_card = source_card_view(self.default_source, &self.sources);
            let output_stream_cards: Vec<Element<Message>> = self
                .output_streams
                .values()
                .filter_map(|value| output_stream_card_view(value.clone(), &self.sources))
                .collect();
            let mut col = column![];
            col = col.push(source_card);
            for elem in output_stream_cards {
                col = col.push(elem);
            }
            col.into()
        };
        let base = match self.audio_variant {
            AudioVariant::Cards => cards,
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
                    .on_press(wrap(AudioMsg::SetAudioVariant(AudioVariant::Output))),
                button("Cards", ButtonVariant::Primary)
                    .on_press(wrap(AudioMsg::SetAudioVariant(AudioVariant::Cards))),
            ]
            .padding(20),
            base
        ]
        .padding(20)
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
fn sink_card_view<'a>(sink: u32, sink_map: &'a HashMap<u32, AudioSink>) -> Element<'a, Message> {
    // TODO beforepr number?
    let sink = sink_map.get(&sink).unwrap().clone();
    let sinks: Vec<AudioSink> = sink_map.clone().into_values().collect();
    let current_volume = get_volume_level(&sink.volume);
    let slider = oxi_slider::slider(
        RangeInclusive::new(0, 100_270),
        current_volume,
        move |value| wrap(AudioMsg::SetSinkVolume(sink.index, sink.channels, value)),
    )
    .step(2000_u32);
    let pick_list = oxi_picklist::pick_list(sinks, Some(sink.clone()), move |sink| {
        wrap(AudioMsg::SetDefaultSink(sink.index))
    });
    column![
        pick_list,
        column![text(sink.name.clone()), text(sink.alias.clone())].padding(20),
        row![
            button("Mute", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::SetSinkMute(sink.index, !sink.muted))),
            slider
        ]
        .padding(20)
    ]
    .padding(20)
    .into()
}

fn source_card_view<'a>(
    source: u32,
    source_map: &'a HashMap<u32, AudioSource>,
) -> Element<'a, Message> {
    // TODO beforepr number?
    let source = source_map.get(&source).unwrap().clone();
    let sources: Vec<AudioSource> = source_map.clone().into_values().collect();
    let current_volume = get_volume_level(&source.volume);
    let slider = oxi_slider::slider(
        RangeInclusive::new(0, 100_270),
        current_volume,
        move |value| {
            wrap(AudioMsg::SetSourceVolume(
                source.index,
                source.channels,
                value,
            ))
        },
    )
    .step(2000_u32);
    let pick_list = oxi_picklist::pick_list(sources, Some(source.clone()), move |source| {
        wrap(AudioMsg::SetDefaultSource(source.index))
    });
    column![
        pick_list,
        column![text(source.name.clone()), text(source.alias.clone())].padding(20),
        row![
            button("Mute", ButtonVariant::Primary)
                .on_press(wrap(AudioMsg::SetSourceMute(source.index, !source.muted))),
            slider
        ]
        .padding(20)
    ]
    .padding(20)
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
    column![
        row![
            column![
                text(input_stream.application_name.clone()),
                text(input_stream.name.clone())
            ]
            .padding(20),
            pick_list
        ]
        .padding(20),
        row![
            button("Mute", ButtonVariant::Primary).on_press(wrap(AudioMsg::SetInputStreamMute(
                input_stream.index,
                !input_stream.muted
            ))),
            slider
        ]
        .padding(20)
    ]
    .padding(20)
    .into()
}

fn output_stream_card_view(
    output_stream: OutputStream,
    source_map: &HashMap<u32, AudioSource>,
) -> Option<Element<'_, Message>> {
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
    let current_source = source_map.get(&output_stream.source_index)?;
    let sources: Vec<AudioSource> = source_map.clone().into_values().collect();
    let output_stream_clone = output_stream.clone();
    let pick_list = oxi_picklist::pick_list(sources, Some(current_source.clone()), move |source| {
        wrap(AudioMsg::SetSourceOfOutputStream(
            output_stream_clone.clone(),
            source,
        ))
    });
    Some(
        column![
            row![
                column![
                    text(output_stream.application_name.clone()),
                    text(output_stream.name.clone())
                ]
                .padding(20),
                pick_list
            ]
            .padding(20),
            row![
                button("Mute", ButtonVariant::Primary).on_press(wrap(
                    AudioMsg::SetOutputStreamMute(output_stream.index, !output_stream.muted)
                )),
                slider
            ]
            .padding(20)
        ]
        .padding(20)
        .into(),
    )
}

fn audio_cards(card: &AudioCard) -> Element<'_, Message> {
    let index = card.index;
    let profiles: Vec<String> = card
        .profiles
        .clone()
        .into_iter()
        .map(|value| value.name)
        .collect();
    let pick_list = oxi_picklist::pick_list(
        profiles,
        Some(card.active_profile.clone()),
        move |profile| wrap(AudioMsg::SetProfileOfCard(index, profile)),
    );
    column![
        pick_list,
        text(card.name.clone()),
        text(card.active_profile.clone())
    ]
    .into()
}
