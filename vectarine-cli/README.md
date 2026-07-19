# Vectarine CLI

The vectarine CLI can be used to programmatically manage projects and run tests.
You can use it in a CI to generate builds of your game or generate screenshots, etc.

## Usage examples

```bash
# Display up-to-date information about all available commands
vecta --help
# Display help about the screenshot command specifically
vecta screenshot --help
```

### screenshot

```bash
# Take a screenshot of a project
vecta screenshot --project ./project/project.vecta --output ./screenshot.png
```

### test

```bash
# Test a project
vecta test --testfile ./testfile/Snake/test.toml
```

Read more information about [Vectarine's test system](./testing.md)

### export

```bash
# Export a project (-p is an alias for --project)
vecta export -p game.vecta -o my_game.zip --target mac-os
```

### new

```bash
# Create a new project using the default template
vecta new --name awesome_game
```
