use crate::Plugin;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;
use reverse_geocoder::Locations;
use reverse_geocoder::ReverseGeocoder;

#[derive(Debug, Clone)]
struct File {
    path: PathBuf,
    timestamp: String,
    gps: Option<rexiv2::GpsInfo>,
}

lazy_static! {
    static ref FILES: Mutex<Vec<File>> = Mutex::new(vec![]);
}

fn link_by_location(
    path: &Path,
    target_dir: &Path,
    latitude: f64,
    longitude: f64,
    geocoder: &ReverseGeocoder,
) {
    match path.file_name() {
        Some(filename) => {
            let y = geocoder
                .search((latitude, longitude))
                .unwrap_or_else(|| panic!("No location found for {},{}", latitude, longitude))
                .record;

            let mut dst_dir = target_dir.to_path_buf();
            dst_dir.push(y.cc.as_str());
            dst_dir.push(y.admin1.as_str());
            dst_dir.push(y.name.as_str());

            let mut dst_file = dst_dir.clone();
            dst_file.push(filename);
            crate::utils::create_link(path, &dst_dir, &dst_file);
        }
        None => {
            println!("Error..");
        }
    }
}

pub fn process(path: &Path, _target_dir: &Path) -> bool {
    match rexiv2::Metadata::new_from_path(path) {
        Ok(meta) => {
            match meta.get_gps_info() {
                None => {
                    if let Ok(time) = meta.get_tag_string("Exif.Image.DateTime") {
                        FILES.lock().unwrap().push(File {
                            path: path.to_owned(),
                            timestamp: time,
                            gps: None,
                        });
                    }
                }
                Some(gps) => {
                    if let Ok(time) = meta.get_tag_string("Exif.Image.DateTime") {
                        FILES.lock().unwrap().push(File {
                            path: path.to_owned(),
                            timestamp: time,
                            gps: Some(gps),
                        });
                    }
                }
            }
            true
        }
        Err(_) => {
            println!("No metadata");
            false
        }
    }
}

pub fn finish(target_dir: &Path) {
    let loc = Locations::from_memory();
    let geocoder = ReverseGeocoder::new(&loc);

    let mut files = FILES.lock().unwrap();
    files.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

    for file in &*files {
        match file.gps {
            Some(gps) => {
                link_by_location(
                    &file.path,
                    target_dir,
                    gps.latitude,
                    gps.longitude,
                    &geocoder,
                );
            }
            None => {
                // No position was found in the exif, let's try to infer the position by checking
                // the files before and after the current file. If they have matching location,
                // then the current file should probably also have the same location (since the
                // files are sorted by date).
                let mut prev_pos: Option<rexiv2::GpsInfo> = None;
                let mut next_pos: Option<rexiv2::GpsInfo> = None;
                let mut is_prev = true;
                for tmp in &*files {
                    // check if we are before or after the current file
                    if tmp.path == file.path {
                        is_prev = false;
                        continue;
                    }

                    if tmp.gps.is_some() {
                        if is_prev {
                            prev_pos = tmp.gps;
                        } else {
                            next_pos = tmp.gps;
                            break;
                        }
                    }
                }
                if let (Some(prev_pos), Some(next_pos)) = (prev_pos, next_pos) {
                    let prev_loc = geocoder
                        .search((prev_pos.latitude, prev_pos.longitude))
                        .unwrap_or_else(|| panic!("No location found for {:?}", prev_pos))
                        .record;
                    let next_loc = geocoder
                        .search((next_pos.latitude, next_pos.longitude))
                        .unwrap_or_else(|| panic!("No location found for {:?}", next_pos))
                        .record;

                    // if country matches
                    if prev_loc.cc == next_loc.cc {
                        let mut dst_dir = target_dir.to_path_buf();
                        dst_dir.push(prev_loc.cc.as_str());
                        // add admin1 if matching
                        if prev_loc.admin1 == next_loc.admin1 {
                            dst_dir.push(prev_loc.admin1.as_str());
                            // add name if matching
                            if prev_loc.name == next_loc.name {
                                dst_dir.push(prev_loc.name.as_str());
                            }
                        }

                        let mut dst_file = dst_dir.clone();
                        dst_file.push(file.path.file_name().unwrap());
                        crate::utils::create_link(&file.path, &dst_dir, &dst_file);
                    } else {
                        println!(
                            "failed to inferred location for {}, prev: {:?}, next: {:?}",
                            file.path.to_string_lossy(),
                            prev_loc,
                            next_loc,
                        );
                        // files before and after didn't have matching locations
                        crate::utils::link_by_location_unknown(&file.path, target_dir);
                    }
                } else {
                    // files before and/or after were missing position
                    crate::utils::link_by_location_unknown(&file.path, target_dir);
                }
            }
        }
    }
    println!("Processed {} files", files.len());
}

pub fn setup() -> Plugin {
    Plugin {
        pattern: "jpg".to_string(),
        handle: process,
        finish,
    }
}
