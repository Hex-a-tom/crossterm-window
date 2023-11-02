pub mod buffer;
pub mod terminal;
pub mod text;
pub mod window;

#[cfg(test)]
mod tests {
    use crossterm::{
        cursor::{Hide, Show},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io;

    use crate::window::{Rect, Window};

    #[test]
    fn it_works() -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        crossterm::terminal::enable_raw_mode()?;

        let (width, height) = crossterm::terminal::size()?;

        let win = Window::new(Rect::new(4, 4, 30, 20));

        crossterm::terminal::disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)
    }

    #[test]
    fn reset() {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
    }
}
