# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Add raw parameter to command builder.

## [0.5.4] - 2023-09-18
- Added optional parser parameters: [diondokter/at-commands#9](https://github.com/diondokter/at-commands/pull/9)
- Changed &str to AsRef<[u8]> in CommandBuilder methods: [diondokter/at-commands#10](https://github.com/diondokter/at-commands/pull/10)

## [0.5.3] - 2022-09-11
- More arguments are now supported: [diondokter/at-commands#6](https://github.com/diondokter/at-commands/pull/6)
- Small bugfix for parsing string parameters that have no characters after it in the input string: [diondokter/at-commands#6](https://github.com/diondokter/at-commands/pull/6)

## [0.5.2] - 2022-07-27
- Added optional defmt feature

## [0.5.1] - 2022-03-02
### Changed
- The parser now trims all spaces after each identifier and parameter function making it more robust

## [0.5.0] - 2021-05-24 (~~[0.4.1] - 2021-05-23~~)
### Added
- Integer parsing now discards the leading `+` character if present: [diondokter/at-commands#5](https://github.com/diondokter/at-commands/pull/5)

### Changed
- The parser is no longer behind a feature gate and is always enabled

## [0.4.0] - 2020-11-16
### Added
- Optional and empty parameter support added: [diondokter/at-commands#2](https://github.com/diondokter/at-commands/pull/2)
- Experimental parser [diondokter/at-commands#3](https://github.com/diondokter/at-commands/pull/3)

## [0.3.0] - 2020-08-14
### Changed
- **Breaking**: Command is now terminated with `\r\n` instead of `\n`.

### Added
- The function `finish_with` has been added so users can choose their own termination.

## [0.2.1] - 2020-05-21
### Changed
- All formatting (fmt) has been removed. (This drastically reduces the amount of flash memory used)
- Removed dependency on ArrayVec because it was no longer needed.

## [0.2.0] - 2020-05-20
### Added
- Ability to not have an AT prefix in the command.

### Fixed
- The arrayvec dependency still used the std. Now changed to not use default features.

## [0.1.1] - 2020-04-13
### Added
- Setup required for publishing the crate.

## [0.1.0] - 2020-04-13
### Added
- Initial `CommandBuilder` implementation.
