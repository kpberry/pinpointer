use geo::{BooleanOps, Contains, CoordsIter, Intersects, MultiPolygon, Point, Rect};
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    series::LineSeries,
    style::{BLACK, RED, WHITE},
};
use std::{collections::HashMap, hash::Hash, path::Path};

/// A struct representing a labeled partition tree.
///
/// This structure is used for performing fast point-in-polygon queries by recursively checking 
/// bounding boxes before performing the final point-in-polygon check.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LabeledPartitionTree<T: Eq + Hash> {
    children: Box<Vec<LabeledPartitionTree<T>>>,
    polygons: HashMap<T, MultiPolygon>,
    bbox: Rect,
}

impl<T: Clone + Eq + Hash> LabeledPartitionTree<T> {
     /// Constructs a labeled partition tree from a set of labeled polygons.
    ///
    /// The `selected` parameter contains the labels of the polygons to be included in the tree.
    /// The `polygons` parameter is a map of labels to their corresponding polygons.
    /// The `bbox` parameter represents the bounding box for the current partition.
    /// The `max_depth` parameter determines the maximum depth of the tree.
    /// The `depth` parameter specifies the current depth during recursion.
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

    /// Returns the label of the partition that contains the given point.
    ///
    /// This method recursively searches for the leaf node that contains the point and returns its label.
    /// If no leaf node contains the point, `None` is returned.
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

    /// Returns the size of the labeled partition tree.
    ///
    /// The size represents the total number of leaf nodes in the tree.
    pub fn size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|child| child.size()).sum()
        }
    }

    /// Plots the labeled partition tree and saves the image to the specified path.
    ///
    /// The `out_path` parameter is the path where the resulting image will be saved.
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

    /// Returns a vector of bounding boxes for all leaf nodes in the labeled partition tree.
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

