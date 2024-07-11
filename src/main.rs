use crossterm::{cursor, terminal, ExecutableCommand};
use std::{error::Error, io, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();

    stdout.execute(terminal::EnterAlternateScreen)?;

    sleep(Duration::from_millis(500));

    stdout.execute(cursor::MoveRight(1))?;

    sleep(Duration::from_millis(500));

    stdout.execute(cursor::MoveRight(1))?;

    sleep(Duration::from_millis(500));

    stdout.execute(cursor::MoveRight(1))?;

    sleep(Duration::from_millis(500));

    stdout.execute(cursor::MoveTo(0, 0))?;

    sleep(Duration::from_millis(500));

    stdout.execute(terminal::LeaveAlternateScreen)?;

    Ok(())
}
