use iced::widget::button;
use iced::widget::button::Status;
use iced::widget::button::Style;
use iced::{Background, Border, Color, Shadow, Theme};

use super::style_conf;

pub fn button_main(_theme: &Theme, status: Status) -> Style {
    //let palette = theme.extended_palette();
    let palette = style_conf::palette();
    let base = Style {
        background: Some(Background::Color(palette.control_main)),
        text_color: palette.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.background_main)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(palette.control_disabled)),
            ..base
        },
    }
}

pub fn button_entry(_theme: &Theme, status: button::Status) -> button::Style {
    let palette = style_conf::palette();
    let base = Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: palette.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.background_main)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(palette.control_disabled)),
            ..base
        },
    }
}

pub fn button_entry_selected(_theme: &Theme, status: button::Status) -> button::Style {
    //let palette = theme.extended_palette();
    let palette = style_conf::palette();
    let base = Style {
        background: Some(Background::Color(palette.background_main)),
        text_color: palette.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.background_main)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(palette.control_disabled)),
            ..base
        },
    }
}
