extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn getKey(
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

    let mut lastNonNothing: Option<Key> = None;
    loop {
        let k = getKey(&mut keys);
        match k {
            None => {}
            Some(Err(_)) => {}
            Some(Ok(k)) => lastNonNothing = Some(k),
        }

        print!(
            "{}{}Last key: {:?}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 2),
            lastNonNothing,
            termion::cursor::Goto(1, 1)
        );

        match k {
            None => println!("nothing"),
            Some(Ok(Key::Esc)) => return,
            Some(Err(e)) => panic!("panic! {}", e),
            Some(Ok(k)) => println!("other: {:?}", k),
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    print!(
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Hide,
        termion::cursor::Goto(1, 1)
    );
    stdout.flush().unwrap();
    game();
    print!("{}", termion::cursor::Show);
    stdout.flush().unwrap();
}
