use std::fs;
use std::ops::{Add, Rem};
use rand::{Rng,rngs::OsRng};
use std::io::BufRead;

fn main() {
    // Open stdin
    let input = std::io::stdin();
    let mut input = input.lock();

    // Parse in program
    let args: Vec<String> = std::env::args().collect();
    let src = fs::read_to_string(&args[1]).unwrap();
    let src = Source::parse(&src);

    // Run program
    let mut state = State::initial();
    while !state.halted {
        state.step(&mut input, &src);
    }
}

#[derive(Debug, Clone)]
struct XY<A> {
    x: A,
    y: A
}

impl<A> XY<A> {
    fn new (x: A, y: A) -> Self {
        XY { x, y }
    }
}

impl<A: Add<Output = A>> Add for XY<A> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        XY::new(self.x + other.x, self.y + other.y)
    }
}

impl<A: Rem<Output = A>> Rem for XY<A> {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        XY::new(self.x % other.x, self.y % other.y)
    }
}

impl<'a, A: Copy + Rem<Output = A>> Rem for &'a XY<A> {
    type Output = XY<A>;
    fn rem(self, other: Self) -> Self::Output {
        let x: A = self.x % other.x;
        let y: A = self.y % other.y;
        XY { x, y }
    }
}

type Coord = XY<isize>;
type Direction = XY<isize>;

enum CardinalDirections {
    Up, Right, Down, Left
}

impl CardinalDirections {
    fn to_direction (&self) -> Direction {
        match self {
            CardinalDirections::Up => XY::new(0, -1),
            CardinalDirections::Down => XY::new(0, 1),
            CardinalDirections::Left => XY::new(-1, 0),
            CardinalDirections::Right => XY::new(1, 0)
        }
    }

    fn from_u8 (i: u8) -> Option<Self> {
        match i {
            0 => Some(Self::Up),
            1 => Some(Self::Right),
            2 => Some(Self::Down),
            3 => Some(Self::Left),
            _ => None
        }
    }
}

#[derive(Debug)]
struct Source {
    instrs: Vec<Vec<char>>,
    bounds: Coord
}

impl Source {
    fn parse (src: &String) -> Self {
        let instrs: Vec<Vec<char>> = src.lines().map(|line| String::from(line).chars().collect()).collect();
        let max_x: isize = instrs.iter().map(|v| v.len()).max().unwrap_or(0) as isize;
        let max_y: isize = instrs.len() as isize;
        let bounds = XY::new(max_x, max_y);
        Source { instrs, bounds }
    }

    fn lookup (&self, coord: &Coord) -> Option<char> {
        if self.bounds.x == 0 { None?; }
        if self.bounds.y == 0 { None?; }
        let coord = coord % &self.bounds;
        let row = self.instrs.get(coord.y as usize)?;
        Some(row.get(coord.x as usize).copied().unwrap_or(' '))
    }
}

struct State {
    position: Coord,
    direction: Direction,
    stack: Vec<isize>,
    string_mode: bool,
    halted: bool,
    double_jump: bool
}

impl State {
    fn initial () -> Self {
        State {
            position: XY::new(0, 0),
            direction: XY::new(1, 0),
            stack: Vec::new(),
            string_mode: false,
            halted: false,
            double_jump: false
        }
    }

    fn step<B: BufRead> (&mut self, input: &mut B, source: &Source) {
        if self.halted { return; }
        match source.lookup(&self.position) {
            None => { self.halted = true; },
            Some(instr) => {
                self.update(input, &instr);
                self.move_pointer();
            }
        }
    }

    fn move_pointer (&mut self) {
        self.position = self.position.clone() + self.direction.clone();
        if self.double_jump {
            self.double_jump = false;
            self.position = self.position.clone() + self.direction.clone();
        }
    }

    fn update<B: BufRead> (&mut self, input: &mut B, c: &char) {
        match c {
            '"' => self.string_mode = !self.string_mode,
            _ if self.string_mode => {
                let codepoint: u64 = (*c).into();
                self.stack.push(codepoint as isize);
            },
            ';' => {
                eprintln!("{:?}", self.position);
                eprintln!("{:?}", self.direction);
                eprintln!("{:?}", self.stack);
            },
            'v' => self.direction = CardinalDirections::Down.to_direction(),
            '^' => self.direction = CardinalDirections::Up.to_direction(),
            '>' => self.direction = CardinalDirections::Right.to_direction(),
            '<' => self.direction = CardinalDirections::Left.to_direction(),
            '+' => self.stack_2to1(|x, y| y + x),
            '-' => self.stack_2to1(|x, y| y - x),
            '*' => self.stack_2to1(|x, y| y * x),
            '/' => self.stack_2to1(|x, y| y / x),
            '%' => self.stack_2to1(|x, y| y % x),
            '!' => self.stack_1to1(|x| if x == 0 { 1 } else { 0 }),
            '`' => self.stack_2to1(|x, y| if y > x { 1 } else { 0 }),
            '\\' => self.stack_2to2(|x, y| (y, x)),
            '$' => { self.stack.pop(); },
            '?' =>
                self.direction =
                    CardinalDirections::from_u8(OsRng.gen_range(0..4))
                        .unwrap()
                        .to_direction(),
            '|' => {
                if let Some(top) = self.stack.pop() {
                    if top == 0 {
                        self.direction = CardinalDirections::Down.to_direction();
                    } else {
                        self.direction = CardinalDirections::Up.to_direction();
                    }
                }
            },
            '_' => {
                if let Some(top) = self.stack.pop() {
                    if top == 0 {
                        self.direction = CardinalDirections::Right.to_direction();
                    } else {
                        self.direction = CardinalDirections::Left.to_direction();
                    }
                }
            },
            ':' => {
                if let Some(top) = self.stack.pop() {
                    self.stack.push(top);
                    self.stack.push(top);
                }
            },
            '.' => {
                if let Some(top) = self.stack.pop() {
                    print!("{top}");
                }
            },
            ',' => {
                if let Some(top) = self.stack.pop() {
                    if let Some(c) = char::from_u32(top as u32) {
                        print!("{c}");
                    }
                }
            },
            '#' => self.double_jump = true,
            '@' => self.halted = true,
            '&' => {
                let mut line = String::new();
                if let Ok(_) = input.read_line(&mut line) {
                    line.pop();
                }
                let inp = line.parse().unwrap_or(0);
                self.stack.push(inp);
            },
            _ => {
                match c.to_digit(10) {
                    Some(v) => { self.stack.push(v as isize); },
                    None => {}
                }
            }
        }
    }

    fn stack_2to1<F: FnOnce(isize, isize) -> isize> (&mut self, dyad: F) {
        if let Some(arg1) = self.stack.pop() {
            if let Some(arg2) = self.stack.pop() {
                self.stack.push(dyad(arg1, arg2))
            } else {
                self.stack.push(arg1);
            }
        }
    }

    fn stack_2to2<F: FnOnce(isize, isize) -> (isize, isize)> (&mut self, dyad: F) {
        if let Some(arg1) = self.stack.pop() {
            if let Some(arg2) = self.stack.pop() {
                let (out1, out2) = dyad(arg1, arg2);
                self.stack.push(out2);
                self.stack.push(out1);
            } else {
                self.stack.push(arg1);
            }
        }
    }

    fn stack_1to1<F: FnOnce(isize) -> isize> (&mut self, monad: F) {
        if let Some(arg1) = self.stack.pop() {
            self.stack.push(monad(arg1))
        }
    }
}
