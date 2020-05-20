//! Implementation of the CommandBuilder

/// # CommandBuilder
/// A builder struct for AT Commands
///
/// ## Summary
/// This can be used to build:
/// * A test command in the form `AT{name}=?`
/// * A query command in the form `AT{name}?`
/// * A set command in the form `AT{name}={param},{param},{param}`
/// * An execute command in the form `AT{name}`
///
/// ## Example
/// ```rust
/// use at_commands::builder::CommandBuilder;
///
/// let mut buffer = [0; 128];
///
/// // Make a query command
/// let result = CommandBuilder::create_query(&mut buffer, true)
///     .named("+MYQUERY")
///     .finish()
///     .unwrap();
///
/// // Buffer now contains "AT+MYQUERY?"
/// // Copy or DMA the resulting slice to the device.
///
/// // Make a set command
/// let result = CommandBuilder::create_set(&mut buffer, false)
///     .named("+MYSET")
///     .with_int_parameter(42)
///     .finish()
///     .unwrap();
///
/// // Buffer now contains "+MYSET=42"
/// // Copy or DMA the resulting slice to the device.
/// ```
pub struct CommandBuilder<'a, STAGE> {
    buffer: &'a mut [u8],
    index: usize,
    phantom: core::marker::PhantomData<STAGE>,
}

impl<'a> CommandBuilder<'a, Uninitialized> {
    /// Creates a builder for a test command.
    ///
    /// The given buffer is used to build the command in and must be big enough to contain it.
    pub fn create_test(buffer: &'a mut [u8], at_prefix: bool) -> CommandBuilder<'a, Initialized<Test>> {
        let mut builder = CommandBuilder::<'a, Initialized<Test>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        if at_prefix {
            builder.try_append_data(b"AT");
        }

        builder
    }

    /// Creates a builder for a query command.
    ///
    /// The given buffer is used to build the command in and must be big enough to contain it.
    pub fn create_query(buffer: &'a mut [u8], at_prefix: bool) -> CommandBuilder<'a, Initialized<Query>> {
        let mut builder = CommandBuilder::<'a, Initialized<Query>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        if at_prefix {
            builder.try_append_data(b"AT");
        }

        builder
    }

    /// Creates a builder for a set command.
    ///
    /// The given buffer is used to build the command in and must be big enough to contain it.
    pub fn create_set(buffer: &'a mut [u8], at_prefix: bool) -> CommandBuilder<'a, Initialized<Set>> {
        let mut builder = CommandBuilder::<'a, Initialized<Set>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        if at_prefix {
            builder.try_append_data(b"AT");
        }

        builder
    }

    /// Creates a builder for an test execute.
    ///
    /// The given buffer is used to build the command in and must be big enough to contain it.
    pub fn create_execute(buffer: &'a mut [u8], at_prefix: bool) -> CommandBuilder<'a, Initialized<Execute>> {
        let mut builder = CommandBuilder::<'a, Initialized<Execute>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        if at_prefix {
            builder.try_append_data(b"AT");
        }

        builder
    }
}
impl<'a, ANY> CommandBuilder<'a, ANY> {
    /// Tries to append data to the buffer.
    ///
    /// If it won't fit, it silently fails and won't copy the data.
    /// The index field is incremented no matter what.
    fn try_append_data(&mut self, data: &[u8]) {
        let data_length = data.len();
        let data_left = self.buffer.len().checked_sub(self.index);

        // Does the slice fit in the buffer?
        if let Some(data_left) = data_left {
            if data_left >= data_length {
                // Yes, so let's copy it.
                self.buffer[self.index..(self.index + data_length)].copy_from_slice(data);
            }
        }

        // Increment the index
        self.index += data_length;
    }
}

impl<'a, N: Nameable> CommandBuilder<'a, Initialized<N>> {
    /// Set the name of the command.
    pub fn named(mut self, name: &str) -> CommandBuilder<'a, N> {
        self.try_append_data(name.as_bytes());
        self.try_append_data(N::NAME_SUFFIX);

        CommandBuilder::<'a, N> {
            buffer: self.buffer,
            index: self.index,
            phantom: Default::default(),
        }
    }
}

impl<'a> CommandBuilder<'a, Set> {
    /// Add an integer parameter.
    pub fn with_int_parameter(mut self, value: i32) -> Self {
        use core::fmt::Write;
        use arrayvec::ArrayString;

        if !matches!(self.buffer.get(self.index - 1), Some(b'=')) {
            self.try_append_data(b",");
        }

        let mut conversion_buffer = ArrayString::<[u8; 12]>::new();
        write!(&mut conversion_buffer, "{}", value).expect("Bad Conversion");
        self.try_append_data(conversion_buffer.as_bytes());
        self
    }

    /// Add a string parameter
    pub fn with_string_parameter(mut self, value: &str) -> Self {
        if !matches!(self.buffer.get(self.index - 1), Some(b'=')) {
            self.try_append_data(b",");
        }

        self.try_append_data(b"\"");
        self.try_append_data(value.as_bytes());
        self.try_append_data(b"\"");
        self
    }
}

impl<'a, F: Finishable> CommandBuilder<'a, F> {
    /// Finishes the builder.
    ///
    /// When Ok, it returns a slice with the built command.
    /// The slice points to the same memory as the buffer,
    /// but is only as long as is required to contain the command.
    ///
    /// The command length is thus the length of the slice.
    ///
    /// If the buffer was not long enough,
    /// then an Err is returned with the size that was required for it to succeed.
    pub fn finish(mut self) -> Result<&'a [u8], usize> {
        self.try_append_data(b"\n");

        if self.index > self.buffer.len() {
            Err(self.index)
        } else {
            Ok(&self.buffer[0..self.index])
        }
    }
}

/// Marker struct for uninitialized builders.
pub struct Uninitialized;
/// Marker struct for initialized builders.
/// The T type is the type the builder will be marked after it has been named.
pub struct Initialized<T>(core::marker::PhantomData<T>);

/// Marker struct for builders that produce a test command.
pub struct Test;
/// Marker struct for builders that produce a query command.
pub struct Query;
/// Marker struct for builders that produce a set command.
pub struct Set;
/// Marker struct for builders that produce a execute command.
pub struct Execute;

/// A trait that can be implemented for marker structs to indicate that the command is ready to be finished.
pub trait Finishable {}
impl Finishable for Test {}
impl Finishable for Query {}
impl Finishable for Set {}
impl Finishable for Execute {}

/// A trait that can be implemented for marker structs to indicate that the command is ready to be named.
pub trait Nameable {
    /// The data that must be put after a name to comply with the type of command that is named.
    const NAME_SUFFIX: &'static [u8];
}
impl Nameable for Test {
    const NAME_SUFFIX: &'static [u8] = b"=?";
}
impl Nameable for Query {
    const NAME_SUFFIX: &'static [u8] = b"?";
}
impl Nameable for Set {
    const NAME_SUFFIX: &'static [u8] = b"=";
}
impl Nameable for Execute {
    const NAME_SUFFIX: &'static [u8] = b"";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_test(&mut buffer, true)
            .named("+TEST")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+TEST=?\n");
    }

    #[test]
    fn test_query() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_query(&mut buffer, true)
            .named("+QUERY")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+QUERY?\n");
    }

    #[test]
    fn test_set() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_set(&mut buffer, true)
            .named("+SET")
            .with_int_parameter(12345)
            .with_string_parameter("my_string_param")
            .with_int_parameter(67)
            .with_int_parameter(89)
            .finish()
            .unwrap();

        assert_eq!(
            core::str::from_utf8(value).unwrap(),
            "AT+SET=12345,\"my_string_param\",67,89\n"
        );
    }

    #[test]
    fn test_execute() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_execute(&mut buffer, true)
            .named("+EXECUTE")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+EXECUTE\n");
    }

    #[test]
    fn test_buffer_too_short() {
        let mut buffer = [0; 5];
        assert!(CommandBuilder::create_execute(&mut buffer, true)
            .named("+BUFFERLENGTH")
            .finish()
            .is_err());
    }
}
