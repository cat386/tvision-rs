//! Просмотр базового Markdown без выполнения HTML и открытия ссылок.
use crate::{color, event::Key, text, view::default_handle_event, *};

#[derive(Clone, Debug)]
struct Row(Vec<(char, u8)>);

/// Просмотр Markdown с переносом строк и вертикальной прокруткой.
///
/// Поддерживает ATX-заголовки, списки, цитаты, fenced code, разделители,
/// **жирный**, *курсив*, `код` и ссылки `[текст](адрес)`.
/// HTML, таблицы и изображения не интерпретируются; ссылки не открываются.
/// Это ограниченное подмножество Markdown, а не полный CommonMark.
pub struct MarkdownViewer {
    pub vs: ViewState,
    pub scroller: Scroller,
    source: String,
    rows: Vec<Row>,
}

impl MarkdownViewer {
    pub fn new(bounds: Rect, markdown: &str, v_scroll: Option<ViewId>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE;
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        let mut viewer = Self { vs, scroller: Scroller::new(None, v_scroll), source: String::new(), rows: Vec::new() };
        viewer.set_markdown(markdown);
        viewer
    }

    /// Заменяет документ и возвращает прокрутку к началу.
    pub fn set_markdown(&mut self, markdown: &str) {
        // Управляющие последовательности файла не должны попадать в терминал.
        self.source = markdown.chars().filter(|c| !c.is_control() || matches!(c, '\n' | '\t')).collect();
        self.scroller.delta = Point::default();
        self.reflow();
    }

    pub fn markdown(&self) -> &str {
        &self.source
    }

    fn reflow(&mut self) {
        self.rows = render(&self.source, self.vs.size.x.max(1) as usize);
        self.scroller.set_limit(self.vs.size.x, self.rows.len() as i32, self.vs.size);
    }
}

impl View for MarkdownViewer {
    crate::impl_view_basics!();

    fn size_policy(&self) -> crate::layout::SizePolicy {
        crate::layout::SizePolicy::EXPANDING
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::SCROLLER
    }

    fn draw(&self, dc: &mut DrawCtx) {
        let normal = dc.color(1);
        dc.fill(self.vs.extent(), ' ', normal);
        for y in 0..self.vs.size.y {
            if let Some(row) = self.rows.get((self.scroller.delta.y + y) as usize) {
                let mut x = 0;
                for &(ch, style) in &row.0 {
                    // Сохраняем фон темы, выделяя только цвет текста.
                    let attr = if style == 0 { normal } else { color::with_fore(normal, style) };
                    dc.write_str(x, y, &ch.to_string(), attr);
                    x += text::char_width(ch) as i32;
                }
            }
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        // В Scroller Home/End относятся к горизонтали; здесь — к документу.
        if let Event::KeyDown(k) = ev {
            if self.vs.is_focused() && k.mods == Mods::NONE && matches!(k.key, Key::Home | Key::End) {
                let y = if k.key == Key::Home { 0 } else { self.rows.len() as i32 };
                self.scroller.scroll_to(0, y, self.vs.size);
                ev.clear();
            }
        }
        self.scroller.handle(ev, self.vs.size, self.vs.is_focused());
        self.scroller.sync(self.vs.size, ctx);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.reflow();
    }
}

fn plain(s: &str, style: u8) -> Vec<(char, u8)> {
    s.replace('\t', "    ").chars().map(|c| (c, style)).collect()
}

fn inline(mut s: &str, base: u8) -> Vec<(char, u8)> {
    let mut out = Vec::new();
    while !s.is_empty() {
        if let Some(rest) = s.strip_prefix('\\') {
            if let Some(c) = rest.chars().next() {
                out.push((c, base));
                s = &rest[c.len_utf8()..];
                continue;
            }
        }
        let mut matched = false;
        for (mark, style) in [("**", 15), ("__", 15), ("`", 14), ("*", 11), ("_", 11)] {
            if let Some(rest) = s.strip_prefix(mark) {
                if let Some(end) = rest.find(mark).filter(|&i| i > 0) {
                    out.extend(plain(&rest[..end], style));
                    s = &rest[end + mark.len()..];
                    matched = true;
                    break;
                }
            }
        }
        if matched {
            continue;
        }
        if let Some(rest) = s.strip_prefix('[') {
            if let Some(mid) = rest.find("](") {
                if let Some(end) = rest[mid + 2..].find(')') {
                    out.extend(plain(&format!("{} ({})", &rest[..mid], &rest[mid + 2..mid + 2 + end]), 11));
                    s = &rest[mid + 3 + end..];
                    continue;
                }
            }
        }
        let ch = s.chars().next().unwrap();
        if ch == '\t' {
            out.extend(plain("    ", base));
        } else {
            out.push((ch, base));
        }
        s = &s[ch.len_utf8()..];
    }
    out
}

fn wrap(chars: Vec<(char, u8)>, width: usize, rows: &mut Vec<Row>) {
    if chars.is_empty() {
        rows.push(Row(chars));
        return;
    }
    let mut remaining = chars.as_slice();
    while !remaining.is_empty() {
        let mut used = 0;
        let mut end = 0;
        let mut space = None;
        for (i, &(ch, _)) in remaining.iter().enumerate() {
            let w = text::char_width(ch);
            if used + w > width {
                break;
            }
            used += w;
            end = i + 1;
            if ch == ' ' && i > 0 {
                space = Some(i);
            }
        }
        if end == remaining.len() {
            rows.push(Row(remaining.to_vec()));
            break;
        }
        // Широкий символ в области шириной 1 не рисуем за границей виджета.
        if end == 0 {
            remaining = &remaining[1..];
            rows.push(Row(vec![('?', 0)]));
            continue;
        }
        let cut = space.unwrap_or(end);
        rows.push(Row(remaining[..cut].to_vec()));
        remaining = &remaining[cut + usize::from(space.is_some())..];
    }
}

fn render(source: &str, width: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        let marker = trimmed.chars().next().unwrap_or(' ');
        let count = trimmed.chars().take_while(|&c| c == marker).count();
        if let Some((ch, min)) = fence {
            if marker == ch && count >= min && trimmed[count..].trim().is_empty() {
                fence = None;
            } else {
                wrap(plain(line, 14), width, &mut rows);
            }
            continue;
        }
        if matches!(marker, '`' | '~') && count >= 3 {
            fence = Some((marker, count));
            wrap(plain(&format!("── {}", trimmed[count..].trim()), 14), width, &mut rows);
            continue;
        }
        let hashes = trimmed.bytes().take_while(|&c| c == b'#').count();
        let (content, style) = if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            (trimmed[hashes..].trim(), 15)
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            wrap(inline(&format!("│ {quote}"), 3), width, &mut rows);
            continue;
        } else if ["---", "***", "___"].contains(&trimmed.trim()) {
            rows.push(Row(plain(&"─".repeat(width), 3)));
            continue;
        } else {
            (line, 0)
        };
        let content = if content.starts_with("- ") || content.starts_with("* ") || content.starts_with("+ ") {
            format!("• {}", &content[2..])
        } else {
            content.to_owned()
        };
        wrap(inline(&content, style), width, &mut rows);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lines(v: &MarkdownViewer) -> Vec<String> {
        v.rows.iter().map(|r| r.0.iter().map(|p| p.0).collect()).collect()
    }

    #[test]
    fn blocks_inline_and_unicode_resize() {
        let mut v = MarkdownViewer::new(
            Rect::new(0, 0, 50, 10),
            "# Привет\n- **мир**\n> цитата\n[сайт](https://example.com)\n```rs\n**literal**\n```\n未完成",
            None,
        );
        let l = lines(&v);
        assert_eq!(l[0], "Привет");
        assert_eq!(l[1], "• мир");
        assert_eq!(l[2], "│ цитата");
        assert_eq!(l[3], "сайт (https://example.com)");
        assert!(l.contains(&"**literal**".into()));
        assert!(v.rows[1].0.iter().any(|p| p.1 == 15));
        v.change_bounds(Rect::new(0, 0, 8, 3));
        assert!(lines(&v).iter().all(|s| text::str_width(s) <= 8));
        v.set_markdown("новый\u{1b}[31m\r\nтекст");
        assert!(!v.markdown().contains('\u{1b}'));
        assert_eq!(v.scroller.delta, Point::default());
    }

    #[test]
    fn scroll_and_draw_headless() {
        let mut backend = crate::platform::HeadlessBackend::new(40, 12);
        backend.push_key(KeyEvent::new(Key::End, Mods::NONE));
        let mut app = Program::new(Box::new(backend), None, None);
        let mut window = Window::new(Rect::new(0, 0, 40, 11), "Markdown", 1);
        window.add(MarkdownViewer::new(Rect::new(1, 1, 38, 9), &(0..30).map(|i| format!("Строка {i}\n")).collect::<String>(), None));
        app.insert_window(window);
        app.pump(&mut crate::program::NoHandler);
        assert!(app.screen_text().contains("Строка 29"));
    }
}
