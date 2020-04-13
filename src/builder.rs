use arrayvec::ArrayString;

pub struct CommandBuilder<'a, STAGE> {
    buffer: &'a mut [u8],
    index: usize,
    phantom: core::marker::PhantomData<STAGE>,
}

impl<'a> CommandBuilder<'a, Uninitialized> {
    pub fn create_test(buffer: &'a mut [u8]) -> CommandBuilder<'a, Initialized<Test>> {
        let mut builder = CommandBuilder::<'a, Initialized<Test>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        builder.try_append_data(b"AT");

        builder
    }

    pub fn create_query(buffer: &'a mut [u8]) -> CommandBuilder<'a, Initialized<Query>> {
        let mut builder = CommandBuilder::<'a, Initialized<Query>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        builder.try_append_data(b"AT");

        builder
    }

    pub fn create_set(buffer: &'a mut [u8]) -> CommandBuilder<'a, Initialized<Set>> {
        let mut builder = CommandBuilder::<'a, Initialized<Set>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        builder.try_append_data(b"AT");

        builder
    }

    pub fn create_execute(buffer: &'a mut [u8]) -> CommandBuilder<'a, Initialized<Execute>> {
        let mut builder = CommandBuilder::<'a, Initialized<Execute>> {
            buffer,
            index: 0,
            phantom: Default::default(),
        };

        builder.try_append_data(b"AT");

        builder
    }
}
impl<'a, ANY> CommandBuilder<'a, ANY> {
    fn try_append_data(&mut self, data: &[u8]) {
        let data_length = data.len();
        let data_left = self.buffer.len().checked_sub(self.index);

        if let Some(data_left) = data_left {
            if data_left >= data_length {
                self.buffer[self.index..(self.index + data_length)].copy_from_slice(data);
            }
        }

        self.index += data_length;
    }
}

impl<'a> CommandBuilder<'a, Initialized<Test>> {
    pub fn named(mut self, name: &str) -> CommandBuilder<'a, Test> {
        self.try_append_data(name.as_bytes());
        self.try_append_data(b"=?");

        CommandBuilder::<'a, Test> {
            buffer: self.buffer,
            index: self.index,
            phantom: Default::default(),
        }
    }
}

impl<'a> CommandBuilder<'a, Initialized<Query>> {
    pub fn named(mut self, name: &str) -> CommandBuilder<'a, Query> {
        self.try_append_data(name.as_bytes());
        self.try_append_data(b"?");

        CommandBuilder::<'a, Query> {
            buffer: self.buffer,
            index: self.index,
            phantom: Default::default(),
        }
    }
}

impl<'a> CommandBuilder<'a, Initialized<Set>> {
    pub fn named(mut self, name: &str) -> CommandBuilder<'a, Set> {
        self.try_append_data(name.as_bytes());
        self.try_append_data(b"=");

        CommandBuilder::<'a, Set> {
            buffer: self.buffer,
            index: self.index,
            phantom: Default::default(),
        }
    }
}

impl<'a> CommandBuilder<'a, Initialized<Execute>> {
    pub fn named(mut self, name: &str) -> CommandBuilder<'a, Execute> {
        self.try_append_data(name.as_bytes());

        CommandBuilder::<'a, Execute> {
            buffer: self.buffer,
            index: self.index,
            phantom: Default::default(),
        }
    }
}

impl<'a> CommandBuilder<'a, Set> {
    pub fn with_int_parameter(mut self, value: i32) -> Self {
        if !matches!(self.buffer.get(self.index - 1), Some(b'=')) {
            self.try_append_data(b",");
        }

        use core::fmt::Write;
        let mut conversion_buffer = ArrayString::<[u8; 12]>::new();
        write!(&mut conversion_buffer, "{}", value).expect("Bad Conversion");
        self.try_append_data(conversion_buffer.as_bytes());
        self
    }

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
    pub fn finish(mut self) -> Result<&'a [u8], usize> {
        self.try_append_data(b"\n");

        if self.index > self.buffer.len() {
            Err(self.index)
        } else {
            Ok(&self.buffer[0..self.index])
        }
    }
}

pub struct Uninitialized;
pub struct Initialized<T>(core::marker::PhantomData<T>);

pub struct Test;
pub struct Query;
pub struct Set;
pub struct Execute;
pub trait Finishable {}
impl Finishable for Test {}
impl Finishable for Query {}
impl Finishable for Set {}
impl Finishable for Execute {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_test(&mut buffer)
            .named("+TEST")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+TEST=?\n");
    }

    #[test]
    fn test_query() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_query(&mut buffer)
            .named("+QUERY")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+QUERY?\n");
    }

    #[test]
    fn test_set() {
        let mut buffer = [0; 128];
        let value = CommandBuilder::create_set(&mut buffer)
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
        let value = CommandBuilder::create_execute(&mut buffer)
            .named("+EXECUTE")
            .finish()
            .unwrap();

        assert_eq!(core::str::from_utf8(value).unwrap(), "AT+EXECUTE\n");
    }

    #[test]
    fn test_buffer_too_short() {
        let mut buffer = [0; 5];
        assert!(CommandBuilder::create_execute(&mut buffer)
            .named("+BUFFERLENGTH")
            .finish()
            .is_err());
    }
}
