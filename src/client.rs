use crate::*;
use std::io::ErrorKind;
use std::net::TcpStream;

pub fn chat(win: &Window, name: &mut std::string::String) -> bool {
    let mut stream;
    while {
        // Get the IP address of the room he wants to connect to
        win.printw("  What's the address of the room?\n  [press ESC to return to menu]\n > ");
        // .subwin(1, 15, win.get_cur_y(), win.get_cur_x()).unwrap()
        let (ip, esc) = get_string(&win);
        // Remove the text but don't update the screen
        win.mv(0, 0);
        win.clrtobot();
        if esc {
            return false;
        }
        // Connect to that address
        stream = TcpStream::connect(ip + ":7357");
        stream.is_err()
    } {
        // Notify the error
        win.printw("  Couldn't connect to the server!");
        noecho();
        win.getch();
        echo();
        // Remove the text
        win.mv(0, 0);
        win.clrtobot();
    }
    // Unwrap the stream because it's safe to do it now
    let mut stream = chattest::BlockingStream::new(stream.unwrap());
    println!("Connected!");

    let mut result;
    while {
        // Send the name of the user to the server
        stream.write(chattest::Code::Name(name.clone())).unwrap();
        result = stream.read().unwrap();
        result == chattest::Code::AlreadyHere
    } {
        win.printw("  There is already someone with your name!\n  Write a new name\n  [press ESC to return to the menu]\n > ");
        let (string, esc) =
            // &win.subwin(1, 30, win.get_cur_y(), win.get_cur_x()).unwrap()
            get_string(&win);
        if esc {
            return false;
        }
        *name = string;
        // Remove the text
        win.mv(0, 0);
        win.clrtobot();
    }
    let admin = match result {
        chattest::Code::Welcome(room, admin) => {
            win.printw("  Connected to room ");
            win.printw(room);
            win.printw("\n The admin is ");
            win.printw(&admin);
            win.refresh();
            admin
        }
        _ => panic!("Server didn't respond correctly"),
    };
    let mut stream = stream.non_blocking();
    win.nodelay(true);

    win.mvprintw(LAST, 0, " > ");

    let mut messages = Vec::new();
    let mut selected = 0;

    let mut string = String::new();
    let mut cursor = 0;
    loop {
        match stream.try_read() {
            Ok(Some(code)) => {
                match code {
                    chattest::Code::MessageFrom(name, message) => {
                        println!("{}> {}", name, message);
                        messages.push(format!("  {}> {}", name, message));
                    }
                    chattest::Code::MessageTo(message) => {
                        println!("{}# {}", admin, message);
                        messages.push(format!("  {}# {}", admin, message));
                    }
                    _ => println!("Strange code: {:?}", code),
                }
                if messages.len() == 1 {
                    win.mvprintw(3, 0, messages.last().unwrap());
                    win.clrtobot();
                    win.mvprintw(LAST, 0, " > ");
                    win.printw(&string);
                    win.mv(LAST, 3 + cursor as i32);
                } else if messages.len() == selected + 2 {
                    selected += 1;
                    win.mvprintw(3, 0, messages.last().unwrap());
                    win.clrtobot();
                    win.mvprintw(LAST, 0, " > ");
                    win.printw(&string);
                    win.mv(LAST, 3 + cursor as i32);
                }
            }
            Ok(None) => (),
            Err(error) => match error.kind() {
                ErrorKind::ConnectionReset => {
                    win.mvprintw(
                        0,
                        0,
                        "  Connection lost!\n  [press any key to return to the menu]\n",
                    );
                    win.clrtobot();
                    win.nodelay(false);
                    win.getch();
                    return false;
                }
                _ => println!("Error: {}", error),
            },
        }
        if let Some(input) = try_get_string(&win, &mut string, &mut cursor) {
            match input {
                Input::Character('\n') if string.len() > 1 => {
                    stream
                        .write(chattest::Code::MessageTo(string.clone()))
                        .unwrap();
                    messages.push(format!("  {}", string));
                    if messages.len() == 1 {
                        win.mvprintw(3, 0, messages.last().unwrap());
                        win.clrtobot();
                        win.mvprintw(LAST, 0, " > ");
                        win.printw(&string);
                        win.mv(LAST, 3 + cursor as i32);
                    } else if messages.len() == selected + 2 {
                        selected += 1;
                        win.mvprintw(3, 0, messages.last().unwrap());
                        win.clrtobot();
                        win.mvprintw(LAST, 0, " > ");
                        win.printw(&string);
                        win.mv(LAST, 3 + cursor as i32);
                    }
                    string.clear();
                    cursor = 0;
                    win.mv(LAST, 3);
                    win.clrtobot();
                }
                Input::KeyUp if selected > 0 => {
                    selected -= 1;
                    win.mvprintw(3, 0, &messages[selected]);
                    win.clrtobot();
                    win.mvprintw(LAST, 0, " > ");
                    win.printw(&string);
                    win.mv(LAST, 3 + cursor as i32);
                }
                Input::KeyDown if selected + 1 < messages.len() => {
                    selected += 1;
                    win.mvprintw(3, 0, &messages[selected]);
                    win.clrtobot();
                    win.mvprintw(LAST, 0, " > ");
                    win.printw(&string);
                    win.mv(LAST, 3 + cursor as i32);
                }
                _ => (),
            }
        }
    }
}
