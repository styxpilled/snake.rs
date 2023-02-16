use crossterm::{
    cursor::{self, Hide, Show},
    event::{poll, read, Event, KeyCode},
    queue,
    style::Print,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};
use std::{
    io::{stdout, Write},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(Clone)]
struct Snake {
    body: Vec<Point>,
}

#[derive(PartialEq)]
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
    const WIDTH: u16 = 60;
    const HEIGHT: u16 = 20;
    // draw_border(&mut stdout, WIDTH, HEIGHT)?;
    draw_food(&mut stdout, food.position.x, food.position.y)?;
    draw_snake(&mut stdout, &snake.body, &snake.body)?;
    queue!(stdout, cursor::MoveTo(0, HEIGHT + 2))?;
    // writeln!(stdout, "Score: {}", score)?;
    loop {
        if poll(Duration::from_millis(100))? {
            let event = read()?;
            match event {
                Event::Key(event) => match event.code {
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Up if direction != Direction::Down => {
                        direction = Direction::Up;
                    }
                    KeyCode::Down if direction != Direction::Up => {
                        direction = Direction::Down;
                    }
                    KeyCode::Left if direction != Direction::Right => {
                        direction = Direction::Left;
                    }
                    KeyCode::Right if direction != Direction::Left => {
                        direction = Direction::Right;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        let head = snake.body[0];
        let new_head = match direction {
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
        };
        if snake.body.contains(&new_head) {
            break;
        }
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
