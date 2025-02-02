use iced::{widget::row, Element};

use crate::Message;

use super::wireless::{WirelessModel, WirelessMsg};

#[derive(Default, Debug, Clone)]
pub enum NetworkPageId {
    #[default]
    Wireless,
}

#[derive(Default)]
pub struct NetworkModel {
    current_page: NetworkPageId,
    wireless_model: WirelessModel,
}

#[derive(Debug, Clone)]
pub enum NetworkMsg {
    SubMsgWireless(WirelessMsg),
}

impl NetworkModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: NetworkMsg) {
        match msg {
            NetworkMsg::SubMsgWireless(wireless_msg) => self.wireless_model.update(wireless_msg),
        }
    }

    pub fn view(&self) -> Element<Message> {
        println!("display network");
        match self.current_page {
            NetworkPageId::Wireless => self.wireless_model.view(),
        }
    }
}
