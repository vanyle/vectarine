# Testing a vectarine project

The `vecta` command line tools can be used to test projects using the `test` command:

```bash
vecta test --testfile ./testdata/Snake/test.toml
```

The `toml` file provided contains details about the test. For example:

```toml
# First, you specify what project is tested and what the test actually tests.
[project]
path = "../../gallery/snake/game.vecta"
description = "The game responds to arrow keys"

# Then, you define the steps of your test.
[[step]]
wait_for_frames = 5 # Just let the game run for a bit

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

# You can run lua code to set game variable and test specific states of your game
[[step]]
run_lua_code = "state.paused = true"

# Tests can produce screenshots. You can compare screenshots using your CI to check for regressions
[[step]]
save_screenshot_to = "./screenshot1.png"

# Tests can also save the console content of the game to a file
[[step]]
save_logs_to = "./logs1.txt"

# Make sure that there were no errors in the console (otherwise, the test fails)
[[step]]
expect_no_errors = {}

# You can run "clear" logs to get only the logs related to a specific section of your game.
[[step]]
clear_logs = {}
```

The tested game runs in a simulated environment that runs at 60fps.

You can use tests to make sure that your games starts properly, that the appearance of a screen is consistent, etc.
