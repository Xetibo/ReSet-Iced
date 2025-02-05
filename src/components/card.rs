use std::borrow::Borrow;

use iced::{
    widget::{column, container, row, Button, Slider},
    Element,
};

use super::comborow::CustomPickList;

pub struct Card<'a, T, V, L, C, Message>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    picker: CustomPickList<'a, T, L, V, Message>,
    mute_button: Button<'a, Message>,
    slider: Slider<'a, C, Message>,
}

impl<'a, T, V, L, C, Message> Card<'a, T, V, L, C, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    C: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: std::clone::Clone + 'a,
{
    pub fn new(
        picker: CustomPickList<'a, T, L, V, Message>,
        mute_button: Button<'a, Message>,
        slider: Slider<'a, C, Message>,
    ) -> Self {
        Self {
            picker,
            mute_button,
            slider,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        container(
            column!(
                self.picker,
                row!(self.mute_button, self.slider).padding(20).spacing(20),
            )
            .spacing(20),
        )
        .padding(20)
        .style(container::rounded_box)
        .into()
    }
}
