use std::{convert::TryFrom, fs, path::Path};

use datasets::load_provinces;
use geojson::{FeatureCollection, GeoJson};

mod labeling;
mod datasets;

use labeling::{country_benchmark, province_benchmark};

fn main() {
    let provinces = load_provinces(Path::new("data\\ne_10m_admin_1_states_provinces.json"));
    province_benchmark(&provinces);
}
