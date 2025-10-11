use iced::{
    widget::{
        radio::{Status, Style},
        text::LineHeight,
    },
    Theme,
};
use oxiced::{
    theme::theme_impl::OXITHEME, utils::color::darken_color, widgets::oxi_radio::OxiRadio,
};

// TODO beforepr upstream this to oxiced
pub fn radio_style(_: &Theme, status: Status) -> Style {
    let style = Style {
        background: iced::Background::Color(OXITHEME.mantle),
        text_color: Some(OXITHEME.text),
        dot_color: OXITHEME.mantle,
        border_width: 1.0,
        border_color: OXITHEME.border_color_weak,
    };
    match status {
        Status::Active { is_selected: true } | Status::Hovered { is_selected: true } => Style {
            background: iced::Background::Color(OXITHEME.primary),
            dot_color: OXITHEME.primary,
            ..style
        },
        Status::Hovered { is_selected: false } => Style {
            background: iced::Background::Color(darken_color(&OXITHEME.primary, 10.0)),
            dot_color: darken_color(&OXITHEME.primary, 10.0),
            ..style
        },
        Status::Active { is_selected: false } => style,
    }
}

pub fn reset_radio<'a, V, M>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl Fn(V) -> M + 'a,
) -> OxiRadio<'a, V, M>
where
    V: Copy + Eq,
    M: Clone,
{
    oxiced::widgets::oxi_radio::OxiRadio::<'a, V, M>::new(
        Some(label.into()),
        selected,
        value,
        Some(on_click),
    )
    .size(20)
    .spacing(10)
    .text_line_height(LineHeight::Relative(2.0))
}
