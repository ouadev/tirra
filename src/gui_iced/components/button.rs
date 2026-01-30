use crate::gui_iced::styles::{self, style_conf};
use iced::{
    alignment,
    widget::{text, Button, Text},
    Padding,
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
    let content_text = if selected {
        content_text.font(style_conf::FONT_STATUS_DATE_BOLD)
    } else {
        content_text.font(style_conf::FONT_STATUS_DATE)
    };
    Button::new(content_text)
        .padding(Padding {
            top: 5.,
            bottom: 5.,
            left: 10.,
            right: 0.,
        })
        .style(if selected {
            styles::button::button_entry_selected
        } else {
            styles::button::button_entry
        })
        .clip(true)
}
