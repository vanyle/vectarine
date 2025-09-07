## read_file_for_rust

Type: Rust -> JS

Reads the content of a network resource and returns it as a string for rust.

TODO:
- Handle failure
- Handle non-UTF8 content

## exit

Type: Rust -> JS

Signals to the JS runtime that the program finished.
The JS needs to display a message so that the user is not left with a black screen.
Maybe have a way to restart?

TODO:
- Everything

## ready

Type: Rust -> JS

Signals to the JS runtime that the program is ready.
The JS needs to stop displaying the loader.

TODO:
- Everything
