# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
