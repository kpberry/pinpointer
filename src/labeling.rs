use geo::{Contains, CoordsIter, Intersects, MultiPolygon, Point, Polygon, Rect};
use geojson::FeatureCollection;
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    series::LineSeries,
    style::{BLACK, RED, WHITE},
};
use std::{collections::HashMap, hash::Hash, path::Path, time::Instant};

pub struct LabeledPartitionTree<T> {
    children: Box<Vec<LabeledPartitionTree<T>>>,
    labels: Vec<T>,
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
        let children = if selected.is_empty() || depth == max_depth {
            Box::new(vec![])
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
            )
        };

        let labels = selected.clone();
        LabeledPartitionTree {
            children,
            labels,
            bbox,
        }
    }

    pub fn get_candidate_labels(&self, point: &Point) -> Vec<T> {
        if self.children.is_empty() {
            self.labels.clone()
        } else {
            self.children
                .iter()
                .filter(|child| child.bbox.contains(point))
                .map(|child| child.get_candidate_labels(point))
                .flatten()
                .collect()
        }
    }

    pub fn label(&self, point: &Point, polygons: &HashMap<T, MultiPolygon>) -> Option<T> {
        let candidates = self.get_candidate_labels(point);
        candidates
            .iter()
            .find(|candidate| {
                polygons
                    .get(&candidate)
                    .map(|polygon| polygon.contains(point))
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|child| child.size()).sum()
        }
    }

    pub fn plot(&self, out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(out_path, (2000, 1500)).into_drawing_area();
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

pub fn country_benchmark(countries: &FeatureCollection) {
    let labeled_polygons: HashMap<String, MultiPolygon> = countries
        .features
        .iter()
        .map(|country| {
            let name = country
                .property("ISO_A2")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let geometry = country.geometry.as_ref().unwrap();
            let multi_polygon = MultiPolygon::try_from(geometry.clone());
            let polygon: MultiPolygon = match multi_polygon {
                Ok(polygon) => polygon,
                Err(_) => MultiPolygon::new(vec![Polygon::try_from(geometry.clone()).unwrap()]),
            };
            (name, polygon)
        })
        .filter(|(name, _)| name != "-99")
        .collect();

    // building depth 10 tree should take 1-2 minutes
    let t0 = Instant::now();
    let max_depth = 10;
    let tree: LabeledPartitionTree<String> = LabeledPartitionTree::from_labeled_polygons(
        &labeled_polygons.keys().cloned().collect(),
        &labeled_polygons,
        Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
        max_depth,
        0,
    );
    println!("{:?}", tree.size());
    println!("{:?}", t0.elapsed().as_secs_f64());

    tree.plot(Path::new(&format!("tree_plot_{max_depth}"))).unwrap();

    // querying 1,000,000 country codes should take < 1 second
    let t0 = Instant::now();
    let mut labels = vec![];
    for _ in 0..1000000 {
        let label = tree.label(&Point::new(-70.013332, 12.557998), &labeled_polygons);
        labels.push(label);
    }
    println!("{:?}", t0.elapsed().as_secs_f64());
}
