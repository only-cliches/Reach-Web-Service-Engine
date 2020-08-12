use walkdir::WalkDir;
use std::path::{ Path, PathBuf, Component };
use std::fs::{ File, create_dir_all };
use std::io::prelude::*;
use std::ffi::OsStr;

use crate::tsc_processor;

const COMPILE_EXTS: [&str; 3] = ["ts", "tsx", "jsx"];
const SRC: &str = "src";
const DST: &str = "dst";

pub fn compile_all() {
    for maybe_entry in WalkDir::new("apps") {
        if let Ok(entry) = maybe_entry {
            let is_in_src = entry
                .path()
                .iter()
                .any(|s| s.to_str().unwrap_or("") == "src");
            let needs_compilation: bool = entry.path().extension().map_or(false, |ext| {
                ext.to_str()
                    .map_or(false, |s| COMPILE_EXTS.iter().any(|x| x == &s))
            });
            if is_in_src && needs_compilation {
                println!("Compiling: {}", entry.path().display());
                compile_specific(entry.path());
            }
        }
    }
}

pub fn compile_specific(path: &Path) {

    let mut destination: PathBuf = path.iter().map(|component| {
        if component == &SRC.as_ref() {
            DST.as_ref()
        } else {
            component
        }
    }).collect();

    destination.set_extension(".js");

    let destination = destination; // Close destination to mutation.

    match File::open(path) {
        Ok(mut f) => {
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();


            create_dir_all(destination.parent().unwrap()).unwrap();


            let mut outfile = File::create(destination).unwrap();

            outfile.write_all(content.as_bytes());
        },
        Err(error) => {
            println!("Cannot open {}: {}", path.display(), error);
        }
    }


}
