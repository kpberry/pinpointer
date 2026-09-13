/// This script demonstrates the usage of the pinpointer library for generating a country label tree.
///
/// It performs the following steps:
/// 1. Downloads the required country map data if it is not already available.
/// 2. Builds a labeled partition tree for countries based on the downloaded map data.
/// 3. Plots the map with the label tree overlaid on top.
use std::path::Path;

use pinpointer::datasets::MapLoader;

pub fn main() {
    let mut loader = MapLoader::new(Path::new("data"));

    for i in 1..10 {
        // build a label tree and plot it
        let country_label_tree = loader.countries_label_tree(i);

        country_label_tree
            .plot(Path::new(&format!("countries_map_{}.png", i)))
            .unwrap();
    }
}
