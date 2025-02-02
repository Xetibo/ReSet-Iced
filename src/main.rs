use audio::{AudioModel, AudioMsg};
use iced::{widget::column, widget::row, Application, Element, Task, Theme};
use network::network::{NetworkModel, NetworkMsg};
use oxiced::widgets::oxi_button::{button, ButtonVariant};

mod audio;
mod network;

#[derive(Default, Debug, Clone)]
enum PageId {
    // Chosen as it is probably the most useful page
    #[default]
    Audio,
    Network,
}

#[derive(Default)]
struct ReSet<'a> {
    current_page: PageId,
    audio_model: AudioModel<'a>,
    network_model: NetworkModel,
}

#[derive(Debug, Clone)]
enum Message {
    SubMsgAudio(AudioMsg),
    SubMsgNetwork(NetworkMsg),
    SetPage(PageId),
}

impl<'a> ReSet<'a> {
    //fn subscription(&self) -> iced::Subscription<Self::Message> {
    //    event::listen_with(|event, _status, _id| match event {
    //        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
    //            modifiers: _,
    //            key: iced::keyboard::key::Key::Named(Named::Escape),
    //            modified_key: _,
    //            physical_key: _,
    //            location: _,
    //            text: _,
    //        }) => Some(Message::Exit),
    //        _ => None,
    //    })
    //}
    //
    fn theme(&self) -> Theme {
        oxiced::theme::get_theme()
    }

    // remove the annoying background color
    //fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
    //    let palette = theme.extended_palette();
    //    iced_layershell::Appearance {
    //        background_color: iced::Color::TRANSPARENT,
    //        text_color: palette.background.base.text,
    //    }
    //}
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                ..Default::default()
            },
            iced::widget::text_input::focus("search_box"),
        )
    }

    fn title(&self) -> String {
        String::from("OxiPaste")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SubMsgAudio(audio_msg) => self.audio_model.update(audio_msg),
            Message::SubMsgNetwork(network_msg) => self.network_model.update(network_msg),
            Message::SetPage(page_id) => self.current_page = page_id,
        }
        Task::none()
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

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

pub fn main() -> Result<(), iced::Error> {
    iced::application(ReSet::title, ReSet::update, ReSet::view)
        .theme(ReSet::theme)
        .run_with(ReSet::new)
}
