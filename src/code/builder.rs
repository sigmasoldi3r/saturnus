pub struct UnevenIndentationError;
impl std::fmt::Debug for UnevenIndentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Uneven Indentation: Attempting to pop further than 0!")
            .finish()
    }
}

/// # String code builder
///
/// This simple immutable builder accounts for raw strings (So it is agnostic
/// from the targeted output), but retains indentation aspects.
///
/// This means that you have an indentation stack, with it's state retained
/// between calls, without having to store it in your code emitter.
///
/// Each call, consumes the builder and returns an extended version of it.
/// If you want to preserve the state, clone the structure by calling `.clone()`
/// explicitly.
///
/// Example:
/// ```rs
/// let out = Builder::new("  ")
///     .put("hello")
///     .push().line()
///     .put("my")
///     .pop().unwrap().line()
///     .put("world!")
///     .collect()
/// ```
/// Yields:
/// ```
/// hello
///   my
/// world
/// ```
#[derive(Clone)]
pub struct Builder {
    level: u16,
    indent: String,
    buffer: String,
}
impl Builder {
    pub fn new<T>(indent: T) -> Self
    where
        T: Into<String>,
    {
        Builder {
            level: 0,
            indent: indent.into(),
            buffer: Default::default(),
        }
    }
    pub fn collect(self) -> String {
        self.buffer
    }
    pub fn push(self) -> Self {
        Builder {
            level: self.level + 1,
            ..self
        }
    }
    pub fn pop(self) -> Result<Self, UnevenIndentationError> {
        if self.level == 0 {
            Err(UnevenIndentationError)
        } else {
            Ok(Builder {
                level: self.level - 1,
                ..self
            })
        }
    }
    pub fn put<T>(self, fragment: T) -> Self
    where
        T: Into<String>,
    {
        Builder {
            buffer: format!("{}{}", self.buffer, fragment.into()),
            ..self
        }
    }
    pub fn _and(self, other: Self) -> Self {
        Builder {
            buffer: format!("{}{}", self.buffer, other.buffer),
            ..self
        }
    }
    pub fn get_level(&self) -> u16 {
        self.level
    }
    pub fn get_spaces(&self) -> String {
        self.get_indent().repeat(self.get_level().into())
    }
    pub fn get_indent(&self) -> String {
        self.indent.clone()
    }
    pub fn line(self) -> Self {
        Builder {
            buffer: format!("{}\n{}", self.buffer, self.get_spaces()),
            ..self
        }
    }
    /// Creates a new copy of this builder, but with empty buffer.
    pub fn clone_like(&self) -> Self {
        Builder {
            level: self.level,
            indent: self.indent.clone(),
            buffer: "".to_string(),
        }
    }
}
