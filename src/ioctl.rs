use std::os::unix::io::AsRawFd;
use termios::{self, tcsetattr, Termios};

#[derive(Debug)]
pub enum Error {
    WinsizeErr
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WinsizeErr => write!(f, "Failed to get window size.")
        }
    }
}

pub fn unecho_noncanonical_cbreak_min_1() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = std::io::stdout();
    let char_device = stdout.as_raw_fd();
    let mut termios = Termios::from_fd(char_device)?;

    termios.c_lflag ^= termios::ECHO | termios::ICANON;
    termios.c_cc[libc::VMIN] = 1;

    tcsetattr(char_device, libc::TCSANOW, &termios)?;

    Ok(())
}


pub fn get_winsize() -> Result<libc::winsize, Error> {
    let stdout = std::io::stdout();
    let char_device = stdout.as_raw_fd();
    let winsize = libc::winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };

    let status = unsafe { libc::ioctl(char_device, libc::TIOCGWINSZ, &winsize) };

    match status {
        0 => Ok(winsize),
        _ => Err(Error::WinsizeErr)
    }
}

