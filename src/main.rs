extern crate termion;

use std::collections::VecDeque;
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

fn game(args: Args) {
    let stdin = termion::async_stdin();

    let mut keys = stdin.keys();

    let mut last_key: Option<Key> = None;

    let mut x: u16 = args.field_width / 2;
    let mut y: u16 = args.field_height / 2;

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    let mut direction = Key::Right;

    let mut now = Instant::now();

    let mut coords: VecDeque<(u16, u16)> = VecDeque::new();

    // TODO(lutzky) collapse these?
    for i in x - args.initial_snake_len..x {
        coords.push_back((i, y));
    }

    for c in &coords {
        print!("{}X", cursor::Goto(c.0 + 1, c.1 + 2));
    }

    stdout.flush();

    const FRAME_DELAY: Duration = Duration::from_millis(5);
    const MOTION_DELAY: Duration = Duration::from_millis(60);

    // TODO(lutzky): Drop me, debug only
    let mut last_popped: (u16, u16) = (0, 0);

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
                Key::Up => y = (y + args.field_height - 1) % args.field_height,
                Key::Down => y = (y + 1) % args.field_height,
                Key::Left => x = (x + args.field_width - 1) % args.field_width,
                Key::Right => x = (x + 1) % args.field_width,
                _ => todo!("use direction-specific enum"),
            }
            coords.push_back((x, y));
            match coords.pop_front() {
                None => {}
                Some((x, y)) => {
                    last_popped = (x, y);
                }
            };
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
            "{}{}Last key: {:?} pos: ({}, {}); last_popped: ({}, {})",
            clear::All,
            cursor::Goto(1, 1),
            last_key,
            x,
            y,
            last_popped.0,
            last_popped.1,
        );

        for c in &coords {
            print!("{}X", cursor::Goto(c.0 + 1, c.1 + 2));
        }

        //        print!("{}X", cursor::Goto(x + 1, y + 2));

        stdout.flush().expect("should be able to flush stdout");

        std::thread::sleep(FRAME_DELAY);
    }
}

#[derive(clap::Parser, Debug)]
struct Args {
    /// Height of the play field
    #[arg(short = 'y', long, default_value_t = 10)]
    field_height: u16,

    /// Width of the play field
    #[arg(short = 'x', long, default_value_t = 10)]
    field_width: u16,

    /// Initial snake length
    #[arg(long, default_value_t = 5)]
    initial_snake_len: u16,
}

fn main() {
    use clap::Parser;
    let args = Args::parse();

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    print!("{}{}{}", clear::All, cursor::Hide, cursor::Goto(1, 1));
    stdout.flush().unwrap();
    game(args);
    print!("{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Show);
    stdout.flush().unwrap();
}
