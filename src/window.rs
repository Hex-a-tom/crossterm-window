
use crate::buffer::{Buffer, BufferDrawIterator};
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Window {
    area: Rect,
    buffer: Buffer,
}


impl Window {
    pub fn new(area: Rect) -> Self {
        Window {
            area,
            buffer: Buffer::empty(area.width, area.height),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.area.width = width;
        self.area.height = height;
        self.buffer.resize(width, height);
    }

    pub fn resize_by(&mut self, width: i16, height: i16) {
        self.resize(
            self.area.width.saturating_add_signed(width),
            self.area.height.saturating_add_signed(height),
        )
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.area.x = x;
        self.area.y = y;
    }

    pub fn move_by(&mut self, x: i16, y: i16) {
        self.area.x = self.area.x.saturating_add_signed(x);
        self.area.y = self.area.y.saturating_add_signed(y);
    }

    pub fn pos(&self) -> (u16, u16) {
        (self.area.x, self.area.y)
    }

    pub fn width(&self) -> u16 {
        self.area.width
    }

    pub fn height(&self) -> u16 {
        self.area.height
    }

    pub fn draw(&self, buffer: &mut Buffer) {
        buffer.insert(self.area.x, self.area.y, &self.buffer);
    }

    pub fn content_iter(&self) -> BufferDrawIterator {
        self.buffer.draw()
    }

    pub fn set_lines<S>(&mut self, x: u16, y: u16, string: S, style: Style)
    where
        S: AsRef<str>,
    {
        self.buffer.set_lines(x, y, string, style)
    }

    pub fn set_string<S>(&mut self, x: u16, y: u16, string: S, style: Style)
    where
        S: AsRef<str>,
    {
        self.buffer.set_string(x, y, string, style)
    }

    pub fn set_stringn<S>(
        &mut self,
        x: u16,
        y: u16,
        string: S,
        width: usize,
        style: Style,
    ) -> (u16, u16)
    where
        S: AsRef<str>,
    {
        self.buffer.set_stringn(x, y, string, width, style)
    }

    pub fn draw_border(&mut self, title: &str) {
        let buf = &mut self.buffer;
        let area = &self.area;

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
            buf.set_stringn(
                area.x + area.width - 1,
                area.y + i,
                "│",
                1,
                Style::default(),
                );
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

    pub fn set_style(&mut self, area: Rect, style: Style) {
        self.buffer.set_style(area, style)
    }
}
