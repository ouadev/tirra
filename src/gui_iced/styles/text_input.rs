use iced::widget::text_input::Status;
use iced::widget::text_input::Style;
use iced::{Background, Border, Color, Theme};

use super::style_conf;

/*
pub struct Style {
    pub background: Background,
    pub border: Border,
    pub icon: Color,
    pub placeholder: Color,
    pub value: Color,
    pub selection: Color,
}
*/
pub fn main_style(theme: &Theme, _status: Status) -> Style {
    let palette = theme.extended_palette();
    let palette_tirra = style_conf::palette();

    let base = Style {
        background: Background::Color(palette_tirra.background_main),
        border: Border {
            radius: 0.0.into(),
            width: 0.,
            color: palette.background.strong.color,
        },
        icon: Color::default(),
        placeholder: Color::default(),
        value: palette_tirra.text,
        selection: palette_tirra.control_main,
    };

    base

    /* match status {
        Status::Active => base,
        Status::Hovered => Style {
            border: Border {
                radius: 0.0.into(),
                width: 0.,
                color: palette.background.base.text,
            },
            ..base
        },
        Status::Focused => Style {
            border: Border {
                radius: 0.0.into(),
                width: 0.,
                color: palette.primary.strong.color,
            },
            ..base
        },
        Status::Disabled => base,
    }
    */
}
