use iced::widget::button;
use iced::widget::button::Status;
use iced::widget::button::Style;
use iced::{Background, Border, Color, Shadow, Theme};

use super::style_conf;

//#[derive(Clone, Copy, Default)]
/*
pub enum ButtonType {
    #[default]
    Standard,
}
    */
/*
pub enum TirraButtonType {
    Entry,
    Main,
}

pub struct TirraButtonStyle {
    pub button_type: TirraButtonType,
    pub selected: bool,
}
*/

pub fn button_main(_theme: &Theme, status: Status) -> Style {
    //let palette = theme.extended_palette();
    let palette = style_conf::TirraPalette::LIGHT;
    let base = Style {
        background: Some(Background::Color(palette.control_main)),
        text_color: Color::BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
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
    let palette = style_conf::TirraPalette::LIGHT;
    let base = Style {
        background: Some(Background::Color(palette.background_secondary)),
        text_color: Color::BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
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
    let palette = style_conf::TirraPalette::LIGHT;
    let base = Style {
        background: Some(Background::Color(palette.background_main)),
        text_color: Color::BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
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
