use super::palette::Palette;
use chrono::{Datelike, Local, Timelike};
use iced::font::{Family, Stretch, Style, Weight};
use iced::widget::{text, Text};
use iced::{alignment, Font};
use std::sync::atomic::{AtomicBool, Ordering};

static DARK_MODE_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn icon<'a>(codepoint: char) -> Text<'a> {
    const ICON_FONT: Font = Font::with_name("tirra-icons");

    text(codepoint)
        .font(ICON_FONT)
        .shaping(text::Shaping::Basic)
        .size(12.0)
        .align_x(text::Alignment::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

pub fn icon_add<'a>() -> Text<'a> {
    icon('\u{0e800}')
}

pub fn icon_sort_create<'a>() -> Text<'a> {
    icon('\u{0e801}')
}

pub fn icon_sort_modify<'a>() -> Text<'a> {
    icon('\u{0e802}')
}

pub fn icon_delete_entry<'a>() -> Text<'a> {
    icon('\u{0e803}')
}

pub fn icon_unlock<'a>() -> Text<'a> {
    icon('\u{0f13e}')
}

pub fn icon_left<'a>() -> Text<'a> {
    icon('\u{f104}')
}

pub fn icon_right<'a>() -> Text<'a> {
    icon('\u{f105}')
}

pub fn icon_undo<'a>() -> Text<'a> {
    icon('\u{e804}')
}

pub fn dark_mode_in_paris() -> () {
    let paris = [
        (8, 17),
        (8, 18),
        (7, 19),
        (7, 21),
        (6, 22),
        (5, 22),
        (6, 22),
        (6, 22),
        (7, 20),
        (8, 19),
        (7, 17),
        (8, 17),
    ];
    // Calculate theme
    let timenow = Local::now();
    let month = timenow.month0() as usize;
    let light = paris.get(month).unwrap_or_else(|| &(5, 22));
    let now_hour = timenow.hour();

    let dark_mode = if now_hour > light.0 && now_hour < light.1 {
        false
    } else {
        true
    };

    DARK_MODE_ENABLED.store(dark_mode, Ordering::Relaxed);
}
/**
* set light/dark mode for the whole application.
*/
pub fn toggle_theme() {
    let current = DARK_MODE_ENABLED.load(Ordering::Relaxed);
    DARK_MODE_ENABLED.store(!current, Ordering::Relaxed);
}
/**
* Return the current Palette of colors to paint the UI.
*/
pub fn palette() -> Palette {
    if DARK_MODE_ENABLED.load(Ordering::Relaxed) {
        Palette::DARK_TIRRA_GREEN
    } else {
        Palette::LIGHT_TIRRA
    }
}
// Font families for different use cases.
pub const FONT_FAMILY_DEFAULT: Family = Family::SansSerif;
#[cfg(target_os = "macos")]
pub const FONT_FAMILY_EDIITING: Family = Family::Name("Georgia");
#[cfg(target_os = "linux")]
pub const FONT_FAMILY_EDIITING: Family = Family::Name("DejaVu Serif");
#[cfg(target_os = "windows")]
pub const FONT_FAMILY_EDIITING: Family = Family::Name("Arial");
pub const FONT_FAMILY_STATUS: Family = FONT_FAMILY_DEFAULT;
pub const FONT_FAMILY_COMMAND: Family = Family::Monospace;

// Acutal used fonts
pub const FONT_DEFAULT: Font = Font {
    family: FONT_FAMILY_DEFAULT,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
pub const FONT_EDITOR: Font = Font {
    family: FONT_FAMILY_EDIITING,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_COMMAND_LINE: Font = Font {
    family: FONT_FAMILY_COMMAND,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_STATUS_DATE: Font = Font {
    family: FONT_FAMILY_STATUS,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_STATUS_DATE_BOLD: Font = Font {
    family: FONT_FAMILY_STATUS,
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_LOGIN_LOGO: Font = Font {
    family: FONT_FAMILY_DEFAULT,
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

// Text Sizes
pub const TEXT_EDITOR_FONT_SIZE: iced::Pixels = iced::Pixels(19.);
pub const STYLE_TEXT_SIZE_DEFAULT: iced::Pixels = iced::Pixels(21.0);
pub const STYLE_TEXT_SIZE_NORMAL: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_COMMAND: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_EDITOR_STATUS_HIGHLIGHT: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_EDITOR_STATUS: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_LOGO: iced::Pixels = iced::Pixels(80.);
