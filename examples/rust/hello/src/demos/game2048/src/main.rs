use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use game2048::*;
use std::io::{self, Write};

fn main() -> crossterm::Result<()> {
    enable_raw_mode()?;
    println!("按方向键进行游戏。按 'Esc' 退出游戏。");

    let mut g = Game2048::default();
    g.random_fill();
    println!("{g}");

    loop {
        if let Event::Key(key_event) = read()? {
            if key_event.kind != KeyEventKind::Release {
                continue;
            }

            let path = match key_event.code {
                KeyCode::Esc => {
                    println!("\n退出游戏");
                    break;
                }
                KeyCode::Up => g.up(),
                KeyCode::Down => g.down(),
                KeyCode::Left => g.left(),
                KeyCode::Right => g.right(),
                _ => continue,
            };

            if path.is_empty() {
                continue;
            }

            dbg!(path);

            if g.is_it_win() {
                println!("{g}");
                println!("2048!");
                println!("游戏结束");
                break;
            }

            g.random_fill();
            println!("{g}");

            if g.is_it_over() {
                println!("游戏结束");
                break;
            }
        }

        io::stdout().flush().ok();
    }

    disable_raw_mode()?;
    Ok(())
}
