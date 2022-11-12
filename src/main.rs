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

struct Board {
    grid: Vec<Vec<BoardItem>>,
}

impl Board {
    fn new((x, y): (u16, u16)) -> Self {
        Board {
            grid: vec![vec![BoardItem::Empty; y.into()]; x.into()],
        }
    }

    fn set_tile(&mut self, (x, y): (u16, u16), item: BoardItem) {
        self.grid[usize::from(x)][usize::from(y)] = item;
        print!(
            "{}{}",
            cursor::Goto(x + 2, y + 3),
            match item {
                BoardItem::Empty => ' ',
                BoardItem::Snake => 'X',
            }
        );
    }

    fn get_tile(&self, (x, y): (u16, u16)) -> BoardItem {
        self.grid[usize::from(x)][usize::from(y)]
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

    board: Board,
}

fn gen_tail_coords(pos: (u16, u16), length: u16) -> VecDeque<(u16, u16)> {
    let mut result = VecDeque::new();

    for i in pos.0 - length..pos.0 {
        result.push_back((i, pos.1));
    }

    result
}

#[derive(Copy, Clone)]
enum BoardItem {
    Empty,
    Snake,
}

#[derive(Debug)]
enum GameResult {
    Quit,
    Lost,
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
            board: Board::new((args.field_width, args.field_height)),
            args,
        };
        res.draw_bounds();
        return res;
    }

    fn draw_bounds(&self) {
        let width = self.args.field_width.into();
        print!(
            "{goto_top_row}╓{blank:─<width$}╖\
            {goto_bottom_row}╙{blank:─<width$}╜",
            goto_top_row = cursor::Goto(1, 2),
            goto_bottom_row = cursor::Goto(1, self.args.field_height + 3),
            blank = "",
        );
        for i in 0..self.args.field_height {
            print!("{}║{:<width$}║", cursor::Goto(1, i + 3), "");
        }
    }

    fn play(mut self) -> Result<GameResult, std::io::Error> {
        for c in &self.tail_coords {
            self.board.set_tile(*c, BoardItem::Snake);
        }

        self.stdout.flush()?;

        const FRAME_DELAY: Duration = Duration::from_millis(5);
        const MOTION_DELAY: Duration = Duration::from_millis(60);

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

                match self.board.get_tile(self.pos) {
                    BoardItem::Empty => {}
                    BoardItem::Snake => return Ok(GameResult::Lost),
                }
                self.tail_coords.push_back(self.pos);
                self.board.set_tile(self.pos, BoardItem::Snake);
                match self.tail_coords.pop_front() {
                    None => {}
                    Some(c) => {
                        self.board.set_tile(c, BoardItem::Empty);
                        last_popped = c;
                    }
                };
            }

            {
                use Key::*;
                match k {
                    Some(Err(e)) => return Err(e),
                    Some(Ok(actual_key)) => match actual_key {
                        Esc => return Ok(GameResult::Quit),
                        Up | Down | Left | Right => self.direction = actual_key,
                        _ => {}
                    },
                    _ => {}
                }
            }

            print!(
                "{}{}Last key: {:?} pos:{:?}; last_popped:{last_popped:?}",
                cursor::Goto(1, 1),
                clear::CurrentLine,
                self.last_key,
                self.pos,
            );

            self.stdout.flush()?;

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

fn must_terminal_size() -> (u16, u16) {
    let (x, y) = termion::terminal_size().expect("should be able to fetch terminal size");
    (x - 2, y - 4)
}
#[derive(clap::Parser, Debug)]
struct Args {
    /// Width of the play field
    #[arg(
      short = 'x', long,
      default_value_t = must_terminal_size().0)]
    field_width: u16,

    /// Height of the play field
    #[arg(
      short = 'y', long,
      default_value_t = must_terminal_size().1)]
    field_height: u16,

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
    let result = Game::new(args).play().expect("game shouldn't error out");
    stdout
        .suspend_raw_mode()
        .expect("raw mode should be suspended");

    println!(
        "{}{}Result: {:?}",
        cursor::Goto(1, termion::terminal_size().unwrap().1),
        cursor::Show,
        result
    );
    stdout.flush().unwrap();
}
