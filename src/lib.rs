//! # pinpointer
//! This crate can be used to get provinces and countries from latitudes and longitudes.
//!
//! ## Usage
//! The main feature of this library is the `LabeledPartitionTree`, which can be used to perform fast point-in-region queries. 
//! A `LabeledPartitionTree` can be built from a mapping from labels to polygons with those labels.
//!
//! This library provides some helper functions to make it easy to get map data and build label trees to perform point-in-country and point-in-province queries.
//! See the examples folder for full code examples for downloading data, computing the label trees, and finally performing millions of point-in-country/point-in-province lookups.

pub mod datasets;
pub mod labeling;