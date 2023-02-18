use crossterm::{
    cursor::{self, Hide, Show},
    event::{poll, read, Event, KeyCode},
    queue,
    style::Print,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};
use pathfinding::prelude::astar;
use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Point {
    x: u16,
    y: u16,
}

const WIDTH: u16 = 60;
const HEIGHT: u16 = 20;

impl Point {
    fn distance(&self, other: &Point) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }

    fn successors(&self, body: &Vec<Point>) -> Vec<(Point, u32)> {
        let &Point { x, y } = self;
        vec![
            Point {
                x: x,
                y: if y > 1 { y - 1 } else { HEIGHT },
            },
            Point {
                x: x,
                y: if y < HEIGHT { y + 1 } else { 1 },
            },
            Point {
                x: if x > 1 { x - 1 } else { WIDTH },
                y: y,
            },
            Point {
                x: if x < WIDTH { x + 1 } else { 1 },
                y: y,
            },
            Point {
                x: if x > 1 { x - 1 } else { WIDTH },
                y: if y > 1 { y - 1 } else { HEIGHT },
            },
            Point {
                x: if x < WIDTH { x + 1 } else { 1 },
                y: if y < HEIGHT { y + 1 } else { 1 },
            },
        ]
        .into_iter()
        .filter(|p| !body.contains(p))
        .map(|p| (p, 1))
        .collect()
    }
}

#[derive(Clone)]
struct Snake {
    body: Vec<Point>,
}

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Food {
    position: Point,
}

fn generate_food(snake_body: &[Point]) -> Food {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    loop {
        let x = rng.gen_range(1..=58);
        let y = rng.gen_range(1..=18);
        let position = Point { x, y };
        if !snake_body.contains(&position) {
            return Food { position };
        }
    }
}

fn draw_border(stdout: &mut std::io::Stdout, width: u16, height: u16) -> Result<()> {
    queue!(stdout, cursor::MoveTo(0, 0))?;
    for _ in 0..=width + 1 {
        queue!(stdout, Print("#"))?;
    }
    queue!(stdout, cursor::MoveTo(0, height + 1))?;
    for _ in 0..=width + 1 {
        queue!(stdout, Print("#"))?;
    }
    for y in 1..=height {
        queue!(stdout, cursor::MoveTo(0, y))?;
        queue!(stdout, Print("#"))?;
        queue!(stdout, cursor::MoveTo(width + 1, y))?;
        queue!(stdout, Print("#"))?;
    }
    stdout.flush()?;
    Ok(())
}

fn draw_food(stdout: &mut std::io::Stdout, x: u16, y: u16) -> Result<()> {
    queue!(stdout, cursor::MoveTo(x + 1, y + 1))?;
    queue!(stdout, Print("@"))?;
    Ok(())
}

fn draw_snake(stdout: &mut std::io::Stdout, body: &[Point], old_body: &[Point]) -> Result<()> {
    for point in old_body.iter() {
        queue!(stdout, cursor::MoveTo(point.x + 1, point.y + 1), Print(" "))?;
    }
    for (i, point) in body.iter().enumerate() {
        queue!(
            stdout,
            cursor::MoveTo(point.x + 1, point.y + 1),
            if i == 0 { Print("O") } else { Print("o") }
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    queue!(stdout, EnterAlternateScreen, Hide)?;
    let mut score = 0;
    let mut snake = Snake {
        body: vec![
            Point { x: 5, y: 5 },
            Point { x: 4, y: 5 },
            Point { x: 3, y: 5 },
        ],
    };
    let mut food = generate_food(&snake.body);
    let mut direction = Direction::Right;
    // draw_border(&mut stdout, WIDTH, HEIGHT)?;
    draw_food(&mut stdout, food.position.x, food.position.y)?;
    draw_snake(&mut stdout, &snake.body, &snake.body)?;
    queue!(stdout, cursor::MoveTo(0, HEIGHT + 2))?;
    // writeln!(stdout, "Score: {}", score)?;
    let mut auto = false;
    let mut event = None;
    loop {
        if poll(Duration::from_millis(10))? {
            event = Some(read()?);
            match event.clone().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Char('a') => {
                        auto = !auto;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        let new_head = if auto && event.is_some() {
            match event.clone().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Up if direction != Direction::Down && !auto => {
                        direction = Direction::Up;
                    }
                    KeyCode::Down if direction != Direction::Up && !auto => {
                        direction = Direction::Down;
                    }
                    KeyCode::Left if direction != Direction::Right && !auto => {
                        direction = Direction::Left;
                    }
                    KeyCode::Right if direction != Direction::Left && !auto => {
                        direction = Direction::Right;
                    }
                    _ => {}
                },
                _ => {}
            };
            let head = snake.body[0];
            match direction {
                Direction::Up => Point {
                    x: head.x,
                    y: if head.y > 1 { head.y - 1 } else { HEIGHT },
                },
                Direction::Down => Point {
                    x: head.x,
                    y: if head.y < HEIGHT { head.y + 1 } else { 1 },
                },
                Direction::Left => Point {
                    x: if head.x > 1 { head.x - 1 } else { WIDTH },
                    y: head.y,
                },
                Direction::Right => Point {
                    x: if head.x < WIDTH { head.x + 1 } else { 1 },
                    y: head.y,
                },
            }
        } else {
            let goal = food.position;
            let result = astar(
                &snake.body[0],
                |p| p.successors(&snake.body),
                |p| p.distance(&goal),
                |p| *p == goal,
            );
            match result {
                Some(steps) => {
                    let mut steps = steps.0.iter();
                    let _ = steps.next().unwrap();
                    let step = steps.next().unwrap();
                    {
                        // sleep(Duration::from_millis(1));
                        *step
                        // if snake.body.contains(&new_head) {
                        //     break;
                        // }
                    }
                }
                None => {
                    auto = false;
                    snake.body[0]
                }
            }
        };
        let old_body = snake.clone();
        snake.body.insert(0, new_head);
        if snake.body[0] == food.position {
            score += 1;
            food = generate_food(&snake.body);
        } else {
            snake.body.pop();
        }
        // queue!(stdout, Clear(ClearType::All))?;
        draw_border(&mut stdout, WIDTH, HEIGHT)?;
        draw_food(&mut stdout, food.position.x, food.position.y)?;
        draw_snake(&mut stdout, &snake.body, &old_body.body)?;
        queue!(stdout, cursor::MoveTo(0, HEIGHT + 2))?;
        println!("Score: {}", score);
        stdout.flush().unwrap();
    }
    draw_border(&mut stdout, WIDTH, HEIGHT)?;
    draw_food(&mut stdout, food.position.x, food.position.y)?;
    draw_snake(&mut stdout, &snake.body, &[])?;
    queue!(
        stdout,
        cursor::MoveTo((WIDTH / 2) - 5, HEIGHT / 2),
        Print("GAME OVER"),
        cursor::MoveTo((WIDTH / 2) - 9, HEIGHT / 2 + 1),
        Print("PRESS ESC TO EXIT")
    )?;
    stdout.flush()?;
    loop {
        if poll(Duration::from_millis(100))? {
            let event = read()?;
            match event {
                Event::Key(event) => match event.code {
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    queue!(stdout, Show, LeaveAlternateScreen)?;
    stdout.flush()?;
    Ok(())
}
