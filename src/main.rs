use std::{fs, net::SocketAddr, path::Path, sync::Arc};

use axum::{extract::Query, routing::get, Router};
use datasets::{
    load_or_compute_country_label_tree, load_or_compute_province_label_tree, load_provinces,
};
use geo::{Point, Rect};

mod datasets;
mod labeling;

use serde::Deserialize;

use crate::{datasets::load_countries, labeling::LabeledPartitionTree};

#[derive(Deserialize, Debug)]
struct LatLon {
    lat: f64,
    lon: f64,
}

async fn lat_lon_to_label(
    lat_lon: LatLon,
    label_tree: Arc<LabeledPartitionTree<String>>,
) -> String {
    label_tree
        .label(&Point::new(lat_lon.lon, lat_lon.lat))
        .unwrap_or(String::from("-99"))
}

#[tokio::main]
async fn main() {
    let country_label_tree = load_or_compute_country_label_tree(
        Path::new("data"),
        Path::new("data\\ne_10m_admin_0_countries.json"),
        6
    );
    let country_label_tree_arc = Arc::new(country_label_tree);

    let province_label_tree = load_or_compute_province_label_tree(
        Path::new("data"),
        Path::new("data\\ne_10m_admin_1_states_provinces.json"),
        6
    );
    let province_label_tree_arc = Arc::new(province_label_tree);

    let app = Router::new()
        .route(
            "/lat_lon_to_country",
            get(move |lat_lon: Query<LatLon>| {
                lat_lon_to_label(
                    LatLon {
                        lat: lat_lon.lat,
                        lon: lat_lon.lon,
                    },
                    country_label_tree_arc.clone(),
                )
            }),
        )
        .route(
            "/lat_lon_to_province",
            get(move |lat_lon: Query<LatLon>| {
                lat_lon_to_label(
                    LatLon {
                        lat: lat_lon.lat,
                        lon: lat_lon.lon,
                    },
                    province_label_tree_arc.clone(),
                )
            }),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
