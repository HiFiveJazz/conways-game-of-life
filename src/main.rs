use std::{thread, time};
use std::mem;
use std::os::unix::io::AsRawFd;
use std::io::{self, Write};

#[repr(C)]
#[derive(Debug)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, ...) -> i32;
}

const TIOCGWINSZ: u64 = 0x5413;

#[repr(C)]
#[derive(Clone, Copy)]
struct Termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_line: u8,
    c_cc: [u8; 32],
    c_ispeed: u32,
    c_ospeed: u32,
}

unsafe extern "C" {
    fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios: *const Termios) -> i32;
}

const TCSANOW: i32 = 0;
const ICANON: u32 = 0x0002;
const ECHO: u32 = 0x0008;

fn enable_raw_mode() -> Termios {
    let fd = std::io::stdin().as_raw_fd();
    let mut termios: Termios = unsafe { std::mem::zeroed() };

    unsafe {
        tcgetattr(fd, &mut termios);
    }

    let original = termios;
    termios.c_lflag &= !(ICANON | ECHO);

    unsafe {
        tcsetattr(fd, TCSANOW, &termios);
    }

    original
}

fn disable_raw_mode(original: Termios) {
    let fd = std::io::stdin().as_raw_fd();
    unsafe {
        tcsetattr(fd, TCSANOW, &original);
    }
}

fn terminal_size() -> Option<(u16, u16)> {
    let mut ws: Winsize = unsafe { mem::zeroed() };
    let fd = io::stdout().as_raw_fd();
    let res = unsafe { ioctl(fd, TIOCGWINSZ, &mut ws) };

    if res == 0 {
        Some((ws.ws_col, ws.ws_row))
    } else {
        None
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn draw_block(x: u16, y: u16) {
    print!("\x1B[{};{}H■", y, x);
}

fn main() {
    let original = enable_raw_mode();
    print!("\x1B[?25l"); // hide cursor

    let (terminal_cols, terminal_rows) =
        terminal_size().expect("Could not get terminal size");

    let x = terminal_cols / 2;
    let y = terminal_rows / 2;

    loop {
        clear_screen();
        draw_block(x, y);
        io::stdout().flush().unwrap();
        thread::sleep(time::Duration::from_millis(50));
    }

    #[allow(unreachable_code)]
    {
        print!("\x1B[?25h"); // show cursor
        disable_raw_mode(original);
    }
}
