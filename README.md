# tc

(T)ime (C)onverter. For those who have to constantly deal with timezones.

## Install

Builds are not provided for `tc` due to issues with cross compilation and the
curses library I'm using (see jeaye/ncurses-rs#207). macOS universal binaries
are available with `brew install amyy54/taps/tc`.

## Usage

```
Usage: tc [COMMAND]

Commands:
  t     Default - Get time based on defined timezones
  d     Define timezone to include on list
  u     Turn provided time into UNIX timestamp
  help  Print this message or the help of the given subcommand(s)

Options:
      --version  Print version
  -h, --help     Print help
```
