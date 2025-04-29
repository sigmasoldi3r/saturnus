use std::fmt::Display;

#[derive(Clone)]
pub struct IndentedBuilder {
    level: usize,
    tab_char: String,
    buffer: String,
}

impl IndentedBuilder {
    pub fn new() -> Self {
        Self {
            level: 0,
            tab_char: "  ".into(),
            buffer: String::new(),
        }
    }
    pub fn push(&mut self) -> &'_ mut Self {
        self.level += 1;
        self
    }
    pub fn pop(&mut self) -> &'_ mut Self {
        if self.level == 0 {
            panic!("Uneven indentation! Trying to pop past 0 indentation level.");
        }
        self.level -= 1;
        self
    }
    pub fn write(&mut self, piece: impl Display) -> &'_ mut Self {
        self.buffer = format!("{}{}", self.buffer, piece);
        self
    }
    pub fn line(&mut self) -> &'_ mut Self {
        self.buffer = format!("{}\n{}", self.buffer, self.tab_char.repeat(self.level));
        self
    }
    pub fn unwrap(self) -> String {
        self.buffer
    }
}
