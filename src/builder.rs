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
    pub fn create_test(
        buffer: &'a mut [u8],
        at_prefix: bool,
    ) -> CommandBuilder<'a, Initialized<Test>> {
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
    pub fn create_query(
        buffer: &'a mut [u8],
        at_prefix: bool,
    ) -> CommandBuilder<'a, Initialized<Query>> {
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
    pub fn create_set(
        buffer: &'a mut [u8],
        at_prefix: bool,
    ) -> CommandBuilder<'a, Initialized<Set>> {
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
    pub fn create_execute(
        buffer: &'a mut [u8],
        at_prefix: bool,
    ) -> CommandBuilder<'a, Initialized<Execute>> {
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
impl<ANY> CommandBuilder<'_, ANY> {
    /// Tries to append data to the buffer.
    ///
    /// If it won't fit, it silently fails and won't copy the data.
    /// The index field is incremented no matter what.
    fn try_append_data(&mut self, data: &[u8]) {
        let data_length = data.len();

        // Why not just use copy_from_slice?
        // That can give a panic and thus dumps a lot of fmt code in the binary.
        // The compiler can check every aspect of this and so the code will never panic.

        // Does the buffer have enough space left?
        if let Some(buffer_slice) = self.buffer.get_mut(self.index..(self.index + data_length)) {
            // Yes, zip the buffer with the data
            for (buffer, data) in buffer_slice.iter_mut().zip(data) {
                // Copy over the bytes.
                *buffer = *data;
            }
        }

        // Increment the index
        self.index += data_length;
    }
}

impl<'a, N: Nameable> CommandBuilder<'a, Initialized<N>> {
    /// Set the name of the command.
    pub fn named<T: AsRef<[u8]>>(mut self, name: T) -> CommandBuilder<'a, N> {
        self.try_append_data(name.as_ref());
        self.try_append_data(N::NAME_SUFFIX);

        CommandBuilder::<'a, N> {
            buffer: self.buffer,
            index: self.index,
            phantom: Default::default(),
        }
    }
}

impl CommandBuilder<'_, Set> {
    /// Add an integer parameter.
    pub fn with_int_parameter<INT: Into<i32>>(mut self, value: INT) -> Self {
        let mut formatting_buffer = [0; crate::formatter::MAX_INT_DIGITS];
        self.try_append_data(crate::formatter::write_int(
            &mut formatting_buffer,
            value.into(),
        ));
        self.try_append_data(b",");
        self
    }

    /// Add a string parameter
    pub fn with_string_parameter<T: AsRef<[u8]>>(mut self, value: T) -> Self {
        self.try_append_data(b"\"");
        self.try_append_data(value.as_ref());
        self.try_append_data(b"\"");
        self.try_append_data(b",");
        self
    }

    /// Add an optional integer parameter.
    pub fn with_optional_int_parameter<INT: Into<i32>>(self, value: Option<INT>) -> Self {
        match value {
            None => self.with_empty_parameter(),
            Some(value) => self.with_int_parameter(value),
        }
    }

    /// Add an optional string parameter.
    pub fn with_optional_string_parameter<T: AsRef<[u8]>>(self, value: Option<T>) -> Self {
        match value {
            None => self.with_empty_parameter(),
            Some(value) => self.with_string_parameter(value),
        }
    }

    /// Add a comma, representing an unset optional parameter.
    pub fn with_empty_parameter(mut self) -> Self {
        self.try_append_data(b",");
        self
    }

    /// Add an unformatted parameter
    pub fn with_raw_parameter<T: AsRef<[u8]>>(mut self, value: T) -> Self {
        self.try_append_data(value.as_ref());
        self.try_append_data(b",");
        self
    }

    /// Add the given value to the array using the hex format for the bytes
    pub fn with_rax_hex_parameter<T: AsRef<[u8]>>(mut self, value: T) -> Self {
        for byte in value.as_ref().iter() {
            use crate::formatter::parse_byte_to_hex;

            let hex_value = parse_byte_to_hex(*byte);
            self.try_append_data(&hex_value);
        }
        self.try_append_data(b",");
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
    pub fn finish(self) -> Result<&'a [u8], usize> {
        self.finish_with(b"\r\n")
    }

    /// Finishes the builder.
    ///
    /// With the terminator variable, you can decide how to end the command.
    /// Normally this is `\r\n`.
    ///
    /// ```rust
    /// use at_commands::builder::CommandBuilder;
    ///
    /// let mut buffer = [0; 128];
    ///
    /// // Make a query command
    /// let result = CommandBuilder::create_query(&mut buffer, true)
    ///     .named("+MYQUERY")
    ///     .finish_with(b"\0")
    ///     .unwrap();
    /// ```
    ///
    /// When Ok, it returns a slice with the built command.
    /// The slice points to the same memory as the buffer,
    /// but is only as long as is required to contain the command.
    ///
    /// The command length is thus the length of the slice.
    ///
    /// If the buffer was not long enough,
    /// then an Err is returned with the size that was required for it to succeed.
    pub fn finish_with(mut self, terminator: &[u8]) -> Result<&'a [u8], usize> {
        // if last byte is a comma, decrement index to drop it
        if let Some(c) = self.buffer.get(self.index - 1) {
            if *c == b',' {
                self.index -= 1;
            }
        }
        self.try_append_data(terminator);

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

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+TEST=?\r\n");
    }

    #[test]
    fn test_query() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_query(&mut buffer, true)
            .named("+QUERY")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+QUERY?\r\n");
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
            "AT+SET=12345,\"my_string_param\",67,89\r\n"
        );
    }

    #[test]
    fn test_execute() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_execute(&mut buffer, true)
            .named("+EXECUTE")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+EXECUTE\r\n");
    }

    #[test]
    fn test_buffer_too_short() {
        let mut buffer = [0; 5];
        assert!(CommandBuilder::create_execute(&mut buffer, true)
            .named("+BUFFERLENGTH")
            .finish()
            .is_err());
        assert!(CommandBuilder::create_execute(&mut buffer, true)
            .named("+A")
            .finish()
            .is_err()); // too short by only one byte
    }

    #[test]
    fn test_buffer_exact_size() {
        let mut buffer = [0; 32];
        let value = CommandBuilder::create_execute(&mut buffer[..8], true)
            .named("+GMR")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+GMR\r\n");

        let value = CommandBuilder::create_set(&mut buffer[..19], true)
            .named("+CWRECONNCFG")
            .with_int_parameter(15)
            .finish()
            .unwrap();

        assert_eq!(
            core::str::from_utf8(value).unwrap(),
            "AT+CWRECONNCFG=15\r\n"
        );

        let value = CommandBuilder::create_query(&mut buffer[..14], true)
            .named("+UART_CUR")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+UART_CUR?\r\n");
    }

    #[test]
    fn test_terminator() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_test(&mut buffer, true)
            .named("+TEST")
            .finish_with(b"\0")
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+TEST=?\0");
    }

    #[test]
    fn test_optional() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_set(&mut buffer, true)
            .named("+CCUG")
            .with_empty_parameter()
            .with_optional_int_parameter(Some(9))
            .finish_with(b"\r")
            .unwrap();
        // see https://www.multitech.com/documents/publications/manuals/s000453c.pdf
        // pages 8 and 85 for command and 150 for CR ending
        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+CCUG=,9\r");

        let value = CommandBuilder::create_set(&mut buffer, true)
            .named("+BLEGATTSSETATTR")
            .with_int_parameter(1)
            .with_int_parameter(1)
            .with_empty_parameter()
            .with_int_parameter(4)
            .finish()
            .unwrap();
        // https://docs.espressif.com/projects/esp-at/en/latest/AT_Command_Set/BLE_AT_Commands.html#cmd-GSSETA
        assert_eq!(
            core::str::from_utf8(value).unwrap(),
            "AT+BLEGATTSSETATTR=1,1,,4\r\n"
        );

        let value = CommandBuilder::create_set(&mut buffer, true)
            .named("+HTTPCLIENT")
            .with_int_parameter(2)
            .with_int_parameter(1)
            .with_optional_string_parameter(Some("http://localpc/ip"))
            .with_empty_parameter()
            .with_empty_parameter()
            .with_int_parameter(1)
            .finish()
            .unwrap();

        assert_eq!(
            core::str::from_utf8(value).unwrap(),
            "AT+HTTPCLIENT=2,1,\"http://localpc/ip\",,,1\r\n"
        );
    }

    #[test]
    fn test_raw_parameter() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_set(&mut buffer, true)
            .named("+CPIN")
            .with_raw_parameter(b"1234")
            .with_optional_int_parameter(Some(9))
            .finish_with(b"\r")
            .unwrap();
        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+CPIN=1234,9\r");
    }

    #[test]
    fn test_hex_parameter() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_set(&mut buffer, true)
            .named("+CPIN")
            .with_rax_hex_parameter(&[0xFF, 0x2F, 0x00])
            .finish()
            .unwrap();

        assert_eq!(value, b"AT+CPIN=ff2f00\r\n");
    }
}
