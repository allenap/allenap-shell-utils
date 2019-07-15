use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub const NAME: &str = "watch";

pub fn argspec<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name(NAME)
        .about("Watch for changes in files or directories")
        .arg(
            clap::Arg::with_name("paths")
                .value_name("PATH")
                .multiple(true)
                .help("The files or directories to watch"),
        )
}

pub fn run(args: &clap::ArgMatches) -> i32 {
    let paths = args.values_of("paths").unwrap();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    for path in paths {
        watcher.watch(path, RecursiveMode::Recursive).unwrap();
    }

    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
