use karabasan::{Chat, Turn};
use std::{
    io::{self, BufRead, IsTerminal, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn render(turn: &Turn, instant: bool) -> io::Result<()> {
    let mut out = io::stdout().lock();
    for line in &turn.lines {
        if instant {
            writeln!(out, "{}", line.text)?;
        } else {
            for c in line.text.chars() {
                write!(out, "{c}")?;
                out.flush()?;
                std::thread::sleep(Duration::from_millis(12));
            }
            writeln!(out)?;
            out.flush()?;
            std::thread::sleep(Duration::from_millis(line.delay_ms as u64));
        }
    }
    if !turn.finished {
        write!(out, "> ")?;
    }
    out.flush()
}

fn run() -> io::Result<()> {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u32;
    let mut instant = !io::stdout().is_terminal() || !io::stdin().is_terminal();
    let mut json = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--instant" => instant = true,
            "--json" => {
                json = true;
                instant = true;
            }
            "--seed" => {
                seed = args.next().and_then(|v| v.parse().ok()).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--seed requires a u32")
                })?
            }
            "-h" | "--help" => {
                println!("Karabasan — DOS sohbetine bir saygı duruşu\n\nUsage: karabasan [--instant] [--seed N] [--json]\n\n/quit: çıkış. Ctrl-D / EOF: çıkış.\n--json emits the shared engine's turn protocol (one JSON object per line).");
                return Ok(());
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument: {arg}"),
                ))
            }
        }
    }
    let mut chat = Chat::new(seed);
    let mut turn = chat.start();
    if json {
        println!("{}", turn.to_json());
    } else {
        render(&turn, instant)?;
    }
    for line in io::stdin().lock().lines() {
        turn = chat.submit(&line?);
        if json {
            println!("{}", turn.to_json());
        } else {
            render(&turn, instant)?;
        }
        if turn.finished {
            break;
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("karabasan: {e}");
            std::process::exit(1);
        }
    }
}
