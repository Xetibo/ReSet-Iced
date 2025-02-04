use std::sync::Arc;

use audio::audio_impl::{watch_audio_dbus_signals, AudioModel, AudioMsg};
use iced::{
    futures::{
        channel::mpsc::{self, Sender},
        executor::block_on,
        SinkExt, Stream, StreamExt,
    },
    stream,
    widget::column,
    Element, Subscription, Task, Theme,
};
use network::network::{NetworkModel, NetworkMsg};
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use zbus::Connection;

mod audio;
mod network;
mod utils;

#[derive(Default, Debug, Clone, Copy)]
enum PageId {
    // Chosen as it is probably the most useful page
    #[default]
    Audio,
    Network,
}

enum SenderOrNone {
    None,
    Sender(Sender<Message>),
}

struct ReSet {
    sender: SenderOrNone,
    ctx: Arc<Connection>,
    current_page: PageId,
    audio_model: AudioModel<'static>,
    network_model: NetworkModel,
}

#[derive(Debug, Clone)]
enum Message {
    SubMsgAudio(AudioMsg),
    SubMsgNetwork(NetworkMsg),
    SetPage(PageId),
    StartWorker(PageId, Arc<Connection>),
    ReceiveSender(Sender<Message>),
}

fn some_worker() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);
        // TODO beforepr handle error
        let _ = output.send(Message::ReceiveSender(sender)).await;

        loop {
            let input = receiver.select_next_some().await;
            if let Message::StartWorker(page_id, conn) = input {
                match page_id {
                    PageId::Audio => watch_audio_dbus_signals(&mut output, conn).await,
                    PageId::Network => (),
                }
            }
        }
    })
}

impl ReSet {
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(some_worker)
    }

    fn theme(&self) -> Theme {
        oxiced::theme::get_theme()
    }

    fn new() -> (Self, Task<Message>) {
        // TODO beforepr handle error
        let ctx = Arc::new(block_on(Connection::session()).unwrap());
        (
            Self {
                sender: SenderOrNone::None,
                ctx: ctx.clone(),
                current_page: Default::default(),
                audio_model: block_on(AudioModel::new(&ctx.clone())),
                network_model: Default::default(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ReSet")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SubMsgAudio(audio_msg) => {
                let update_fn = async || self.audio_model.update(audio_msg).await;
                block_on(update_fn());
                Task::none()
            }
            Message::SubMsgNetwork(network_msg) => {
                self.network_model.update(network_msg);
                Task::none()
            }
            Message::SetPage(page_id) => {
                self.current_page = page_id;
                Task::done(Message::StartWorker(page_id, self.ctx.clone()))
            }
            Message::StartWorker(page_id, connection) => {
                match &mut self.sender {
                    SenderOrNone::None => (),
                    SenderOrNone::Sender(sender) => {
                        let fun =
                            async || sender.send(Message::StartWorker(page_id, connection)).await;
                        let _ = block_on(fun());
                    }
                };
                Task::none()
            }
            Message::ReceiveSender(sender) => {
                self.sender = SenderOrNone::Sender(sender);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            // TODO beforepr set audio and network
            button("SetAudio", ButtonVariant::Primary).on_press(Message::SetPage(PageId::Audio)),
            button("SetNetwork", ButtonVariant::Primary)
                .on_press(Message::SetPage(PageId::Network)),
            // TODO beforepr make a wrapper over everything ->
            // 3 views  -> 1 box without sidebar -> 1 box with sidebar -> 2 boxes with sidebar
            match self.current_page {
                PageId::Audio => self.audio_model.view(),
                PageId::Network => self.network_model.view(),
            }
        ]
        .into()
    }

    //fn scale_factor(&self) -> f64 {
    //    1.0
    //}
}

#[tokio::main]
pub async fn main() -> Result<(), iced::Error> {
    iced::application(ReSet::title, ReSet::update, ReSet::view)
        .theme(ReSet::theme)
        .subscription(ReSet::subscription)
        .run_with(ReSet::new)
}
