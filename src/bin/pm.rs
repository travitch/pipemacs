use std::{fs::File, io::{self, Read as _, Write}, net::TcpListener, process::{Command, Stdio}, thread};

use clap::{Arg, ArgAction};

const ELISP_LIBRARY: &str = include_str!("../../resources/pipemacs.el");

struct Arguments {
    /// If true, pass the `-nw` argument to emacs to start in TTY mode.
    no_window: bool,

    /// If true, use emacsclient to connect to an emacs server instead of starting a new instance.
    client: bool,

    /// If true, write the contents of the emacs buffer back to this process, to be printed
    /// to standard output.
    writeback: bool,

    /// The mode to use for the buffer in emacs
    ///
    /// If not provided, use fundamental-mode
    mode: Option<String>,

    /// The name to use for the emacs buffer.
    ///
    /// If none is provided, use a default
    buffer_name: Option<String>,

    /// The filenames to open in emacs.
    ///
    /// Passing this disables the standard input collecting mode.
    filenames: Vec<String>,
}

/// Create the elisp function call to pass to emacs
fn call_emacs_entry_point(bound_port: u16, mode: Option<&String>, buffer_name: Option<&String>, writeback: bool) -> String {
    let mode_arg = mode.map(|s| format!("{}", s)).unwrap_or_else(|| "fundamental-mode".into());
    let buffer_name_arg = buffer_name.map(|s| format!("{}", s)).unwrap_or_else(|| "pipemacs-input".into());
    let writeback_arg = if writeback { "t" } else { "nil" };
    format!("(pipemacs-read-data-into-buffer {} \"{}\" \"{}\" {})", bound_port, mode_arg, buffer_name_arg, writeback_arg)
}

/// Copy all bytes from stdin to the first client that connects to the TCP listener.
fn feed_data_to_emacs(listener: TcpListener, writeback: bool) -> anyhow::Result<Option<String>> {
    // This blocks until emacs connects to us, so we know it is ready once we get a stream.
    let (mut stream, _sockaddr) = listener.accept()?;

    for line in io::stdin().lines() {
        stream.write_all(line?.as_bytes())?;
        stream.write_all("\n".as_bytes())?;
    }

    stream.flush()?;

    if writeback {
        let mut buffer = String::new();
        let _bytes_read = stream.read_to_string(&mut buffer)?;

        Ok(Some(buffer))
    } else {
        Ok(None)
    }
}

struct ServerState {
    listener: TcpListener,
    bound_port: u16,
}

fn create_listener(args: &Arguments) -> anyhow::Result<Option<ServerState>> {
    if args.filenames.is_empty() {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let bound_port = listener.local_addr()?.port();

        Ok(Some(ServerState {listener, bound_port }))
    } else {
        Ok(None)
    }
}

fn main() -> anyhow::Result<()> {
    let matches = clap::Command::new("pm")
        .arg(Arg::new("no-window").long("no-window").short('n').action(ArgAction::SetTrue)
            .help("Start a TTY emacs frame (instead of a GUI frame)"))
        .arg(Arg::new("client").long("client").short('c').action(ArgAction::SetTrue)
            .help("Connect to a running emacs server using emacsclient"))
        .arg(Arg::new("writeback").long("writeback").short('w').action(ArgAction::SetTrue)
            .help("Write the contents of the emacs buffer back to standard output"))
        .arg(Arg::new("mode").long("mode").short('m').action(ArgAction::Set)
            .help("Specify the mode to use for the piped buffer (default: fundamental-mode)"))
        .arg(Arg::new("buffer-name").long("buffer-name").short('b').action(ArgAction::Set)
            .help("Specify the name to assign to the buffer containing the piped content (default: pipemacs-input)"))
        .arg(Arg::new("filename").action(ArgAction::Append)
            .help("Files to open in emacs; if this is specified, do not redirect stdin to emacs"))
        .get_matches();

    let args = Arguments {
        no_window: matches.get_flag("no-window"),
        client: matches.get_flag("client"),
        writeback: matches.get_flag("writeback"),
        mode: matches.get_one::<String>("mode").cloned(),
        buffer_name: matches.get_one::<String>("buffer-name").cloned(),
        filenames: matches.get_many("filename").map(|vs| vs.cloned().collect()).unwrap_or_else(|| Vec::new()),
    };

    let server_state = create_listener(&args)?;

    let mut emacs_process = if args.client {
        Command::new("emacsclient")
    } else {
        Command::new("emacs")
    };
    if args.no_window {
        emacs_process.arg("-nw");

        // We have to give emacs a TTY to draw into.  The stdin of this process was redirected to
        // so if we do nothing, the inherited stdin is not a TTY, which will crash terminal emacs.
        let dev_tty = File::options().read(true).write(true).open("/dev/tty")?;
        let stdin: Stdio = dev_tty.into();
        emacs_process.stdin(stdin);
    }

    if let Some(server_state) = &server_state {
        emacs_process.arg("--eval");
        emacs_process.arg(ELISP_LIBRARY);
        emacs_process.arg("--eval");
        emacs_process.arg(call_emacs_entry_point(server_state.bound_port, args.mode.as_ref(), args.buffer_name.as_ref(), args.writeback));
    } else {
        for filename in &args.filenames {
            emacs_process.arg(filename);
        }
    }

    let mut data_thread_handle = None;
    // Bind to any local address, but only if we are going to need to send data over the socket
    if let Some(server_state) = server_state {
        let handle = thread::spawn(move || {
            match feed_data_to_emacs(server_state.listener, args.writeback) {
                Ok(result) => result,
                Err(e) => {
                    panic!("Error feeding data to emacs {}", e);
                }
            }
        });

        data_thread_handle = Some(handle);
    }

    let mut child_emacs = emacs_process.spawn()?;
    child_emacs.wait()?;
    if let Some(handle) = data_thread_handle {
        let result = handle.join().expect("Could not join the data thread");
        // We have to print this out after emacs returns so that control over the tty is returned
        if let Some(output) = result {
            io::stdout().write_all(output.as_bytes())?;
        }
    }

    Ok(())
}
