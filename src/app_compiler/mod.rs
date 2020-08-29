pub mod file_watcher;
mod tsc_processor;

use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use std::fs::{copy, create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

const COMPILE_EXTS: [&str; 3] = ["ts", "tsx", "jsx"];
pub const TARGET: &str = "target";
const SERVER: &str = "server";
const CLIENT: &str = "client";

pub fn compile_all() {
    let started = Instant::now();

    let mut walker = WalkDir::new("apps").min_depth(2).into_iter();

    // Not the most efficient, but WalkDir is live. Walking the directory fully before processing
    // puts a nice stopper in case of accidental recursion.
    let mut list = vec![];

    while let Some(Ok(entry)) = walker.next() {
        let file_type = entry.file_type();
        let path = entry.path();

        let is_in_target = path.iter().skip(2).next().unwrap_or("".as_ref()) == TARGET;

        if file_type.is_file() && !is_in_target {
            list.push(path.to_path_buf())
        }
    }

    // Show a progress spinner and a final duration once everything's compiled.
    let spinner_style = ProgressStyle::default_spinner().template("Compiling {wide_msg}");

    let count: u64 = list.len() as u64;
    let pb = ProgressBar::new(count);
    pb.set_style(spinner_style);

    for path in list.iter() {
        pb.set_message(&path.to_string_lossy());
        pb.inc(1);

        compile_specific(path);
    }

    let completed = format!("completed in {}", HumanDuration(started.elapsed()));

    pb.finish_with_message(&completed);
}

pub struct Targets {
    server: PathBuf,
    client: PathBuf,
}

fn needs_compilation(path: &Path) -> bool {
    path.extension().map_or(false, |ext| {
        ext.to_str()
            .map_or(false, |s| COMPILE_EXTS.iter().any(|x| x == &s))
    })
}

// Accepts a relative path to the root directory.
pub fn construct_target_paths(path_buf: &PathBuf) -> Targets {
    let path = path_buf;

    let mut server = PathBuf::new();
    let mut client = PathBuf::new();

    let mut path_iter = path.iter();

    let apps_folder = path_iter.next().unwrap();

    server.push(apps_folder);
    client.push(apps_folder);

    let app = path_iter.next().unwrap();

    server.push(app);
    client.push(app);

    server.push(TARGET);
    client.push(TARGET);

    server.push(SERVER);
    client.push(CLIENT);

    while let Some(component) = path_iter.next() {
        server.push(component);
        client.push(component);
    }

    if needs_compilation(&path) {
        server.set_extension("js");
        client.set_extension("js");
    }

    Targets { server, client }
}

// Accepts a relative path to the root directory.
pub fn compile_specific(path: &PathBuf) {
    let target = construct_target_paths(path);

    create_dir_all(target.server.parent().unwrap()).unwrap();
    create_dir_all(target.client.parent().unwrap()).unwrap();

    if needs_compilation(&path) {
        match File::open(path) {
            Ok(mut f) => {
                let mut content = String::new();
                f.read_to_string(&mut content).unwrap();

                let mut server_outfile = File::create(target.server).unwrap();
                let mut client_outfile = File::create(target.client).unwrap();

                let compiled = tsc_processor::run(content);

                server_outfile.write_all(compiled.as_bytes()).unwrap();
                client_outfile.write_all(compiled.as_bytes()).unwrap();
            }
            Err(error) => {
                println!("Cannot open {}: {}", path.display(), error);
            }
        }
    } else {
        copy(path, target.server).unwrap();
        copy(path, target.client).unwrap();
    }
}
