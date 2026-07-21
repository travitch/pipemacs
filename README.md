# Overview

`pipemacs` is a tool to pipe data from its standard input to a new emacs process.

Emacs does not provide a native way to pipe data into a new buffer through standard input.  Between eshell and other built-in emacs facilities, that is generally fine.  `pipemacs` exists for when you just need to pipe data into emacs.

## Usage

Example:

```
jq . file.json | pm --no-window --mode json-mode
```

```
Usage: pm [OPTIONS] [filename]...

Arguments:
  [filename]...  Files to open in emacs; if this is specified, do not redirect stdin to emacs

Options:
  -n, --no-window                  Start a TTY emacs frame (instead of a GUI frame)
  -c, --client                     Connect to a running emacs server using emacsclient
  -m, --mode <mode>                Specify the mode to use for the piped buffer (default: fundamental-mode)
  -b, --buffer-name <buffer-name>  Specify the name to assign to the buffer containing the piped content (default: pipemacs-input)
  -h, --help                       Print help
```

# Related Work

The most closely related tool is [mxp](https://github.com/agzam/mxp), which is a shell script that accomplishes something similar to `pipemacs`.  Unlike `mxp`, `pipemacs` starts a new emacs process.  `mxp` requires a running emacs server instance.  Most people will probably prefer `mxp`, but `pipemacs` exists for those who prefer to start a new emacs process.
