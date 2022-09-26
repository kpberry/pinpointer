use std::{convert::TryFrom, fs, path::Path};

use geojson::{FeatureCollection, GeoJson};

mod labeling;

use labeling::{country_benchmark};

fn main() {
    let geojson_str =
        fs::read_to_string(Path::new("data\\ne_10m_admin_0_countries_lakes.json")).unwrap();
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    let countries: FeatureCollection = FeatureCollection::try_from(geojson).unwrap();
    country_benchmark(&countries);
}
