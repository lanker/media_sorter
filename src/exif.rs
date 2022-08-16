use crate::Plugin;
use std::path::Path;

use reverse_geocoder::ReverseGeocoder;

pub fn process(path: &Path, target_dir: &Path, geocoder: &ReverseGeocoder) -> bool {
    match rexiv2::Metadata::new_from_path(path) {
        Ok(meta) => {
            match meta.get_gps_info() {
                None => {
                    // no gps info in exif
                    crate::utils::link_by_location_unknown(&path, target_dir);
                },
                Some(gps) => {
                    crate::utils::link_by_location(
                        &path, target_dir, gps.latitude, gps.longitude, geocoder);
                }
            }
        },
        Err(_) => {
            println!("No metadata");
            return false;
        }
    }
    return true;
}

pub fn setup() -> Plugin {
    Plugin { pattern: "jpg".to_string(), handle: process }
}
