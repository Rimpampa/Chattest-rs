use pancurses::*;

pub fn get_string(win: &Window) -> (String, bool) {
    let mut string = String::new();
    let y = win.get_cur_y();
    let mut cursor = 0;
    loop {
        let ch = win.getch();
        if let Some(Input::Character(ch)) = ch {
            println!("Pressed: {}({:x})\nLength: {}", ch, ch as u32, string.len());
            match ch {
                '\n' => break,
                '\u{8}' => {
                    if cursor > 0 {
                        if cursor < string.len() {
                            string.remove(cursor);
                        } else {
                            string.pop();
                        }
                        win.addch('\u{8}');
                        win.delch();
                        cursor -= 1;
                    }
                }
                '\u{1b}' => return (string, true),
                '\u{7f}' => {
                    win.mv(y, win.get_cur_x() - cursor as i32);
                    win.clrtoeol();
                    if let Some(idx) = string[0..cursor].rfind(char::is_whitespace) {
                        let (take, divide) = string.split_at(idx);
                        string = format!("{}{}", take, divide.split_at(cursor - take.len()).1);
                        cursor = idx;
                    } else {
                        use std::iter::FromIterator;
                        string = String::from_iter(
                            string.split_at(cursor).1.chars().skip_while(|c| *c == ' '),
                        );
                        cursor = 0;
                    }
                    win.printw(&string);
                    win.mv(y, win.get_cur_x() - (string.len() - cursor) as i32);
                }
                _ => {
                    if ch.is_ascii() && !ch.is_ascii_control() {
                        // if let Some(max) = max {
                        //     if max == string.len() {
                        //         win.mvprintw(
                        //             y,
                        //             0,
                        //             format!("  Maximum number of characters is {}!", max),
                        //         );
                        //         noecho();
                        //         win.getch();
                        //         echo();
                        //         win.mv(y, 0);
                        //         win.clrtoeol();
                        //         win.printw(format!(" > {}", string));
                        //         continue;
                        //     }
                        // }
                        string.insert(cursor, ch);
                        cursor += 1;
                        win.insch(ch);
                        win.mv(y, win.get_cur_x() + 1);
                    }
                }
            }
        } else if let Some(input) = ch {
            println!("Pressed: {:?}\nLength: {}", input, string.len());
            match input {
                Input::KeyLeft if cursor > 0 => {
                    cursor -= 1;
                    win.mv(y, win.get_cur_x() - 1);
                }
                Input::KeyRight if cursor < string.len() => {
                    cursor += 1;
                    win.mv(y, win.get_cur_x() + 1);
                }
                Input::KeyDC if cursor < string.len() => {
                    string.remove(cursor);
                    win.delch();
                }
                Input::KeyHome => {
                    win.mv(y, win.get_cur_x() - cursor as i32);
                    cursor = 0;
                }
                _ => (),
            }
        }
    }
    (string.trim().to_string(), false)
}

pub fn try_get_string(win: &Window, string: &mut String, cursor: &mut usize) -> Option<Input> {
    let ch = win.getch();
    let y = win.get_cur_y();
    if let Some(Input::Character(ch)) = ch {
        println!("Pressed: {}({:x})\nLength: {}", ch, ch as u32, string.len());
        match ch {
            '\u{8}' => {
                if *cursor > 0 {
                    if *cursor < string.len() {
                        string.remove(*cursor);
                    } else {
                        string.pop();
                    }
                    win.addch('\u{8}');
                    win.delch();
                    *cursor -= 1;
                }
            }
            '\u{7f}' => {
                win.mv(y, win.get_cur_x() - *cursor as i32);
                win.clrtoeol();
                if let Some(idx) = string[0..*cursor].rfind(char::is_whitespace) {
                    let (take, divide) = string.split_at(idx);
                    *string = format!("{}{}", take, divide.split_at(*cursor - take.len()).1);
                    *cursor = idx;
                } else {
                    use std::iter::FromIterator;
                    *string = String::from_iter(
                        string.split_at(*cursor).1.chars().skip_while(|c| *c == ' '),
                    );
                    *cursor = 0;
                }
                win.printw(&string);
                win.mv(y, win.get_cur_x() - (string.len() - *cursor) as i32);
            }
            _ => {
                if ch.is_ascii() && !ch.is_ascii_control() {
                    string.insert(*cursor, ch);
                    *cursor += 1;
                    win.insch(ch);
                    win.mv(y, win.get_cur_x() + 1);
                }
            }
        }
    } else if let Some(input) = ch {
        println!("Pressed: {:?}\nLength: {}", input, string.len());
        match input {
            Input::KeyLeft if *cursor > 0 => {
                *cursor -= 1;
                win.mv(y, win.get_cur_x() - 1);
            }
            Input::KeyRight if *cursor < string.len() => {
                *cursor += 1;
                win.mv(y, win.get_cur_x() + 1);
            }
            Input::KeyDC if *cursor < string.len() => {
                string.remove(*cursor);
                win.delch();
            }
            Input::KeyHome => {
                win.mv(y, win.get_cur_x() - *cursor as i32);
                *cursor = 0;
            }
            _ => (),
        }
    }
    ch
}

/*
#[derive(Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl From<(usize, usize)> for Position {
    fn from(val: (usize, usize)) -> Position {
        Position { line: val.0, column: val.1 }
    }
}

pub struct TextBox<'a> {
    window: &'a Window,
    start: Position,
    lines: usize,
    columns: usize,

    string: String,
    cursor: Position,
    vcursor: usize,
}

impl<'a> TextBox<'a> {
    pub fn new(window: &'a Window, start_line: usize, lines: usize, start_column: usize, columns: usize) -> Self {
        TextBox {
            window,
            start: (start_line, start_column).into(),
            lines,
            columns,
            string: String::with_capacity(columns * lines),
            cursor: (start_line, start_column).into(),
            vcursor: 0,
        }
    }

    pub fn clear(&self) {
        for i in 0..self.lines {
            self.window.mvprintw(
                (self.start.line + i) as i32,
                self.start.column as i32,
                std::iter::repeat(' ').take(self.columns).collect::<String>()
            );
        }
    }

    pub fn getch(&mut self) -> Option<Input> {
        let input = self.window.getch();
        if let Some(input) = input {
            match input {
                Input::Character(ch) if ch.is_ascii() => match ch {
                    '\n' => {
                        self.string.insert(self.vcursor, '\n');
                        self.vcursor += 1;
                        if self.cursor.line != self.start.line + self.lines - 1 {
                            self.cursor.line += 1;
                            self.cursor.column = 0;
                        }
                    }
                    _ => {

                    }
                }
                Input::KeyUp => {
                }
                Input::KeyDown => {
                }
                Input::KeyLeft => {
                }
                Input::KeyRight => {
                }
                _ => (),
            }
            println!("Pressed: {:?}", input);
        }
        self.clear();
        input
    }

    pub fn take(&mut self) -> String {
        let string = self.string.clone();
        self.string.clear();
        self.cursor = self.start;
        string
    }
}
 */
