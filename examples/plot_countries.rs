/// This script demonstrates the usage of the pinpointer library for generating a country label tree.
///
/// It performs the following steps:
/// 1. Downloads the required country map data if it is not already available.
/// 2. Builds a labeled partition tree for countries based on the downloaded map data.
/// 3. Plots the map with the label tree overlaid on top.
use std::path::Path;

use pinpointer::datasets::{lazy_download_map_data, load_or_compute_country_label_tree};

pub fn main() {
    lazy_download_map_data().unwrap(); // make sure we can access the country maps we need

    for i in 1..10 {
        // build a label tree and plot it
        let country_label_tree = load_or_compute_country_label_tree(
            Path::new("data"),
            Path::new("data/ne_10m_admin_0_countries_lakes.geojson"),
            i,
        );

        country_label_tree
            .plot(Path::new(&format!("countries_map_{}.png", i)))
            .unwrap();
    }
}
