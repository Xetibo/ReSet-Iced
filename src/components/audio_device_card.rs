use iced::{
    alignment::{Horizontal, Vertical},
    widget::{column, row, Button, Slider},
    Element,
};
use oxiced::widgets::oxi_radio::OxiRadio;

use crate::utils::{rounded_card, OxiPadding};

pub struct AudioDeviceCard<'a, C, V, Message>
where
    V: std::clone::Clone + PartialEq + 'a,
{
    mute_button: Button<'a, Message>,
    radio: OxiRadio<'a, V, Message>,
    slider: Slider<'a, C, Message>,
}

impl<'a, C, V, Message> AudioDeviceCard<'a, C, V, Message>
where
    C: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: std::clone::Clone + 'a,
    V: std::clone::Clone + PartialEq + 'a,
{
    pub fn new(
        mute_button: Button<'a, Message>,
        slider: Slider<'a, C, Message>,
        radio: OxiRadio<'a, V, Message>,
    ) -> Self {
        Self {
            mute_button,
            radio,
            slider,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        iced::widget::container(
            column!(
                row!(self.radio)
                    .padding(OxiPadding::Large)
                    .align_y(Vertical::Center),
                row!(self.mute_button, self.slider)
                    .padding(OxiPadding::Large)
                    .spacing(OxiPadding::Large)
                    .align_y(Vertical::Center),
            )
            .spacing(OxiPadding::Large)
            .align_x(Horizontal::Left),
        )
        .padding(OxiPadding::Small)
        .style(rounded_card)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}
