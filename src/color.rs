//! Цвета и палитры.
//!
//! Turbo Vision использует атрибуты в стиле BIOS: один байт, где младшие
//! 4 бита — цвет текста, старшие 4 бита — цвет фона.
//! Цвета BIOS: 0 чёрный, 1 синий, 2 зелёный, 3 бирюзовый, 4 красный,
//! 5 малиновый, 6 коричневый, 7 светло-серый, 8–15 — их яркие варианты.
//!
//! Каждое представление имеет **палитру** — массив индексов, который
//! переводит «логический» номер цвета представления в номер цвета владельца.
//! Цепочка палитр заканчивается палитрой приложения, которая уже содержит
//! реальные атрибуты. Подробнее — в [`crate::DrawCtx::map_color`].

/// Атрибут ячейки: байт BIOS (фон << 4 | текст). Аналог `TColorAttr`
/// в упрощённом (16-цветном) варианте.
pub type Attr = u8;

/// Пара атрибутов: младший байт — обычный цвет, старший — цвет
/// подсвеченного текста между `~...~`. Аналог `TAttrPair`.
pub type AttrPair = u16;

/// Атрибут ошибки палитры (мигающий белый на красном в оригинале).
pub const ERROR_ATTR: Attr = 0xCF;

/// Атрибут тени под окнами (тёмно-серый на чёрном), как `shadowAttr`.
pub const SHADOW_ATTR: Attr = 0x08;

/// Создаёт атрибут из цвета текста и фона.
pub const fn attr(fg: u8, bg: u8) -> Attr {
    (bg << 4) | (fg & 0x0F)
}

/// Цвет текста из атрибута.
pub const fn fore(a: Attr) -> u8 {
    a & 0x0F
}

/// Цвет фона из атрибута.
pub const fn back(a: Attr) -> u8 {
    a >> 4
}

/// Перевод номера цвета BIOS в номер цвета ANSI (0–15).
/// В BIOS синий = 1, красный = 4, а в ANSI наоборот.
pub const fn bios_to_ansi(c: u8) -> u8 {
    const MAP: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];
    MAP[(c & 7) as usize] | (c & 8)
}

/// Палитры. Байтовые строки перенесены из C++ без изменений
/// (`#define cpXXX "..."`), только без завершающего нуля.
pub mod palettes {
    /// Палитра приложения в цвете (`cpAppColor`, 135 элементов).
    pub const APP_COLOR: &[u8] = b"\x71\x70\x78\x74\x20\x28\x24\x17\x1F\x1A\x31\x31\x1E\x71\x1F\
\x37\x3F\x3A\x13\x13\x3E\x21\x3F\x70\x7F\x7A\x13\x13\x70\x7F\x7E\
\x70\x7F\x7A\x13\x13\x70\x70\x7F\x7E\x20\x2B\x2F\x78\x2E\x70\x30\
\x3F\x3E\x1F\x2F\x1A\x20\x72\x31\x31\x30\x2F\x3E\x31\x13\x38\x00\
\x17\x1F\x1A\x71\x71\x1E\x17\x1F\x1E\x20\x2B\x2F\x78\x2E\x10\x30\
\x3F\x3E\x70\x2F\x7A\x20\x12\x31\x31\x30\x2F\x3E\x31\x13\x38\x00\
\x37\x3F\x3A\x13\x13\x3E\x30\x3F\x3E\x20\x2B\x2F\x78\x2E\x30\x70\
\x7F\x7E\x1F\x2F\x1A\x20\x32\x31\x71\x70\x2F\x7E\x71\x13\x78\x00\
\x37\x3F\x3A\x13\x13\x30\x3E\x1E";

    /// Чёрно-белая палитра приложения (`cpAppBlackWhite`).
    pub const APP_BLACK_WHITE: &[u8] = b"\x70\x70\x78\x7F\x07\x07\x0F\x07\x0F\x07\x70\x70\x07\x70\x0F\
\x07\x0F\x07\x70\x70\x07\x70\x0F\x70\x7F\x7F\x70\x07\x70\x07\x0F\
\x70\x7F\x7F\x70\x07\x70\x70\x7F\x7F\x07\x0F\x0F\x78\x0F\x78\x07\
\x0F\x0F\x0F\x70\x0F\x07\x70\x70\x70\x07\x70\x0F\x07\x07\x08\x00\
\x07\x0F\x0F\x07\x70\x07\x07\x0F\x0F\x70\x78\x7F\x08\x7F\x08\x70\
\x7F\x7F\x7F\x0F\x70\x70\x07\x70\x70\x70\x07\x7F\x70\x07\x78\x00\
\x70\x7F\x7F\x70\x07\x70\x70\x7F\x7F\x07\x0F\x0F\x78\x0F\x78\x07\
\x0F\x0F\x0F\x70\x0F\x07\x70\x70\x70\x07\x70\x0F\x07\x07\x08\x00\
\x07\x0F\x07\x70\x70\x07\x0F\x70";

    /// Монохромная палитра приложения (`cpAppMonochrome`).
    pub const APP_MONOCHROME: &[u8] = b"\x70\x07\x07\x0F\x70\x70\x70\x07\x0F\x07\x70\x70\x07\x70\x00\
\x07\x0F\x07\x70\x70\x07\x70\x00\x70\x70\x70\x07\x07\x70\x07\x00\
\x70\x70\x70\x07\x07\x70\x70\x70\x0F\x07\x07\x0F\x70\x0F\x70\x07\
\x0F\x0F\x07\x70\x07\x07\x70\x07\x07\x07\x70\x0F\x07\x07\x70\x00\
\x70\x70\x70\x07\x07\x70\x70\x70\x0F\x07\x07\x0F\x70\x0F\x70\x07\
\x0F\x0F\x07\x70\x07\x07\x70\x07\x07\x07\x70\x0F\x07\x07\x01\x00\
\x70\x70\x70\x07\x07\x70\x70\x70\x0F\x07\x07\x0F\x70\x0F\x70\x07\
\x0F\x0F\x07\x70\x07\x07\x70\x07\x07\x07\x70\x0F\x07\x07\x01\x00\
\x07\x0F\x07\x70\x70\x07\x0F\x70";

    // --- Палитры окон (views.h) ---
    pub const BLUE_WINDOW: &[u8] = b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    pub const CYAN_WINDOW: &[u8] = b"\x10\x11\x12\x13\x14\x15\x16\x17";
    pub const GRAY_WINDOW: &[u8] = b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F";

    // --- Палитры диалогов (dialogs.h) ---
    pub const GRAY_DIALOG: &[u8] = b"\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2A\x2B\x2C\x2D\x2E\x2F\
\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3A\x3B\x3C\x3D\x3E\x3F";
    pub const BLUE_DIALOG: &[u8] = b"\x40\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\
\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f";
    pub const CYAN_DIALOG: &[u8] = b"\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\
\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f";

    // --- Палитры отдельных элементов (из соответствующих .cpp) ---
    pub const BACKGROUND: &[u8] = b"\x01";
    pub const FRAME: &[u8] = b"\x01\x01\x02\x02\x03";
    pub const MENU_VIEW: &[u8] = b"\x02\x03\x04\x05\x06\x07";
    pub const STATUS_LINE: &[u8] = b"\x02\x03\x04\x05\x06\x07";
    pub const BUTTON: &[u8] = b"\x0A\x0B\x0C\x0D\x0E\x0E\x0E\x0F";
    pub const CLUSTER: &[u8] = b"\x10\x11\x12\x12\x1f";
    pub const INPUT_LINE: &[u8] = b"\x13\x13\x14\x15";
    pub const LABEL: &[u8] = b"\x07\x08\x09\x09";
    pub const STATIC_TEXT: &[u8] = b"\x06";
    pub const LIST_VIEWER: &[u8] = b"\x1A\x1A\x1B\x1C\x1D";
    pub const SCROLL_BAR: &[u8] = b"\x04\x05\x05";
    pub const SCROLLER: &[u8] = b"\x06\x07";
    pub const HISTORY: &[u8] = b"\x16\x17";
    /// `cpHistoryWindow`, уже пересчитанная через палитру серого диалога:
    /// в C++ окно истории вставлялось в диалог, а здесь — на рабочий стол.
    pub const HISTORY_WINDOW: &[u8] = b"\x32\x32\x34\x37\x36\x32\x33";
    pub const HISTORY_VIEWER: &[u8] = b"\x06\x06\x07\x06\x06";
    pub const INDICATOR: &[u8] = b"\x02\x03";
    pub const EDITOR: &[u8] = b"\x06\x07";
    pub const MEMO: &[u8] = b"\x1A\x1B";
}

#[cfg(test)]
mod tests {
    use super::palettes::*;

    #[test]
    fn app_palette_sizes() {
        // В оригинале палитра приложения содержит ровно 135 цветов.
        assert_eq!(APP_COLOR.len(), 135);
        assert_eq!(APP_BLACK_WHITE.len(), 135);
        assert_eq!(APP_MONOCHROME.len(), 135);
        assert_eq!(GRAY_DIALOG.len(), 32);
    }
}
