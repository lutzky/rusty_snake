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
        let c: char = item.into();
        print!("{}{}", cursor::Goto(x + 2, y + 3), c,);
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

    prev_direction: Direction,
    direction: Direction,
    last_motion: std::time::Instant,

    // Times to lengthen the tail
    lengthenings: u8,

    board: Board,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl TryFrom<Key> for Direction {
    type Error = String;

    fn try_from(k: Key) -> Result<Self, Self::Error> {
        match k {
            Key::Left => Ok(Self::Left),
            Key::Right => Ok(Self::Right),
            Key::Up => Ok(Self::Up),
            Key::Down => Ok(Self::Down),
            _ => Err(format!("cannot convert {:?} to direction", k)),
        }
    }
}

#[derive(Copy, Clone)]
enum BoardItem {
    Empty,
    Horizontal,
    Vertical,
    TopRight,
    TopLeft,
    BottomLeft,
    BottomRight,
}

impl BoardItem {
    fn from_pair(prev: Direction, current: Direction) -> Self {
        use Direction::{Down, Left, Right, Up};

        match (prev, current) {
            (Right, Right) | (Left, Left) => BoardItem::Horizontal,
            (Up, Up) | (Down, Down) => BoardItem::Vertical,
            (Right, Down) | (Up, Left) => BoardItem::TopRight,
            (Up, Right) | (Left, Down) => BoardItem::TopLeft,
            (Down, Right) | (Left, Up) => BoardItem::BottomLeft,
            (Right, Up) | (Down, Left) => BoardItem::BottomRight,

            // Technically correct, though should-be-impossible options
            (Up, Down) | (Down, Up) => BoardItem::Vertical,
            (Right, Left) | (Left, Right) => BoardItem::Horizontal,
        }
    }
}

impl From<BoardItem> for char {
    fn from(val: BoardItem) -> Self {
        match val {
            BoardItem::Empty => ' ',
            BoardItem::Horizontal => '═',
            BoardItem::Vertical => '║',
            BoardItem::TopRight => '╗',
            BoardItem::TopLeft => '╔',
            BoardItem::BottomLeft => '╚',
            BoardItem::BottomRight => '╝',
        }
    }
}

#[derive(Debug)]
enum GameResult {
    Quit,
    Lost,
}

impl Direction {
    fn opposite(self) -> Self {
        use Direction::{Down, Left, Right, Up};

        match self {
            Left => Right,
            Right => Left,
            Up => Down,
            Down => Up,
        }
    }
}

impl Game {
    fn new(args: Args) -> Self {
        let pos = (args.field_width / 2, args.field_height / 2);
        let res = Self {
            stdout: stdout().lock().into_raw_mode().unwrap(),
            keys: termion::async_stdin().keys(),
            lengthenings: args.initial_snake_len,
            last_key: None,
            pos,
            tail_coords: VecDeque::from([pos]),
            direction: Direction::Right,
            prev_direction: Direction::Right,
            last_motion: Instant::now(),
            board: Board::new((args.field_width, args.field_height)),
            args,
        };
        res.draw_bounds();
        res
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
            self.board.set_tile(*c, BoardItem::Horizontal);
        }

        self.stdout.flush()?;

        let mut last_popped: (u16, u16) = (0, 0);

        loop {
            let k = get_key(&mut self.keys);
            match k {
                None => {}
                Some(Err(_)) => {}
                Some(Ok(k)) => self.last_key = Some(k),
            }

            if self.last_motion.elapsed() > self.args.motion_delay {
                if self.direction == self.prev_direction.opposite() {
                    self.direction = self.prev_direction;
                }
                self.last_motion = Instant::now();
                self.board.set_tile(
                    self.pos,
                    BoardItem::from_pair(self.prev_direction, self.direction),
                );
                self.prev_direction = self.direction;
                self.move_head();

                match self.board.get_tile(self.pos) {
                    BoardItem::Empty => {}
                    _ => return Ok(GameResult::Lost),
                }
                self.tail_coords.push_back(self.pos);
                self.board.set_tile(
                    self.pos,
                    BoardItem::from_pair(self.prev_direction, self.direction),
                );
                match self.lengthenings {
                    0 => match self.tail_coords.pop_front() {
                        None => {}
                        Some(c) => {
                            self.board.set_tile(c, BoardItem::Empty);
                            last_popped = c;
                        }
                    },
                    _ => self.lengthenings -= 1,
                };
            }

            {
                match k {
                    Some(Err(e)) => return Err(e),
                    Some(Ok(actual_key)) => match actual_key {
                        Key::Esc => return Ok(GameResult::Quit),
                        Key::Char('+') => self.lengthenings += 1,
                        _ => {
                            if let Ok(d) = actual_key.try_into() {
                                self.direction = d;
                            }
                        }
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

            std::thread::sleep(self.args.frame_delay);
        }
    }

    fn move_head(&mut self) {
        use Direction::{Down, Left, Right, Up};

        match self.direction {
            Up => self.pos.1 = (self.pos.1 + self.args.field_height - 1) % self.args.field_height,
            Down => self.pos.1 = (self.pos.1 + 1) % self.args.field_height,
            Left => self.pos.0 = (self.pos.0 + self.args.field_width - 1) % self.args.field_width,
            Right => self.pos.0 = (self.pos.0 + 1) % self.args.field_width,
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
    initial_snake_len: u8,

    /// Amount of milliseconds to wait after each frame
    #[arg(long, value_parser = parse_duration_from_millis, default_value = "5")]
    frame_delay: Duration,

    /// Amount of milliseconds to wait between movements of snake
    #[arg(long,  value_parser = parse_duration_from_millis, default_value = "60")]
    motion_delay: Duration,
}

fn parse_duration_from_millis(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let millis = arg.parse()?;
    Ok(Duration::from_millis(millis))
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
