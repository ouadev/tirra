use iced::widget::text_editor::default;
use iced::widget::text_editor::Status;
use iced::widget::text_editor::Style;
use iced::{Background, Border, Theme};

use super::style_conf;

pub fn main_style(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let palette_tirra = style_conf::Palette::LIGHT;
    let base = Style {
        background: Background::Color(palette_tirra.background_main),
        border: Border {
            radius: 0.0.into(),
            width: 0.,
            color: palette.background.strong.color,
        },
        value: palette_tirra.text,
        ..default(theme, status)
    };

    match status {
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
}
