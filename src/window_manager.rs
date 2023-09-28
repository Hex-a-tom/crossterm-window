use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{poll, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{
    Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::{cursor::MoveTo, queue};

use crate::buffer::BufferDiffIterator;
use crate::window::Window;
use crate::{buffer::Buffer, text::Modifier};

pub struct WindowManager {
    buffers: [Buffer; 2],
    current: usize,

    should_exit: bool,

    // TODO: Unessessary, remove these
    width: u16,
    height: u16,

    // Windows
    windows: Vec<Window>,
    current_window: usize,
}

impl WindowManager {
    pub fn new(width: u16, height: u16) -> Self {
        WindowManager {
            buffers: [Buffer::empty(width, height), Buffer::empty(width, height)],
            current: 0,
            should_exit: false,
            width,
            height,
            windows: vec![],
            current_window: 0,
        }
    }

    pub fn add_window(&mut self, win: Window) {
        self.windows.push(win);
    }

    pub fn handle_manager_keys(&mut self, e: KeyEvent) -> bool {
        let mut passthrough = false;

        // Handle window manager keybinds
        if e.modifiers.contains(KeyModifiers::CONTROL) {
            if e.modifiers.contains(KeyModifiers::SHIFT) {
                // Ctrl + Shift
                match e.code {
                    KeyCode::Left => self.windows[self.current_window].resize_by(-1, 0),
                    KeyCode::Up => self.windows[self.current_window].resize_by(0, -1),
                    KeyCode::Right => self.windows[self.current_window].resize_by(1, 0),
                    KeyCode::Down => self.windows[self.current_window].resize_by(0, 1),
                    _ => passthrough = true,
                }
            } else {
                // Ctrl
                match e.code {
                    KeyCode::Char(c) => match c {
                        'c' => self.should_exit = true,
                        _ => passthrough = true,
                    },
                    KeyCode::Left => self.windows[self.current_window].move_by(-1, 0),
                    KeyCode::Up => self.windows[self.current_window].move_by(0, -1),
                    KeyCode::Right => self.windows[self.current_window].move_by(1, 0),
                    KeyCode::Down => self.windows[self.current_window].move_by(0, 1),
                    KeyCode::PageDown => {
                        if self.windows.len() != 0 {
                            let win = self.windows.remove(0);
                            self.windows.push(win);
                        }
                    }
                    KeyCode::PageUp => {
                        if let Some(win) = self.windows.pop() {
                            self.windows.insert(0, win)
                        }
                    }
                    _ => passthrough = true,
                }
            }
        } else {
            passthrough = true
        }

        passthrough
    }

    const FRAMETIME: Duration = Duration::from_millis(50);

    pub fn run(&mut self) -> io::Result<()> {
        let mut now = Instant::now();
        let mut next_frame = Instant::now() + WindowManager::FRAMETIME;
        while !self.should_exit {
            if poll(next_frame.duration_since(now))? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(e) => {
                        let passthrough = self.handle_manager_keys(e);

                        if passthrough && !self.windows.is_empty() {
                            self.windows[self.current_window].event(crate::window::Event::Key(e));
                        }

                        self.draw_windows();
                        self.update_screen()?;
                    }
                    crossterm::event::Event::Mouse(_) => return Ok(()),
                    crossterm::event::Event::Paste(_) => return Ok(()),
                    crossterm::event::Event::Resize(width, height) => {
                        self.buffers[0].resize(width, height);
                        self.buffers[1].resize(width, height);
                        self.width = width;
                        self.height = height;
                    }
                    _ => (),
                }
            } else {
                for win in &mut self.windows {
                    win.update();
                }
                self.draw_windows();
                self.update_screen()?;

                next_frame = Instant::now() + WindowManager::FRAMETIME;
            }

            now = Instant::now();
        }

        Ok(())
    }

    pub fn draw_windows(&mut self) {
        for win in &mut self.windows {
            win.draw(&mut self.buffers[self.current]);
        }
    }

    pub fn update_screen(&mut self) -> io::Result<()> {
        self.flush()?;
        self.swap_buffers();
        io::stdout().flush()?;
        Ok(())
    }

    /// Obtains a difference between the previous and the current buffer and passes it to the
    /// current backend for drawing.
    pub fn flush(&mut self) -> io::Result<()> {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let updates = previous_buffer.diff(current_buffer);
        self.draw(io::stdout(), updates)
    }

    /// Clears the inactive buffer and swaps it with the current buffer
    pub fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }

    pub fn draw<'a, W>(&self, mut writer: W, diff: BufferDiffIterator) -> io::Result<()>
    where
        W: Write,
    {
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut modifier = Modifier::empty();
        let mut last_pos: Option<(u16, u16)> = None;
        for (x, y, cell) in diff {
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
            SetAttribute(Attribute::Reset)
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
