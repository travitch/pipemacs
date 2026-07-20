# Overview

`pipemacs` is a tool to pipe data from its standard input to a new emacs process.

Emacs does not provide a native way to pipe data into a new buffer through standard input.  Between eshell and other built-in emacs facilities, that is generally fine.  `pipemacs` exists for when you just need to pipe data into emacs.

## Usage

Example:

```
jq . file.json | pm --no-window --mode json-mode
```

```
Usage: pm [OPTIONS]

Options:
      --no-window
          If true, pass the `-nw` argument to emacs to start in TTY mode

  -m, --mode <MODE>
          The mode to use for the buffer in emacs

          If not provided, use fundamental-mode

  -f, --filename <FILENAME>
          The filename to use for the emacs buffer.

          If none is provided, use a default

  -h, --help
          Print help (see a summary with '-h')
```

# Related Work

The most closely related tool is [mxp](https://github.com/agzam/mxp), which is a shell script that accomplishes something similar to `pipemacs`.  Unlike `mxp`, `pipemacs` starts a new emacs process.  `mxp` requires a running emacs server instance.  Most people will probably prefer `mxp`, but `pipemacs` exists for those who prefer to start a new emacs process.
