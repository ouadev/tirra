use crate::gui_iced::styles::{self, style_conf};
use iced::{
    alignment,
    widget::{text, Button, Text},
};

pub fn button_submit<'a, T: 'a>(content_text: Text<'a>) -> Button<'a, T> {
    Button::new(
        content_text
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .align_x(text::Alignment::Center)
            .align_y(alignment::Vertical::Center),
    )
    .style(styles::button::button_main)
}
