# Changelog
All notable changes to this project will be documented in this file.

## v1.2 - 2022-01-11

### Added
- Added a setting which changes the pool of words to guess from to a smaller hand picked list. This word list should contain less strange unused / dialect words. Guesses can still be submitted from the full list. For 6 character mode the common words list is still mostly unchanged, but is coming soon.
- The default word list is still the full list, but this setting is persisted

### Changed

### Fixed

## v1.1 - 2022-01-11

### Added

### Changed

### Fixed
- Fix a bug where on current tile state border highlights the count of known characters wasn't considered. Even if the exact count of characters was already known the absent tiles were still displayed with yellow "present" border instead of as known absent. Thanks /u/Allium_Senescens for the bug report!

## v1.0 - 2022-01-09

### Added
- Added version number and changelog

### Changed
- Refactor project structure into components

### Fixed
- Fix board sometimes not rerendering after keypresses