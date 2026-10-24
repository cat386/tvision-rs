//! Диалоговое окно (`TDialog`).
//!
//! Отличия от обычного окна: серая палитра, нельзя менять размер и
//! распахивать; Esc = «Отмена», Enter = нажать кнопку по умолчанию;
//! в модальном режиме команды OK/Cancel/Yes/No завершают диалог.

use crate::color::palettes;
use crate::event::{cm, Event, Key, Message};
use crate::geometry::{Point, Rect};
use crate::view::{of, sf, Ctx, DrawCtx, View, ViewId};
use crate::widgets::window::{wf, Window};

/// Цветовая схема диалога (`dpGrayDialog`, `dpBlueDialog`, `dpCyanDialog`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogPalette {
    Gray,
    Blue,
    Cyan,
}

pub struct Dialog {
    pub win: Window,
}

impl Dialog {
    /// Создаёт диалог (аналог `TDialog(bounds, title)`).
    pub fn new(bounds: Rect, title: &str) -> Self {
        let mut win = Window::new(bounds, title, 0);
        win.group.vs.grow_mode = 0;
        win.flags = wf::MOVE | wf::CLOSE;
        win.palette = palettes::GRAY_DIALOG;
        Dialog { win }
    }

    /// Диалог заданного размера, который будет отцентрирован при вставке.
    pub fn centered(width: i32, height: i32, title: &str) -> Self {
        let mut d = Dialog::new(Rect::new(0, 0, width, height), title);
        d.win.group.vs.options |= of::CENTERED;
        d
    }

    pub fn set_palette(&mut self, p: DialogPalette) {
        self.win.palette = match p {
            DialogPalette::Gray => palettes::GRAY_DIALOG,
            DialogPalette::Blue => palettes::BLUE_DIALOG,
            DialogPalette::Cyan => palettes::CYAN_DIALOG,
        };
    }

    pub fn id(&self) -> ViewId {
        self.win.id()
    }

    /// Вставить элемент. Порядок вставки = порядок перехода по Tab.
    pub fn insert(&mut self, v: Box<dyn View>) -> ViewId {
        self.win.insert(v)
    }

    pub fn add<T: View>(&mut self, v: T) -> ViewId {
        self.win.add(v)
    }

    /// Выбрать (дать фокус) элемент диалога.
    pub fn select(&mut self, id: ViewId) {
        if let Some(i) = self.win.group.index_of(id) {
            self.win.group.set_current_raw(Some(i));
        }
    }
}

impl View for Dialog {
    crate::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
    }

    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        match ev {
            Event::KeyDown(k) if k.key == Key::Esc && k.mods == Default::default() => {
                ctx.post_command(cm::CANCEL, Default::default());
                ev.clear();
            }
            Event::KeyDown(k) if k.key == Key::Enter && k.mods == Default::default() => {
                // Нажать кнопку по умолчанию.
                let mut b = Event::Broadcast(Message::new(cm::DEFAULT));
                self.win.group.broadcast_children(&mut b, ctx);
                ev.clear();
            }
            Event::Command(m) if matches!(m.command, cm::OK | cm::CANCEL | cm::YES | cm::NO) => {
                if self.win.group.vs.state & sf::MODAL != 0 {
                    ctx.end_modal(m.command);
                    ev.clear();
                }
            }
            _ => {}
        }
    }

    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
    }

    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        // «Отмена» допустима всегда (как в `TDialog::valid`).
        command == cm::CANCEL || self.win.group.valid(command, ctx)
    }

    fn size_limits(&self, owner_size: Point) -> (Point, Point) {
        (crate::widgets::window::MIN_WIN_SIZE, owner_size)
    }
}
