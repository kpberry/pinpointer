use geo::{Contains, Intersects, MultiPolygon, Point, Rect};
use std::{collections::HashMap, hash::Hash};

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
}
