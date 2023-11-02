use crate::{text::{Modifier, Style}, window::Rect};
use crossterm::style::Color;
use std::cmp::min;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Cell {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
    pub skip: bool,
}

impl Cell {
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Cell {
        self.symbol.clear();
        self.symbol.push_str(symbol);
        self
    }

    pub fn set_char(&mut self, ch: char) -> &mut Cell {
        self.symbol.clear();
        self.symbol.push(ch);
        self
    }

    pub fn set_fg(&mut self, color: Color) -> &mut Cell {
        self.fg = color;
        self
    }

    pub fn set_bg(&mut self, color: Color) -> &mut Cell {
        self.bg = color;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Cell {
        if let Some(c) = style.fg {
            self.fg = c;
        }
        if let Some(c) = style.bg {
            self.bg = c;
        }
        self.modifier.insert(style.add_modifier);
        self.modifier.remove(style.sub_modifier);
        self
    }

    pub fn style(&self) -> Style {
        Style::default()
            .fg(self.fg)
            .bg(self.bg)
            .add_modifier(self.modifier)
    }

    /// Sets the cell to be skipped when copying (diffing) the buffer to the screen.
    ///
    /// This is helpful when it is necessary to prevent the buffer from overwriting a cell that is
    /// covered by an image from some terminal graphics protocol (Sixel / iTerm / Kitty ...).
    pub fn set_skip(&mut self, skip: bool) -> &mut Cell {
        self.skip = skip;
        self
    }

    pub fn reset(&mut self) {
        self.symbol.clear();
        self.symbol.push(' ');
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        self.modifier = Modifier::empty();
        self.skip = false;
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            symbol: " ".into(),
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::empty(),
            skip: false,
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    pub content: Vec<Cell>,
}

impl Buffer {
    pub fn empty(width: u16, height: u16) -> Buffer {
        let cell = Cell::default();
        Buffer::filled(width, height, &cell)
    }

    pub fn filled(width: u16, height: u16, cell: &Cell) -> Buffer {
        let size = (width * height) as usize;
        let mut content = Vec::with_capacity(size);
        for _ in 0..size {
            content.push(cell.clone())
        }
        Buffer {
            width,
            height,
            content,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.content
            .resize((width * height) as usize, Cell::default());
    }

    fn area(&self) -> u16 {
        self.width * self.height
    }

    /// Reset all cells in the buffer
    pub fn reset(&mut self) {
        for c in &mut self.content {
            c.reset();
        }
    }

    /// Returns the index in the `Vec<Cell>` for the given global (x, y) coordinates.
    ///
    /// Global coordinates are offset by the Buffer's area offset (`x`/`y`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::prelude::*;
    /// let rect = Rect::new(200, 100, 10, 10);
    /// let buffer = Buffer::empty(rect);
    /// // Global coordinates to the top corner of this buffer's area
    /// assert_eq!(buffer.index_of(200, 100), 0);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when given an coordinate that is outside of this Buffer's area.
    ///
    /// ```should_panic
    /// # use ratatui::prelude::*;
    /// let rect = Rect::new(200, 100, 10, 10);
    /// let buffer = Buffer::empty(rect);
    /// // Top coordinate is outside of the buffer in global coordinate space, as the Buffer's area
    /// // starts at (200, 100).
    /// buffer.index_of(0, 0); // Panics
    /// ```
    pub fn index_of(&self, x: u16, y: u16) -> usize {
        debug_assert!(
            x < self.width && y < self.height,
            "Trying to access position outside the buffer: x={x}, y={y}",
        );
        (y * self.width + x) as usize
    }

    /// Returns the (global) coordinates of a cell given its index
    ///
    /// Global coordinates are offset by the Buffer's area offset (`x`/`y`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::prelude::*;
    /// let rect = Rect::new(200, 100, 10, 10);
    /// let buffer = Buffer::empty(rect);
    /// assert_eq!(buffer.pos_of(0), (200, 100));
    /// assert_eq!(buffer.pos_of(14), (204, 101));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when given an index that is outside the Buffer's content.
    ///
    /// ```should_panic
    /// # use ratatui::prelude::*;
    /// let rect = Rect::new(0, 0, 10, 10); // 100 cells in total
    /// let buffer = Buffer::empty(rect);
    /// // Index 100 is the 101th cell, which lies outside of the area of this Buffer.
    /// buffer.pos_of(100); // Panics
    /// ```
    pub fn pos_of(&self, i: usize) -> (u16, u16) {
        debug_assert!(
            i < self.content.len(),
            "Trying to get the coords of a cell outside the buffer: i={i} len={}",
            self.content.len()
        );
        ((i as u16) % self.width, (i as u16) / self.width)
    }

    pub fn set_lines<S>(&mut self, x: u16, y: u16, string: S, style: Style)
    where
        S: AsRef<str>,
    {
        for (i, s) in string.as_ref().lines().enumerate() {
            self.set_string(x, y + i as u16, s, style)
        }
    }

    /// Print a string, starting at the position (x, y)
    pub fn set_string<S>(&mut self, x: u16, y: u16, string: S, style: Style)
    where
        S: AsRef<str>,
    {
        self.set_stringn(x, y, string, usize::MAX, style);
    }

    /// Print at most the first n characters of a string if enough space is available
    /// until the end of the line
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
        let mut index = self.index_of(x, y);
        let mut x_offset = x as usize;
        let graphemes = UnicodeSegmentation::graphemes(string.as_ref(), true);
        let max_offset = min(self.width as usize, width.saturating_add(x as usize));
        for s in graphemes {
            let width = s.width();
            if width == 0 {
                continue;
            }
            // `x_offset + width > max_offset` could be integer overflow on 32-bit machines if we
            // change dimensions to usize or u32 and someone resizes the terminal to 1x2^32.
            if width > max_offset.saturating_sub(x_offset) {
                break;
            }

            self.content[index].set_symbol(s);
            self.content[index].set_style(style);
            // Reset following cells if multi-width (they would be hidden by the grapheme),
            for i in index + 1..index + width {
                self.content[i].reset();
            }
            index += width;
            x_offset += width;
        }
        (x_offset as u16, y)
    }

    pub fn set_style(&mut self, area: Rect, style: Style) {
        for x in area.x..area.width+area.x {
            for y in area.y..area.height+area.y {
                let i = self.index_of(x, y);
                self.content[i].set_style(style);
            }
        }
    }

    pub fn insert(&mut self, x: u16, y: u16, other: &Self) {
        for (i, cell) in other.content.iter().enumerate() {
            let (xc, yc) = other.pos_of(i);
            let index = self.index_of(x + xc, y + yc);
            self.content[index] = cell.clone();
        }
    }

    pub fn draw(&self) -> BufferDrawIterator {
        BufferDrawIterator {
            buffer: self,
            index: 0,
        }
    }

    pub fn diff<'a>(&'a self, other: &'a Self) -> BufferDiffIterator {
        BufferDiffIterator {
            buffer_one: self,
            buffer_two: other,
            index: 0,
        }
    }
}

pub struct BufferDiffIterator<'a> {
    buffer_one: &'a Buffer,
    buffer_two: &'a Buffer,
    index: usize,
}

impl<'a> Iterator for BufferDiffIterator<'a> {
    type Item = (u16, u16, &'a crate::buffer::Cell);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index + 1 > self.buffer_one.area() as usize {
                return None;
            }

            if self.buffer_one.content[self.index] != self.buffer_two.content[self.index] {
                let out = Some((
                    self.index as u16 % self.buffer_two.width,
                    self.index as u16 / self.buffer_two.width,
                    &self.buffer_two.content[self.index],
                ));

                self.index += 1;
                return out;
            }

            self.index += 1;
        }
    }
}

pub struct BufferDrawIterator<'a> {
    buffer: &'a Buffer,
    index: usize,
}

impl<'a> Iterator for BufferDrawIterator<'a> {
    type Item = (u16, u16, &'a crate::buffer::Cell);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index + 1 > self.buffer.area() as usize {
                return None;
            }

            if self.buffer.content[self.index].skip {
                self.index += 1;
                continue;
            }

            let out = Some((
                self.index as u16 % self.buffer.width,
                self.index as u16 / self.buffer.width,
                &self.buffer.content[self.index],
            ));

            self.index += 1;
            return out;
        }
    }
}
