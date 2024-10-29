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
    let base = Style {
        background: Some(Background::Color(style_conf::COLOR_BUTTON_MAIN)),
        text_color: Color::BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(style_conf::STYLE_EDITOR_BG_COLOR)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(style_conf::STYLE_DISABLED_BUTTON_COLOR)),
            ..base
        },
    }
}

pub fn button_entry(_theme: &Theme, status: button::Status) -> button::Style {
    //let palette = theme.extended_palette();
    let base = Style {
        background: Some(Background::Color(style_conf::STYLE_BUTTON_ENTRY_COLOR)),
        text_color: Color::BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(style_conf::STYLE_EDITOR_BG_COLOR)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(style_conf::STYLE_DISABLED_BUTTON_COLOR)),
            ..base
        },
    }
}

pub fn button_entry_selected(_theme: &Theme, status: button::Status) -> button::Style {
    //let palette = theme.extended_palette();
    let base = Style {
        background: Some(Background::Color(
            style_conf::STYLE_BUTTON_ENTRY_COLOR_SELECTED,
        )),
        text_color: Color::BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(style_conf::STYLE_EDITOR_BG_COLOR)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(style_conf::STYLE_DISABLED_BUTTON_COLOR)),
            ..base
        },
    }
}
