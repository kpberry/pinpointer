use std::{collections::HashMap, fs, path::Path};

use geo::{MultiPolygon, Polygon};
use geojson::{FeatureCollection, GeoJson};

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