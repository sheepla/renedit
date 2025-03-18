<div align="center">

# 🚀 renedit

</div>

<div align="center">

A simple and efficient command line bulk file renaming tool that works in concert with your favorite text editors

</div>

## Demo


[Demo.webm](https://github.com/user-attachments/assets/2d89b984-bd6a-4eb0-be7a-f8365d8b5fc1)

## Features

- Efficient batch renaming workflow: 💨 Run → 🗒️Edit → 💾 Save → ✨ Rename
- Can invoke your favorite text editors to determine the name of the file to be changed. e.g. Vim, Emacs, nano, etc. (`-e`, `--editor` option or `$EDITOR` environment variable)
- Fail-safe: **dry-run** mode by default (run by `-x`, `--execute` option)

## Usage

The arguments are as follows:

```
Usage: renedit [OPTIONS] --editor <EDITOR> <PATH>...

Arguments:
  <PATH>...  Target directories or files

Options:
  -e, --editor <EDITOR>                    Command of text editor [env: EDITOR=nvim]
  -d, --definition-file <DEFINITION_FILE>  Path to definition file
  -x, --execute                            Execute renaming (disable DRY-RUN mode)
  -h, --help                               Print help
```

1. 💨 **Run**: Run this command like: `renedit --editor nvim path/to/files --execute`
1. 🗒️**Edit**: When the command is invoked, the editor is automatically launched. A text file with the path to the file will open, edit it to the name you want to change
1. 💾 **Save**: Overwrites the file and exits. For example, in Vim/Neovim, type `:wq`.
1. ✨ **Rename**: If the `-x`, `--execute` option is specified, you can batch rename to the changed name when you exit the editor. If not specified, the paths before and after the rename will be displayed.


## Installation

To build from source, run `cargo install --path=.`

## License

MIT

## Author

[sheepla](https://github.com/sheepla)
