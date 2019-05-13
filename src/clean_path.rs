use std::collections;
use std::env;
use std::error;
use std::ffi;
use std::path::{self, PathBuf};

pub const NAME: &str = "clean-path";

pub fn argspec<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name(NAME)
        .about("cleans a PATH-like string")
        .arg(
            clap::Arg::with_name("path")
                .value_name("PATH")
                .env("PATH")
                .hide_env_values(true)
                .help("The PATH-like string to clean"),
        )
}

pub fn run(args: &clap::ArgMatches) -> i32 {
    let path = args.value_of("path").unwrap();
    match clean_path(path) {
        Ok(path) => {
            println!("{}", path.to_string_lossy());
            0
        }
        Err(err) => {
            eprintln!("{}", err);
            2
        }
    }
}

fn clean_path(path: &str) -> Result<std::ffi::OsString, impl error::Error> {
    let paths: Vec<PathBuf> = env::split_paths(path)
        .filter_map(|pathbuf| expand_path(pathbuf))
        .collect();
    let mut unseen: collections::HashSet<&PathBuf> = paths.iter().collect();
    env::join_paths(
        paths
            .iter()
            .filter(|pathbuf| unseen.remove(pathbuf) && pathbuf.exists()),
    )
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
