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

pub fn button_action<'a, T: 'a>(content_text: Text<'a>) -> Button<'a, T> {
    Button::new(
        content_text
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .align_x(text::Alignment::Center)
            .align_y(alignment::Vertical::Center),
    )
    .style(styles::button::button_action)
}

pub fn button_list_entry<'a, T: 'a>(content_text: Text<'a>, selected: bool) -> Button<'a, T> {
    Button::new(content_text)
        .style(if selected {
            styles::button::button_entry_selected
        } else {
            styles::button::button_entry
        })
        .clip(true)
}
