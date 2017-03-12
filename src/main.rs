extern crate yaml_rust;
use yaml_rust::{yaml, Yaml};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use std::io::BufReader;
use std::io::BufRead;

#[derive(Debug)]
struct Bookmark<'a> {
    local_bookmark_path: &'a Path,
    shared_bookmark_path: &'a Path,
    projects: Vec<Project<'a>>,
}

#[derive(Debug)]
struct Project<'a> {
    name: &'a str,
    directory: &'a Path,
}

fn main() {
    match load_from_file() {
        Ok(yaml_vec) => {
            let yaml = &yaml_vec[0];
            let bookmark = build_bookmark(yaml);
            match bookmark {
                Some(b) => {
                    sync_bookmarks(b);
                }
                None => {}
            }
        }
        Err(err) => println!("{}", err),
    }
}

fn build_bookmark<'a>(yaml: &'a Yaml) -> Option<Bookmark> {
    match yaml["projects"].as_vec() {
        Some(projects) => {
            let ps: Vec<_> = projects.into_iter()
                .map(|p| {
                    Project {
                        name: p["name"].as_str().unwrap_or("default"),
                        directory: Path::new(p["directory"]
                            .as_str()
                            .unwrap_or("/")),
                    }
                })
                .collect();
            let bookmark = Bookmark {
                local_bookmark_path: Path::new(yaml["local_bookmark_repository"]
                    .as_str()
                    .unwrap_or("/")),
                shared_bookmark_path: Path::new(yaml["shared_bookmark_repository"]
                    .as_str()
                    .unwrap_or("/")),
                projects: ps,
            };
            Some(bookmark)
        }
        None => {
            println!("there is no project");
            None
        }
    }
}

fn load_from_file() -> Result<Vec<Yaml>, String> {
    env::home_dir()
        .ok_or("no home directory".to_owned())
        .and_then(|mut home| {
            home.push(".unite_bookmark_sync.yml");
            Ok(home)
        })
        .and_then(|path| File::open(path).map_err(|err| err.to_string()))
        .map_err(|err| err.to_string())
        .and_then(|mut file| {
            let mut yaml_string = String::new();
            file.read_to_string(&mut yaml_string)
                .map_err(|err| err.to_string())
                .map(|_| yaml_string)
        })
        .and_then(|mut yaml_string| {
            yaml::YamlLoader::load_from_str(&mut yaml_string).map_err(|err| err.to_string())
        })
}

fn sync_bookmarks(bookmark: Bookmark) {
    for project in bookmark.projects {
        let shared_file_name = format!("{}/{}",
                                       bookmark.shared_bookmark_path.to_str().unwrap(),
                                       project.name);
        let local_file_name = format!("{}/{}",
                                      bookmark.local_bookmark_path.to_str().unwrap(),
                                      project.name);
        let mut shared_file = File::create(shared_file_name).unwrap();
        match File::open(local_file_name) {
            Ok(f) => {
                let mut file = BufReader::new(&f);
                let mut lines = file.lines();
                lines.next();
                shared_file.write_fmt(format_args!("0.1.0\n"));
                for line in lines {
                    match line {
                        Ok(l) => {
                            let bookmark_info: Vec<&str> = l.split('\t').collect();
                            let bookmarked_file = bookmark_info[1]
                                .to_string()
                                .replacen(project.directory.to_str().unwrap(), "", 1);
                            shared_file.write_fmt(format_args!("{}\t{}\t{}\t{}\n",
                                                               bookmark_info[0],
                                                               bookmarked_file,
                                                               bookmark_info[2],
                                                               bookmark_info[3]));
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {
                continue;
            }
        }
    }
}
