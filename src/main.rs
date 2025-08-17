use audio::{
    audio_impl::{watch_audio_dbus_signals, AudioModel, AudioMsg, AudioVariant},
    dbus_interface::AudioDbusProxy,
};
use bluetooth::{
    bluetooth_impl::{watch_bluetooth_dbus_signals, BluetoothModel, BluetoothMsg},
    dbus_interface::BluetoothDbusProxy,
};
use components::{
    icons::{icon_widget, Icon},
    sidebar::sidebar,
};
use dbus_interface::ReSetDbusProxy;
use iced::{
    event,
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
    widget::{column, container, mouse_area, opaque, row, scrollable, stack, Column},
    window::Settings,
    Color, Element, Event, Font,
    Length::{self, Fill},
    Size, Subscription, Task, Theme,
};
use libloading::Symbol;
use network::{
    dbus_interface::WifiDbusProxy,
    network_impl::{NetworkModel, NetworkMsg},
    wireless_impl::{watch_wireless_dbus_signals, WirelessModel},
};
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use plugins::{SETUP_LIBS, SETUP_PLUGIN_DIR};
use re_set_lib::{
    utils::{any::ReSetAny, iced_sidebar::EntryButton},
    write_log_to_file,
};
use re_set_lib::{
    utils::{error::ReSetError, iced_sidebar::EntryCategory},
    LOG,
};
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

const NAME: &str = "ReSet-Iced";
const COMPATIBLE_API: &str = "2.2.0";

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
    pub sidebar_entries: libloading::Symbol<'static, unsafe extern "C" fn() -> EntryCategory>,
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

impl From<PageId> for u8 {
    fn from(val: PageId) -> Self {
        match val {
            PageId::Audio => 0,
            PageId::Network => 1,
            PageId::Bluetooth => 2,
            PageId::Plugin(id) => 3 + id,
        }
    }
}

impl From<u8> for PageId {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Audio,
            1 => Self::Bluetooth,
            2 => Self::Network,
            id => PageId::Plugin(id - 3),
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
                let leave = plugin_funcs.get(id).unwrap().enter.clone();
                let newid = *id;
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
                let leave = plugin_funcs.get(id).unwrap().leave.clone();
                let newid = *id;
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
    proxy: ReSetDbusProxy<'static>,
    sender: SenderOrNone,
    ctx: Arc<Connection>,
    current_page: PageId,
    model_map: HashMap<PageId, &'static mut dyn ReSetAny>,
    plugin_funcs: HashMap<u8, PluginFuncs>,
    layout: HorizontalLayout,
    sidebar_open: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum HorizontalLayout {
    ThreeColumnsWithSidebar,
    TwoColumnsWithSidebar,
    OneColumnWithSidebar,
    OneColumn,
}

impl From<&Size> for HorizontalLayout {
    fn from(size: &Size) -> Self {
        match size.width {
            0.0..800.0 => HorizontalLayout::OneColumn,
            801.0..1600.0 => HorizontalLayout::OneColumnWithSidebar,
            1601.0..2400.0 => HorizontalLayout::TwoColumnsWithSidebar,
            _ => HorizontalLayout::ThreeColumnsWithSidebar,
        }
    }
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
    Event(Event),
    LayoutMsg(HorizontalLayout),
    ExpandSidebar(bool),
    Exit,
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
            ReSetMessage::Event(event) => Self::Event(event.clone()),
            ReSetMessage::LayoutMsg(layout) => Self::LayoutMsg(*layout),
            ReSetMessage::ExpandSidebar(open) => Self::ExpandSidebar(*open),
            ReSetMessage::Exit => Self::Exit,
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
                return;
            },
            default => {
                daemonfn(output, &mut signals).await.expect("")
            }
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
            default => unsafe {
                daemonfn(output as &mut dyn ReSetAny, &mut signals).expect("")}
        }
    }
}

fn some_worker() -> impl Stream<Item = ReSetMessage> {
    stream::channel(100, move |mut output: Sender<ReSetMessage>| async move {
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

// TODO
fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let modal_element = opaque(
        mouse_area(
            container(opaque(content))
                .style(|_theme| container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                })
                .align_left(Length::Fill),
        )
        .on_press(on_blur),
    );
    stack![base.into(), modal_element]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn split_elems<'a, Message>(elems: Vec<Element<'a, Message>>, count: i32) -> Element<'a, Message>
where
    Message: 'a,
{
    fn split_col<'a, Message>(
        mut elems: Vec<Element<'a, Message>>,
        split: usize,
        leftover: &mut usize,
    ) -> (Vec<Element<'a, Message>>, Vec<Element<'a, Message>>) {
        if *leftover > 0 {
            *leftover -= 1;
            let new = elems.split_off(split + 1);
            (elems, new)
        } else {
            let new = elems.split_off(split);
            (elems, new)
        }
    }

    if elems.len() < 2 {
        return Column::from_vec(elems).width(Fill).into();
    }
    let mut leftover = elems.len() % count as usize;
    let split = elems.len() / count as usize;

    if count == 2 || elems.len() == 2 {
        let (first, second) = split_col(elems, split, &mut leftover);
        row!(
            Column::from_vec(first).width(Fill),
            Column::from_vec(second).width(Fill)
        )
        .width(Fill)
        .into()
    } else {
        let (first, second_and_third) = split_col(elems, split, &mut leftover);
        let (second, third) = split_col(second_and_third, split, &mut leftover);
        row!(
            Column::from_vec(first).width(Fill),
            Column::from_vec(second).width(Fill),
            Column::from_vec(third).width(Fill)
        )
        .width(Fill)
        .into()
    }
}

impl ReSet {
    fn subscription(&self) -> Subscription<ReSetMessage> {
        let subs = [
            Subscription::run(some_worker),
            event::listen().map(ReSetMessage::Event),
        ];
        Subscription::batch(subs)
    }

    fn theme(&self) -> Theme {
        oxiced::theme::theme::get_derived_iced_theme()
    }

    async fn setup_daemon() -> Result<ReSetDbusProxy<'static>, iced::Error> {
        let conn = Connection::session().await.unwrap();
        let reset_proxy = ReSetDbusProxy::new(&conn).await.unwrap();

        let res = reset_proxy.register_client(NAME).await;

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
            let res = reset_proxy.api_version().await;
            if let Ok(api_version) = res {
                if api_version != COMPATIBLE_API {
                    return Err(iced::Error::WindowCreationFailed(Box::from(format!(
                        "mismatch on api version of daemon: got {} but need {}",
                        api_version, COMPATIBLE_API
                    ))));
                }
            } else {
                return Err(iced::Error::WindowCreationFailed(Box::from(
                    "Could not get api version from daemon",
                )));
            }
            // Second try without any catch, this means there was no way to connect to the daemon
            reset_proxy
                .register_client(NAME)
                .await
                .expect("Failed to get daemon");
            LOG!("Using Bundled Daemon")
        }
        Ok(reset_proxy)
    }

    fn new() -> (Self, Task<ReSetMessage>) {
        // TODO how to handle this error without panic?
        let proxy =
            block_on(ReSet::setup_daemon()).expect("Could not create a connection to ReSet daemon");
        // TODO get from initial size instead
        let layout = HorizontalLayout::OneColumnWithSidebar;
        let sidebar_open = false;

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

        let plugin_funcs: HashMap<u8, PluginFuncs> = HashMap::new();
        let index = 0;
        // TODO
        //for plugin in load_plugins() {
        //    let modelfn = plugin.model.clone();
        //    let model = unsafe { (modelfn)(&ctx.clone(), &mut () as &mut dyn ReSetAny) };
        //    model_map.insert(PageId::Plugin(index), model);
        //    plugin_funcs.insert(index as u8, plugin);
        //    index += 1;
        //}
        (
            Self {
                proxy,
                sender: SenderOrNone::None,
                ctx: ctx.clone(),
                current_page: Default::default(),
                model_map,
                plugin_funcs,
                layout,
                sidebar_open,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ReSet")
    }

    fn handle_event(event: &Event) -> Task<ReSetMessage> {
        match event {
            Event::Keyboard(keyboard_event) => match keyboard_event {
                iced::keyboard::Event::KeyPressed {
                    key,
                    modified_key,
                    physical_key,
                    location,
                    modifiers,
                    text,
                } => Task::none(),
                iced::keyboard::Event::KeyReleased {
                    key,
                    location,
                    modifiers,
                    modified_key,
                    physical_key,
                } => Task::none(),
                iced::keyboard::Event::ModifiersChanged(modifiers) => Task::none(),
            },
            Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::CursorEntered => Task::none(),
                iced::mouse::Event::CursorLeft => Task::none(),
                iced::mouse::Event::CursorMoved { position } => Task::none(),
                iced::mouse::Event::ButtonPressed(button) => Task::none(),
                iced::mouse::Event::ButtonReleased(button) => Task::none(),
                iced::mouse::Event::WheelScrolled { delta } => Task::none(),
            },
            Event::Window(window_event) => match window_event {
                iced::window::Event::Opened { position, size } => Task::none(),
                iced::window::Event::Closed => Task::none(),
                iced::window::Event::Moved(point) => Task::none(),
                iced::window::Event::Resized(size) => {
                    let layout = HorizontalLayout::from(size);
                    Task::done(ReSetMessage::LayoutMsg(layout))
                }
                iced::window::Event::RedrawRequested(instant) => Task::none(),
                iced::window::Event::CloseRequested => Task::done(ReSetMessage::Exit),
                iced::window::Event::Focused => Task::none(),
                iced::window::Event::Unfocused => Task::none(),
                iced::window::Event::FileHovered(path_buf) => Task::none(),
                iced::window::Event::FileDropped(path_buf) => Task::none(),
                iced::window::Event::FilesHoveredLeft => Task::none(),
            },
            Event::Touch(touch_event) => match touch_event {
                iced::touch::Event::FingerPressed { id, position } => Task::none(),
                iced::touch::Event::FingerMoved { id, position } => Task::none(),
                iced::touch::Event::FingerLifted { id, position } => Task::none(),
                iced::touch::Event::FingerLost { id, position } => Task::none(),
            },
            Event::InputMethod(event) => Task::none(),
        }
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
            ReSetMessage::Event(event) => Self::handle_event(&event),
            ReSetMessage::LayoutMsg(layout) => {
                self.layout = layout;
                Task::none()
            }
            ReSetMessage::ExpandSidebar(open) => {
                self.sidebar_open = open;
                Task::none()
            }
            ReSetMessage::Exit => {
                block_on(async move {
                    let _ = self.proxy.register_client(NAME).await;
                });
                iced::exit::<ReSetMessage>()
            }
        }
    }

    fn sidebar_elements(&self) -> Vec<EntryCategory> {
        let audio_sub =
            vec![
                EntryButton::sub_level(
                    "Input",
                    Some(Icon::Mic),
                    Box::new(&ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                        AudioVariant::Input,
                    )) as &dyn ReSetAny),
                ),
                EntryButton::sub_level(
                    "Output",
                    Some(Icon::Volume),
                    Box::new(&ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                        AudioVariant::Output,
                    )) as &dyn ReSetAny),
                ),
                EntryButton::sub_level(
                    "Cards",
                    Some(Icon::AudioCards),
                    Box::new(&ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                        AudioVariant::Cards,
                    )) as &dyn ReSetAny),
                ),
                EntryButton::sub_level(
                    "Devices",
                    Some(Icon::AudioDevices),
                    Box::new(&ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                        AudioVariant::Devices,
                    )) as &dyn ReSetAny),
                ),
            ];
        let base_audio = EntryButton::top_level(
            "Audio",
            Some(Icon::Audio),
            Box::new(&ReSetMessage::SubMsgAudio(AudioMsg::SetAudioVariant(
                AudioVariant::InputAndOutput,
            )) as &dyn ReSetAny),
        );
        let audio = EntryCategory {
            main_entry: base_audio,
            sub_entries: audio_sub,
        };
        let network = EntryCategory {
            main_entry: EntryButton::top_level(
                "Network",
                Some(Icon::Wifi),
                Box::new(&ReSetMessage::SetPage(PageId::Network) as &dyn ReSetAny),
            ),
            sub_entries: Vec::new(),
        };
        let bluetooth = EntryCategory {
            main_entry: EntryButton::top_level(
                "Bluetooth",
                Some(Icon::Bluetooth),
                Box::new(&ReSetMessage::SetPage(PageId::Bluetooth) as &dyn ReSetAny),
            ),
            sub_entries: Vec::new(),
        };
        let mut entries = vec![audio, network, bluetooth];
        unsafe {
            let mut plugin_entries: Vec<EntryCategory> = self
                .plugin_funcs
                .clone()
                .iter()
                .map(|funcs| (funcs.1.sidebar_entries)())
                .collect();
            entries.append(&mut plugin_entries);
        }
        entries
    }

    fn top_row(&self) -> Element<ReSetMessage> {
        // TODO use icons
        let close_button: Element<'_, ReSetMessage> = container(
            button(
                icon_widget(Icon::Exit)
                    .width(Length::Fill)
                    .height(Length::Fill),
                ButtonVariant::Neutral,
            )
            .on_press(ReSetMessage::Exit)
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0)),
        )
        .align_right(Length::Fill)
        .into();
        let sidebar_button: Element<'_, ReSetMessage> = match self.layout {
            HorizontalLayout::OneColumn => button(
                match self.sidebar_open {
                    true => icon_widget(Icon::SidebarOpen)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    false => icon_widget(Icon::SidebarClose)
                        .width(Length::Fill)
                        .height(Length::Fill),
                },
                ButtonVariant::Neutral,
            )
            .on_press(ReSetMessage::ExpandSidebar(!self.sidebar_open))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .into(),
            _ => row!().into(),
        };
        row!(sidebar_button, close_button)
            .height(Length::Fixed(50.0))
            .width(Length::Fill)
            .padding(5)
            .into()
    }

    fn main_elements(&self) -> Vec<Element<ReSetMessage>> {
        let main_elements = match self.current_page {
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
                        Ok(vec![view.map(move |msg| {
                            ReSetMessage::SubPluginMsg(id, Arc::new(msg))
                        })])
                    }
                    Err(err) => Err(err),
                }
            }
        };
        display_view_or_error(main_elements)
    }

    fn view(&self) -> Element<ReSetMessage> {
        let sidebar_element = sidebar(self.sidebar_elements());
        let main_elements = self.main_elements();

        match self.layout {
            HorizontalLayout::ThreeColumnsWithSidebar => row!(
                sidebar_element,
                column!(
                    self.top_row(),
                    scrollable(split_elems(main_elements, 3)).width(Fill)
                )
                .width(Fill)
            )
            .width(Fill)
            .into(),
            HorizontalLayout::TwoColumnsWithSidebar => row!(
                sidebar_element,
                column!(
                    self.top_row(),
                    scrollable(split_elems(main_elements, 2)).width(Fill)
                )
                .width(Fill)
            )
            .width(Fill)
            .into(),
            HorizontalLayout::OneColumnWithSidebar => row!(
                sidebar_element,
                column!(
                    self.top_row(),
                    scrollable(Column::from_vec(main_elements).width(Fill)).width(Fill)
                )
                .width(Fill)
            )
            .width(Fill)
            .into(),
            HorizontalLayout::OneColumn => {
                if self.sidebar_open {
                    modal(
                        column!(
                            self.top_row(),
                            scrollable(Column::from_vec(main_elements).width(Fill)).width(Fill)
                        )
                        .width(Fill),
                        sidebar_element,
                        ReSetMessage::ExpandSidebar(false),
                    )
                } else {
                    row!(column!(
                        self.top_row(),
                        scrollable(Column::from_vec(main_elements).width(Fill)).width(Fill)
                    )
                    .width(Fill))
                    .width(Fill)
                    .into()
                }
            }
        }
    }

    //fn scale_factor(&self) -> f64 {
    //    1.0
    //}
}

pub fn main() -> Result<(), iced::Error> {
    let icon = iced::window::icon::from_file("./assets/ReSet.png"); //.ok();
    let icon = if let Ok(icon) = icon {
        Some(icon)
    } else {
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
            application_id: NAME.into(),
            override_redirect: false,
        },
        exit_on_close_request: false,
        maximized: false,
        fullscreen: false,
    };

    SETUP_PLUGIN_DIR();
    SETUP_LIBS();

    iced::application(ReSet::new, ReSet::update, ReSet::view)
        .title(ReSet::title)
        .window(window_settings)
        .theme(ReSet::theme)
        .default_font(Font::with_name("Adwaita Sans"))
        .subscription(ReSet::subscription)
        .exit_on_close_request(true)
        .run()
}
