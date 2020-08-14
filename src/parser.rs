pub struct CommandParser<'a, ProgressState, OkDataType> {
    buffer: &'a [u8],
    buffer_index: usize,
    ok_data: OkDataType,
    error_index: Option<u8>,
    errors_tested: u8,
    ok_data_valid: bool,
    progress_phantom: core::marker::PhantomData<ProgressState>,
}

pub struct Uninitialized;
pub struct OkUninitialized;
pub struct OkStarted;
pub struct ErrUninitialized;
pub struct ErrStarted;

impl<'a> CommandParser<'a, Uninitialized, ()> {
    pub fn parse<OkDataType>(
        buffer: &'a [u8],
        initial_ok_data: OkDataType,
    ) -> CommandParser<'a, OkUninitialized, OkDataType> {
        CommandParser {
            buffer,
            buffer_index: 0,
            ok_data: initial_ok_data,
            error_index: None,
            errors_tested: 0,
            ok_data_valid: true,
            progress_phantom: Default::default(),
        }
    }
}

impl<'a, AnyOkData> CommandParser<'a, OkUninitialized, AnyOkData> {
    pub fn with_ok(self) -> CommandParser<'a, OkStarted, AnyOkData> {
        CommandParser::<'a, _, _> {
            buffer: self.buffer,
            buffer_index: 0,
            ok_data: self.ok_data,
            error_index: self.error_index,
            errors_tested: self.errors_tested,
            ok_data_valid: self.ok_data_valid,
            progress_phantom: Default::default(),
        }
    }
}

impl<'a, AnyOkData> CommandParser<'a, OkStarted, AnyOkData> {
    pub fn expect_identifier(mut self, identifier: &[u8]) -> Self {
        // If we're already not valid, then quit
        if !self.ok_data_valid {
            return self;
        }

        if self.buffer[self.buffer_index..].len() < identifier.len() {
            self.ok_data_valid = false;
            return self;
        }

        // Zip together the identifier and the buffer data. If all bytes are the same, the data is valid.
        self.ok_data_valid = self.buffer[self.buffer_index..]
            .iter()
            .zip(identifier)
            .all(|(buffer, id)| *buffer == *id);
        // Advance the index
        self.buffer_index += identifier.len();

        self
    }

    pub fn expect_int_parameter<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut AnyOkData, i32) -> Result<(), ()>,
    {
        // If we're already not valid, then quit
        if !self.ok_data_valid {
            return self;
        }

        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_parameter();
        // Get the bytes in which the int should reside.
        let int_slice = match self.buffer.get(self.buffer_index..parameter_end) {
            None => {
                self.ok_data_valid = false;
                return self;
            }
            Some(int_slice) => int_slice,
        };
        if int_slice.is_empty() {
            // We probably hit the end of the buffer.
            // The parameter is empty so it is always invalid.
            self.ok_data_valid = false;
            return self;
        }

        // Parse the int
        let parsed_int = crate::formatter::parse_int(int_slice);

        // Advance the index to the character after the parameter separator (comma) if it's there.
        self.buffer_index =
            parameter_end + (self.buffer.get(parameter_end) == Some(&b',')) as usize;
        // If we've found an int, then the data may be valid and we allow the closure to set the result ok data.
        if let Some(parameter_value) = parsed_int {
            self.ok_data_valid = f(&mut self.ok_data, parameter_value).is_ok();
        } else {
            self.ok_data_valid = false;
        }

        self
    }

    pub fn expect_string_parameter<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut AnyOkData, &str) -> Result<(), ()>,
    {
        // If we're already not valid, then quit
        if !self.ok_data_valid {
            return self;
        }

        // Get the end index of the current parameter.
        let parameter_end = self.find_end_of_parameter();
        // Get the bytes in which the string should reside.
        let string_slice = &self.buffer[self.buffer_index..parameter_end];
        if string_slice.is_empty() {
            // We probably hit the end of the buffer.
            // The parameter is empty so it is always invalid.
            self.ok_data_valid = false;
            return self;
        }

        let has_comma_after_parameter = if let Some(next_char) = self.buffer.get(parameter_end) {
            *next_char == b','
        } else {
            false
        };

        // Advance the index to the character after the parameter separator.
        self.buffer_index = parameter_end + has_comma_after_parameter as usize;
        // If we've found a valid string, then the data may be valid and we allow the closure to set the result ok data.
        if let Ok(parameter_value) = core::str::from_utf8(string_slice) {
            self.ok_data_valid = f(&mut self.ok_data, parameter_value).is_ok();
        } else {
            self.ok_data_valid = false;
        }

        self
    }

    pub fn expect_ending_with_ok(self) -> CommandParser<'a, ErrStarted, AnyOkData> {
        self.expect_ending_with_identifier(b"OK\n")
    }
    pub fn expect_ending_with_newline_ok(self) -> CommandParser<'a, ErrStarted, AnyOkData> {
        self.expect_ending_with_identifier(b"\nOK\n")
    }
    pub fn expect_ending_with_newline(self) -> CommandParser<'a, ErrStarted, AnyOkData> {
        self.expect_ending_with_identifier(b"\n")
    }
    pub fn expect_ending_with_identifier(
        mut self,
        identifier: &[u8],
    ) -> CommandParser<'a, ErrStarted, AnyOkData> {
        self = self.expect_identifier(identifier);

        CommandParser::<'a, _, _> {
            buffer: self.buffer,
            buffer_index: if self.ok_data_valid {
                self.buffer_index
            } else {
                0
            },
            ok_data: self.ok_data,
            error_index: self.error_index,
            errors_tested: self.errors_tested,
            ok_data_valid: self.ok_data_valid,
            progress_phantom: Default::default(),
        }
    }

    /// Finds the index of the character after the parameter or the end of the data.
    fn find_end_of_parameter(&mut self) -> usize {
        self.buffer_index
            + self
                .buffer
                .get(self.buffer_index..)
                .map(|buffer| {
                    buffer
                        .iter()
                        .take_while(|byte| **byte != b',' && **byte != b'\n')
                        .count()
                })
                .unwrap_or(self.buffer.len())
    }
}

impl<'a, AnyOkData> CommandParser<'a, ErrStarted, AnyOkData> {
    pub fn or_error(mut self, error: &[u8]) -> Self {
        if !self.ok_data_valid && self.error_index.is_none() {
            if self
                .buffer
                .iter()
                .zip(error)
                .all(|(buffer, error)| *buffer == *error)
            {
                self.error_index = Some(self.errors_tested);
                self.buffer_index = error.len();
            }
            self.errors_tested += 1;
        }

        self
    }

    pub fn get_result(self) -> Result<AnyOkData, Option<u8>> {
        if self.ok_data_valid {
            Ok(self.ok_data)
        } else {
            Err(self.error_index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let parse_result =
            CommandParser::parse(b"+SYSGPIOREAD:654,true,-65154\nOK\n", (0, false, 0))
                .with_ok()
                .expect_identifier(b"+SYSGPIOREAD:")
                .expect_int_parameter(|data, value| {
                    data.0 = value;
                    Ok(())
                })
                .expect_string_parameter(|data, value| {
                    value.parse().map(|val| data.1 = val).map_err(|_| ())
                })
                .expect_int_parameter(|data, value| {
                    data.2 = value;
                    Ok(())
                })
                .expect_ending_with_newline_ok()
                .get_result();

        assert_eq!(parse_result, Ok((654, true, -65154)))
    }
}
