use iced::widget::text_editor;
use iced::{theme, Background, Border, Color};

use super::style_constants;

/// The style of a text input.
/*
#[derive(Default)]
pub enum TextEditor {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn text_editor::StyleSheet<Style = Theme>>),
}
    */
pub struct EditorStyle {}

impl text_editor::StyleSheet for EditorStyle {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> text_editor::Appearance {
        let palette = style.extended_palette();

        text_editor::Appearance {
            background: Background::Color(style_constants::STYLE_EDITOR_BG_COLOR),
            border: Border {
                radius: 0.0.into(),
                width: 0.,
                color: palette.background.strong.color,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> text_editor::Appearance {
        let palette = style.extended_palette();

        text_editor::Appearance {
            background: Background::Color(style_constants::STYLE_EDITOR_BG_COLOR),
            border: Border {
                radius: 0.0.into(),
                width: 0.,
                color: palette.background.base.text,
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_editor::Appearance {
        let palette = style.extended_palette();

        text_editor::Appearance {
            background: Background::Color(style_constants::STYLE_EDITOR_BG_COLOR),
            border: Border {
                radius: 0.0.into(),
                width: 0.,
                color: palette.primary.strong.color,
            },
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();

        palette.background.strong.color
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();

        palette.background.base.text
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();

        palette.primary.weak.color
    }

    fn disabled(&self, style: &Self::Style) -> text_editor::Appearance {
        let palette = style.extended_palette();

        text_editor::Appearance {
            background: Background::Color(style_constants::STYLE_EDITOR_BG_COLOR),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: palette.background.strong.color,
            },
        }
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.placeholder_color(style)
    }
}
