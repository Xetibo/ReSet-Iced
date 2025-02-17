use std::{
    collections::HashMap,
    error::Error,
    sync::{atomic::AtomicU8, Arc},
};

use iced::{
    futures::{channel::mpsc::Sender, SinkExt, StreamExt},
    widget::{column, row},
    Element, Task,
};
use zbus::{Connection, Proxy};

use crate::{
    components::{
        audio_card::{device_card_view, populate_audio_cards},
        comborow::{ComboPickerTitle, CustomPickList, PickerVariant},
        select_row::picklist_to_row,
    },
    utils::ignore,
    PageId, ReSetMessage,
};

use super::dbus_interface::{
    AudioCard, AudioDbusProxy, AudioSink, AudioSource, InputStream, OutputStream, TIndex,
};

#[derive(Debug, Clone, Default)]
pub enum AudioVariant {
    Input,
    #[default]
    Output,
    Cards,
    Devices,
    InputAndOutput,
}

pub struct AudioModel<'a> {
    audio_proxy: Arc<AudioDbusProxy<'a>>,
    default_sink: u32,
    // TODO beforepr find a better way to handle broken defaults
    default_sink_dummy: bool,
    default_source: u32,
    default_source_dummy: bool,
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
    AddAudioCard(AudioCard),
    RemoveAudioCard(u32),
    SetProfileOfCard(u32, String),
}

async fn create_audio_proxy(ctx: &Connection) -> Result<AudioDbusProxy<'static>, Box<dyn Error>> {
    let proxy = AudioDbusProxy::new(ctx).await?;
    Ok(proxy)
}

fn wrap(audio_msg: AudioMsg) -> ReSetMessage {
    ReSetMessage::SubMsgAudio(audio_msg)
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
pub async fn watch_audio_dbus_signals(
    sender: &mut Sender<ReSetMessage>,
    conn: Arc<Connection>,
    current_page_id: Arc<AtomicU8>,
) -> Result<(), zbus::Error> {
    let proxy = AudioDbusProxy::new(&conn).await.expect("no proxy");
    let mut signals = Proxy::receive_all_signals(&proxy.into_inner()).await?;
    loop {
        if current_page_id.load(std::sync::atomic::Ordering::SeqCst) != PageId::Audio.into() {
            break;
        }
        if let Some(msg) = signals.next().await {
            match msg.header().member().unwrap().to_string().as_str() {
                "OutputStreamAdded" | "OutputStreamChanged" => {
                    let obj: OutputStream = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::AddOutputStream(obj))).await;
                }
                "OutputStreamRemoved" => {
                    let obj: u32 = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::RemoveOutputStream(obj))).await;
                }
                "InputStreamAdded" | "InputStreamChanged" => {
                    let obj: InputStream = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::AddInputStream(obj))).await;
                }
                "InputStreamRemoved" => {
                    let obj: u32 = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::RemoveInputStream(obj))).await;
                }
                "SinkAdded" | "SinkChanged" => {
                    let obj: AudioSink = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::AddSink(obj))).await;
                }
                "SinkRemoved" => {
                    let obj: u32 = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::RemoveSink(obj))).await;
                }
                "SourceAdded" | "SourceChanged" => {
                    let obj: AudioSource = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::AddSource(obj))).await;
                }
                "SourceRemoved" => {
                    let obj: u32 = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::RemoveSource(obj))).await;
                }
                "CardAdded" | "CardChanged" => {
                    let obj: AudioCard = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::AddAudioCard(obj))).await;
                }
                "CardRemoved" => {
                    let obj: u32 = msg.body().deserialize()?;
                    let _res = sender.send(wrap(AudioMsg::RemoveAudioCard(obj))).await;
                }
                _ => (),
            }
        }
    }

    println!("end audio dbus listener");
    Ok(())
}

impl AudioModel<'_> {
    pub async fn new(ctx: &Connection) -> Result<Self, zbus::Error> {
        let proxy = Arc::new(
            create_audio_proxy(ctx)
                .await
                .expect("Could not create proxy for audio"),
        ); // TODO beforepr expect
        let sinks = to_map(proxy.list_sinks().await?);
        let default_sink = proxy.get_default_sink().await?;
        let input_streams = to_map(proxy.list_input_streams().await?);
        let sources = to_map(proxy.list_sources().await?);
        let default_source = proxy.get_default_source().await?;
        let output_streams = to_map(proxy.list_output_streams().await?);
        let cards = to_map(proxy.list_cards().await?);
        Ok(Self {
            audio_proxy: proxy,
            default_sink: default_sink.index,
            default_source: default_source.index,
            sinks,
            sources,
            input_streams,
            output_streams,
            audio_variant: Default::default(),
            cards,
            default_sink_dummy: false,
            default_source_dummy: false,
        })
    }

    pub async fn update(&mut self, msg: AudioMsg) -> Option<Task<ReSetMessage>> {
        let cmd = match msg {
            AudioMsg::SetAudioVariant(audio_variant) => {
                self.audio_variant = audio_variant;
                Task::done(ReSetMessage::SetPage(crate::PageId::Audio))
            }
            AudioMsg::SetSinkVolume(index, channels, volume) => {
                let current_sink = self.sinks.get_mut(&index)?;
                set_volume(&mut current_sink.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_sink_volume(index, channels, volume)
                        .await,
                );
                Task::none()
            }
            AudioMsg::SetSinkMute(index, muted) => {
                // TODO beforepr handle unwrap
                self.sinks.get_mut(&index)?.muted = muted;
                ignore(self.audio_proxy.set_sink_mute(index, muted).await);
                Task::none()
            }
            AudioMsg::AddSink(sink) => {
                if self.default_sink_dummy {
                    self.default_sink = sink.index;
                    self.default_sink_dummy = false;
                }
                ignore(self.sinks.insert(sink.index, sink));
                Task::none()
            }
            AudioMsg::RemoveSink(index) => {
                if index == self.default_sink {
                    if self.sinks.len() <= 1 {
                        self.default_sink_dummy = true;
                    } else {
                        self.default_sink =
                            self.sinks.values().find(|sink| sink.index != index)?.index;
                    }
                }
                ignore(self.sinks.remove(&index));
                Task::none()
            }
            AudioMsg::SetInputStreamMute(index, muted) => {
                self.input_streams.get_mut(&index)?.muted = muted;
                ignore(self.audio_proxy.set_input_stream_mute(index, muted).await);
                Task::none()
            }
            AudioMsg::SetInputStreamVolume(index, channels, volume) => {
                let current_input_stream = self.input_streams.get_mut(&index)?;
                set_volume(&mut current_input_stream.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_input_stream_volume(index, channels, volume)
                        .await,
                );
                Task::none()
            }
            AudioMsg::SetSinkOfInputStream(input_stream, sink) => {
                self.input_streams.get_mut(&input_stream.index)?.sink_index = sink.index;
                ignore(
                    self.audio_proxy
                        .set_sink_of_input_stream(input_stream, sink)
                        .await,
                );
                Task::none()
            }
            AudioMsg::AddInputStream(input_stream) => {
                ignore(self.input_streams.insert(input_stream.index, input_stream));
                Task::none()
            }
            AudioMsg::RemoveInputStream(index) => {
                ignore(self.input_streams.remove(&index));
                Task::none()
            }
            AudioMsg::SetSourceVolume(index, channels, volume) => {
                let current_source = self.sources.get_mut(&index)?;
                set_volume(&mut current_source.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_source_volume(index, channels, volume)
                        .await,
                );
                Task::none()
            }
            AudioMsg::SetSourceMute(index, muted) => {
                self.sources.get_mut(&index)?.muted = muted;
                ignore(self.audio_proxy.set_source_mute(index, muted).await);
                Task::none()
            }
            AudioMsg::AddSource(source) => {
                if self.default_source_dummy {
                    self.default_source = source.index;
                    self.default_source_dummy = false;
                }
                ignore(self.sources.insert(source.index, source));
                Task::none()
            }
            AudioMsg::RemoveSource(index) => {
                if index == self.default_source {
                    if self.sources.len() <= 1 {
                        self.default_source_dummy = true;
                    } else {
                        self.default_source = self
                            .sources
                            .values()
                            .find(|source| source.index != index)?
                            .index;
                    }
                }
                ignore(self.sources.remove(&index));
                Task::none()
            }
            AudioMsg::SetOutputStreamMute(index, muted) => {
                self.output_streams.get_mut(&index)?.muted = muted;
                ignore(self.audio_proxy.set_output_stream_mute(index, muted).await);
                Task::none()
            }
            AudioMsg::SetOutputStreamVolume(index, channels, volume) => {
                let current_output_stream = self.output_streams.get_mut(&index)?;
                set_volume(&mut current_output_stream.volume, volume);
                ignore(
                    self.audio_proxy
                        .set_output_stream_volume(index, channels, volume)
                        .await,
                );
                Task::none()
            }
            AudioMsg::SetSourceOfOutputStream(output_stream, source) => {
                self.output_streams
                    .get_mut(&output_stream.index)?
                    .source_index = source.index;
                ignore(
                    self.audio_proxy
                        .set_source_of_output_stream(output_stream, source)
                        .await,
                );
                Task::none()
            }
            AudioMsg::AddOutputStream(output_stream) => {
                ignore(
                    self.output_streams
                        .insert(output_stream.index, output_stream),
                );
                Task::none()
            }
            AudioMsg::RemoveOutputStream(index) => {
                ignore(self.output_streams.remove(&index));
                Task::none()
            }
            // TODO beforepr handle these properly when sink or source changes
            AudioMsg::SetDefaultSink(index) => {
                self.default_sink = index;
                let sink = self.sinks.get(&index)?;
                ignore(self.audio_proxy.set_default_sink(sink.name.clone()).await);
                Task::none()
            }
            AudioMsg::SetDefaultSource(index) => {
                self.default_source = index;
                let source = self.sources.get(&index)?;
                ignore(
                    self.audio_proxy
                        .set_default_source(source.name.clone())
                        .await,
                );
                Task::none()
            }
            AudioMsg::AddAudioCard(audio_card) => {
                ignore(self.cards.insert(audio_card.index, audio_card));
                Task::none()
            }
            AudioMsg::RemoveAudioCard(index) => {
                ignore(self.cards.remove(&index));
                Task::none()
            }
            AudioMsg::SetProfileOfCard(index, profile) => {
                self.cards.get_mut(&index)?.active_profile = profile.clone();
                ignore(
                    self.audio_proxy
                        .set_card_profile_of_device(index, profile)
                        .await,
                );
                Task::none()
            }
        };
        Some(cmd)
    }

    // TODO beforepr handle errors
    pub fn view(&self) -> Option<Element<ReSetMessage>> {
        let cards = {
            let card_elements: Vec<Element<ReSetMessage>> = self
                .cards
                .values()
                .enumerate()
                .map(|(index, card)| audio_cards(card, index, self.cards.len()))
                .collect();
            let mut col = column![];
            for elem in card_elements {
                col = col.push(elem);
            }
            col.into()
        };
        let output: Element<ReSetMessage> =
            populate_audio_cards(self.default_sink, &self.sinks, &self.input_streams)?;
        let input = populate_audio_cards(self.default_source, &self.sources, &self.output_streams)?;
        // TODO beforepr, should these be combined??
        let devices = {
            row!(
                device_card_view(self.default_source, &self.sources),
                device_card_view(self.default_sink, &self.sinks)
            )
            .spacing(20)
            .into()
        };
        let base = match self.audio_variant {
            AudioVariant::Cards => cards,
            AudioVariant::Input => input,
            AudioVariant::Output => output,
            AudioVariant::InputAndOutput => row![output, input].into(),
            AudioVariant::Devices => devices,
        };
        // Make an enum to buttons function
        Some(column![base].padding(20).into())
    }
}

fn set_volume(volume: &mut [u32], new_volume: u32) {
    for line in volume.iter_mut() {
        *line = new_volume;
    }
}

fn audio_cards(card: &AudioCard, vec_index: usize, length: usize) -> Element<'_, ReSetMessage> {
    let index = card.index;
    let profiles: Vec<String> = card
        .profiles
        .clone()
        .into_iter()
        .map(|value| value.name)
        .collect();
    picklist_to_row(
        CustomPickList::new(
            PickerVariant::ComboPicker(ComboPickerTitle::new(
                card.name.clone(),
                Some(card.active_profile.clone()),
            )),
            profiles,
            Some(card.active_profile.clone()),
            move |profile| wrap(AudioMsg::SetProfileOfCard(index, profile)),
        ),
        vec_index,
        length,
    )
    .into()
}
