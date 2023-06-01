use std::collections;
use std::env;
use std::error;
use std::ffi;
use std::ffi::OsStr;
use std::path::{self, PathBuf};
use std::process;

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
        #[arg(short, long, env = "PATH", hide_env_values = true)]
        path: PathBuf,
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

fn clean_path<T>(path: T) -> Result<std::ffi::OsString, impl error::Error>
where
    T: AsRef<OsStr>,
{
    let paths: Vec<PathBuf> = env::split_paths(&path).filter_map(expand_path).collect();
    let mut unseen: collections::HashSet<&PathBuf> = paths.iter().collect();
    env::join_paths(paths.iter().filter(|p| unseen.remove(p) && p.exists()))
}

fn expand_path(path: PathBuf) -> Option<path::PathBuf> {
    let mut abspath = PathBuf::new();
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
