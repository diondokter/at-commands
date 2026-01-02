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

    /// Tries reading an optional identifier.
    pub fn expect_optional_identifier(mut self, identifier: &[u8]) -> Self {
        // If we're already not valid, then quit
        if !self.data_valid {
            return self;
        }

        // empty identifier is always valid
        if self.buffer[self.buffer_index..].is_empty() {
            return self;
        }

        if self.buffer[self.buffer_index..].len() < identifier.len() {
            self.data_valid = false;
            return self;
        }

        // Zip together the identifier and the buffer data. If all bytes are the same, the data is valid.
        let found_optional = self.buffer[self.buffer_index..]
            .iter()
            .zip(identifier)
            .all(|(buffer, id)| *buffer == *id);

        if found_optional {
            // If we found the optional advance the index
            // Advance the index
            self.buffer_index += identifier.len();

            self.trim_space()
        } else {
            // If we did not find the optional just keep the index
            self
        }
    }

    /// Moves the internal buffer index over the next bit of space characters, if any
    fn trim_space(mut self) -> Self {
        // If we're already not valid, then quit
        if !self.data_valid {
            return self;
        }

        while let Some(c) = self.buffer.get(self.buffer_index) {
            if *c == b' ' {
                self.buffer_index += 1;
            } else {
                break;
            }
        }

        self
    }

    /// Finds the index of the character after the int parameter or the end of the data.
    fn find_end_of_int_parameter(&self) -> usize {
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
    fn find_end_of_string_parameter(&self) -> usize {
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
    fn find_end_of_raw_string(&self) -> usize {
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

    /// Finds the index of the character after the raw string parameter (comma or end of data).
    fn find_end_of_raw_string_parameter(&self) -> usize {
        self.buffer_index
            + self
                .buffer
                .get(self.buffer_index..)
                .map(|buffer| buffer.iter().take_while(|byte| **byte != b',').count())
                .unwrap_or(self.buffer.len())
    }

    fn parse_int_parameter(&self) -> (usize, bool, Option<i32>) {
        let mut new_buffer_index = self.buffer_index;
        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_int_parameter();
        // Get the bytes in which the int should reside.
        let int_slice = match self.buffer.get(self.buffer_index..parameter_end) {
            None => {
                return (new_buffer_index, false, None);
            }
            Some(int_slice) => int_slice,
        };
        if int_slice.is_empty() {
            // We probably hit the end of the buffer.
            // The parameter is empty but as it is optional not invalid
            // Advance the index to the character after the parameter separator (comma) if it's there.
            new_buffer_index =
                parameter_end + (self.buffer.get(parameter_end) == Some(&b',')) as usize;
            return (new_buffer_index, true, None);
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
        new_buffer_index = parameter_end + (self.buffer.get(parameter_end) == Some(&b',')) as usize;
        // If we've found an int, then the data may be valid and we allow the closure to set the result ok data.
        if let Some(parameter_value) = parsed_int {
            (new_buffer_index, true, Some(parameter_value))
        } else {
            (new_buffer_index, false, None)
        }
    }

    fn parse_string_parameter(&self) -> (usize, bool, Option<&'a str>) {
        let mut new_buffer_index = self.buffer_index;
        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_string_parameter();
        if parameter_end > self.buffer.len() {
            // We hit the end of the buffer.
            // The parameter is empty but as it is optional not invalid
            return (new_buffer_index, true, None);
        }
        // Get the bytes in which the string should reside.
        let string_slice = &self.buffer[(new_buffer_index + 1)..(parameter_end - 1)];

        let has_comma_after_parameter = if let Some(next_char) = self.buffer.get(parameter_end) {
            *next_char == b','
        } else {
            false
        };

        // Advance the index to the character after the parameter separator.
        new_buffer_index = parameter_end + has_comma_after_parameter as usize;
        // If we've found a valid string, then the data may be valid and we allow the closure to set the result ok data.
        if let Ok(parameter_value) = core::str::from_utf8(string_slice) {
            (new_buffer_index, true, Some(parameter_value))
        } else {
            (new_buffer_index, false, None)
        }
    }

    fn parse_raw_string(&self) -> (usize, bool, Option<&'a str>) {
        let mut new_buffer_index = self.buffer_index;
        // Get the end index of the current string.
        let end = self.find_end_of_raw_string();
        // Get the bytes in which the string should reside.
        let string_slice = &self.buffer[new_buffer_index..(end - 1)];

        // Advance the index to the character after the string.
        new_buffer_index = end - 1usize;

        // If we've found a valid string, then the data may be valid and we allow the closure to set the result ok data.
        if let Ok(parameter_value) = core::str::from_utf8(string_slice) {
            (new_buffer_index, true, Some(parameter_value))
        } else {
            (new_buffer_index, false, None)
        }
    }

    fn parse_raw_string_parameter(&self) -> (usize, bool, Option<&'a str>) {
        let mut new_buffer_index = self.buffer_index;
        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_raw_string_parameter();
        // Get the bytes in which the string should reside.
        let string_slice = &self.buffer[new_buffer_index..parameter_end];

        if string_slice.is_empty() {
            // We probably hit the end of the buffer.
            // The parameter is empty but as it is optional not invalid
            // Advance the index to the character after the parameter separator (comma) if it's there.
            new_buffer_index =
                parameter_end + (self.buffer.get(parameter_end) == Some(&b',')) as usize;
            return (new_buffer_index, true, None);
        }

        let has_comma_after_parameter = if let Some(next_char) = self.buffer.get(parameter_end) {
            *next_char == b','
        } else {
            false
        };

        // Advance the index to the character after the parameter separator.
        new_buffer_index = parameter_end + has_comma_after_parameter as usize;
        // If we've found a valid string, then the data may be valid and we allow the closure to set the result ok data.
        if let Ok(parameter_value) = core::str::from_utf8(string_slice) {
            (new_buffer_index, true, Some(parameter_value))
        } else {
            (new_buffer_index, false, None)
        }
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
    pub fn expect_int_parameter(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(0),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_int_parameter();
        if let Some(parameter_value) = data {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid,
                data: self.data.tup_cat(parameter_value),
            }
            .trim_space()
        } else {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid: false,
                data: self.data.tup_cat(0),
            }
            .trim_space()
        }
    }
}

impl<'a, D: TupleConcat<&'a str>> CommandParser<'a, D> {
    /// Tries reading a string parameter
    pub fn expect_string_parameter(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_string_parameter();
        if let Some(parameter_value) = data {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid,
                data: self.data.tup_cat(parameter_value),
            }
            .trim_space()
        } else {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid: false,
                data: self.data.tup_cat(""),
            }
            .trim_space()
        }
    }

    /// Tries reading a non-parameter, non-quoted string
    pub fn expect_raw_string(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_raw_string();
        if let Some(parameter_value) = data {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid,
                data: self.data.tup_cat(parameter_value),
            }
            .trim_space()
        } else {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid: false,
                data: self.data.tup_cat(""),
            }
            .trim_space()
        }
    }

    /// Tries reading a raw string parameter (non-quoted string separated by commas)
    pub fn expect_raw_string_parameter(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(""),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_raw_string_parameter();
        if let Some(parameter_value) = data {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid,
                data: self.data.tup_cat(parameter_value),
            }
            .trim_space()
        } else {
            CommandParser {
                buffer: self.buffer,
                buffer_index,
                data_valid: false,
                data: self.data.tup_cat(""),
            }
            .trim_space()
        }
    }
}

//
// Optional parameters
//

impl<'a, D: TupleConcat<Option<i32>>> CommandParser<'a, D> {
    /// Tries reading an int parameter
    pub fn expect_optional_int_parameter(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(None),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_int_parameter();
        CommandParser {
            buffer: self.buffer,
            buffer_index,
            data_valid,
            data: self.data.tup_cat(data),
        }
        .trim_space()
    }
}

impl<'a, D: TupleConcat<Option<&'a str>>> CommandParser<'a, D> {
    /// Tries reading a string parameter
    pub fn expect_optional_string_parameter(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(None),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_string_parameter();
        CommandParser {
            buffer: self.buffer,
            buffer_index,
            data_valid,
            data: self.data.tup_cat(data),
        }
        .trim_space()
    }

    /// Tries reading a non-parameter, non-quoted string
    pub fn expect_optional_raw_string(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(None),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_raw_string();
        CommandParser {
            buffer: self.buffer,
            buffer_index,
            data_valid,
            data: self.data.tup_cat(data),
        }
        .trim_space()
    }

    /// Tries reading an optional raw string parameter (non-quoted string separated by commas)
    pub fn expect_optional_raw_string_parameter(self) -> CommandParser<'a, D::Out> {
        // If we're already not valid, then quit
        if !self.data_valid {
            return CommandParser {
                buffer: self.buffer,
                buffer_index: self.buffer_index,
                data_valid: self.data_valid,
                data: self.data.tup_cat(None),
            };
        }

        let (buffer_index, data_valid, data) = self.parse_raw_string_parameter();
        CommandParser {
            buffer: self.buffer,
            buffer_index,
            data_valid,
            data: self.data.tup_cat(data),
        }
        .trim_space()
    }
}

/// Error type for parsing
///
/// The number is the index of up to where it was correctly parsed
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

    #[test]
    fn string_param_at_end() {
        let (x, y) = CommandParser::parse(br#"+SYSGPIOREAD: 42, "param at end""#)
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_int_parameter()
            .expect_string_parameter()
            .finish()
            .unwrap();

        assert_eq!(x, 42);
        assert_eq!(y, "param at end");
    }

    #[test]
    fn test_optional_int_parameter_all_present() {
        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:654,\"true\",-65154\r\nOK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, Some(654));
        assert_eq!(y, Some("true"));
        assert_eq!(z, Some(-65154));
    }

    #[test]
    fn test_optional_int_parameter_middle_not_present() {
        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:,\"true\"\r\nOK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, None);
        assert_eq!(y, Some("true"));
        assert_eq!(z, None);
    }

    #[test]
    fn test_optional_int_parameter_end_not_present() {
        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:654,\"true\",\r\nOK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_optional_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, Some(654));
        assert_eq!(y, Some("true"));
        assert_eq!(z, None);
    }

    #[test]
    fn test_optional_identifier() {
        let r = CommandParser::parse(b"+SYSGPIOREAD:,\"true\"\r\nK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_optional_identifier(b"\r\nOK\r\n")
            .finish();

        assert_eq!(r, Err(ParseError(20)));

        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:,\"true\"\r\nOK\r\n")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_optional_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, None);
        assert_eq!(y, Some("true"));
        assert_eq!(z, None);

        let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:,\"true\"")
            .expect_identifier(b"+SYSGPIOREAD:")
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_optional_identifier(b"\r\nOK\r\n")
            .finish()
            .unwrap();

        assert_eq!(x, None);
        assert_eq!(y, Some("true"));
        assert_eq!(z, None);
    }

    /// On this test we will check multiple possible variables for the optional identifier
    #[test]
    fn test_optional_identifier_multiple_cases() {
        const OK_1: &str = "\r\nOK";
        const OK_2: &str = "\r\nOK\r\n";
        const OK_3: &str = "OK\r\n";
        const OK_4: &str = "OK";

        static TEST_CASES: [&str; 4] = [OK_1, OK_2, OK_3, OK_4];

        for test_case in TEST_CASES {
            let result = CommandParser::parse(test_case.as_bytes())
                .expect_optional_identifier(b"\r")
                .expect_optional_identifier(b"\n")
                .expect_identifier(b"OK")
                .expect_optional_identifier(b"\r")
                .expect_optional_identifier(b"\n")
                .finish();

            assert_eq!(result, Ok(()), "Failed test case: {:?}", test_case);
        }
    }

    #[test]
    fn test_raw_string_parameter() {
        let (x, y, raw, z) =
            CommandParser::parse(b"+SYSGPIOREAD:654,\"true\",123ABC,-65154\r\nOK\r\n")
                .expect_identifier(b"+SYSGPIOREAD:")
                .expect_int_parameter()
                .expect_string_parameter()
                .expect_raw_string_parameter()
                .expect_int_parameter()
                .expect_identifier(b"\r\nOK\r\n")
                .finish()
                .unwrap();

        assert_eq!(x, 654);
        assert_eq!(y, "true");
        assert_eq!(raw, "123ABC");
        assert_eq!(z, -65154);
    }

    #[test]
    fn raw_string_parameters_includes_non_ascii_characters() {
        let (x, y, raw, z) =
            CommandParser::parse("+SYSGPIOREAD:654,\"true\",123àABC,-65154\r\nOK\r\n".as_bytes())
                .expect_identifier(b"+SYSGPIOREAD:")
                .expect_int_parameter()
                .expect_string_parameter()
                .expect_raw_string_parameter()
                .expect_int_parameter()
                .expect_identifier(b"\r\nOK\r\n")
                .finish()
                .unwrap();

        assert_eq!(x, 654);
        assert_eq!(y, "true");
        assert_eq!(raw, "123àABC");
        assert_eq!(z, -65154);
    }
}
