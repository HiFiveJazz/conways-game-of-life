use std::{thread, time, vec};
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

fn initial_runtime() {
    print!("\x1B[?25l"); // hide cursor
}



fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn draw_block(coords: &[(u16, u16)]) {
	for coord in coords {
		print!("\x1B[{};{}H■", coord.1, coord.0);
	}
}

fn check_neighbors(coords: &[(u16,u16)]) -> std::vec::Vec<(u16,u16)>{
	let mut neighbor_stack: Vec<(u16, u16)> = Vec::new(); // neighbors of the live cells this frame
	let mut next_coordinate_stack: Vec<(u16, u16)> = Vec::new(); // live cells for next frame
	let mut live_neighbor:u8;
	
	for this_coord in coords {
		live_neighbor = 0;
		neighbor_stack.push((this_coord.0 - 1, this_coord.1));
		neighbor_stack.push((this_coord.0 - 1, this_coord.1 + 1));
		neighbor_stack.push((this_coord.0, this_coord.1 + 1));
		neighbor_stack.push((this_coord.0 + 1, this_coord.1 + 1));
		neighbor_stack.push((this_coord.0 + 1, this_coord.1));
		neighbor_stack.push((this_coord.0 + 1, this_coord.1 - 1));
		neighbor_stack.push((this_coord.0, this_coord.1 - 1));
		neighbor_stack.push((this_coord.0 - 1, this_coord.1 - 1));

		for all_coord in coords {
			if this_coord.1  == all_coord.1 - 1 && this_coord.0 == all_coord.0 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 - 1 && this_coord.0 == all_coord.0 + 1 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 && this_coord.0 == all_coord.0 + 1 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 + 1 && this_coord.0 == all_coord.0 + 1 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 + 1 && this_coord.0 == all_coord.0 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 + 1 && this_coord.0 == all_coord.0 - 1 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 && this_coord.0 == all_coord.0 - 1 {
				live_neighbor+=1;
			}
			if this_coord.1  == all_coord.1 - 1 && this_coord.0 == all_coord.0 - 1 {
				live_neighbor+=1;
			}
		}
		// Only Live cells with 2 or 3 neighbors live, rest are not alive
		let cell = (this_coord.0, this_coord.1);

		if (live_neighbor == 2 || live_neighbor == 3)
		    && !next_coordinate_stack.contains(&cell)
		{
		    next_coordinate_stack.push(cell);
		}
	}
	for neighbor_coords in neighbor_stack {
		let mut live_neighbor:u8 = 0;
		let mut already_alive = false;
		for all_coord in coords {
			if neighbor_coords.0 == all_coord.0 && neighbor_coords.1 == all_coord.1 {
				already_alive = true;
				break;
			}
			if neighbor_coords.1  == all_coord.1 - 1 && neighbor_coords.0 == all_coord.0 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 - 1 && neighbor_coords.0 == all_coord.0 + 1 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 && neighbor_coords.0 == all_coord.0 + 1 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 + 1 && neighbor_coords.0 == all_coord.0 + 1 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 + 1 && neighbor_coords.0 == all_coord.0 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 + 1 && neighbor_coords.0 == all_coord.0 - 1 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 && neighbor_coords.0 == all_coord.0 - 1 {
				live_neighbor+=1;
			}
			if neighbor_coords.1  == all_coord.1 - 1 && neighbor_coords.0 == all_coord.0 - 1 {
				live_neighbor+=1;
			}
		}
		// Dead cells with exactly 3 neighbors become live
		if !already_alive && live_neighbor == 3 && !next_coordinate_stack.contains(&neighbor_coords) {
			next_coordinate_stack.push(neighbor_coords);
		}
	}
	next_coordinate_stack
}

fn main() {
    let _original = enable_raw_mode();
    let (terminal_cols, terminal_rows) =
        terminal_size().expect("Could not get terminal size");
    let x = terminal_cols / 2;
    let y = terminal_rows / 2;
    // let mut coordinate_stack: Vec<(u16, u16)> = vec![(x,y), (x+1,y), (x+1,y+1), (x, y+1)];
    // let mut coordinate_stack: Vec<(u16, u16)> = vec![(x-1,y), (x,y), (x+1,y)]; 
//     let mut coordinate_stack: Vec<(u16, u16)> = vec![
//     (x, y),
//     (x+1, y-1),
//     (x+2, y-1),
//     (x+3, y),
//     (x+1, y+1),
//     (x+2, y+1),
// ];

    let mut coordinate_stack: Vec<(u16, u16)> = 
vec![
    (x, y),
    (x+1, y),
    (x+2, y),
    (x-1, y+1),
    (x, y+1),
    (x+1, y+1),
];

    initial_runtime();
    loop {
        clear_screen();
	draw_block(&coordinate_stack);
	coordinate_stack = check_neighbors(&coordinate_stack);
        io::stdout().flush().unwrap();
        thread::sleep(time::Duration::from_millis(64));
    }

    #[allow(unreachable_code)]
    {
        print!("\x1B[?25h"); // show cursor
        disable_raw_mode(_original);
    }
}
