extern crate termion;

use std::io::{stdout, Write};
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

    loop {
        let k = get_key(&mut keys);
        match k {
            None => {}
            Some(Err(_)) => {}
            Some(Ok(k)) => last_key = Some(k),
        }

        match direction {
            Key::Up => y = (y + 10 - 1) % 10,
            Key::Down => y = (y + 1) % 10,
            Key::Left => x = (x + 10 - 1) % 10,
            Key::Right => x = (x + 1) % 10,
            _ => todo!("use direction-specific enum"),
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
            cursor::Goto(1, 2),
            last_key,
            x,
            y,
        );

        print!("{}X", cursor::Goto(x + 1, y + 1));

        stdout.flush();

        std::thread::sleep(std::time::Duration::from_millis(100));
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
