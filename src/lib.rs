pub mod buffer;
pub mod text;
pub mod window;
pub mod window_manager;

#[cfg(test)]
mod tests {
    use crossterm::{
        cursor::{Hide, Show},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io;

    use crate::{
        text::Style,
        window::{ContentInfo, Rect, Window, WindowContent},
        window_manager::WindowManager,
    };

    struct WindowContentTest {
        int: usize,
    }

    impl WindowContent for WindowContentTest {
        fn redraw(&mut self, buf: &mut crate::buffer::Buffer) -> ContentInfo {
            buf.set_stringn(0, 0, format!("number: {}", self.int), 20, Style::default());
            ContentInfo::pos(0.3)
        }

        fn update(&mut self, buf: &mut crate::buffer::Buffer) -> ContentInfo {
            self.int += 1;
            buf.set_stringn(0, 0, format!("number: {}", self.int), 20, Style::default());
            ContentInfo::pos(0.3)
        }

        fn event(
            &mut self,
            _buf: &mut crate::buffer::Buffer,
            _event: crate::window::Event,
        ) -> ContentInfo {
            ContentInfo::pos(0.3)
        }
    }

    #[test]
    fn it_works() -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        crossterm::terminal::enable_raw_mode()?;

        let (width, height) = crossterm::terminal::size()?;
        let mut manager = WindowManager::new(width, height);

        let win = Window::new(
            "Test".to_string(),
            Rect::new(4, 4, 30, 20),
            Box::new(WindowContentTest { int: 0 }),
        );

        manager.add_window(win);

        manager.run()?;

        crossterm::terminal::disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)
    }

    #[test]
    fn reset() {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
    }
}
