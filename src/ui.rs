use crate::ioctl;
use std::io::{stdout, stdin, Stdout, Stdin, Write, Read};

pub const CURSOR_INVISIBLE: &'static str = "?25l";
pub const CURSOR_VISIBLE: &'static str = "?25h";
pub const CURSOR_SAVE_POSITION: &'static str = "s";
pub const CURSOR_RESTORE_POSITION: &'static str = "u";
pub const CURSOR_HOME: &'static str = "H";
pub const CURSOR_UP: &'static str = "1A";
pub const LINE_CLEAR: &'static str = "2K";
pub const SCREEN_CLEAR: &'static str = "2J";

const UP_ARROW: &'static str = "\x1B[38;2;226;44;44m\u{25B2}\x1B[0m";
const DOWN_ARROW: &'static str = "\x1B[38;2;226;44;44m\u{25BC}\x1B[0m";

pub struct ScrollableList {
    stdin: Stdin,
    stdout: Stdout,
    height: usize,
    selected: usize,
    arrow_offset: usize,
    items: Vec<String>,
    frame: std::ops::Range<usize>,
}

#[derive(Debug)]
pub enum Error {
    InvalidHeight,
    EmptyItems,
    SelectedOutsideRange,
    CmdError
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidHeight => write!(f, "Height can't be less than 1, greater than length of items, nor greater than the window height."),
            Error::EmptyItems => write!(f, "Items can't be empty."),
            Error::SelectedOutsideRange => write!(f, "Selected index is outside the index range of items."),
            Error::CmdError => write!(f, "There was an error executing command on selection."),
        }
    }
}

impl ScrollableList {
    pub fn new(items: Vec<String>, selected: usize, mut height: usize) -> Result<Self, Error> {
        let winheight = ioctl::get_winsize().unwrap().ws_row;

        if height > winheight as usize {
            height = winheight as usize
        }

        if height < 1 || height > items.len() {
            return Err(Error::InvalidHeight)
        }

        if items.len() == 0 {
            return Err(Error::EmptyItems)
        }

        if selected >= items.len() {
            return Err(Error::SelectedOutsideRange)
        }

        let mut arrow_offset = 0;
        for i in items.iter() {
            if i.len() > arrow_offset {
                arrow_offset = i.len()
            }
        }

        let stdin = stdin();
        let stdout = stdout();

        let (head, tail) = if selected + height > items.len() - 1 {
            (items.len() - height, items.len()) 
        } else {
            (selected, selected + height)
        };

        let frame = head..tail;
        let mut list = ScrollableList {
            stdin, stdout, height, selected, arrow_offset, items, frame
        };

        list.set_frame();

        Ok(list)
    }

    pub fn event_loop<F>(&mut self, mut func: F) -> Result<(), Error> where
        F: FnMut(&str) -> Result<(), Error>
    {
        let mut bytes_buffer: [u8; 3] = [0; 3];

        self.buffer_window();
        self.render();

        loop {
            if let Ok(_) = self.stdin.read(&mut bytes_buffer) {
                match bytes_buffer[0] {
                    10 => self.act_on_selection(&mut func)?,
                    27 => match bytes_buffer[2] {
                        65 => self.decrement_selected(),
                        66 => self.increment_selected(),
                        _ => ()
                    },
                    106 => self.increment_selected(),
                    107 => self.decrement_selected(),
                    _ => ()
                }
            };

            self.render();

            bytes_buffer = [0; 3]
        }
    }

    fn act_on_selection<F>(&self, func: &mut F) -> Result<(), Error> where
        F: FnMut(&str) -> Result<(), Error>
    {
        match func(&self.items[self.selected]) {
            Err(e) => Err(e),
            _ => Ok(())
        }
    }

    fn render(&mut self) {
        //self.clear_lines();

        let fmt_selected = format!("\x1B[36m{}\x1B[0m", &self.items[self.selected]);
        let first_item_in_frame = self.fmt_first_item_in_frame();
        let last_item_in_frame = self.fmt_last_item_in_frame();

        let mut items = self.items.clone();
        items[self.selected] = fmt_selected;
        items[self.frame.start] = first_item_in_frame;
        items[self.frame.end - 1] = last_item_in_frame;

        ansi_execute(CURSOR_SAVE_POSITION, &mut self.stdout);

        for i in &items[self.frame.start..self.frame.end] {
            ansi_execute(LINE_CLEAR, &mut self.stdout);
            println!("{}", i);
        }    
        ansi_execute(CURSOR_RESTORE_POSITION, &mut self.stdout);
    }

    fn set_frame(&mut self) {
        if self.height == self.items.len() {
            self.frame = 0..self.items.len();
            return
        }

        if self.frame.start <= self.selected && self.selected <= self.frame.end - 1 {
            return
        }

        if self.selected > self.frame.end - 1 {
            self.frame = (self.frame.start + 1)..(self.frame.end + 1);
            return
        }

        let head = if self.selected + self.height - 1 < self.items.len() - 1 {
            self.selected
        } else {
            self.items.len() - self.height - 2
        };

        let tail = head + self.height - 1;

        self.frame = head..tail + 1;
    }

    fn buffer_window(&mut self) {
        for _ in 0..self.height {
            println!();
        }

        for _ in 0..self.height {
            ansi_execute(CURSOR_UP, &mut self.stdout);
        }
    }

    fn increment_selected(&mut self) {
        if self.selected + 1 >= self.items.len() {
            return 
        }

        self.selected += 1;
        self.set_frame();
    }

    fn decrement_selected(&mut self) {
        let selected = self.selected as i64;
        if selected - 1 < 0 {
            return
        }

        self.selected -= 1;
        self.set_frame();
    }

    fn fmt_first_item_in_frame(&self) -> String {
        let mut first = self.items[self.frame.start].clone();
        let offset = self.arrow_offset - first.len() + 1;

        if self.selected == self.frame.start {
            for _ in 0..offset {
                first.push_str(" ")    
            }
            first = format!("\x1B[36m{}\x1B[0m", first);

            if self.selected == 0 {
                first.push_str(" ")

            } else {
                first.push_str(UP_ARROW)
            }
        } else if self.frame.start > 0 {
            for _ in 0..offset {
                first.push_str(" ")
            }

            first.push_str(UP_ARROW)
        } else {
            for _ in 0..=offset {
                first.push_str(" ")
            }
        }

        first
    }

    fn fmt_last_item_in_frame(&self) -> String {
        let mut last = self.items[self.frame.end - 1].clone();
        let offset = self.arrow_offset - last.len() + 1;

        if self.selected + 1 == self.frame.end {
            for _ in 0..offset {
                last.push_str(" ")
            }
            last = format!("\x1B[36m{}\x1B[0m", last);
            
            if self.selected + 1 < self.items.len() {
                last.push_str(DOWN_ARROW)
            } else {
                last.push_str(" ")
            }
        } else if self.frame.end < self.items.len() {
            for _ in 0..offset {
                last.push_str(" ")
            }

            last.push_str(DOWN_ARROW);
        } else {
            for _ in 0..=offset {
                last.push_str(" ")
            }
        }

        last
    }
}

pub fn ansi_execute(esc_seq: &'static str, stdout: &mut Stdout) {
    print!("\x1B[{}", esc_seq);
    stdout.flush().unwrap();
}

