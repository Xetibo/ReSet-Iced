use iced::{
    color,
    widget::{
        radio::{Status, Style},
        text::LineHeight,
        Radio,
    },
    Theme,
};
use oxiced::widgets::common::{darken_color, lighten_color};

// TODO beforepr upstream this to oxiced
pub fn radio_style(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut style = Style {
        background: iced::Background::Color(color!(0x1E1E2E)),
        text_color: Some(palette.background.base.text),
        dot_color: color!(0x1E1E2E),
        border_width: 1.0,
        border_color: color!(0x333444),
    };
    match status {
        Status::Active { is_selected: true } => Style {
            background: iced::Background::Color(color!(0x89B4FA)),
            border_color: lighten_color(color!(0x333444)),
            ..style
        },
        Status::Active { is_selected: false } => Style {
            background: iced::Background::Color(color!(0x1E1E2E)),
            border_color: lighten_color(color!(0x1E1E2E)),
            ..style
        },
        Status::Hovered { is_selected: _ } => {
            style.background = iced::Background::Color(darken_color(color!(0x89B4FA)));
            style
        }
    }
}

pub fn reset_radio<'a, V, M>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> M + 'a,
) -> Radio<'a, M>
where
    V: Copy + Eq,
    M: Clone,
{
    iced::widget::radio(label, value, selected, on_click)
        .size(20)
        .spacing(10)
        .style(radio_style)
        .text_line_height(LineHeight::Relative(2.0))
}
