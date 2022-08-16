extern crate kdtree;
extern crate time;
extern crate rexiv2;

use reverse_geocoder::ReverseGeocoder;

use std::path::Path;
use std::os::unix::fs;

pub fn link_by_location_unknown(path: &Path, target_dir: &Path) -> bool {
    match path.file_name() {
        Some(filename) => {
            let dst_dir = target_dir.join("unknown_location");
            let mut dst_file = dst_dir.clone();
            dst_file.push(filename);
            create_link(&path, &dst_dir, &dst_file);
            return true;
        }
        None => {
            println!("Error..");
            return false;
        }
    }
}

pub fn link_by_location(
    path: &Path,
    target_dir: &Path,
    latitude: f64,
    longitude: f64,
    geocoder: &ReverseGeocoder)
{
    match path.file_name() {
        Some(filename) => {
            let y = geocoder.search((latitude, longitude)).expect("Nothing found.").record;

            let mut dst_dir = target_dir.to_path_buf();
            dst_dir.push(y.cc.as_str());
            dst_dir.push(y.admin1.as_str());
            dst_dir.push(y.name.as_str());

            let mut dst_file = dst_dir.clone();
            dst_file.push(filename);
            create_link(&path, &dst_dir, &dst_file);
        },
        None => {
            println!("Error..");
        }
    }
}

fn create_link(
    src: &Path,
    dst_dir: &Path, dst_file: &Path)
{
    match std::fs::create_dir_all(dst_dir) {
        Ok(_) => {},
        Err(e) => println!("failed to create dir: {}", e)
    }
    println!("{} => {}", src.display(), dst_file.display());
    match fs::symlink(src, dst_file) {
        Ok(_) => {},
        Err(e) => println!("failed to create symlink: {}", e)
    }
}
