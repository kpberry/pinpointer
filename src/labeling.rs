use geo::{BooleanOps, Contains, CoordsIter, Intersects, MultiPolygon, Point, Rect};
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    series::LineSeries,
    style::{BLACK, RED, WHITE},
};
use std::{
    collections::HashMap,
    hash::Hash,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

/// A struct representing a labeled partition tree.
///
/// This structure is used for performing fast point-in-polygon queries by recursively checking
/// bounding boxes before performing the final point-in-polygon check.
#[derive(serde::Serialize, serde::Deserialize, std::clone::Clone, PartialEq, Debug)]
pub struct LabeledPartitionTree<T: Eq + Hash> {
    children: Box<Vec<LabeledPartitionTree<T>>>,
    polygons: HashMap<T, MultiPolygon>,
    bbox: Rect,
}

impl<T: Clone + Eq + Hash + Sync + Send + 'static> LabeledPartitionTree<T> {
    /// Constructs a labeled partition tree from a set of labeled polygons.
    ///
    /// # Arguments
    /// * `selected` - The labels of the polygons to be included in the tree.
    /// * `polygons` - A map of labels to their corresponding polygons.
    /// * `bbox` - The bounding box for the current partition.
    /// * `max_depth` - The maximum depth of the tree. Deeper trees tend to result in faster queries,
    ///                 but take much longer to construct.
    /// * `depth` - The current depth during recursion.
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

    fn label_leaf_polygons(
        selected: &[T],
        polygons: &HashMap<T, MultiPolygon>,
        bbox: Rect,
        depth: usize,
        max_depth: usize,
    ) -> Option<HashMap<T, MultiPolygon>> {
        if depth == max_depth {
            Some(
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
            Some(HashMap::new())
        } else if selected.len() == 1 && polygons.get(&selected[0]).unwrap().contains(&bbox) {
            // TODO the check for this is slow
            Some(
                vec![(selected[0].clone(), MultiPolygon::from(bbox))]
                    .into_iter()
                    .collect(),
            )
        } else {
            None
        }
    }

    fn select_child_bboxes(
        selected: &[T],
        polygons: &HashMap<T, MultiPolygon>,
        bbox: Rect,
    ) -> Vec<(Vec<T>, Rect)> {
        // TODO check if a different branching factor can speed things up
        let [ab, cd] = bbox.split_x();
        let [a, b] = ab.split_y();
        let [c, d] = cd.split_y();
        let bboxes = vec![a, b, c, d];

        bboxes
            .iter()
            .map(|bbox| {
                // TODO it might be possible to speed up this intersection check
                (
                    selected
                        .iter()
                        .filter(|&label| bbox.intersects(polygons.get(label).unwrap()))
                        .cloned()
                        .collect(),
                    bbox.clone(),
                )
            })
            .collect()
    }

    /// Constructs a labeled partition tree from a set of labeled polygons.
    ///
    /// This uses a queue instead of an implicit stack with recursion. In practice,
    /// this is slightly faster than the recursive version.
    ///
    /// # Arguments
    /// * `polygons` - A map of labels to their corresponding polygons.
    /// * `bbox` - The bounding box for the current partition.
    /// * `max_depth` - The maximum depth of the tree. Deeper trees tend to result in faster queries,
    ///                 but take much longer to construct.
    pub fn from_labeled_polygons_queue(
        polygons: &HashMap<T, MultiPolygon>,
        bbox: Rect,
        max_depth: usize,
    ) -> LabeledPartitionTree<T> {
        let root = LabeledPartitionTree {
            children: Box::new(Vec::new()),
            polygons: HashMap::new(),
            bbox: bbox,
        };
        let mut queue: Vec<(usize, Vec<T>, Rect, usize)> =
            vec![(0, polygons.keys().cloned().collect(), bbox, 0)];
        let mut nodes = vec![root];
        let mut parents = Vec::new();

        while let Some((parent_id, selected, bbox, depth)) = queue.pop() {
            if let Some(labeling) = LabeledPartitionTree::label_leaf_polygons(
                &selected, polygons, bbox, depth, max_depth,
            ) {
                let child_id = nodes.len();
                nodes.push(LabeledPartitionTree {
                    children: Box::new(Vec::new()),
                    polygons: labeling,
                    bbox: bbox,
                });
                parents.push((parent_id, child_id))
            } else {
                LabeledPartitionTree::select_child_bboxes(&selected, polygons, bbox)
                    .iter()
                    .cloned()
                    .for_each(|(selected, bbox)| {
                        let child = LabeledPartitionTree {
                            children: Box::new(Vec::new()),
                            polygons: HashMap::new(),
                            bbox: bbox,
                        };
                        let child_id = nodes.len();
                        nodes.push(child);
                        parents.push((parent_id, child_id));
                        queue.push((child_id, selected, bbox, depth + 1));
                    });
            }
        }

        // Annoyingly, it's important that we go in reverse order here since we clone
        // the child nodes.
        // This would be simpler if we just re-represented the tree structure as indexes
        // and a linear node array, as we have them here.
        for (parent_id, child_id) in parents.into_iter().rev() {
            let child = nodes[child_id].clone();
            nodes[parent_id].children.push(child);
        }

        nodes[0].clone()
    }

    /// Constructs a labeled partition tree from a set of labeled polygons in parallel.
    ///
    /// This function is a bit messy. See the single-threaded version for a cleaner
    /// implementation of this algorithm.
    ///
    /// # Arguments
    /// * `polygons` - A map of labels to their corresponding polygons.
    /// * `bbox` - The bounding box for the current partition.
    /// * `max_depth` - The maximum depth of the tree. Deeper trees tend to result in faster queries,
    ///                 but take much longer to construct.
    /// * `threads` - The number of threads to use when computing the tree.
    pub fn from_labeled_polygons_queue_pool(
        polygons: HashMap<T, MultiPolygon>,
        bbox: Rect,
        max_depth: usize,
        threads: usize,
    ) -> LabeledPartitionTree<T> {
        let root = LabeledPartitionTree {
            children: Box::new(Vec::new()),
            polygons: HashMap::new(),
            bbox: bbox,
        };
        let queue: Arc<Mutex<Vec<(usize, Vec<T>, Rect, usize)>>> = Arc::new(Mutex::new(vec![(
            0,
            polygons.keys().cloned().collect(),
            bbox,
            0,
        )]));
        let active = Arc::new(Mutex::new(1));
        let polygons_ref = Arc::new(polygons);
        let nodes = Arc::new(Mutex::new(vec![root]));
        let parents = Arc::new(Mutex::new(Vec::new()));

        let mut handles = vec![];

        for _ in 0..threads {
            let active = Arc::clone(&active);
            let queue = Arc::clone(&queue);
            let polygons = Arc::clone(&polygons_ref);
            let parents = Arc::clone(&parents);
            let nodes = Arc::clone(&nodes);
            let handle = thread::spawn(move || {
                while { *active.lock().unwrap() } > 0 {
                    let next = { queue.lock().unwrap().pop() };
                    if let Some((parent_id, selected, bbox, depth)) = next {
                        if let Some(labeling) = LabeledPartitionTree::label_leaf_polygons(
                            &selected, &polygons, bbox, depth, max_depth,
                        ) {
                            let child_id = {
                                let mut nodes = nodes.lock().unwrap();
                                let child_id = nodes.len();
                                nodes.push(LabeledPartitionTree {
                                    children: Box::new(Vec::new()),
                                    polygons: labeling,
                                    bbox: bbox,
                                });
                                child_id
                            };
                            {
                                parents.lock().unwrap().push((parent_id, child_id));
                            }
                        } else {
                            LabeledPartitionTree::select_child_bboxes(&selected, &polygons, bbox)
                                .iter()
                                .cloned()
                                .for_each(|(selected, bbox)| {
                                    let child = LabeledPartitionTree {
                                        children: Box::new(Vec::new()),
                                        polygons: HashMap::new(),
                                        bbox: bbox,
                                    };
                                    let child_id = {
                                        let mut nodes = nodes.lock().unwrap();
                                        let child_id = nodes.len();
                                        nodes.push(child);
                                        child_id
                                    };
                                    {
                                        parents.lock().unwrap().push((parent_id, child_id));
                                    }
                                    {
                                        queue.lock().unwrap().push((
                                            child_id,
                                            selected,
                                            bbox,
                                            depth + 1,
                                        ));
                                        *active.lock().unwrap() += 1;
                                    }
                                });
                        }
                        *active.lock().unwrap() -= 1;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Annoyingly, it's important that we go in reverse order here since we clone
        // the child nodes.
        // This would be simpler if we just re-represented the tree structure as indexes
        // and a linear node array, as we have them here.
        let mut nodes = nodes.lock().unwrap();
        for (parent_id, child_id) in parents.clone().lock().unwrap().clone().into_iter().rev() {
            let child = nodes[child_id].clone();
            nodes[parent_id].children.push(child);
        }

        nodes[0].clone()
    }

    /// Returns the label of the partition that contains the given point.
    ///
    /// This method recursively searches for the leaf node that contains the point and returns its label.
    /// If no leaf node contains the point, `None` is returned.
    ///
    /// # Arguments
    /// * `point` - The point to check.
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

    /// Returns the number of leaf nodes in the tree.
    pub fn size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|child| child.size()).sum()
        }
    }

    /// Plots the labeled partition tree and saves the image to the specified path.
    ///
    /// # Arguments
    /// * `out_path` - The path where the resulting image will be saved.
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
            return vec![self.bbox];
        } else {
            self.children
                .iter()
                .map(|child| child.bboxes())
                .flatten()
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use geo::{Point, Rect};

    use crate::{datasets::MapLoader, labeling::LabeledPartitionTree};

    #[test]
    fn test_tree_computation_methods_produce_same_query_results() {
        let mut map_loader = MapLoader::new(Path::new("test_data"));
        let provinces = map_loader.provinces();

        let recursive_tree = LabeledPartitionTree::from_labeled_polygons(
            &provinces.keys().cloned().collect(),
            &provinces,
            Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
            5,
            0,
        );
        println!("computed recursive tree");

        let queue_tree = LabeledPartitionTree::from_labeled_polygons_queue(
            &provinces,
            Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
            5,
        );
        println!("computed queue tree");

        let parallel_queue_tree = LabeledPartitionTree::from_labeled_polygons_queue_pool(
            provinces.clone(),
            Rect::new(Point::new(-180.0, 90.0), Point::new(180.0, -90.0)),
            5,
            10,
        );
        println!("computed parallel queue tree");

        let w = 1000;
        let h = 1000;
        for i in 0..w {
            for j in 0..h {
                let lat = -90.0 + (j as f64 / h as f64) * 180.0;
                let lon = -180.0 + (i as f64 / w as f64) * 360.0;
                let p = Point::new(lon, lat);

                let r_tree_label = recursive_tree.label(&p);
                let q_tree_label = queue_tree.label(&p);
                let pq_tree_label = parallel_queue_tree.label(&p);

                assert_eq!(r_tree_label, q_tree_label);
                assert_eq!(q_tree_label, pq_tree_label);
            }
        }
    }
}
