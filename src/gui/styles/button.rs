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

/*
impl button::StyleSheet for TirraButtonStyle {
    type Style = theme::Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let button_background: Background;
        match self.button_type {
            TirraButtonType::Main => {
                button_background = Background::Color(style_conf::COLOR_BUTTON_MAIN);
            }

            TirraButtonType::Entry => {
                if self.selected {
                    button_background =
                        Background::Color(style_conf::STYLE_BUTTON_ENTRY_COLOR_SELECTED);
                } else {
                    button_background = Background::Color(style_conf::STYLE_BUTTON_ENTRY_COLOR);
                }
            }
        }
        button::Appearance {
            shadow_offset: Vector::default(),
            background: Some(button_background),
            text_color: Color::BLACK,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut appear = button::Appearance::default();
        appear.background = Some(Background::Color(style_conf::STYLE_EDITOR_BG_COLOR));

        appear
    }
}
*/
