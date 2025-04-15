use audio::{
    audio_impl::{watch_audio_dbus_signals, AudioModel, AudioMsg, AudioVariant},
    dbus_interface::AudioDbusProxy,
};
use bluetooth::{
    bluetooth_impl::{watch_bluetooth_dbus_signals, BluetoothModel, BluetoothMsg},
    dbus_interface::BluetoothDbusProxy,
};
use components::{
    icons::Icon,
    sidebar::{sidebar, EntryButton, EntryButtonLevel, EntryCategory},
};
use dbus_interface::ReSetDbusProxy;
use iced::{
    futures::{
        self,
        channel::{
            mpsc::{self, Sender},
            oneshot::{self, Receiver},
        },
        executor::block_on,
        FutureExt, SinkExt, Stream, StreamExt,
    },
    stream,
    widget::{row, scrollable},
    window::Settings,
    Element, Font, Size, Subscription, Task, Theme,
};
use libloading::Symbol;
use network::{
    dbus_interface::WifiDbusProxy,
    network_impl::{NetworkModel, NetworkMsg},
    wireless_impl::{watch_wireless_dbus_signals, WirelessModel},
};
use plugins::{load_plugins, SETUP_LIBS, SETUP_PLUGIN_DIR};
use re_set_lib::{utils::any::ReSetAny, write_log_to_file};
use re_set_lib::{utils::error::ReSetError, LOG};
use reset_daemon::run_daemon;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use utils::{display_view_or_error, TPage};

use zbus::{proxy::SignalStream, Connection, Proxy};

mod audio;
mod bluetooth;
mod components;
mod dbus_interface;
mod network;
mod plugins;
mod utils;

// TODO make this work over C ABI
//pub struct PluginFuncs2 {
//    pub enter: libloading::Symbol<'static, unsafe extern "C" fn()>,
//    pub leave: libloading::Symbol<'static, unsafe extern "C" fn()>,
//    pub model: libloading::Symbol<
//        'static,
//        unsafe extern "C" fn(ctx: &zbus::Connection, additional_data: c_void) -> c_void,
//    >,
//    pub update:
//        libloading::Symbol<'static, unsafe extern "C" fn(data: c_void, msg: c_void) -> c_void>,
//    pub view: libloading::Symbol<'static, unsafe extern "C" fn(data: c_void) -> c_void>,
//}

//TODO move error into lib
#[derive(Clone, Debug)]
pub struct PluginFuncs {
    pub enter:
        libloading::Symbol<'static, unsafe extern "C" fn() -> Task<&'static mut dyn ReSetAny>>,
    pub leave:
        libloading::Symbol<'static, unsafe extern "C" fn() -> Task<&'static mut dyn ReSetAny>>,
    pub model: libloading::Symbol<
        'static,
        unsafe extern "C" fn(
            ctx: &zbus::Connection,
            additional_data: &mut dyn ReSetAny,
        ) -> &'static mut dyn ReSetAny,
    >,
    pub update: libloading::Symbol<
        'static,
        unsafe extern "C" fn(
            data: &&mut dyn ReSetAny,
            msg: &dyn ReSetAny,
        ) -> Option<&'static dyn ReSetAny>,
    >,
    pub view: libloading::Symbol<
        'static,
        unsafe extern "C" fn(
            data: &dyn ReSetAny,
        ) -> Result<Element<&'static mut dyn ReSetAny>, ReSetError>,
    >,
    pub signals: libloading::Symbol<
        'static,
        unsafe extern "C" fn(conn: &Connection) -> Option<SignalStream<'static>>,
    >,
    pub watch_signals: libloading::Symbol<
        'static,
        unsafe extern "C" fn(
            sender: &mut dyn ReSetAny,
            signals: &mut SignalStream<'static>,
        ) -> Result<(), ReSetError>,
    >,
}

unsafe impl Send for PluginFuncs {}
unsafe impl Sync for PluginFuncs {}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum PageId {
    // Chosen as it is probably the most useful page
    #[default]
    Audio,
    Network,
    Bluetooth,
    Plugin(u8),
}

impl Into<u8> for PageId {
    fn into(self) -> u8 {
        match self {
            PageId::Audio => 0,
            PageId::Network => 1,
            PageId::Bluetooth => 2,
            PageId::Plugin(id) => 3 + id, // TODO
        }
    }
}

impl From<u8> for PageId {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Audio,
            1 => Self::Bluetooth,
            2 => Self::Network,
            id => PageId::Plugin(id - 3), // TODO
        }
    }
}

impl PageId {
    pub fn enter(&self, plugin_funcs: HashMap<u8, PluginFuncs>) -> Task<ReSetMessage> {
        match self {
            PageId::Audio => AudioModel::enter(),
            PageId::Network => WirelessModel::enter(),
            PageId::Bluetooth => BluetoothModel::enter(),
            PageId::Plugin(id) => unsafe {
                let leave = plugin_funcs.get(&id).unwrap().enter.clone();
                let newid = id.clone();
                (leave()).map(move |msg: &'static mut dyn ReSetAny| {
                    ReSetMessage::SubPluginMsg(newid, Arc::new(msg))
                })
            },
        }
    }

    pub fn leave(&self, plugin_funcs: HashMap<u8, PluginFuncs>) -> Task<ReSetMessage> {
        match self {
            PageId::Audio => AudioModel::leave(),
            PageId::Network => NetworkModel::leave(),
            PageId::Bluetooth => BluetoothModel::leave(),
            PageId::Plugin(id) => unsafe {
                let leave = plugin_funcs.get(&id).unwrap().leave.clone();
                let newid = id.clone();
                (leave()).map(move |msg: &'static mut dyn ReSetAny| {
                    ReSetMessage::SubPluginMsg(newid, Arc::new(msg))
                })
            },
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
    model_map: HashMap<PageId, &'static mut dyn ReSetAny>,
    plugin_funcs: HashMap<u8, PluginFuncs>,
}

#[derive(Debug)]
pub enum ReSetMessage {
    SubMsgAudio(AudioMsg),
    SubMsgNetwork(NetworkMsg),
    SubMsgBluetooth(BluetoothMsg),
    SubPluginMsg(u8, Arc<dyn ReSetAny>),
    SetPage(PageId),
    StartWorker(PageId, Arc<Connection>, HashMap<u8, PluginFuncs>),
    ReceiveSender(Sender<ReSetMessage>),
}

impl Clone for ReSetMessage {
    fn clone(&self) -> Self {
        match self {
            ReSetMessage::SubMsgAudio(audio_msg) => Self::SubMsgAudio(audio_msg.clone()),
            ReSetMessage::SubMsgNetwork(network_msg) => Self::SubMsgNetwork(network_msg.clone()),
            ReSetMessage::SubMsgBluetooth(bluetooth_msg) => {
                Self::SubMsgBluetooth(bluetooth_msg.clone())
            }
            ReSetMessage::SubPluginMsg(id, any) => Self::SubPluginMsg(*id, any.clone()),
            ReSetMessage::SetPage(page_id) => Self::SetPage(*page_id),
            ReSetMessage::StartWorker(page_id, connection, plugins) => {
                Self::StartWorker(*page_id, connection.clone(), plugins.clone())
            }
            ReSetMessage::ReceiveSender(sender) => Self::ReceiveSender(sender.clone()),
        }
    }
}

unsafe impl Send for ReSetMessage {}
unsafe impl Sync for ReSetMessage {}

// This stops each page daemon on page transition
async fn wrap_daemon(
    shutdown_rx: &mut Receiver<()>,
    output: &mut Sender<ReSetMessage>,
    mut signals: SignalStream<'static>,
    daemonfn: impl AsyncFn(
        &mut Sender<ReSetMessage>,
        &mut SignalStream<'static>,
    ) -> Result<(), ReSetError>,
) {
    let shutdown_future = shutdown_rx.map(|_| ());
    let mut shutdown_future = Box::pin(shutdown_future);

    loop {
        futures::select! {
            _ = shutdown_future => {
                break;
            },
            default => daemonfn(output, &mut signals).await.expect("")
        }
    }
}

fn wrap_daemon_plugin(
    shutdown_rx: &mut Receiver<()>,
    output: &mut Sender<ReSetMessage>,
    mut signals: SignalStream<'static>,
    daemonfn: &Symbol<
        'static,
        unsafe extern "C" fn(
            &mut dyn ReSetAny,
            &mut SignalStream<'static>,
        ) -> Result<(), ReSetError>,
    >,
) {
    let shutdown_future = shutdown_rx.map(|_| ());
    let mut shutdown_future = Box::pin(shutdown_future);

    loop {
        futures::select! {
            _ = shutdown_future => {
                break;
            },
            default => unsafe {daemonfn(output as &mut dyn ReSetAny, &mut signals).expect("")}
        }
    }
}

fn some_worker() -> impl Stream<Item = ReSetMessage> {
    stream::channel(100, move |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);
        // TODO beforepr handle error
        let _ = output.send(ReSetMessage::ReceiveSender(sender)).await;

        loop {
            let (shutdown_sender, mut shutdown_receiver) = oneshot::channel();
            // TODO resize event
            let input = receiver.select_next_some().await;
            if let ReSetMessage::StartWorker(page_id, conn, plugins) = input {
                // TODO beforepr handle error
                let _ = shutdown_sender.send(());
                match page_id {
                    PageId::Audio => {
                        let proxy = AudioDbusProxy::new(&conn).await.expect("no proxy");
                        let signals = Proxy::receive_all_signals(&proxy.into_inner())
                            .await
                            .expect("no proxy");
                        let _ = wrap_daemon(
                            &mut shutdown_receiver,
                            &mut output,
                            signals,
                            watch_audio_dbus_signals,
                        )
                        .await;
                        ()
                    }
                    PageId::Network => {
                        let proxy = WifiDbusProxy::new(&conn).await.expect("no proxy");
                        let signals = Proxy::receive_all_signals(&proxy.into_inner())
                            .await
                            .expect("no proxy");
                        let _ = wrap_daemon(
                            &mut shutdown_receiver,
                            &mut output,
                            signals,
                            watch_wireless_dbus_signals,
                        )
                        .await;
                        ()
                    }
                    PageId::Bluetooth => {
                        let proxy = BluetoothDbusProxy::new(&conn).await.expect("no proxy");
                        let signals = Proxy::receive_all_signals(&proxy.into_inner())
                            .await
                            .expect("no proxy");
                        let _ = wrap_daemon(
                            &mut shutdown_receiver,
                            &mut output,
                            signals,
                            watch_bluetooth_dbus_signals,
                        )
                        .await;
                        ()
                    }
                    PageId::Plugin(id) => {
                        let funcs = plugins.get(&id).unwrap();
                        unsafe {
                            if let Some(signals) = (funcs.signals)(&conn) {
                                wrap_daemon_plugin(
                                    &mut shutdown_receiver,
                                    &mut output,
                                    signals,
                                    &funcs.watch_signals,
                                )
                            }
                        }
                    }
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
            AudioModel::new(&ctx.clone(), ())
                .await
                .expect("Failed to create audio")
            // TODO beforepr expect
        };
        let bluetooth_context = async || {
            BluetoothModel::new(&ctx.clone(), ())
                .await
                .expect("Failed to create bluetooth")
            // TODO beforepr expect
        };
        let network_context = async || {
            NetworkModel::new(&ctx.clone(), ())
                .await
                .expect("Failed to create network")
            // TODO beforepr expect
        };
        let audio_model = Box::<AudioModel<'_>>::leak(Box::new(block_on(audio_context())));
        let network_model = Box::<NetworkModel<'_>>::leak(Box::new(block_on(network_context())));
        let bluetooth_model =
            Box::<BluetoothModel<'_>>::leak(Box::new(block_on(bluetooth_context())));
        let mut model_map = HashMap::new();
        model_map.insert(PageId::Audio, audio_model as &mut dyn ReSetAny);
        model_map.insert(PageId::Network, network_model as &mut dyn ReSetAny);
        model_map.insert(PageId::Bluetooth, bluetooth_model as &mut dyn ReSetAny);

        let mut plugin_funcs: HashMap<u8, PluginFuncs> = HashMap::new();
        let mut index = 0;
        // TODO
        for plugin in load_plugins() {
            let modelfn = plugin.model.clone();
            let model = unsafe { (modelfn)(&ctx.clone(), &mut () as &mut dyn ReSetAny) };
            model_map.insert(PageId::Plugin(index), model);
            plugin_funcs.insert(index as u8, plugin);
            index += 1;
        }
        (
            Self {
                sender: SenderOrNone::None,
                ctx: ctx.clone(),
                current_page: Default::default(),
                model_map,
                plugin_funcs,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ReSet")
    }

    fn update_submodel<Model, Message, A>(
        &mut self,
        page_id: PageId,
        message: Message,
    ) -> Option<Task<ReSetMessage>>
    where
        Model: TPage<Message, Model, A> + ReSetAny + Debug + 'static,
    {
        let update_fn = async || {
            self.model_map
                .get_mut(&page_id)
                .unwrap()
                .downcast_mut::<Model>()
                .unwrap()
                .update(message)
                .await
        };
        block_on(update_fn())
    }

    fn update(&mut self, message: ReSetMessage) -> Task<ReSetMessage> {
        match message {
            ReSetMessage::SubMsgAudio(audio_msg) => {
                let output = self
                    .update_submodel::<AudioModel<'static>, AudioMsg, ()>(PageId::Audio, audio_msg);
                if let Some(task) = output {
                    task
                } else {
                    Task::none()
                }
            }
            ReSetMessage::SubMsgNetwork(network_msg) => {
                let _ = self.update_submodel::<NetworkModel<'static>, NetworkMsg, ()>(
                    PageId::Network,
                    network_msg,
                );
                Task::none()
            }
            ReSetMessage::SubMsgBluetooth(bluetooth_msg) => {
                let output = self.update_submodel::<BluetoothModel<'static>, BluetoothMsg, ()>(
                    PageId::Bluetooth,
                    bluetooth_msg,
                );
                if let Some(task) = output {
                    task
                } else {
                    Task::none()
                }
            }
            ReSetMessage::SubPluginMsg(id, msg) => {
                let plugin = self.plugin_funcs.get(&id).unwrap();
                let model = self.model_map.get(&PageId::Plugin(id)).unwrap();
                let update_func = plugin.update.clone();
                unsafe {
                    let _ = (update_func)(model, &msg);
                }
                Task::none()
            }
            ReSetMessage::SetPage(page_id) => {
                if page_id == self.current_page {
                    Task::none()
                } else {
                    self.current_page = page_id;
                    Task::batch([
                        PageId::leave(&self.current_page, self.plugin_funcs.clone()),
                        PageId::enter(&page_id, self.plugin_funcs.clone()),
                        Task::done(ReSetMessage::StartWorker(
                            page_id,
                            self.ctx.clone(),
                            self.plugin_funcs.clone(),
                        )),
                    ])
                }
            }
            ReSetMessage::StartWorker(page_id, connection, plugins) => {
                match &mut self.sender {
                    SenderOrNone::None => (),
                    SenderOrNone::Sender(sender) => {
                        let fun = async || {
                            sender
                                .send(ReSetMessage::StartWorker(page_id, connection, plugins))
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
            let plugin = EntryCategory {
                main_entry: EntryButton {
                    title: "Plugin",
                    icon: None,
                    msg: ReSetMessage::SetPage(PageId::Plugin(0)),
                    level: EntryButtonLevel::TopLevel,
                },
                sub_entries: Vec::new(),
            };
            vec![audio, network, bluetooth, plugin]
        };
        row!(
            // TODO beforepr set audio and network
            sidebar(entries),
            // TODO beforepr make a wrapper over everything ->
            // 3 views  -> 1 box without sidebar -> 1 box with sidebar -> 2 boxes with sidebar
            scrollable(display_view_or_error(match self.current_page {
                PageId::Audio => self
                    .model_map
                    .get(&PageId::Audio)
                    .unwrap()
                    .downcast_ref::<AudioModel<'_>>()
                    .unwrap()
                    .view(),
                PageId::Network => self
                    .model_map
                    .get(&PageId::Network)
                    .unwrap()
                    .downcast_ref::<NetworkModel<'_>>()
                    .unwrap()
                    .view(),
                PageId::Bluetooth => self
                    .model_map
                    .get(&PageId::Bluetooth)
                    .unwrap()
                    .downcast_ref::<BluetoothModel<'_>>()
                    .unwrap()
                    .view(),
                PageId::Plugin(id) => {
                    let plugin = self.plugin_funcs.get(&id).unwrap();
                    let model = self.model_map.get(&PageId::Plugin(id)).unwrap();
                    let view_func = plugin.view.clone();
                    let view_res = unsafe { (view_func)(model) };
                    match view_res {
                        Ok(view) => {
                            Ok(view.map(move |msg| ReSetMessage::SubPluginMsg(id, Arc::new(msg))))
                        }
                        Err(err) => Err(err),
                    }
                }
            }))
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

    SETUP_PLUGIN_DIR();
    SETUP_LIBS();

    iced::application(ReSet::title, ReSet::update, ReSet::view)
        .window(window_settings)
        .theme(ReSet::theme)
        .default_font(Font::with_name("Adwaita Sans"))
        .subscription(ReSet::subscription)
        .run_with(ReSet::new)
}
