extern crate termion;

use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

fn get_key(
    keys: &mut termion::input::Keys<termion::AsyncReader>,
) -> Option<Result<Key, std::io::Error>> {
    let mut result: Option<Result<Key, std::io::Error>> = None;

    loop {
        let k = keys.next();
        match k {
            None => {
                return result;
            }
            Some(Err(e)) => return Some(Err(e)),
            Some(Ok(Key::Esc)) => return Some(Ok(Key::Esc)),
            Some(Ok(Key::Up)) => result = Some(Ok(Key::Up)),
            Some(Ok(other)) => result = Some(Ok(other)),
        }
    }
}

fn game() {
    let stdin = termion::async_stdin();

    let mut keys = stdin.keys();

    let mut last_key: Option<Key> = None;

    let mut x: u16 = 5;
    let mut y: u16 = 5;

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    let mut direction = Key::Right;

    let mut now = Instant::now();

    const FRAME_DELAY: Duration = Duration::from_millis(5);
    const MOTION_DELAY: Duration = Duration::from_millis(60);
    const FIELD_WIDTH: u16 = 10;
    const FIELD_HEIGHT: u16 = 10;

    loop {
        let k = get_key(&mut keys);
        match k {
            None => {}
            Some(Err(_)) => {}
            Some(Ok(k)) => last_key = Some(k),
        }

        if now.elapsed() > MOTION_DELAY {
            now = Instant::now();
            match direction {
                Key::Up => y = (y + FIELD_HEIGHT - 1) % FIELD_HEIGHT,
                Key::Down => y = (y + 1) % FIELD_HEIGHT,
                Key::Left => x = (x + FIELD_WIDTH - 1) % FIELD_WIDTH,
                Key::Right => x = (x + 1) % FIELD_WIDTH,
                _ => todo!("use direction-specific enum"),
            }
        }

        match k {
            None => {}
            Some(Ok(Key::Esc)) => return,
            Some(Err(e)) => panic!("panic! {}", e),

            // TODO: Collapse these?
            Some(Ok(Key::Up)) => direction = Key::Up,
            Some(Ok(Key::Down)) => direction = Key::Down,
            Some(Ok(Key::Left)) => direction = Key::Left,
            Some(Ok(Key::Right)) => direction = Key::Right,

            _ => {}
        }

        print!(
            "{}{}Last key: {:?} (x: {}, y: {})",
            clear::All,
            cursor::Goto(1, 1),
            last_key,
            x,
            y,
        );

        print!("{}X", cursor::Goto(x + 1, y + 2));

        stdout.flush().expect("should be able to flush stdout");

        std::thread::sleep(FRAME_DELAY);
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    print!("{}{}{}", clear::All, cursor::Hide, cursor::Goto(1, 1));
    stdout.flush().unwrap();
    game();
    print!("{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Show);
    stdout.flush().unwrap();
}
