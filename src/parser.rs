//! Module that defines the at command parser

use crate::tuple_concat::TupleConcat;

/// ```
/// use at_commands::parser::CommandParser;
/// let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:654,\"true\",-65154\r\nOK\r\n")
///    .expect_identifier(b"+SYSGPIOREAD:")
///    .expect_int_parameter()
///    .expect_string_parameter()
///    .expect_int_parameter()
///    .expect_identifier(b"\r\nOK\r\n")
///    .finish()
///    .unwrap();
///
/// assert_eq!(x, 654);
/// assert_eq!(y, "true");
/// assert_eq!(z, -65154);
///
/// let (w,) = CommandParser::parse(b"+STATUS: READY\r\nOK\r\n")
///    .expect_identifier(b"+STATUS: ")
///    .expect_raw_string()
///    .expect_identifier(b"\r\nOK\r\n")
///    .finish()
///    .unwrap();
///
/// assert_eq!(w, "READY");
/// ```
#[must_use]
pub struct CommandParser<'a, D> {
    buffer: &'a [u8],
    buffer_index: usize,
    data_valid: bool,
    data: D,
}

impl<'a> CommandParser<'a, ()> {
    /// Start parsing the command
    pub fn parse(buffer: &'a [u8]) -> CommandParser<'a, ()> {
        CommandParser {
            buffer,
            buffer_index: 0,
            data_valid: true,
            data: (),
        }
    }
}
impl<'a, D> CommandParser<'a, D> {
    /// Tries reading an identifier
    pub fn expect_identifier(mut self, identifier: &[u8]) -> Self {
        // If we're already not valid, then quit
        if !self.data_valid {
            return self;
        }

        if self.buffer[self.buffer_index..].len() < identifier.len() {
            self.data_valid = false;
            return self;
        }

        // Zip together the identifier and the buffer data. If all bytes are the same, the data is valid.
        self.data_valid = self.buffer[self.buffer_index..]
            .iter()
            .zip(identifier)
            .all(|(buffer, id)| *buffer == *id);
        // Advance the index
        self.buffer_index += identifier.len();

        self.trim_space()
    }

    /// Moves the internal buffer index over the next bit of space characters, if any
    fn trim_space(mut self) -> Self {
        // If we're already not valid, then quit
        if !self.data_valid {
            return self;
        }

        loop {
            if let Some(c) = self.buffer.get(self.buffer_index) {
                if *c == b' ' {
                    self.buffer_index += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        self
    }

    /// Finds the index of the character after the int parameter or the end of the data.
    fn find_end_of_int_parameter(&mut self) -> usize {
        self.buffer_index
            + self
                .buffer
                .get(self.buffer_index..)
                .map(|buffer| {
                    buffer
                        .iter()
                        .take_while(|byte| {
                            byte.is_ascii_digit() || **byte == b'-' || **byte == b'+'
                        })
                        .count()
                })
                .unwrap_or(self.buffer.len())
    }

    /// Finds the index of the character after the string parameter or the end of the data.
    fn find_end_of_string_parameter(&mut self) -> usize {
        let mut counted_quotes = 0;

        self.buffer_index
            + self
                .buffer
                .get(self.buffer_index..)
                .map(|buffer| {
                    buffer
                        .iter()
                        .take_while(|byte| {
                            counted_quotes += (**byte == b'"') as u8;
                            counted_quotes < 2
                        })
                        .count()
                        + 1
                })
                .unwrap_or(self.buffer.len())
    }

    /// Finds the index of the control character after the non-quoted string or the end of the data.
    fn find_end_of_raw_string(&mut self) -> usize {
        self.buffer_index
            + self
                .buffer
                .get(self.buffer_index..)
                .map(|buffer| {
                    buffer
                        .iter()
                        .take_while(|byte| !(**byte as char).is_ascii_control())
                        .count()
                        + 1
                })
                .unwrap_or(self.buffer.len())
    }

    /// Finish parsing the command and get the results
    pub fn finish(self) -> Result<D, ParseError> {
        if self.data_valid {
            Ok(self.data)
        } else {
            Err(ParseError(self.buffer_index))
        }
    }
}

impl<'a, D: TupleConcat<i32>> CommandParser<'a, D> {
    /// Tries reading an int parameter
    pub fn expect_int_parameter(mut self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(0),
            };
        }

        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_int_parameter();
        // Get the bytes in which the int should reside.
        let int_slice = match self.buffer.get(self.buffer_index..parameter_end) {
            None => {
                self.data_valid = false;
                return CommandParser {
                    buffer: self.buffer,
                    buffer_index: self.buffer_index,
                    data_valid: self.data_valid,
                    data: self.data.tup_cat(0),
                };
            }
            Some(int_slice) => int_slice,
        };
        if int_slice.is_empty() {
            // We probably hit the end of the buffer.
            // The parameter is empty so it is always invalid.
            self.data_valid = false;
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(0),
            };
        }

        // Skip the leading '+'
        let int_slice = if int_slice[0] == b'+' {
            &int_slice[1..]
        } else {
            int_slice
        };

        // Parse the int
        let parsed_int = crate::formatter::parse_int(int_slice);

        // Advance the index to the character after the parameter separator (comma) if it's there.
        self.buffer_index =
            parameter_end + (self.buffer.get(parameter_end) == Some(&b',')) as usize;
        // If we've found an int, then the data may be valid and we allow the closure to set the result ok data.
        if let Some(parameter_value) = parsed_int {
            CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(parameter_value),
            }
        } else {
            self.data_valid = false;
            CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(0),
            }
        }
        .trim_space()
    }
}
impl<'a, D: TupleConcat<&'a str>> CommandParser<'a, D> {
    /// Tries reading a string parameter
    pub fn expect_string_parameter(mut self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            };
        }

        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_string_parameter();
        if parameter_end == self.buffer.len() {
            // We hit the end of the buffer.
            // The parameter is empty so it is always invalid.
            self.data_valid = false;
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            };
        }
        // Get the bytes in which the string should reside.
        let string_slice = &self.buffer[(self.buffer_index + 1)..(parameter_end - 1)];

        let has_comma_after_parameter = if let Some(next_char) = self.buffer.get(parameter_end) {
            *next_char == b','
        } else {
            false
        };

        // Advance the index to the character after the parameter separator.
        self.buffer_index = parameter_end + has_comma_after_parameter as usize;
        // If we've found a valid string, then the data may be valid and we allow the closure to set the result ok data.
        if let Ok(parameter_value) = core::str::from_utf8(string_slice) {
            CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(parameter_value),
            }
        } else {
            self.data_valid = false;
            CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            }
        }
        .trim_space()
    }

    /// Tries reading a non-parameter, non-quoted string
    pub fn expect_raw_string(mut self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            };
        }

        // Get the end index of the current string.
        let end = self.find_end_of_raw_string();
        // Get the bytes in which the string should reside.
        let string_slice = &self.buffer[self.buffer_index..(end - 1)];

        // Advance the index to the character after the string.
        self.buffer_index = end - 1 as usize;

        // If we've found a valid string, then the data may be valid and we allow the closure to set the result ok data.
        if let Ok(parameter_value) = core::str::from_utf8(string_slice) {
            CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(parameter_value),
            }
        } else {
            self.data_valid = false;
            CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            }
        }
        .trim_space()
    }
}

/// Error type for parsing
///
/// The number is the index of up to where it was correctly parsed
#[derive(Debug, Clone)]
pub struct ParseError(usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:654,\"true\",-65154\r\nOK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_int_parameter()
            .expect_string_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, 654);
        assert_eq!(y, "true");
        assert_eq!(z, -65154);
    }

    #[test]
    fn test_positive_int_param() {
        let (x,) = CommandParser::parse(b"OK+RP:+20dBm\r\n")
            .expect_identifier(b"OK+RP:")
            .expect_int_parameter()
            .expect_identifier(b"dBm\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, 20);
    }

    #[test]
    fn test_whitespace() {
        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD: 654, \"true\", -65154 \r\nOK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_int_parameter()
            .expect_string_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, 654);
        assert_eq!(y, "true");
        assert_eq!(z, -65154);
    }
}
