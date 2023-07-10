## rman

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

#### Configuration File Format

The configuration file should have the following format:

- For JSON:

  ```json
  [
    {
      "name": "string",
      "cmd": "string",
      "args": ["string"],
      "stdin": "Inherit",
      "stdout": "Inherit"
    }
  ]
  ```

- For YAML:

  ```yaml
  - name: string
    cmd: string
    args:
      - string
    stdin: Inherit
    stdout: Inherit
  ```

- For Procfile (plain text):

  ```
  name: cmd [args]
  ```

  - `name`: The name of the command.
  - `cmd`: The command to be executed.
  - `args`: (Optional) An array of command arguments.

#### Limitations

Currently only `null` and `inherit` are implemented for stdio. However I am planning to add support for `file` and `pipe` soon.

Also procfile config file does not support stdio configuration just yet!

#### Example Configuration File (JSON)

```json
[
  {
    "name": "command1",
    "cmd": "echo",
    "args": ["Hello, World!"],
    "stdin": "Inherit",
    "stdout": "Inherit"
  },
  {
    "name": "command2",
    "cmd": "python",
    "args": ["script.py"],
    "stdin": "Null",
    "stdout": "Null"
  }
]
```

#### Example Configuration File (Procfile)

```
command1: echo "Hello, World!"
command2: python script.py
```

### License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).
