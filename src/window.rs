use crossterm::event::{KeyEvent, MouseEvent};

use crate::buffer::Buffer;
use crate::text::Style;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn area(&self) -> u16 {
        self.width * self.height
    }

    pub const fn left(self) -> u16 {
        self.x
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn top(self) -> u16 {
        self.y
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }
}

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
}

pub trait WindowContent {
    /// Full window redraw (On buffer resize mostly)
    fn redraw(&mut self, buf: &mut Buffer) -> ContentInfo;

    /// Update
    fn update(&mut self, buf: &mut Buffer) -> ContentInfo;

    /// Event
    fn event(&mut self, buf: &mut Buffer, event: Event) -> ContentInfo;
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ContentInfo {
    scroll: bool,
    scroll_pos: f32,
}

impl ContentInfo {
    pub fn pos(pos: f32) -> Self {
        ContentInfo {
            scroll: true,
            scroll_pos: pos,
        }
    }
}

pub struct WindowContentNone {}

impl WindowContent for WindowContentNone {
    fn redraw(&mut self, _buf: &mut Buffer) -> ContentInfo {
        ContentInfo::default()
    }

    fn update(&mut self, _buf: &mut Buffer) -> ContentInfo {
        ContentInfo::default()
    }

    fn event(&mut self, _buf: &mut Buffer, _event: Event) -> ContentInfo {
        ContentInfo::default()
    }
}

pub struct Window {
    title: String,
    area: Rect,
    buffer: Buffer,

    // Inner Window
    content: Box<dyn WindowContent>,
    content_info: ContentInfo,
}

fn draw_border(buf: &mut Buffer, area: Rect, title: &str, info: &ContentInfo) {
    // Top
    buf.set_stringn(area.x, area.y, "╭", 1, Style::default());
    let len = title.len().min(area.width as usize - 2);
    buf.set_stringn(area.x + 1, area.y, title, len, Style::default());
    for i in (len as u16 + 1)..(area.width - 1) {
        buf.set_stringn(area.x + i, area.y, "─", 1, Style::default());
    }
    buf.set_stringn(area.x + area.width - 1, area.y, "╮", 1, Style::default());

    // Middle
    for i in 1..area.height {
        buf.set_stringn(area.x, area.y + i, "│", 1, Style::default());
    }

    if info.scroll {
        buf.set_stringn(
            area.x + area.width - 1,
            area.y + 1,
            "∧",
            1,
            Style::default(),
        );
        buf.set_stringn(
            area.x + area.width - 1,
            area.y + area.height - 1,
            "∨",
            1,
            Style::default(),
        );
        for i in 2..area.height - 1 {
            buf.set_stringn(
                area.x + area.width - 1,
                area.y + i,
                "╎",
                1,
                Style::default(),
            );
        }
        let pos = ((area.height - 4) as f32 * info.scroll_pos) as u16;
        buf.set_stringn(
            area.x + area.width - 1,
            area.y + 2 + pos,
            "█",
            1,
            Style::default(),
        );
    } else {
        for i in 1..area.height {
            buf.set_stringn(
                area.x + area.width - 1,
                area.y + i,
                "│",
                1,
                Style::default(),
            );
        }
    }

    // Bottom
    buf.set_stringn(area.x, area.y + area.height, "╰", 1, Style::default());
    for i in 1..area.width - 1 {
        buf.set_stringn(area.x + i, area.y + area.height, "─", 1, Style::default());
    }
    buf.set_stringn(
        area.x + area.width - 1,
        area.y + area.height,
        "╯",
        1,
        Style::default(),
    );
}

impl Window {
    pub fn new(title: String, area: Rect, content: Box<dyn WindowContent>) -> Self {
        Window {
            title,
            area,
            buffer: Buffer::empty(area.width - 2, area.height - 2),
            content,
            content_info: ContentInfo::default(),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.area.width = width;
        self.area.height = height;
        self.buffer.resize(width, height);
        self.content_info = self.content.redraw(&mut self.buffer);
    }

    pub fn resize_by(&mut self, width: i16, height: i16) {
        self.resize(
            self.area.width.saturating_add_signed(width),
            self.area.height.saturating_add_signed(height),
        )
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.area.x = x;
        self.area.y = y;
    }

    pub fn move_by(&mut self, x: i16, y: i16) {
        self.area.x = self.area.x.saturating_add_signed(x);
        self.area.y = self.area.y.saturating_add_signed(y);
    }

    pub fn event(&mut self, event: Event) {
        self.content_info = self.content.event(&mut self.buffer, event);
    }

    pub fn update(&mut self) {
        self.content_info = self.content.update(&mut self.buffer);
    }

    pub fn draw(&self, buffer: &mut Buffer) {
        buffer.insert(self.area.x + 1, self.area.y + 1, &self.buffer);
        draw_border(buffer, self.area, &self.title, &self.content_info);
    }
}
