use std::{collections, env, error, ffi, path, process};

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
    Watch {
        /// The files or directories to watch
        paths: Vec<path::PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    process::exit(match cli.command {
        Commands::CleanPath { path } => match clean_path(path) {
            Ok(path) => {
                println!("{}", path.to_string_lossy());
                0
            }
            Err(err) => {
                eprintln!("{}", err);
                2
            }
        },
        Commands::Watch { paths } => {
            use notify::Watcher;

            // Create a channel to receive the events.
            let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();

            // Create a watcher object, delivering debounced events.
            // The notification back-end is selected based on the platform.
            let mut watcher: notify::RecommendedWatcher =
                notify::recommended_watcher(move |res| tx.send(res).unwrap()).unwrap();

            eprintln!("Watcher: {:?}", watcher);

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            for path in paths {
                watcher
                    .watch(&path, notify::RecursiveMode::Recursive)
                    .unwrap();
            }

            loop {
                match rx.recv() {
                    Ok(event) => println!("{:?}", event),
                    Err(e) => println!("watch error: {:?}", e),
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
