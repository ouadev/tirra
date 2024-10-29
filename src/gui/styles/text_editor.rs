use iced::widget::text_editor::default;
use iced::widget::text_editor::Status;
use iced::widget::text_editor::Style;
use iced::{Background, Border, Theme};

use super::style_conf;

pub fn main_style(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = Style {
        background: Background::Color(style_conf::STYLE_EDITOR_BG_COLOR),
        border: Border {
            radius: 0.0.into(),
            width: 0.,
            color: palette.background.strong.color,
        },
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

