use std::env::current_dir;
use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;

use notify::event::ModifyKind::{Data, Name};
use notify::event::{RemoveKind, RenameMode};
use notify::EventKind::{Create, Modify, Remove};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use super::{compile_specific, construct_target_paths, TARGET};

const FOLDER: &str = "root/apps";

// MaybeWatcher lets start and stop be idempotent operations. Pass in whatever you've got, and the
// functions will figure out what needs to happen.
pub type MaybeWatcher = Option<RecommendedWatcher>;

pub fn start(maybe_watcher: MaybeWatcher) -> MaybeWatcher {
    match maybe_watcher {
        Some(watcher) => Some(watcher),
        None => {
            let mut watcher: RecommendedWatcher = Watcher::new_immediate(|res| match res {
                Ok(event) => handle_event(event),
                Err(e) => println!("watch error: {:?}", e),
            })
            .unwrap();

            watcher.watch(FOLDER, RecursiveMode::Recursive).unwrap();

            Some(watcher)
        }
    }
}

pub fn stop(maybe_watcher: MaybeWatcher) -> MaybeWatcher {
    match maybe_watcher {
        Some(mut watcher) => {
            watcher
                .unwatch(FOLDER)
                .expect("File watcher in inconsistent state.");
            None
        }
        None => None,
    }
}

fn handle_event(event: Event) {
    let local_path = event.paths[0]
        .strip_prefix(current_dir().expect("Cannot access current dir."))
        .expect("Application directory invalid.");

    let is_in_target = local_path.iter().skip(3).next().unwrap_or("".as_ref()) == TARGET;

    if !is_in_target {
        let local_path = local_path.to_path_buf();
        match event.kind {
            Remove(RemoveKind::File) => delete_file(local_path),
            Remove(RemoveKind::Folder) => delete_folder(local_path),
            Modify(Name(RenameMode::From)) => delete_file(local_path),
            Modify(Name(RenameMode::To)) => recompile(local_path),
            Modify(Data(_)) => recompile(local_path),
            Create(_) => recompile(local_path),
            _ => (),
        };
    }
}

fn delete_file(local_path: PathBuf) {
    let paths = construct_target_paths(&local_path);
    println!("Deleting: {:?}", paths.server);
    println!("Deleting: {:?}", paths.client);
    if let Err(e) = remove_file(paths.server) {
        eprintln!("{:?}", e);
    };
    if let Err(e) = remove_file(paths.client) {
        eprintln!("{:?}", e);
    };
}

// This gets called on a folder rename too. Catch folders and compile them differently. Probably
// via another file walker TODO
fn recompile(local_path: PathBuf) {
    println!("Compiling: {:?}", &local_path);
    compile_specific(&local_path);
}

fn delete_folder(local_path: PathBuf) {
    let paths = construct_target_paths(&local_path);
    println!("Deleting: {:?}", paths.server);
    println!("Deleting: {:?}", paths.client);
    if let Err(e) = remove_dir_all(paths.server) {
        eprintln!("{:?}", e);
    };
    if let Err(e) = remove_dir_all(paths.client) {
        eprintln!("{:?}", e);
    };
}
