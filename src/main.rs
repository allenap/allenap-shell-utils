#[macro_use]
extern crate clap;
extern crate dirs;
extern crate notify;

use std::process;

mod clean_path;
mod watch;

fn main() {
    let matches = clap::App::new("allenap-shell-utils")
        .version(crate_version!())
        .author(crate_authors!())
        .about("allenap's shell utilities.")
        .after_help("Doing shell-like things better and faster.")
        .subcommand(clean_path::argspec())
        .subcommand(watch::argspec())
        .get_matches();

    process::exit(match matches.subcommand() {
        (clean_path::NAME, Some(submatches)) => clean_path::run(submatches),
        (watch::NAME, Some(submatches)) => watch::run(submatches),
        (_, _) => {
            // We'll only get here if no subcommand was given; a subcommand with
            // an unrecognised name is picked up by `App.get_matches()`.
            eprintln!("{}", matches.usage());
            1
        }
    })
}
