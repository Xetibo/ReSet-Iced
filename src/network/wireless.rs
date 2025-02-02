use iced::{widget::row, Element};

use crate::Message;

#[derive(Default)]
pub struct WirelessModel {}

#[derive(Debug, Clone)]
pub enum WirelessMsg {}

impl WirelessModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: WirelessMsg) {
        // TODO beforepr
        match msg {}
    }

    pub fn view(&self) -> Element<Message> {
        println!("display wireless page");
        row![].into()
    }
}
