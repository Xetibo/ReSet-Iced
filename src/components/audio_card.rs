use std::{borrow::Borrow, collections::HashMap, ops::RangeInclusive};

use iced::{
    alignment::{Horizontal, Vertical},
    border,
    widget::{column, container::Style, row, text, Button, Slider},
    Element, Length, Theme,
};
use oxiced::widgets::{
    oxi_button::{button, ButtonVariant},
    oxi_radio::radio,
    oxi_slider,
};

use crate::{
    audio::{audio_impl::AudioMsg, dbus_interface::TAudioObject},
    ReSetMessage,
};

use super::{
    audio_device_card::AudioDeviceCard,
    comborow::{ComboPickerTitle, CustomPickList, PickerVariant},
    icons::{icon_widget, Icon},
};

pub trait TCardUser {
    fn volume_fn(index: u32, channels: u16, volume: u32) -> AudioMsg;
    fn mute_fn(index: u32, muted: bool) -> AudioMsg;
    fn default_fn(index: u32) -> AudioMsg;
    fn muted_icon() -> Icon;
    fn unmuted_icon() -> Icon;
    fn title() -> String;
}

pub trait TStreamCardUser<C> {
    fn volume_fn(index: u32, channels: u16, volume: u32) -> AudioMsg;
    fn mute_fn(index: u32, muted: bool) -> AudioMsg;
    fn default_fn(self, obj: C) -> AudioMsg;
    fn muted_icon() -> Icon;
    fn unmuted_icon() -> Icon;
    // TODO beforepr implement and use
    fn title() -> String;
    fn obj_index(&self) -> u32;
}

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

pub fn populate_audio_cards<'a, OBJ, STREAM>(
    index: u32,
    object_map: &'a HashMap<u32, OBJ>,
    stream_map: &'a HashMap<u32, STREAM>,
) -> Element<'a, ReSetMessage>
where
    OBJ: TAudioObject + TCardUser + std::fmt::Display + Clone + PartialEq + 'a,
    STREAM: TAudioObject + TStreamCardUser<OBJ> + Clone + PartialEq + 'a,
{
    let object = card_from_audio_object::<OBJ>(index, object_map).view();
    let stream_cards: Vec<Element<ReSetMessage>> = stream_map
        .values()
        .filter_map(|value| stream_card_view::<STREAM, OBJ>(value.clone(), object_map))
        .collect();
    let mut col = column!(
        object,
        iced::widget::Space::with_height(10),
        iced::widget::Rule::horizontal(2),
        iced::widget::Space::with_height(10),
    );
    let stream_count = if stream_cards.is_empty() {
        0
    } else {
        stream_cards.len() - 1
    };
    for (i, elem) in stream_cards.into_iter().enumerate() {
        col = col.push(elem);
        if i != stream_count {
            col = col.push(iced::widget::Rule::horizontal(2));
        }
    }
    column!(text(OBJ::title()).size(30), col.spacing(20))
        .padding(20)
        .spacing(20)
        .into()
}

fn get_volume_level(volume: &[u32]) -> u32 {
    // TODO beforepr does this always exist?
    *volume.first().unwrap()
}

fn wrap(audio_msg: AudioMsg) -> ReSetMessage {
    ReSetMessage::SubMsgAudio(audio_msg)
}

pub fn card_from_audio_object<T>(
    index: u32,
    object_map: &HashMap<u32, T>,
) -> Card<'_, T, T, Vec<T>, ReSetMessage>
where
    T: Clone + ToString + PartialEq,
    T: TAudioObject + TCardUser,
{
    let object = object_map.get(&index).unwrap().clone();

    let current_volume = get_volume_level(&object.volume());
    let channels = object.channels();
    let slider = oxi_slider::slider(
        RangeInclusive::new(0, 100_270),
        current_volume,
        move |value| wrap(T::volume_fn(index, channels, value)),
    )
    .step(2000_u32);

    let objects: Vec<T> = object_map.clone().into_values().collect();
    let pick_list = CustomPickList::new(
        PickerVariant::ComboPicker(ComboPickerTitle::new(object.alias(), None::<String>)),
        objects,
        Some(object.clone()),
        move |object| wrap(T::default_fn(object.index())),
    );

    let icon = if object.muted() {
        icon_widget(T::muted_icon())
    } else {
        icon_widget(T::unmuted_icon())
    }
    .width(Length::Shrink);
    let mute_button =
        button(icon, ButtonVariant::Primary).on_press(wrap(T::mute_fn(index, !object.muted())));

    Card::new(pick_list, mute_button, slider, current_volume)
}

pub fn device_card_view<T>(
    default_index: u32,
    object_map: &HashMap<u32, T>,
) -> Element<'_, ReSetMessage>
where
    T: Clone + ToString + PartialEq,
    T: TAudioObject + TCardUser,
{
    let objects: Vec<T> = object_map.clone().into_values().collect();

    let create_card = |object: T| {
        let radio = radio("", object.index(), Some(default_index), |index| {
            wrap(T::default_fn(index))
        });

        let icon = if object.muted() {
            icon_widget(T::muted_icon())
        } else {
            icon_widget(T::unmuted_icon())
        }
        .width(Length::Shrink);
        let mute_button = button(icon, ButtonVariant::Primary)
            .on_press(wrap(T::mute_fn(object.index(), !object.muted())));

        let current_volume = get_volume_level(&object.volume());
        let index = object.index();
        let channels = object.channels();
        let slider = oxi_slider::slider(
            RangeInclusive::new(0, 100_270),
            current_volume,
            move |value| wrap(T::volume_fn(index, channels, value)),
        );
        AudioDeviceCard::new(mute_button, slider, radio, object.name())
    };

    let cards: Vec<Element<ReSetMessage>> = objects
        .into_iter()
        .map(create_card)
        .map(AudioDeviceCard::view)
        .collect();

    column!(
        text(format!("{} devices", T::title())).size(30),
        iced::widget::Column::with_children(cards).spacing(20)
    )
    .spacing(20)
    .padding(20)
    .into()
}

pub fn stream_card_view<'a, T, C>(
    stream: T,
    object_map: &HashMap<u32, C>,
) -> Option<Element<'a, ReSetMessage>>
where
    T: TAudioObject + TStreamCardUser<C> + Clone + PartialEq + 'a,
    C: TAudioObject + TCardUser + 'a + ToString + Clone + PartialEq,
{
    // TODO beforepr number?
    let current_obj = object_map.get(&stream.obj_index())?;

    let current_volume = get_volume_level(&stream.volume());
    let index = stream.index();
    let channels = stream.channels();
    let slider = oxi_slider::slider(
        RangeInclusive::new(0, 100_270),
        current_volume,
        move |value| wrap(T::volume_fn(index, channels, value)),
    )
    .step(2000_u32);

    let objects: Vec<C> = object_map.clone().into_values().collect();
    let stream_clone = stream.clone();
    let pick_list = CustomPickList::new(
        PickerVariant::ComboPicker(ComboPickerTitle::new(
            format!("{}: {}", stream.alias(), stream.name()),
            Some(current_obj.alias()),
        )),
        objects,
        Some(current_obj.clone()),
        move |obj| wrap(T::default_fn(stream_clone.clone(), obj)),
    );

    let icon = if stream.muted() {
        icon_widget(T::muted_icon())
    } else {
        icon_widget(T::unmuted_icon())
    }
    .width(Length::Shrink);
    let mute_button =
        button(icon, ButtonVariant::Primary).on_press(wrap(T::mute_fn(index, !stream.muted())));

    let card = Card::new(pick_list, mute_button, slider, current_volume);
    Some(card.view())
}
