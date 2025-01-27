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
