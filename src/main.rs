// spell-checker: ignore abspath

use std::{
    collections, env, error, ffi,
    io::{self, Write},
    path, process,
};

use anyhow::{anyhow, Context as _, Result};
use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(
    author, version, about,
    after_help = "Doing shell-like things better and faster.",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cleans a PATH-like string
    CleanPath {
        /// The PATH-like string to clean
        #[arg(env = "PATH", hide_env_values = true)]
        path: ffi::OsString,
    },
    /// Watch for changes in files or directories
    ///
    /// Events are emitted as JSON, with one object per line. This can be
    /// readily processed by `jq` or `jaq`, e.g.: `allenap-shell-utils watch . |
    /// jaq .paths`
    Watch {
        /// The files or directories to watch
        paths: Vec<path::PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    process::exit(match cli.command {
        Commands::CleanPath { path } => match clean_path(path) {
            Ok(path) => {
                println!(
                    "{}",
                    path.into_string()
                        .map_err(|path| anyhow!("Could not convert path {path:?} to UTF-8"))?
                );
                0
            }
            Err(err) => {
                eprintln!("{}", err);
                2
            }
        },
        Commands::Watch { paths } => {
            use notify::Watcher;

            // Create a channel to receive the events. Sending `None` denotes
            // the end of the stream; the receiver should not attempt to receive
            // any more messages.
            let (tx, rx) = std::sync::mpsc::channel::<Option<notify::Result<notify::Event>>>();

            // Handle Ctrl-C to stop watching.
            ctrlc::set_handler({
                let tx = tx.clone();
                move || tx.send(None).unwrap()
            })?;

            // Create a watcher object, delivering debounced events.
            // The notification back-end is selected based on the platform.
            let mut watcher: notify::RecommendedWatcher = notify::recommended_watcher(move |res| {
                tx.send(Some(res)).unwrap_or(()) // Ignore errors when sending to `tx`.
            })
            .context("Could not create watcher")?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            for path in paths.iter() {
                watcher
                    .watch(path, notify::RecursiveMode::Recursive)
                    .with_context(|| format!("Could not watch {path:?}"))?
            }

            // Loop over events until asked to stop. Errors from `watcher` are
            // printed to `stderr` but otherwise ignored.
            let mut stdout = io::stdout().lock();
            loop {
                match rx.recv()? {
                    None => break 0, // Stop requested, probably Ctrl-C; exit loop cleanly.
                    Some(Err(error)) => eprintln!("Error: {error}"),
                    Some(Ok(event)) => {
                        serde_json::to_writer(&mut stdout, &event)?;
                        stdout.write_all(b"\n")?;
                        stdout.flush()?;
                    }
                }
            }
        }
    })
}

fn clean_path<T>(path: T) -> Result<ffi::OsString, impl error::Error>
where
    T: AsRef<ffi::OsStr>,
{
    let paths: Vec<path::PathBuf> = env::split_paths(&path).filter_map(expand_path).collect();
    let mut unseen: collections::HashSet<&path::PathBuf> = paths.iter().collect();
    env::join_paths(paths.iter().filter(|p| unseen.remove(p) && p.exists()))
}

fn expand_path(path: path::PathBuf) -> Option<path::PathBuf> {
    let mut abspath = path::PathBuf::new();
    for (index, component) in path.components().enumerate() {
        match component {
            path::Component::CurDir => {
                if index == 0 {
                    abspath.push(env::current_dir().ok()?)
                }
            }
            path::Component::Normal(element) => {
                if index == 0 && element == ffi::OsStr::new("~") {
                    abspath.push(dirs::home_dir()?)
                } else {
                    abspath.push(element)
                }
            }
            other => abspath.push(other),
        }
    }
    Some(abspath)
}
