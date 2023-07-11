## rman [![CI](https://github.com/gmanninglive/rman/actions/workflows/ci.yml/badge.svg)](https://github.com/gmanninglive/rman/actions/workflows/ci.yml)

rman is a command line program inspired by foreman that allows you to configure standard input/output (stdio) for each command.

Note this is intended for development purposes only and should never be used in production!

### Features

- Configure stdio for individual commands
- Run multiple commands in a single terminal session
- Customize command execution and control flow

### Installation

To install rman, follow these steps:

1. Clone the repository from GitHub: `git clone https://github.com/gmanninglive/rman.git`
2. Navigate to the project directory: `cd rman`
3. Run or build: `cargo run -- [path/to/config]` or `cargo build --release`

### Usage

To use rman, follow the instructions below:

1. Create a configuration file in either JSON, YAML, or Procfile format. This file will define the list of commands you want to run, along with their stdio configurations.
2. Pass the path to the configuration file as an argument when running the `rman` command: `./rman config_file_path`

### Options

- `name`: The name of the command.
- `stdin`: (Optional) The source of stdin. `null` | `inherit` | `file={path}`
  Defaults to inherit.
- `stdout`: (Optional) The source of stdout. `null` | `inherit` | `file={path}`
  Defaults to inherit.
- `cmd`: The command to be executed.
- `args`: (Optional) An array of command arguments.

#### Configuration File Format

The configuration file should have the following format:

- For JSON:
  Note for stdio options `null` must be expressed as a string, to be deserialized correctly.

  ```json
  [
    {
      "name": "string",
      "cmd": "string",
      "args": ["string"],
      "stdin": "inherit",
      "stdout": "null"
    }
  ]
  ```

- For YAML:

  ```yaml
  - name: string
    cmd: string
    args:
      - string
    stdin: inherit
    stdout: inherit
  ```

- For Procfile (plain text):

  ```
  name: stdin>{stdin} stdout>{stdout} cmd [args]
  ```

#### Example Configuration File (JSON)

```json
[
  {
    "name": "command1",
    "cmd": "echo",
    "args": ["Hello, World!"],
    "stdin": "inherit",
    "stdout": "inherit"
  },
  {
    "name": "command2",
    "cmd": "python",
    "args": ["script.py"],
    "stdin": "null",
    "stdout": "null"
  }
]
```

#### Example Configuration File (Procfile)

```
command1: stdin>null stdout>file=log.txt echo "Hello, World!"
command2: stdout>null python script.py
```

### License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).
