/// This script demonstrates the usage of the pinpointer library for performing point-in-country queries.
///
/// It performs the following steps:
/// 1. Downloads the required country map data if it is not already available.
/// 2. Builds a labeled partition tree for countries based on the downloaded map data.
/// 3. Generates a list of random latitude-longitude coordinates.
/// 4. Queries the country label for each coordinate using the partition tree.

use std::{path::Path, time::Instant};

use geo_types::Point;
use pinpointer::datasets::{lazy_download_map_data, load_or_compute_country_label_tree};
use rand::Rng;

pub fn main() {
    lazy_download_map_data().unwrap(); // make sure we can access the country maps we need

    // build a label tree so we can do point-in-country queries (should take about 1 minute)
    let country_label_tree = load_or_compute_country_label_tree(
        Path::new("data"),
        Path::new("data\\ne_10m_admin_0_countries_lakes.geojson"),
        6,
    );

    let mut rng = rand::thread_rng();
    let latlons: Vec<(f64, f64)> = (0..10000000)
        .map(|_| (rng.gen_range(-180.0..180.0), rng.gen_range(-90.0..90.0)))
        .collect();

    // query 10,000,000 country codes (should take about 4 seconds)
    let t0 = Instant::now();
    let mut labels = vec![];
    for (lat, lon) in latlons.iter() {
        let label = country_label_tree.label(&Point::new(*lon, *lat));
        labels.push(label);
    }

    let duration = t0.elapsed().as_secs_f64();

    println!(
        "{} point-in-country queries completed in {:.4} seconds ({:.2} queries per second).",
        latlons.len(),
        duration,
        latlons.len() as f64 / duration
    );
}
