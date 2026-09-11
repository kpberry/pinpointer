/// This script demonstrates the usage of the pinpointer library for generating a province label tree.
///
/// It performs the following steps:
/// 1. Downloads the required province map data if it is not already available.
/// 2. Builds a labeled partition tree for countries based on the downloaded map data.
/// 3. Plots the map with the label tree overlaid on top.
use std::path::Path;

use pinpointer::datasets::{lazy_download_map_data, load_or_compute_province_label_tree};

pub fn main() {
    lazy_download_map_data().unwrap(); // make sure we can access the province maps we need

    for i in 1..10 {
        // build a label tree and plot it
        let province_label_tree = load_or_compute_province_label_tree(
            Path::new("data"),
            Path::new("data/ne_10m_admin_1_states_provinces_lakes.geojson"),
            i,
        );

        province_label_tree
            .plot(Path::new(&format!("provinces_map_{}.png", i)))
            .unwrap();
    }
}
