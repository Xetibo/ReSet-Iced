use iced::{
    alignment::{Horizontal, Vertical},
    widget::{column, row, text, Button, Radio, Slider},
    Element, Length,
};

use crate::utils::rounded_card;

pub struct AudioDeviceCard<'a, C, Message> {
    mute_button: Button<'a, Message>,
    radio: Radio<'a, Message>,
    name: String,
    slider: Slider<'a, C, Message>,
}

impl<'a, C, Message> AudioDeviceCard<'a, C, Message>
where
    C: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: std::clone::Clone + 'a,
{
    pub fn new(
        mute_button: Button<'a, Message>,
        slider: Slider<'a, C, Message>,
        radio: Radio<'a, Message>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            mute_button,
            radio,
            name: name.into(),
            slider,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        iced::widget::container(
            column!(
                row!(text(self.name).width(Length::Fill), self.radio)
                    .padding(20)
                    .align_y(Vertical::Center),
                row!(self.mute_button, self.slider)
                    .padding(20)
                    .spacing(20)
                    .align_y(Vertical::Center),
            )
            .spacing(20)
            .align_x(Horizontal::Left),
        )
        .padding(5)
        .style(rounded_card)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}
