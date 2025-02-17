use std::{
    ptr::null,
    sync::{
        atomic::{AtomicBool, AtomicPtr, AtomicU8},
        Arc,
    },
    time::Duration,
};

use audio::audio_impl::{watch_audio_dbus_signals, AudioModel, AudioMsg, AudioVariant};
use bluetooth::bluetooth_impl::{watch_bluetooth_dbus_signals, BluetoothModel, BluetoothMsg};
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
use network::network_impl::{NetworkModel, NetworkMsg};
use re_set_lib::write_log_to_file;
use re_set_lib::LOG;
use reset_daemon::run_daemon;
use zbus::Connection;

mod audio;
mod bluetooth;
mod components;
mod dbus_interface;
mod network;
mod utils;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
enum PageId {
    // Chosen as it is probably the most useful page
    #[default]
    Audio,
    Network,
    Bluetooth,
}

impl Into<u8> for PageId {
    fn into(self) -> u8 {
        match self {
            PageId::Audio => 0,
            PageId::Network => 1,
            PageId::Bluetooth => 2,
        }
    }
}

impl From<u8> for PageId {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Audio,
            1 => Self::Bluetooth,
            _ => Self::Network,
        }
    }
}

impl PageId {
    pub fn task(&self) -> Option<ReSetMessage> {
        match self {
            PageId::Audio => None,
            PageId::Network => None,
            PageId::Bluetooth => Some(ReSetMessage::SubMsgBluetooth(
                BluetoothMsg::StartBluetoothListener,
            )),
        }
    }
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
    bluetooth_model: BluetoothModel<'static>,
}

#[derive(Debug, Clone)]
enum ReSetMessage {
    SubMsgAudio(AudioMsg),
    SubMsgNetwork(NetworkMsg),
    SubMsgBluetooth(BluetoothMsg),
    SetPage(PageId),
    StartWorker(PageId, Arc<Connection>),
    ReceiveSender(Sender<ReSetMessage>),
}

fn some_worker() -> impl Stream<Item = ReSetMessage> {
    let mut page_id = PageId::Audio;
    stream::channel(100, move |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);
        let current_page_id = Arc::new(AtomicU8::new(page_id.into()));
        // TODO beforepr handle error
        let _ = output.send(ReSetMessage::ReceiveSender(sender)).await;

        println!("start");
        loop {
            // TODO resize event
            println!("blet");
            let input = receiver.select_next_some().await;
            if let ReSetMessage::StartWorker(page_id, conn) = input {
                current_page_id.store(page_id.into(), std::sync::atomic::Ordering::SeqCst);
                match page_id {
                    PageId::Audio => {
                        watch_audio_dbus_signals(&mut output, conn, current_page_id.clone())
                            .await
                            .expect("audio watcher failed")
                    } // TODO beforepr
                    PageId::Network => (),
                    PageId::Bluetooth => {
                        watch_bluetooth_dbus_signals(&mut output, conn, current_page_id.clone())
                            .await
                            .expect("audio watcher failed")
                    } // TODO beforepr
                }
                println!("what");
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
        let bluetooth_context = async || {
            BluetoothModel::new(&ctx.clone())
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
                bluetooth_model: block_on(bluetooth_context()),
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
            ReSetMessage::SubMsgBluetooth(bluetooth_msg) => {
                let update_fn = async || self.bluetooth_model.update(bluetooth_msg).await;
                let output = block_on(update_fn());
                if let Some(task) = output.ok() {
                    task
                } else {
                    Task::none()
                }
            }
            ReSetMessage::SetPage(page_id) => {
                if page_id == self.current_page {
                    Task::none()
                } else {
                    self.current_page = page_id;
                    Task::batch([
                        if let Some(msg) = PageId::task(&page_id) {
                            Task::done(msg)
                        } else {
                            Task::none()
                        },
                        Task::done(ReSetMessage::StartWorker(page_id, self.ctx.clone())),
                    ])
                }
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
                PageId::Bluetooth => self.bluetooth_model.view(),
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

    let icon = iced::window::icon::from_file("./assets/ReSet.png"); //.ok();
    let icon = if let Ok(icon) = icon {
        Some(icon)
    } else {
        dbg!(icon.err());
        None
    };
    let window_settings = Settings {
        size: Size::default(),
        position: iced::window::Position::Default,
        min_size: None,
        max_size: None,
        visible: true,
        resizable: true,
        decorations: true,
        transparent: false,
        level: iced::window::Level::Normal,
        // NOTE: this doesn't work on wayland
        // Use a .desktop file instead
        // https://github.com/iced-rs/iced/issues/1944
        icon,
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
