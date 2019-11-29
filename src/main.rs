// #![windows_subsystem = "windows"]

use pancurses::*;

mod utilities;
use utilities::*;

mod chattest;
mod client;
mod server;

const TITLE: &str = "    ___ _           _   _            _   
   / __\\ |__   __ _| |_| |_ ___  ___| |_ 
  / /  | '_ \\ / _` | __| __/ _ \\/ __| __|
 / /___| | | | (_| | |_| ||  __/\\__ \\ |_ 
 \\____/|_| |_|\\__,_|\\__|\\__\\___||___/\\__|
                              by Rimpampa";

const TITLE_HEIGTH: i32 = 5;

const WIDTH: i32 = 42;
const HEIGHT: i32 = 20;
const LAST: i32 = HEIGHT - TITLE_HEIGTH - 4;

fn main() {
    let window = initscr();
    resize_term(20, 42);
    set_title("Chattest");
    window.printw(TITLE);
    noecho();
    window.refresh();
    let window = window
        .subwin(HEIGHT - TITLE_HEIGTH - 2, WIDTH, TITLE_HEIGTH + 2, 0)
        .unwrap();
    window.keypad(true);

    let mut name;
    while {
        window.printw("  What's your name?\n > ");
        // .subwin(1, 30, window.get_cur_y(), window.get_cur_x()).unwrap()
        match get_string(&window) {
            (n, false) => name = n,
            (_, true) => return,
        }
        window.mv(0, 0);
        window.clrtobot();
        name.is_empty()
    } {}

    window.printw(format!(
        "  Hi {}!\n  Use Up and Down to move the cursor\n  Press Enter to confirm the selection\n\n",
        name
    ));

    let mut selected = 0;
    loop {
        window.mv(4, 0);
        window.clrtobot();
        match selected {
            0 => {
                window.printw("           > CREATE ROOM < \n");
                window.printw("              JOIN ROOM    \n");
                window.printw("                 EXIT      \n");
            }
            1 => {
                window.printw("             CREATE ROOM   \n");
                window.printw("            > JOIN ROOM <  \n");
                window.printw("                 EXIT      \n");
            }
            2 => {
                window.printw("             CREATE ROOM   \n");
                window.printw("              JOIN ROOM    \n");
                window.printw("               > EXIT <    \n");
            }
            _ => unreachable!(),
        }
        let ch = window.getch();
        if let Some(ch) = ch {
            match ch {
                Input::KeyEnter | Input::Character('\n') => {
                    window.mv(0, 0);
                    window.clrtobot();
                    match selected {
                        0 => {
                            if server::chat(&window, name.clone()) {
                                break;
                            }
                        }
                        1 => {
                            if client::chat(&window, &mut name) {
                                break;
                            }
                        }
                        2 => break,
                        _ => unreachable!(),
                    }
                    window.mv(0, 0);
                    window.clrtobot();
                    window.printw(
                        format!(
                            "  Hi {}!\n  Use Up and Down to move the cursor\n  Press Enter to confirm the selection\n",
                            name
                        ),
                    );
                }
                Input::KeyUp => {
                    if selected == 0 {
                        selected = 2;
                    } else {
                        selected -= 1;
                    }
                }
                Input::KeyDown => {
                    if selected == 2 {
                        selected = 0;
                    } else {
                        selected += 1;
                    }
                }
                _ => (),
            }
        }
    }
}
