use std::{fs::File, io::{self, Write}, net::TcpListener, process::{Command, Stdio}, thread};

use clap::{Arg, ArgAction};

const ELISP_LIBRARY: &str = include_str!("../../resources/pipemacs.el");

struct Arguments {
    /// If true, pass the `-nw` argument to emacs to start in TTY mode.
    no_window: bool,

    /// The mode to use for the buffer in emacs
    ///
    /// If not provided, use fundamental-mode
    mode: Option<String>,

    /// The filename to use for the emacs buffer.
    ///
    /// If none is provided, use a default
    buffer_name: Option<String>,
}

/// Create the elisp function call to pass to emacs
fn call_emacs_entry_point(bound_port: u16, mode: Option<&String>, buffer_name: Option<&String>) -> String {
    let mode_arg = mode.map(|s| format!("\"{}\"", s)).unwrap_or_else(|| "fundamental-mode".into());
    let buffer_name_arg = buffer_name.map(|s| format!("\"{}\"", s)).unwrap_or_else(|| "\"pipemacs-input\"".into());
    format!("(pipemacs-read-data-into-buffer {} {} {})", bound_port, mode_arg, buffer_name_arg)
}

/// Copy all bytes from stdin to the first client that connects to the TCP listener.
fn feed_data_to_emacs(listener: TcpListener) -> anyhow::Result<()> {
    // This blocks until emacs connects to us, so we know it is ready once we get a stream.
    let (mut stream, _sockaddr) = listener.accept()?;

    for line in io::stdin().lines() {
        stream.write_all(line?.as_bytes())?;
        stream.write_all("\n".as_bytes())?;
    }

    stream.flush()?;

    Ok(())
}


fn main() -> anyhow::Result<()> {
    let matches = clap::Command::new("pm")
        .arg(Arg::new("no-window").long("no-window").short('n').action(ArgAction::SetTrue))
        .arg(Arg::new("mode").long("mode").short('m').action(ArgAction::Set))
        .arg(Arg::new("buffer-name").long("buffer-name").short('b').action(ArgAction::Set))
        .get_matches();

    let args = Arguments {
        no_window: matches.get_flag("no-window"),
        mode: matches.get_one::<String>("mode").cloned(),
        buffer_name: matches.get_one::<String>("buffer-name").cloned(),
    };

    // Bind to any local address
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let bound_port = listener.local_addr()?.port();
    let data_thread = thread::spawn(move || {
        match feed_data_to_emacs(listener) {
            Ok(()) => {}
            Err(e) => {
                panic!("Error feeding data to emacs {}", e);
            }
        }
    });

    let mut emacs_process = Command::new("emacs");
    if args.no_window {
        emacs_process.arg("-nw");

        // We have to give emacs a TTY to draw into.  The stdin of this process was redirected to
        // so if we do nothing, the inherited stdin is not a TTY, which will crash terminal emacs.
        let dev_tty = File::options().read(true).write(true).open("/dev/tty")?;
        let stdin: Stdio = dev_tty.into();
        emacs_process.stdin(stdin);
    }

    emacs_process.arg("--eval");
    emacs_process.arg(ELISP_LIBRARY);
    emacs_process.arg("--eval");
    emacs_process.arg(call_emacs_entry_point(bound_port, args.mode.as_ref(), args.buffer_name.as_ref()));


    let mut child_emacs = emacs_process.spawn()?;
    child_emacs.wait()?;
    data_thread.join().expect("Could not join the data thread");

    Ok(())
}
