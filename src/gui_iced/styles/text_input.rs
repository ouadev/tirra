use iced::color;
use iced::widget::text_input::Status;
use iced::widget::text_input::Style;
use iced::{Background, Border, Color, Theme};

use super::style_conf;

pub fn main_style(_theme: &Theme, _status: Status) -> Style {
    let palette_tirra = style_conf::palette();

    let base = Style {
        background: Background::Color(palette_tirra.background_main),
        border: Border {
            radius: 5.0.into(),
            width: 0.,
            color: Color::default(),
        },
        icon: Color::default(),
        placeholder: color!(0x8f8f8f),
        value: palette_tirra.text,
        selection: palette_tirra.control_main,
    };

    base
}

pub fn transparent_style(_theme: &Theme, _status: Status) -> Style {
    let palette_tirra = style_conf::palette();

    let base = Style {
        background: Background::Color(palette_tirra.background_secondary),
        border: Border {
            radius: 5.0.into(),
            width: 0.,
            color: Color::default(),
        },
        icon: Color::default(),
        placeholder: color!(0x8f8f8f),
        value: palette_tirra.text,
        selection: palette_tirra.control_main,
    };

    base
}

pub fn main_style_borders(_theme: &Theme, _status: Status) -> Style {
    let palette_tirra = style_conf::palette();

    let base = Style {
        background: Background::Color(palette_tirra.background_main),
        border: Border {
            radius: 5.0.into(),
            width: 2.,
            color: palette_tirra.background_secondary,
        },
        icon: Color::default(),
        placeholder: color!(0x8f8f8f),
        value: palette_tirra.text,
        selection: palette_tirra.control_main,
    };

    base
}
