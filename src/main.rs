extern crate termion;

use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, AsyncReader};

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

struct Game {
    args: Args,

    stdout: termion::raw::RawTerminal<std::io::StdoutLock<'static>>,
    keys: termion::input::Keys<AsyncReader>,

    last_key: Option<Key>,

    pos: (u16, u16),
    tail_coords: VecDeque<(u16, u16)>,

    direction: Key,
    last_motion: std::time::Instant,
}

fn gen_tail_coords(pos: (u16, u16), length: u16) -> VecDeque<(u16, u16)> {
    let mut result = VecDeque::new();

    for i in pos.0 - length..pos.0 {
        result.push_back((i, pos.1));
    }

    result
}

impl Game {
    fn new(args: Args) -> Self {
        let pos = (args.field_width / 2, args.field_height / 2);
        let res = Self {
            stdout: stdout().lock().into_raw_mode().unwrap(),
            keys: termion::async_stdin().keys(),
            last_key: None,
            pos,
            tail_coords: gen_tail_coords(pos, args.initial_snake_len),
            direction: Key::Right,
            last_motion: Instant::now(),
            args,
        };
        res.draw_bounds();
        return res;
    }

    fn draw_bounds(&self) {
        let width = self.args.field_width.into();
        print!(
            "{goto_top_row}.{blank:-<width$}.\
            {goto_bottom_row}`{blank:-<width$}'",
            goto_top_row=cursor::Goto(1, 2),
            goto_bottom_row=cursor::Goto(1, self.args.field_height + 3),
            blank="",
        );
        for i in 0..self.args.field_height {
            print!("{}|{:<width$}|", cursor::Goto(1, i + 3), "");
        }
    }

    fn position_cursor(&self, pos: (u16, u16)) -> cursor::Goto {
        cursor::Goto(pos.0 + 2, pos.1 + 3)
    }

    fn play(mut self) -> Result<(), std::io::Error> {
        // TODO(lutzky) collapse these?
        for c in &self.tail_coords {
            print!("{}X", self.position_cursor(*c));
        }

        self.stdout.flush()?;

        const FRAME_DELAY: Duration = Duration::from_millis(5);
        const MOTION_DELAY: Duration = Duration::from_millis(60);

        // TODO(lutzky): Drop me, debug only
        let mut last_popped: (u16, u16) = (0, 0);

        loop {
            let k = get_key(&mut self.keys);
            match k {
                None => {}
                Some(Err(_)) => {}
                Some(Ok(k)) => self.last_key = Some(k),
            }

            if self.last_motion.elapsed() > MOTION_DELAY {
                self.last_motion = Instant::now();
                self.move_head();
                self.tail_coords.push_back(self.pos);
                print!("{}X", self.position_cursor(self.pos));
                match self.tail_coords.pop_front() {
                    None => {}
                    Some(c) => {
                        print!("{} ", self.position_cursor(c));
                        last_popped = c;
                    }
                };
            }

            match k {
                None => {}
                Some(Ok(Key::Esc)) => return Ok(()),
                Some(Err(e)) => panic!("panic! {}", e),

                // TODO: Collapse these?
                Some(Ok(Key::Up)) => self.direction = Key::Up,
                Some(Ok(Key::Down)) => self.direction = Key::Down,
                Some(Ok(Key::Left)) => self.direction = Key::Left,
                Some(Ok(Key::Right)) => self.direction = Key::Right,

                _ => {}
            }

            print!(
                "{}{}Last key: {:?} pos: ({}, {}); last_popped: ({}, {})",
                cursor::Goto(1, 1),
                clear::CurrentLine,
                self.last_key,
                self.pos.0,
                self.pos.1,
                last_popped.0,
                last_popped.1,
            );

            self.stdout.flush().expect("should be able to flush stdout");

            std::thread::sleep(FRAME_DELAY);
        }
    }

    fn move_head(&mut self) {
        match self.direction {
            Key::Up => {
                self.pos.1 = (self.pos.1 + self.args.field_height - 1) % self.args.field_height
            }
            Key::Down => self.pos.1 = (self.pos.1 + 1) % self.args.field_height,
            Key::Left => {
                self.pos.0 = (self.pos.0 + self.args.field_width - 1) % self.args.field_width
            }
            Key::Right => self.pos.0 = (self.pos.0 + 1) % self.args.field_width,
            _ => todo!("use direction-specific enum"),
        }
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
    Game::new(args).play().expect("game shouldn't error out");
    print!("{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Show);
    stdout.flush().unwrap();
}
