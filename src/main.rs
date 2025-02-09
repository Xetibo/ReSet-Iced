use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use audio::audio_impl::{watch_audio_dbus_signals, AudioModel, AudioMsg, AudioVariant};
use components::{
    icons::Icon,
    sidebar::{sidebar, EntryButton, EntryButtonLevel, EntryCategory},
};
use dbus_interface::ReSetDbusProxy;
use iced::{
    futures::{
        channel::mpsc::{self, Sender},
        executor::block_on,
        SinkExt, Stream, StreamExt,
    },
    stream,
    widget::{column, row, scrollable, text},
    window::Settings,
    Element, Font, Size, Subscription, Task, Theme,
};
use network::network::{NetworkModel, NetworkMsg};
use re_set_lib::write_log_to_file;
use re_set_lib::LOG;
use reset_daemon::run_daemon;
use zbus::Connection;

mod audio;
mod components;
mod dbus_interface;
mod network;
mod utils;

#[derive(Default, Debug, Clone, Copy)]
enum PageId {
    // Chosen as it is probably the most useful page
    #[default]
    Audio,
    Network,
    Bluetooth,
}

enum SenderOrNone {
    None,
    Sender(Sender<ReSetMessage>),
}

struct ReSet {
    sender: SenderOrNone,
    ctx: Arc<Connection>,
    current_page: PageId,
    audio_model: AudioModel<'static>,
    network_model: NetworkModel,
}

#[derive(Debug, Clone)]
enum ReSetMessage {
    SubMsgAudio(AudioMsg),
    SubMsgNetwork(NetworkMsg),
    SetPage(PageId),
    StartWorker(PageId, Arc<Connection>),
    ReceiveSender(Sender<ReSetMessage>),
}

fn some_worker() -> impl Stream<Item = ReSetMessage> {
    stream::channel(100, |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);
        // TODO beforepr handle error
        let _ = output.send(ReSetMessage::ReceiveSender(sender)).await;

        loop {
            let input = receiver.select_next_some().await;
            if let ReSetMessage::StartWorker(page_id, conn) = input {
                match page_id {
                    PageId::Audio => watch_audio_dbus_signals(&mut output, conn)
                        .await
                        .expect("audio watcher failed"), // TODO beforepr
                    PageId::Network => (),
                    PageId::Bluetooth => (),
                }
            }
        }
    })
}

impl ReSet {
    fn subscription(&self) -> Subscription<ReSetMessage> {
        Subscription::run(some_worker)
    }

    fn theme(&self) -> Theme {
        oxiced::theme::get_theme()
    }

    fn new() -> (Self, Task<ReSetMessage>) {
        // TODO beforepr handle error
        let ctx = Arc::new(block_on(Connection::session()).unwrap());
        let audio_context = async || {
            AudioModel::new(&ctx.clone())
                .await
                .expect("Failed to create audio")
            // TODO beforepr expect
        };
        (
            Self {
                sender: SenderOrNone::None,
                ctx: ctx.clone(),
                current_page: Default::default(),
                audio_model: block_on(audio_context()),
                network_model: Default::default(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ReSet")
    }

    fn update(&mut self, message: ReSetMessage) -> Task<ReSetMessage> {
        match message {
            ReSetMessage::SubMsgAudio(audio_msg) => {
                let update_fn = async || self.audio_model.update(audio_msg).await;
                let output = block_on(update_fn());
                if let Some(task) = output {
                    task
                } else {
                    Task::none()
                }
            }
            ReSetMessage::SubMsgNetwork(network_msg) => {
                self.network_model.update(network_msg);
                Task::none()
            }
            ReSetMessage::SetPage(page_id) => {
                self.current_page = page_id;
                Task::done(ReSetMessage::StartWorker(page_id, self.ctx.clone()))
            }
            ReSetMessage::StartWorker(page_id, connection) => {
                match &mut self.sender {
                    SenderOrNone::None => (),
                    SenderOrNone::Sender(sender) => {
                        let fun = async || {
                            sender
                                .send(ReSetMessage::StartWorker(page_id, connection))
                                .await
                        };
                        let _ = block_on(fun());
                    }
                };
                Task::none()
            }
            ReSetMessage::ReceiveSender(sender) => {
                self.sender = SenderOrNone::Sender(sender);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<ReSetMessage> {
        let entries = {
            let audio_sub = vec![
                EntryButton {
                    title: "Input",
                    icon: Some(Icon::Mic),
                    msg: ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(AudioVariant::Input)),
                    level: EntryButtonLevel::SubLevel,
                },
                EntryButton {
                    title: "Output",
                    icon: Some(Icon::Volume),
                    msg: ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(AudioVariant::Output)),
                    level: EntryButtonLevel::SubLevel,
                },
                EntryButton {
                    title: "Cards",
                    icon: Some(Icon::AudioCards),
                    msg: ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(AudioVariant::Cards)),
                    level: EntryButtonLevel::SubLevel,
                },
                EntryButton {
                    title: "Devices",
                    icon: Some(Icon::AudioDevices),
                    msg: ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                        AudioVariant::Devices,
                    )),
                    level: EntryButtonLevel::SubLevel,
                },
            ];
            let base_audio = EntryButton {
                title: "Audio",
                icon: Some(Icon::Audio),
                msg: ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                    AudioVariant::InputAndOutput,
                )),
                level: EntryButtonLevel::TopLevel,
            };
            let audio = EntryCategory {
                main_entry: base_audio,
                sub_entries: audio_sub,
            };
            let network = EntryCategory {
                main_entry: EntryButton {
                    title: "Network",
                    icon: Some(Icon::Wifi),
                    msg: ReSetMessage::SetPage(PageId::Network),
                    level: EntryButtonLevel::TopLevel,
                },
                sub_entries: Vec::new(),
            };
            let bluetooth = EntryCategory {
                main_entry: EntryButton {
                    title: "Bluetooth",
                    icon: Some(Icon::Bluetooth),
                    msg: ReSetMessage::SetPage(PageId::Bluetooth),
                    level: EntryButtonLevel::TopLevel,
                },
                sub_entries: Vec::new(),
            };
            vec![audio, network, bluetooth]
        };
        row!(
            // TODO beforepr set audio and network
            sidebar(entries),
            // TODO beforepr make a wrapper over everything ->
            // 3 views  -> 1 box without sidebar -> 1 box with sidebar -> 2 boxes with sidebar
            scrollable(match self.current_page {
                PageId::Audio => self
                    .audio_model
                    .view()
                    .unwrap_or(column!(text("le error has happened")).into()),
                PageId::Network => self.network_model.view(),
                PageId::Bluetooth => row!(text("TODO")).into(),
            }),
        )
        .into()
    }

    //fn scale_factor(&self) -> f64 {
    //    1.0
    //}
}

#[tokio::main]
pub async fn main() -> Result<(), iced::Error> {
    let conn = Connection::session().await.unwrap();
    let reset_proxy = ReSetDbusProxy::new(&conn).await.unwrap();

    let res = reset_proxy.register_client("ReSet-Iced").await;

    if res.is_err() {
        // Start daemon and retry
        let ready = Arc::new(AtomicBool::new(false));
        let start = std::time::SystemTime::now();
        tokio::task::spawn(run_daemon(Some(ready.clone())));
        while !ready.load(std::sync::atomic::Ordering::SeqCst) {
            if start.elapsed().unwrap_or(Duration::from_secs(1)) >= Duration::from_secs(1) {
                return Err(iced::Error::WindowCreationFailed(Box::from(
                    "Failed to get daemon",
                )));
            }
        }
        // Second try without any catch, this means there was no way to connect to the daemon
        reset_proxy
            .register_client("ReSet-Iced")
            .await
            .expect("Failed to get daemon");
        LOG!("Using Bundled Daemon")
    }

    let window_settings = Settings {
        size: Size::default(),
        position: iced::window::Position::Default,
        min_size: None,
        max_size: None,
        visible: true,
        resizable: true,
        decorations: true, // TODO beforepr
        transparent: false,
        level: iced::window::Level::Normal,
        icon: None, // TODO beforepr add reset item
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: "ReSet-Iced".into(),
            override_redirect: false,
        },
        exit_on_close_request: true,
    };

    iced::application(ReSet::title, ReSet::update, ReSet::view)
        .window(window_settings)
        .theme(ReSet::theme)
        .default_font(Font::with_name("Adwaita Sans"))
        .subscription(ReSet::subscription)
        .run_with(ReSet::new)
}
