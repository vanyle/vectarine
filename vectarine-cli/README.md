# Vectarine CLI

The vectarine CLI can be used to programmatically manage projects and run tests.
You can use it in a CI to generate builds of your game or generate screenshots, etc.

## Usage examples

```bash
# Take a screenshot of a project
vecta-cli screenshot --project ./project/project.vecta --output ./screenshot.png
```

Goals:

- [x] Run a project and generate a screenshot.
- [x] Export a project
- [ ] Run a project and get console output
- [ ] Scaffold a new project
- [ ] Run a test file
