use std::borrow::Borrow;

use iced::{
    alignment::{Horizontal, Vertical},
    border,
    widget::{column, container::Style, row, text, Button, Slider},
    Element, Theme,
};

use super::comborow::CustomPickList;

pub struct Card<'a, T, V, L, Message>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    picker: CustomPickList<'a, T, L, V, Message>,
    mute_button: Button<'a, Message>,
    slider: Slider<'a, u32, Message>,
    current_value: u32,
}

impl<'a, T, V, L, Message> Card<'a, T, V, L, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: std::clone::Clone + 'a,
{
    pub fn new(
        picker: CustomPickList<'a, T, L, V, Message>,
        mute_button: Button<'a, Message>,
        slider: Slider<'a, u32, Message>,
        current_value: u32,
    ) -> Self {
        Self {
            picker,
            mute_button,
            slider,
            current_value,
        }
    }

    fn style(theme: &Theme) -> Style {
        let palette = theme.extended_palette();

        Style {
            background: Some(palette.background.weak.color.into()),
            border: border::rounded(10),
            ..Style::default()
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        // TODO beforepr is this correct?? (prob not)
        let percentage = (100.0 / 65536.0 * self.current_value as f32) as u32;
        iced::widget::container(
            column!(
                self.picker,
                row!(
                    self.mute_button,
                    self.slider,
                    text(format!("{}%", percentage))
                )
                .padding(20)
                .spacing(20)
                .align_y(Vertical::Center),
            )
            .align_x(Horizontal::Left),
        )
        .padding(5)
        .style(Self::style)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}
