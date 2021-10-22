use std::{fmt::Display, io::{self, Write, stdout}, sync::atomic::{AtomicBool, Ordering}};
use termion::{clear, color::{self, LightGreen}, cursor, style::{self, Bold}};

/// Whether we're using verbose output or not
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

pub fn init(verbose: bool) {
    VERBOSE.store(verbose, Ordering::Relaxed);

    if !verbose {
        let mut stdout = io::stdout();
        let _ = write!(stdout, "{}", cursor::Hide);
        let _ = stdout.flush();
    }
}

#[ctor::dtor]
fn restore_cursor() {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "{}", cursor::Show);
    let _ = stdout.flush();
}

fn write(
    mut out: impl Write,
    status: &str,
    message: impl Display,
) -> io::Result<()> {
    writeln!(out, "{}{}{:>width$}{}{} {}",
        LightGreen.fg_str(), Bold, status, color::Reset.fg_str(), style::Reset, message, width=12)
}

/// Show message on screen
pub fn message(status: &str, message: impl Display) -> io::Result<()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    write(stdout, status, message)
}

/// Update last message
pub fn update(status: &str, message: impl Display) -> io::Result<()> {
    let verbose = VERBOSE.load(Ordering::Relaxed);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if !verbose {
        write!(stdout, "{}", cursor::Up(1))?;
    }

    write(stdout, status, message)
}
