use std::{collections::HashMap, fs, path::{Path, self}};

use geo::{MultiPolygon, Polygon, Rect, Point};
use geojson::{FeatureCollection, GeoJson};

use crate::labeling::LabeledPartitionTree;

pub fn load_countries(path: &Path) -> HashMap<String, MultiPolygon> {
    let geojson_str = fs::read_to_string(path).unwrap();
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    let countries: FeatureCollection = FeatureCollection::try_from(geojson).unwrap();

    let mut labeled_polygons: HashMap<String, Vec<Polygon>> = HashMap::new();
    countries.features.iter().for_each(|country| {
        let name = country
            .property("ISO_A2")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        if name != "-99" {
            let geometry = country.geometry.as_ref().unwrap();
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





pub fn load_provinces(path: &Path) -> HashMap<String, MultiPolygon> {
    let geojson_str = fs::read_to_string(path).unwrap();
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    let provinces: FeatureCollection = FeatureCollection::try_from(geojson).unwrap();
    
    let mut labeled_polygons: HashMap<String, Vec<Polygon>> = HashMap::new();
    provinces.features.iter().for_each(|province| {
        let name = province
            .property("iso_3166_2")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        if name != "-99" {
            let geometry = province.geometry.as_ref().unwrap();
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

pub fn load_or_compute_country_label_tree(cache_dir: &Path, countries_path: &Path, max_depth: usize) -> LabeledPartitionTree<String> {
    let cache_path = cache_dir.join(format!("country_label_tree_{max_depth}.json"));
    let tree = match fs::read_to_string(&cache_path) {
        Ok(string) => {
            serde_json::from_str(&string).unwrap()
        },
        Err(e) => {
            println!("{e}");
            println!("Could not load country label tree; computing from scratch.");
            let countries = load_countries(countries_path);
            let tree = LabeledPartitionTree::from_labeled_polygons(
                &countries.keys().cloned().collect(),
                &countries,
                Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
                max_depth,
                0,
            );
            let tree_json = serde_json::to_string(&tree).unwrap();
            fs::write(cache_path, tree_json).unwrap();
            tree
        }
    };
    println!("Loaded country label tree.");
    tree
}

pub fn load_or_compute_province_label_tree(cache_dir: &Path, provinces_path: &Path, max_depth: usize) -> LabeledPartitionTree<String> {
    let cache_path = cache_dir.join(format!("province_label_tree_{max_depth}.json"));
    let tree = match fs::read_to_string(&cache_path) {
        Ok(string) => {
            serde_json::from_str(&string).unwrap()
        },
        Err(e) => {
            println!("{e}");
            println!("Could not load province label tree; computing from scratch.");
            let provinces = load_provinces(provinces_path);
            let tree = LabeledPartitionTree::from_labeled_polygons(
                &provinces.keys().cloned().collect(),
                &provinces,
                Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
                max_depth,
                0,
            );
            let tree_json = serde_json::to_string(&tree).unwrap();
            fs::write(cache_path, tree_json).unwrap();
            tree
        }
    };
    println!("Loaded province label tree.");
    tree
}