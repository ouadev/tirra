use super::palette::Palette;
use chrono::{DateTime, Datelike, Timelike, Utc};
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
    let paris_sunrises = [
        461, 461, 461, 461, 460, 460, 460, 460, 459, 459, 458, 458, 457, 457, 456, 455, 455, 454,
        453, 452, 451, 450, 449, 448, 447, 446, 445, 444, 442, 441, 440, 439, 437, 436, 434, 433,
        432, 430, 429, 427, 425, 424, 422, 420, 419, 417, 415, 414, 412, 410, 408, 406, 405, 403,
        401, 399, 397, 395, 393, 391, 389, 387, 385, 383, 381, 379, 377, 375, 373, 371, 369, 367,
        365, 363, 360, 358, 356, 354, 352, 350, 348, 346, 343, 341, 339, 337, 335, 333, 331, 329,
        327, 324, 322, 320, 318, 316, 314, 312, 310, 308, 306, 304, 302, 300, 298, 296, 294, 292,
        290, 288, 286, 284, 282, 280, 279, 277, 275, 273, 271, 270, 268, 266, 265, 263, 261, 260,
        258, 257, 255, 254, 252, 251, 249, 248, 246, 245, 244, 243, 241, 240, 239, 238, 237, 236,
        235, 234, 233, 232, 231, 230, 230, 229, 228, 227, 227, 226, 226, 225, 225, 224, 224, 224,
        224, 223, 223, 223, 223, 223, 223, 223, 223, 223, 224, 224, 224, 224, 225, 225, 226, 226,
        227, 227, 228, 229, 229, 230, 231, 232, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241,
        242, 243, 245, 246, 247, 248, 250, 251, 252, 253, 255, 256, 257, 259, 260, 261, 263, 264,
        265, 267, 268, 270, 271, 273, 274, 275, 277, 278, 280, 281, 282, 284, 285, 287, 288, 290,
        291, 293, 294, 295, 297, 298, 300, 301, 303, 304, 305, 307, 308, 310, 311, 312, 314, 315,
        317, 318, 319, 321, 322, 324, 325, 327, 328, 329, 331, 332, 334, 335, 336, 338, 339, 341,
        342, 344, 345, 346, 348, 349, 351, 352, 354, 355, 357, 358, 360, 361, 363, 364, 366, 367,
        369, 370, 372, 373, 375, 376, 378, 379, 381, 383, 384, 386, 387, 389, 391, 392, 394, 395,
        397, 398, 400, 402, 403, 405, 406, 408, 410, 411, 413, 414, 416, 417, 419, 421, 422, 424,
        425, 427, 428, 429, 431, 432, 434, 435, 436, 438, 439, 440, 442, 443, 444, 445, 446, 447,
        448, 449, 450, 451, 452, 453, 454, 455, 455, 456, 457, 457, 458, 458, 459, 459, 460, 460,
        460, 460, 461, 461, 461, 461,
    ];

    let paris_sunsets = [
        966, 967, 968, 969, 971, 972, 973, 974, 975, 976, 978, 979, 980, 982, 983, 984, 986, 987,
        989, 990, 992, 993, 995, 996, 998, 1000, 1001, 1003, 1004, 1006, 1008, 1009, 1011, 1013,
        1014, 1016, 1018, 1019, 1021, 1022, 1024, 1026, 1027, 1029, 1031, 1032, 1034, 1036, 1037,
        1039, 1041, 1042, 1044, 1045, 1047, 1049, 1050, 1052, 1053, 1055, 1057, 1058, 1060, 1061,
        1063, 1064, 1066, 1068, 1069, 1071, 1072, 1074, 1075, 1077, 1078, 1080, 1081, 1083, 1084,
        1086, 1087, 1089, 1090, 1092, 1093, 1095, 1096, 1098, 1099, 1101, 1102, 1104, 1105, 1107,
        1108, 1110, 1111, 1113, 1114, 1116, 1117, 1119, 1120, 1122, 1123, 1125, 1126, 1128, 1129,
        1131, 1132, 1134, 1135, 1137, 1138, 1140, 1141, 1143, 1144, 1146, 1147, 1149, 1150, 1152,
        1153, 1155, 1156, 1157, 1159, 1160, 1162, 1163, 1165, 1166, 1167, 1169, 1170, 1171, 1173,
        1174, 1175, 1177, 1178, 1179, 1180, 1181, 1183, 1184, 1185, 1186, 1187, 1188, 1189, 1190,
        1191, 1192, 1193, 1193, 1194, 1195, 1196, 1196, 1197, 1198, 1198, 1199, 1199, 1200, 1200,
        1200, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1201, 1200,
        1200, 1200, 1199, 1199, 1198, 1198, 1197, 1196, 1196, 1195, 1194, 1193, 1192, 1192, 1191,
        1190, 1189, 1187, 1186, 1185, 1184, 1183, 1182, 1180, 1179, 1178, 1176, 1175, 1174, 1172,
        1171, 1169, 1168, 1166, 1164, 1163, 1161, 1159, 1158, 1156, 1154, 1153, 1151, 1149, 1147,
        1145, 1143, 1142, 1140, 1138, 1136, 1134, 1132, 1130, 1128, 1126, 1124, 1122, 1120, 1118,
        1116, 1114, 1112, 1110, 1107, 1105, 1103, 1101, 1099, 1097, 1095, 1093, 1090, 1088, 1086,
        1084, 1082, 1080, 1078, 1075, 1073, 1071, 1069, 1067, 1065, 1063, 1060, 1058, 1056, 1054,
        1052, 1050, 1048, 1046, 1044, 1041, 1039, 1037, 1035, 1033, 1031, 1029, 1027, 1025, 1023,
        1021, 1019, 1017, 1016, 1014, 1012, 1010, 1008, 1006, 1005, 1003, 1001, 999, 998, 996, 994,
        993, 991, 990, 988, 986, 985, 984, 982, 981, 979, 978, 977, 976, 974, 973, 972, 971, 970,
        969, 968, 967, 966, 965, 964, 963, 963, 962, 961, 961, 960, 959, 959, 959, 958, 958, 958,
        957, 957, 957, 957, 957, 957, 957, 957, 957, 957, 958, 958, 958, 959, 959, 960, 960, 961,
        961, 962, 963, 964, 964, 965, 966,
    ];

    // Calculate theme
    let timenow: DateTime<Utc> = Utc::now();
    let ordinal = timenow.ordinal();
    let minutes_since_midnight = timenow.hour() * 60 + timenow.minute();

    let (today_sr, today_ss) = (
        paris_sunrises[ordinal as usize],
        paris_sunsets[ordinal as usize],
    );

    let dark_mode = if minutes_since_midnight > today_sr && minutes_since_midnight < today_ss {
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
