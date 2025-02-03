use enum_stringify::EnumStringify;
use oxiced::widgets::oxi_svg::SvgStyleVariant;

#[derive(EnumStringify)]
pub enum Icon {
    MicMuted,
    Mic,
    Volume,
    VolumeMuted,
}

fn path(icon: Icon) -> String {
    format!("./assets/{}.svg", icon)
}

pub fn icon_widget<'a>(icon: Icon) -> iced::widget::Svg<'a> {
    oxiced::widgets::oxi_svg::svg_from_path(SvgStyleVariant::Primary, path(icon))
}
