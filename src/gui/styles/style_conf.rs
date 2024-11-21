use super::palette::Palette;
use iced::font::{Family, Stretch, Style, Weight};
use iced::Font;

/**
* Return the current Palette of colors to paint the UI.
*/
pub fn palette() -> Palette {
    Palette::DARK
}
// Font families for different use cases.
pub const FONT_FAMILY_DEFAULT: Family = Family::SansSerif;
#[cfg(target_os = "macos")]
pub const FONT_FAMILY_EDIITING: Family = Family::Name("Georgia");
#[cfg(target_os = "linux")]
pub const FONT_FAMILY_EDIITING: Family = Family::Name("DejaVu Serif");
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

// Text Sizes
pub const STYLE_TEXT_SIZE_EDITOR: iced::Pixels = iced::Pixels(23.0);
pub const STYLE_TEXT_SIZE_NORMAL: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_COMMAND: iced::Pixels = iced::Pixels(20.);
pub const STYLE_TEXT_SIZE_EDITOR_STATUS_HIGHLIGHT: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_EDITOR_STATUS: iced::Pixels = iced::Pixels(15.);
