use std::os::unix::fs;
use std::path::Path;

pub fn link_by_location_unknown(path: &Path, target_dir: &Path) -> bool {
    match path.file_name() {
        Some(filename) => {
            let dst_dir = target_dir.join("unknown_location");
            let mut dst_file = dst_dir.clone();
            dst_file.push(filename);
            create_link(path, &dst_dir, &dst_file);
            true
        }
        None => {
            println!("Error..");
            false
        }
    }
}

pub fn create_link(src: &Path, dst_dir: &Path, dst_file: &Path) {
    match std::fs::create_dir_all(dst_dir) {
        Ok(_) => {}
        Err(e) => println!("failed to create dir: {}", e),
    }
    // println!("{} => {}", src.display(), dst_file.display());
    match fs::symlink(src, dst_file) {
        Ok(_) => {}
        Err(e) => println!("failed to create symlink: {}", e),
    }
}
