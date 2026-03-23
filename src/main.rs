use std::{thread, time};
use std::mem;
use std::os::unix::io::AsRawFd;
use std::io::{self, Read, Write};
use std::sync::mpsc::{channel, Receiver};
// use fastrand;

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
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
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

    // Disable canonical mode & echo
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

fn draw_tail(stack: &[(u16,u16)]) {
    for &(tail_x, tail_y) in stack {
        draw_block(tail_x, tail_y);
    }
}


fn apple_coordinates(terminal_cols: u16, terminal_rows: u16) -> (u16, u16) {
    let x = fastrand::u16(1..terminal_cols);
    let y = fastrand::u16(1..terminal_rows);
    (x, y)
    // TODO: Include logic to ensure apple cannot spawn on top of snake
}




fn get_input() -> Receiver<char> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        for byte in stdin.bytes() {
            if let Ok(b) = byte {
                let c = b as char;
                tx.send(c).unwrap();
            }
        }
    });
    rx
}

fn draw_apple(x: u16, y: u16) {
    print!("\x1B[{};{}H\x1B[31m■\x1B[0m", y, x);
}

fn main() {
    let mut stack: Vec<(u16, u16)> = Vec::new();
    let mut apple_count:u16 = 0;
    enable_raw_mode();
    // let original = enable_raw_mode();
    let (terminal_cols, terminal_rows) = terminal_size().expect("Could not get terminal size");
    let (mut apple_x, mut apple_y) = apple_coordinates(terminal_cols, terminal_rows);
    // let (mut apple_y, mut apple_x) = apple_coordinates(&terminal_cols, &terminal_rows);
    // let (mut apple_x, mut apple_y) = (50, 20);
    let mut trailing_direction = fastrand::u16(0..=3); // Up = 0, Right = 1, Down = 2, Left = 3
    //TODO: If on the left side of the terminal make it right? just ensure that it doesn't allow for auto losing, maybe spawn snake 
    // in the middle if continuing to use this rand function for cardinal directions 
    let input = get_input();
    let mut x = terminal_cols / 4;
    let mut y = terminal_rows / 2;
    loop {
        clear_screen();
        if x == apple_x && y == apple_y { // If we eat the apple
            (apple_x, apple_y) = apple_coordinates(terminal_cols, terminal_rows); // Regenerate the apple
            apple_count += 1;
            
        } 
        draw_apple(apple_x, apple_y);
        draw_block(x, y);
        draw_tail(&stack);
        io::stdout().flush().unwrap();
        print!("\x1B[?25l"); // hide cursor
        if let Ok(key) = input.try_recv() {
            match key {
                // Up
                'w'|'k' => {
                    if trailing_direction != 2 {
                        trailing_direction = 0;
                    }
                },
                // Right
                'd'|'l' => {
                    if trailing_direction != 3 {
                        trailing_direction = 1;
                    }
                },
                // Down
                's'|'j' => {
                    if trailing_direction != 0 {
                        trailing_direction = 2;
                    }
                },
                // Left
                'a'|'h' => {
                    if trailing_direction != 1 {
                        trailing_direction = 3; 
                    }
                },
                'q' => break,
                _ => {}
            }
        }

        stack.push((x, y));
        
        if stack.len() > apple_count as usize {
            stack.remove(0);
        }
        
        match trailing_direction {
            // Up
            0 => {
               y -= 1; 
               thread::sleep(time::Duration::from_millis(75));
            }
            //Right
            1 => {
                x +=1;
                thread::sleep(time::Duration::from_millis(30));
            }
            //Down
            2 => {
                y +=1;
                thread::sleep(time::Duration::from_millis(75));
            }
            //Left
            3 => {
                x -=1;
                thread::sleep(time::Duration::from_millis(30));
            }
            _ => {
                panic!()
            }
        }
        if y == terminal_rows {
            y = 1
            // break
        }
        if x  == terminal_cols {
            x = 1
        }
        if y == 0 {
            y = terminal_rows 
            // break
        }
        if x  == 0 {
            x = terminal_cols 
            // break
        }
    }
    clear_screen();
    println!("Game Over!, Score: {}", apple_count);
    print!("\x1B[?25h");
}
