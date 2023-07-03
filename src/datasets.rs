use std::{collections::HashMap, fs, path::Path};

use geo::{MultiPolygon, Point, Polygon, Rect};
use geojson::{FeatureCollection, GeoJson};

use crate::labeling::LabeledPartitionTree;

use reqwest::blocking::get;
use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;

/// Downloads map data lazily if it doesn't exist in the specified directory.
///
/// # Errors
///
/// Returns an error if there is an issue with downloading or writing the files.
pub fn lazy_download_map_data() -> Result<(), Box<dyn std::error::Error>> {
    let filenames = vec![
        "ne_10m_admin_0_countries_lakes",
        "ne_10m_admin_1_states_provinces",
    ];
    for filename in filenames {
        let output_filename = &format!("data/{}.json", filename);
        let output_path = Path::new(output_filename);
        if !output_path.exists() {
            let url = format!(
                "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/{}.geojson",
                filename
            );
            let data = get(&url)?.bytes()?;

            let mut file = File::create(&output_path)?;
            file.write_all(&data)?;
        }
    }

    Ok(())
}


/// Loads labeled polygons from a GeoJSON file and returns them as a HashMap.
///
/// # Arguments
///
/// * `path` - The path to the GeoJSON file.
/// * `label` - The property to use as the label for the polygons.
///
/// # Returns
///
/// A HashMap where the keys are labels and the values are MultiPolygons.
pub fn load_labeled_collection_polygons(path: &Path, label: &str) -> HashMap<String, MultiPolygon> {
    let geojson_str = fs::read_to_string(path).unwrap();
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    let collection: FeatureCollection = FeatureCollection::try_from(geojson).unwrap();

    let mut labeled_polygons: HashMap<String, Vec<Polygon>> = HashMap::new();
    collection.features.iter().for_each(|region| {
        let name = region
            .property(label)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        if name != "-99" {
            let geometry = region.geometry.as_ref().unwrap();
            let mut polygons: Vec<Polygon> = vec![];
            if let Ok(polygon) = Polygon::try_from(geometry) {
                polygons = vec![polygon];
            }
            if let Ok(multi_polygon) = MultiPolygon::try_from(geometry) {
                polygons.extend(multi_polygon)
            }
            labeled_polygons
                .entry(name)
                .or_insert(Vec::new())
                .extend(polygons);
        }
    });

    labeled_polygons
        .iter()
        .map(|(name, polygons)| (name.clone(), MultiPolygon::new(polygons.clone())))
        .collect()
}

/// Loads a HashMap from ISO_A2 country names to their borders from a GeoJSON file.
/// 
/// # Arguments
///
/// * `path` - The path to the GeoJSON file.
///
/// # Returns
///
/// A HashMap where each key is a country name and the value is the MultiPolygons representing
/// the country's borders. 
pub fn load_countries(path: &Path) -> HashMap<String, MultiPolygon> {
    load_labeled_collection_polygons(path, "ISO_A2")
}

/// Loads a HashMap from iso_3166_2 province names to their borders from a GeoJSON file.
/// 
/// # Arguments
///
/// * `path` - The path to the GeoJSON file.
///
/// # Returns
///
/// A HashMap where each key is a province name and the value is the MultiPolygons representing
/// the province's borders. 
pub fn load_provinces(path: &Path) -> HashMap<String, MultiPolygon> {
    load_labeled_collection_polygons(path, "iso_3166_2")
}


/// Loads or computes a labeled partition tree from the given GeoJSON file and property label.
/// If a cached version of the tree exists, it is loaded; otherwise, the tree is computed from scratch and saved.
///
/// # Arguments
///
/// * `cache_dir` - The directory where the tree cache will be stored.
/// * `collection_path` - The path to the GeoJSON file.
/// * `label` - The property to use as the label for the polygons.
/// * `max_depth` - The maximum depth of the partition tree.
///
/// # Returns
///
/// A `LabeledPartitionTree` with labels of type `String`.
pub fn load_or_compute_label_tree(
    cache_dir: &Path,
    collection_path: &Path,
    label: &str,
    max_depth: usize,
) -> LabeledPartitionTree<String> {
    let cache_path = cache_dir.join(format!("{label}_label_tree_{max_depth}.json"));
    let tree = match fs::read_to_string(&cache_path) {
        Ok(string) => serde_json::from_str(&string).unwrap(),
        Err(e) => {
            println!("{e}");
            println!("Could not load saved {label} label tree; computing from scratch.");
            let collection = load_labeled_collection_polygons(collection_path, label);
            let tree = LabeledPartitionTree::from_labeled_polygons(
                &collection.keys().cloned().collect(),
                &collection,
                Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
                max_depth,
                0,
            );
            let tree_json = serde_json::to_string(&tree).unwrap();
            fs::write(cache_path, tree_json).unwrap();
            tree
        }
    };
    println!("Loaded {label} label tree.");
    tree
}

/// Loads or computes a labeled country partition tree.
/// If a cached version of the tree exists, it is loaded; otherwise, the tree is computed from scratch and saved.
///
/// # Arguments
///
/// * `cache_dir` - The directory where the tree cache will be stored.
/// * `countries_path` - The path to the GeoJSON file containing country data.
/// * `max_depth` - The maximum depth of the partition tree.
///
/// # Returns
///
/// A `LabeledPartitionTree` with ISO_A2 country names as labels.
pub fn load_or_compute_country_label_tree(
    cache_dir: &Path,
    countries_path: &Path,
    max_depth: usize,
) -> LabeledPartitionTree<String> {
    load_or_compute_label_tree(cache_dir, countries_path, "ISO_A2", max_depth)
}


/// Loads or computes a labeled province partition tree.
/// If a cached version of the tree exists, it is loaded; otherwise, the tree is computed from scratch and saved.
///
/// # Arguments
///
/// * `cache_dir` - The directory where the tree cache will be stored.
/// * `provinces_path` - The path to the GeoJSON file containing province data.
/// * `max_depth` - The maximum depth of the partition tree.
///
/// # Returns
///
/// A `LabeledPartitionTree` with iso_3166_2 province names as labels.
pub fn load_or_compute_province_label_tree(
    cache_dir: &Path,
    provinces_path: &Path,
    max_depth: usize,
) -> LabeledPartitionTree<String> {
    load_or_compute_label_tree(cache_dir, provinces_path, "iso_3166_2", max_depth)
}
