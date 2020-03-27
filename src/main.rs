#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::env;
use std::fs::{copy, create_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use regex::Regex;

static FILE_FOLDER: &str = "files/";

fn process_md(path_buf: &mut PathBuf) {
    if !path_buf.exists() {
        return;
    }
    let mut file_dir = path_buf.clone();
    let mut file_dir_rel = PathBuf::from("./");
    let pop = file_dir.pop();
    assert_eq!(pop, true);
    file_dir.push(FILE_FOLDER);
    file_dir_rel.push(FILE_FOLDER);

    let mut file = match File::open(path_buf.clone()) {
        Ok(file) => file,
        Err(_) => return,
    };

    lazy_static! {
        static ref FILE_LINK: Regex = Regex::new(r"\[(.*)\]\((.*)\)").unwrap();
    }
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        println!(
            "Processing {} failed: read_to_string",
            path_buf.to_string_lossy()
        );
        return;
    }

    if !file_dir.exists() {
        if create_dir(&file_dir).is_err() {
            println!(
                "Processing {} failed: create_dir",
                path_buf.to_string_lossy()
            );
            return;
        }
    }

    let mut modified = false;
    let mut contents_new = contents.clone();
    for cap in FILE_LINK.captures_iter(contents.as_str()) {
        let original = cap.get(0).unwrap().as_str();
        let text = cap.get(1).map_or("", |m| m.as_str());
        let link = match cap.get(2) {
            Some(link) => link.as_str(),
            None => continue,
        };

        if link.starts_with(FILE_FOLDER) {
            continue;
        }

        if link.starts_with("http") {
            continue;
        }
        let old_path = Path::new(link);

        if !old_path.exists() {
            continue;
        }

        let file_name = match old_path.file_name() {
            Some(name) => name,
            None => continue,
        };

        let file_stem = match old_path.file_stem() {
            Some(stem) => stem,
            None => continue,
        };

        let extension = match old_path.extension() {
            Some(ext) => ext,
            None => continue,
        };

        let mut file_path = file_dir.clone();
        let mut file_path_rel = file_dir_rel.clone();

        file_path.push(file_name);
        file_path_rel.push(file_name);
        let mut num = 1;
        while file_path.exists() {
            let new_file_name = format!(
                "{}_{}.{}",
                file_stem.to_string_lossy(),
                num.to_string(),
                extension.to_string_lossy()
            );

            file_path.set_file_name(&new_file_name);
            file_path_rel.set_file_name(&new_file_name);
            num += 1;
        }

        println!(
            "{} -> {}",
            old_path.to_str().unwrap(),
            file_path.to_str().unwrap()
        );
        if copy(old_path, file_path).is_err() {
            println!("Processing {} failed: copy", path_buf.to_string_lossy());
            continue;
        }
        contents_new = contents_new.replace(
            original,
            format!("[{}]({})", text, file_path_rel.to_str().unwrap()).as_str(),
        );
        modified = true;
    }
    if modified {
        let mut dst = File::create(path_buf.clone()).unwrap();
        if dst.write(contents_new.as_bytes()).is_err() {
            println!("Processing {} failed: write", path_buf.to_string_lossy());
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let key = "WATCHMAN_ROOT";
    let root = match env::var(key) {
        Ok(val) => PathBuf::from(val),
        Err(_) => PathBuf::from("."),
    };

    for arg in &args[1..] {
        let mut path = root.clone();
        path.push(arg);
        process_md(&mut path)
    }
}
