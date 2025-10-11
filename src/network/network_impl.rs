use iced::{Element, Task};
use re_set_lib::utils::error::ReSetError;
use zbus::Connection;

use crate::{utils::TPage, ReSetMessage};

use super::wireless_impl::{WirelessModel, WirelessMsg};

#[derive(Default, Debug, Clone)]
pub enum NetworkPageId {
    #[default]
    Wireless,
}

#[derive(Debug)]
pub struct NetworkModel<'a> {
    current_page: NetworkPageId,
    wireless_model: WirelessModel<'a>,
}

impl<'a> TPage<NetworkMsg, NetworkModel<'a>, ()> for NetworkModel<'a> {
    fn enter() -> Task<ReSetMessage> {
        WirelessModel::enter()
    }

    fn leave() -> Task<ReSetMessage> {
        WirelessModel::leave()
    }

    async fn update(&mut self, msg: NetworkMsg) -> Option<Task<ReSetMessage>> {
        match msg {
            NetworkMsg::SubMsgWireless(wireless_msg) => {
                let _ = self.wireless_model.update(wireless_msg).await?;
            }
        }
        None
    }

    async fn new(ctx: &Connection, _: ()) -> Result<Self, ReSetError> {
        let wireless_model = WirelessModel::new(ctx, ()).await?;
        Ok(Self {
            current_page: NetworkPageId::Wireless,
            wireless_model,
        })
    }

    fn view(&self) -> Result<Vec<Element<'_, ReSetMessage>>, ReSetError> {
        match self.current_page {
            NetworkPageId::Wireless => self.wireless_model.view(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NetworkMsg {
    SubMsgWireless(WirelessMsg),
}
