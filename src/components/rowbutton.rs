use iced::{border::Radius, widget, Border};

pub enum RowbuttonPosition {
    Start,
    Between,
    End,
    Only,
}

pub fn radius(at: RowbuttonPosition) -> Radius {
    match at {
        RowbuttonPosition::Start => Radius::new(10).top_right(0).top_left(0),
        RowbuttonPosition::Between => Radius::new(0),
        RowbuttonPosition::End => Radius::new(10).bottom_right(0).bottom_left(0),
        RowbuttonPosition::Only => Radius::new(10),
    }
}

pub fn style(style: widget::button::Style, at: RowbuttonPosition) -> widget::button::Style {
    widget::button::Style {
        border: Border {
            radius: radius(at),
            ..style.border
        },
        ..style
    }
}
