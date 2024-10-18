use iced::widget::button;
use iced::{theme, Background, Border, Color, Shadow, Vector};

use super::style_constants;

//#[derive(Clone, Copy, Default)]
/*
pub enum ButtonType {
    #[default]
    Standard,
}
    */

pub enum TirraButtonType {
    Entry,
    Main,
}

pub struct TirraButtonStyle {
    pub button_type: TirraButtonType,
    pub selected: bool,
}

impl button::StyleSheet for TirraButtonStyle {
    type Style = theme::Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let button_background: Background;
        match self.button_type {
            TirraButtonType::Main => {
                button_background = Background::Color(style_constants::COLOR_BUTTON_MAIN);
            }

            TirraButtonType::Entry => {
                if self.selected {
                    button_background =
                        Background::Color(style_constants::STYLE_BUTTON_ENTRY_COLOR_SELECTED);
                } else {
                    button_background =
                        Background::Color(style_constants::STYLE_BUTTON_ENTRY_COLOR);
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
        appear.background = Some(Background::Color(style_constants::STYLE_EDITOR_BG_COLOR));

        appear
    }
}
