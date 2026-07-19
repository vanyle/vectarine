# Testing a vectarine project

The `vecta` command line tools can be used to test projects using the `test` command:

```bash
vecta test --testfile ./testdata/Snake/test.toml
```

The `vecta` CLI will run your game in a deterministic simulated environment where you control the keyboard inputs, can run code and take screenshots to
compare to a baseline.

The `toml` file provided contains details about the test. For example:
A minimal test looks like this:

```toml
# First, you specify what project is tested and what the test actually tests.
[project]
path = "../../gallery/snake/game.vecta"
description = "The game loads without errors and looks normal"

# Then, you define the steps of your test.
[[step]]
wait_for_frames = 5 # Just let the game run for a bit

# Compare the appearance of the game to a reference. If the reference does not exist, it is automatically created.
# All paths are relative to the location of this toml file.
[[step]]
compare_screenshot_to = "./reference_screenshot.png"

[[step]]
expect_no_errors = {}
```

A more complex test, that runs code in the game and interacts with it using the keyboard could look like this:

```toml
[project]
path = "../../gallery/snake/game.vecta"
description = "The game responds to arrow keys"

[[step]]
wait_for_frames = 5

# Press the up button for 2 frames.
[[step]]
press_keys = ["up"]

[[step]]
wait_for_frames = 2

[[step]]
release_keys = ["up"]

# You can also simulate mouse presses

# [[step]]
# press_mouse_at = [100, 200]

# [[step]]
# release_mouse_at = [100, 200]

# In general, it is a good idea to wait between keyboard inputs to let the game
# process them.

[[step]]
wait_for_frames = 5

# You can run lua code to set game variable and test specific states of your game.
# You can also use this to check that a function runs without error inside your game.
[[step]]
run_lua_code = "state.paused = true"

[[step]]
compare_screenshot_to = "./reference_screenshot.png"

# Tests can also save the console content of the game to a file
# You can also use this to make sure your game output stays consistent.
[[step]]
compare_logs_to = "./reference_logs.txt"

[[step]]
expect_no_errors = {}

# You can run "clear" logs to get only the logs related to a specific section of your game.
# If you clear the logs, any errors inside them will be cleared and "expect_no_errors" will always succeed.
[[step]]
clear_logs = {}
```

The tested game runs in a simulated environment that runs at 60fps.

You can use tests to make sure that your games starts properly, that the appearance of a screen is consistent, etc.

If the test fails, `vecta` will exit with a code 1 (indicating a failure) and print something like:

```
❌ Test failed:
There was a difference in the bytes of the screenshot taken and the saved one at /Users/antoinedelegue/projects/game_related/vectarine-enhanced/testdata/Snake/screenshot1.png. Use a hex editor or image diff tool to see the difference.
```

Otherwise, it will just print

```
✅ Test passed.
```

If there is a difference between the reference screenshot / logs and the ones generated, you can pass the `-r` option to override them. You can then use `git` to see the difference in a convenient way, or in a Git GUI program.

## Running multiple tests

You might have multiple tests for your game. In that case, run:

```bash
vecta test --testfile ./testdata
```

`vecta` will look for all tests ending with `vecta-test.toml` in the folders provided and run them.
