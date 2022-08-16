use std::env;
use std::fmt;
use std::process;
use std::fs;
use std::path::Path;

mod exif;
mod utils;
use reverse_geocoder::Locations;
use reverse_geocoder::ReverseGeocoder;

pub struct Plugin {
    pattern: String,
    handle: fn(&Path, &Path, &ReverseGeocoder) -> bool,
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <input directory> <output directory>", &args[0]);
        process::exit(0);
    }

    let dir_in = &args[1];
    println!("Using in dir: {}", dir_in);

    let dir_out = &args[2];
    println!("Using out dir: {}", dir_out);
    let dir_out_path = Path::new(dir_out);

    let paths = match fs::read_dir(dir_in) {
        Ok(paths) => paths,
        Err(_) => {
            println!("'{}' is not a directory", dir_in);
            process::exit(0);
        }
    };

    let loc = Locations::from_memory();
    let geocoder = ReverseGeocoder::new(&loc);

    let mut plugins = Vec::new();

    plugins.push(exif::setup());

    for entry in paths {
        if let Ok(entry) = entry {
            let path = entry.path();
            let ext = match path.extension() {
                Some(ext) => ext,
                None => continue
            };
            let ext_str = match ext.to_str() {
                Some(ext_str) => ext_str.to_lowercase(),
                None => continue
            };
            for p in &plugins {
                if p.pattern.to_lowercase() == ext_str {
                    (p.handle)(path.as_path(), dir_out_path, &geocoder);
                }
            }
        }
    }
}
