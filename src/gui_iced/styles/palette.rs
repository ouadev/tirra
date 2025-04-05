use iced::color;
use iced::Color;

pub struct Palette {
    pub background_main: Color,
    pub background_secondary: Color,
    pub control_main: Color,
    pub control_disabled: Color,
    pub text: Color,
    // ...
}

impl Palette {
    pub const LIGHT_TIRRA: Self = Self {
        background_main: color!(0xffffff),
        text: color!(0x000000),

        background_secondary: color!(0xc1dfc2),
        control_main: color!(0x72b775),
        control_disabled: color!(0x0e6174),
    };

    pub const DARK_TIRRA: Self = Self {
        background_main: color!(0x2C3930),
        text: color!(0xcccccc),

        background_secondary: color!(0x3F4F44),
        control_main: color!(0xA27B5C),
        control_disabled: color!(0x0e6174),
    };

    pub const LIGHT: Self = Self {
        background_main: color!(0xffffff),
        background_secondary: color!(0xd9d9d9),
        control_main: color!(0xb6deca),
        control_disabled: color!(0x000000),
        text: color!(0x000000),
    };

    pub const LIGHT_CATPPUCCIN: Self = Self {
        background_main: color!(0xeff1f5),
        background_secondary: color!(0xdce0e8),
        control_main: color!(0x04a5e5),
        control_disabled: color!(0x000000),
        text: color!(0x4c4f69),
    };

    pub const DARK: Self = Self {
        background_main: color!(0x282A36),
        background_secondary: color!(0x44475a),
        control_main: color!(0x6272a4),
        control_disabled: color!(0x000000),
        text: color!(0xFFFFFF),
    };
}
