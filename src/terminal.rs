use std::io::{self, Write};
use crossterm::style::{
    Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::{cursor::MoveTo, queue};

use crate::window::Window;
use crate::{buffer::Buffer, text::Modifier};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Terminal {
    width: u16,
    height: u16,
    buffer: Buffer,
}

impl Terminal {
    pub fn init() -> Self {
        let (width, height) = crossterm::terminal::size().expect("Unable to get terminal size");
        Terminal {
            width,
            height,
            buffer: Buffer::empty(width, height),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.buffer.resize(width, height);
        self.buffer.reset();
    }

    pub fn set_cursor_pos(x: u16, y: u16) -> io::Result<()> {
        crossterm::execute!(io::stdout(), MoveTo(x, y))
    }

    pub fn put(&mut self, win: &Window) -> io::Result<()> {
        let (width, height) = crossterm::terminal::size().expect("Unable to get terminal size");
        if width != self.width || height != self.height {
            self.buffer.resize(width, height);
            self.width = width;
            self.height = height;
        }
        self.draw(io::stdout(), win)?;
        io::stdout().flush()?;
        Ok(())
    }

    pub fn draw<'a, W>(&mut self, mut writer: W, win: &Window) -> io::Result<()>
    where
        W: Write,
    {
        let (win_x, win_y) = win.pos();

        let cursor = crossterm::cursor::position().expect("Unable to get cursor position");

        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut modifier = Modifier::empty();
        let mut last_pos: Option<(u16, u16)> = None;
        for (x, y, cell) in win.content_iter() {
            let x = x+win_x;
            let y = y+win_y;

            let index = self.buffer.index_of(x, y);

            if *cell == self.buffer.content[index] {
                continue;
            }

            self.buffer.content[index] = cell.clone();

            // Move the cursor if the previous location was not (x - 1, y)
            if !matches!(last_pos, Some(p) if x == p.0 + 1 && y == p.1) {
                queue!(writer, MoveTo(x, y))?;
            }
            last_pos = Some((x, y));
            if cell.modifier != modifier {
                let diff = ModifierDiff {
                    from: modifier,
                    to: cell.modifier,
                };
                diff.queue(&mut writer)?;
                modifier = cell.modifier;
            }
            if cell.fg != fg {
                let color = cell.fg;
                queue!(writer, SetForegroundColor(color))?;
                fg = cell.fg;
            }
            if cell.bg != bg {
                let color = cell.bg;
                queue!(writer, SetBackgroundColor(color))?;
                bg = cell.bg;
            }

            queue!(writer, Print(&cell.symbol))?;
        }

        queue!(
            writer,
            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset),
            SetAttribute(Attribute::Reset),
            MoveTo(cursor.0, cursor.1),
        )
    }
}

/// The `ModifierDiff` struct is used to calculate the difference between two `Modifier`
/// values. This is useful when updating the terminal display, as it allows for more
/// efficient updates by only sending the necessary changes.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
struct ModifierDiff {
    pub from: Modifier,
    pub to: Modifier,
}

impl ModifierDiff {
    fn queue<W>(&self, mut w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        //use crossterm::Attribute;
        let removed = self.from - self.to;
        if removed.contains(Modifier::REVERSED) {
            queue!(w, SetAttribute(Attribute::NoReverse))?;
        }
        if removed.contains(Modifier::BOLD) {
            queue!(w, SetAttribute(Attribute::NormalIntensity))?;
            if self.to.contains(Modifier::DIM) {
                queue!(w, SetAttribute(Attribute::Dim))?;
            }
        }
        if removed.contains(Modifier::ITALIC) {
            queue!(w, SetAttribute(Attribute::NoItalic))?;
        }
        if removed.contains(Modifier::UNDERLINED) {
            queue!(w, SetAttribute(Attribute::NoUnderline))?;
        }
        if removed.contains(Modifier::DIM) {
            queue!(w, SetAttribute(Attribute::NormalIntensity))?;
        }
        if removed.contains(Modifier::CROSSED_OUT) {
            queue!(w, SetAttribute(Attribute::NotCrossedOut))?;
        }
        if removed.contains(Modifier::SLOW_BLINK) || removed.contains(Modifier::RAPID_BLINK) {
            queue!(w, SetAttribute(Attribute::NoBlink))?;
        }

        let added = self.to - self.from;
        if added.contains(Modifier::REVERSED) {
            queue!(w, SetAttribute(Attribute::Reverse))?;
        }
        if added.contains(Modifier::BOLD) {
            queue!(w, SetAttribute(Attribute::Bold))?;
        }
        if added.contains(Modifier::ITALIC) {
            queue!(w, SetAttribute(Attribute::Italic))?;
        }
        if added.contains(Modifier::UNDERLINED) {
            queue!(w, SetAttribute(Attribute::Underlined))?;
        }
        if added.contains(Modifier::DIM) {
            queue!(w, SetAttribute(Attribute::Dim))?;
        }
        if added.contains(Modifier::CROSSED_OUT) {
            queue!(w, SetAttribute(Attribute::CrossedOut))?;
        }
        if added.contains(Modifier::SLOW_BLINK) {
            queue!(w, SetAttribute(Attribute::SlowBlink))?;
        }
        if added.contains(Modifier::RAPID_BLINK) {
            queue!(w, SetAttribute(Attribute::RapidBlink))?;
        }

        Ok(())
    }
}
