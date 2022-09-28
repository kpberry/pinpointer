use geo::{BooleanOps, Contains, CoordsIter, Intersects, MultiPolygon, Point, Rect};
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    series::LineSeries,
    style::{BLACK, RED, WHITE},
};
use rand::Rng;
use std::{collections::HashMap, hash::Hash, path::Path, time::Instant};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LabeledPartitionTree<T: Eq + Hash> {
    children: Box<Vec<LabeledPartitionTree<T>>>,
    polygons: HashMap<T, MultiPolygon>,
    bbox: Rect,
}

impl<T: Clone + Eq + Hash> LabeledPartitionTree<T> {
    pub fn from_labeled_polygons(
        selected: &Vec<T>,
        polygons: &HashMap<T, MultiPolygon>,
        bbox: Rect,
        max_depth: usize,
        depth: usize,
    ) -> LabeledPartitionTree<T> {
        let (children, inner_polygons) = if depth == max_depth {
            (
                Box::new(vec![]),
                selected
                    .iter()
                    .map(|label| {
                        (
                            label.clone(),
                            polygons
                                .get(label)
                                .unwrap()
                                .intersection(&MultiPolygon::from(bbox)), // TODO this intersection is slow
                        )
                    })
                    .collect(),
            )
        } else if selected.len() == 0 {
            (Box::new(vec![]), HashMap::new())
        } else if selected.len() == 1 && polygons.get(&selected[0]).unwrap().contains(&bbox) {
            // TODO the check for this is slow
            (
                Box::new(vec![]),
                vec![(selected[0].clone(), MultiPolygon::from(bbox))]
                    .into_iter()
                    .collect(),
            )
        } else {
            // TODO check if a different branching factor can speed things up
            let [ab, cd] = bbox.split_x();
            let [a, b] = ab.split_y();
            let [c, d] = cd.split_y();
            let bboxes = vec![a, b, c, d];

            let bbox_selected_polygons: Vec<Vec<T>> = bboxes
                .iter()
                .map(|bbox| {
                    // TODO it might be possible to speed up this intersection check
                    selected
                        .iter()
                        .filter(|&label| bbox.intersects(polygons.get(label).unwrap()))
                        .cloned()
                        .collect()
                })
                .collect();

            (
                Box::new(
                    bbox_selected_polygons
                        .iter()
                        .zip(bboxes)
                        .map(|(selected, bbox)| {
                            LabeledPartitionTree::from_labeled_polygons(
                                selected,
                                polygons,
                                bbox,
                                max_depth,
                                depth + 1,
                            )
                        })
                        .collect(),
                ),
                HashMap::new(),
            )
        };

        LabeledPartitionTree {
            children,
            bbox,
            polygons: inner_polygons,
        }
    }

    pub fn label(&self, point: &Point) -> Option<T> {
        if self.children.is_empty() {
            self.polygons.iter().find_map(|(label, polygon)| {
                if polygon.contains(point) {
                    Some(label.clone())
                } else {
                    None
                }
            })
        } else {
            self.children
                .iter()
                .filter(|child| child.bbox.contains(point))
                .find_map(|child| child.label(point))
        }
    }

    pub fn size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|child| child.size()).sum()
        }
    }

    pub fn plot(&self, out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(out_path, (4000, 3000)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-180f32..180f32, -90f32..90f32)?;

        chart.configure_mesh().draw()?;

        let bboxes = self.bboxes();
        bboxes.iter().for_each(|bbox| {
            chart
                .draw_series(LineSeries::new(
                    bbox.coords_iter()
                        .map(|coord| (coord.x as f32, coord.y as f32)),
                    &RED,
                ))
                .unwrap();
        });

        chart
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    fn bboxes(&self) -> Vec<Rect> {
        if self.children.is_empty() {
            vec![self.bbox]
        } else {
            self.children
                .iter()
                .map(|child| child.bboxes())
                .flatten()
                .collect()
        }
    }
}

pub fn country_benchmark(countries: &HashMap<String, MultiPolygon>) {
    // building depth 8 tree should take ~5 minutes
    let t0 = Instant::now();
    let max_depth = 6;
    let tree: LabeledPartitionTree<String> = LabeledPartitionTree::from_labeled_polygons(
        &countries.keys().cloned().collect(),
        &countries,
        Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
        max_depth,
        0,
    );
    println!("{:?}", tree.size());
    println!("{:?}", t0.elapsed().as_secs_f64());

    tree.plot(Path::new(&format!("tree_plot_{max_depth}.png")))
        .unwrap();

    let mut rng = rand::thread_rng();
    let latlons: Vec<(f64, f64)> = (0..10000000)
        .map(|_| (rng.gen_range(-180.0..180.0), rng.gen_range(-90.0..90.0)))
        .collect();

    // querying 10,000,000 country codes should take 1-2 seconds
    let t0 = Instant::now();
    let mut labels2 = vec![];
    for (lat, lon) in latlons.iter() {
        let label = tree.label(&Point::new(*lon, *lat));
        labels2.push(label);
    }
    println!("{:?}", t0.elapsed().as_secs_f64());
}


pub fn province_benchmark(provinces: &HashMap<String, MultiPolygon>) {
    // building depth 8 tree should take ~2 minutes
    let t0 = Instant::now();
    let max_depth = 8;
    let tree: LabeledPartitionTree<String> = LabeledPartitionTree::from_labeled_polygons(
        &provinces.keys().cloned().collect(),
        &provinces,
        Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
        max_depth,
        0,
    );
    println!("{:?}", tree.size());
    println!("{:?}", t0.elapsed().as_secs_f64());

    tree.plot(Path::new(&format!("tree_plot_{max_depth}.png")))
        .unwrap();

    let mut rng = rand::thread_rng();
    let latlons: Vec<(f64, f64)> = (0..10000000)
        .map(|_| (rng.gen_range(-180.0..180.0), rng.gen_range(-90.0..90.0)))
        .collect();

    // querying 10,000,000 province codes should take 2-3 seconds
    let t0 = Instant::now();
    let mut labels2 = vec![];
    for (lat, lon) in latlons.iter() {
        let label = tree.label(&Point::new(*lon, *lat));
        labels2.push(label);
    }
    println!("{:?}", t0.elapsed().as_secs_f64());
}
