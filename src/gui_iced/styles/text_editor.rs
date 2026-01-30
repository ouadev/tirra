use iced::widget::container;
use iced::widget::scrollable;
use iced::widget::text_editor::default;
use iced::widget::text_editor::Status;
use iced::widget::text_editor::Style;
use iced::{Background, Border, Theme};

use super::style_conf;



pub fn main_style(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let palette_tirra = style_conf::palette();
    let base = Style {
        background: Background::Color(palette_tirra.background_neutral),
        border: Border {
            radius: 0.0.into(),
            width: 0.,
            color: palette.background.strong.color,
        },
        value: palette_tirra.text,
        selection: palette_tirra.control_main,
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
        Status::Focused { is_hovered: _ } => Style {
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

pub fn scroller_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let palette = style_conf::palette();
    let mut return_style = scrollable::default(theme, status);

    //container background
    return_style.container =
        container::Style::default().background(Background::Color(palette.background_neutral));

    //color of the rail
    return_style.vertical_rail.background = Some(Background::Color(palette.background_secondary));

    match status {
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered: _,
            is_vertical_scrollbar_hovered,
            ..
        } => {
            if is_vertical_scrollbar_hovered {
                return_style.vertical_rail.scroller.background =
                    Background::Color(palette.control_main);
            }
        }
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged: _,
            is_vertical_scrollbar_dragged,
            ..
        } => {
            if is_vertical_scrollbar_dragged {
                return_style.vertical_rail.scroller.background =
                    Background::Color(palette.control_main);
            }
        }
        _ => {}
    }
    //scrollable::Status::Dragged { .. } |
    return_style
}
