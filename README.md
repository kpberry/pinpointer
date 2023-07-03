# pinpointer
Gets provinces and countries from latitudes and longitudes.

## Installation:
### From source:
You will need to install [Rust](https://www.rust-lang.org/learn/get-started) in order to install this program from source. 
After installing Rust, installation is simply:
```
cargo install pinpointer
```

## Usage
### As a library:
The main feature of this library is the `LabeledPartitionTree`, which can be used to perform fast point-in-region queries. 
A `LabeledPartitionTree` can be built from a mapping from labels to polygons with those labels.

This library provides some helper functions to make it easy to get map data and build label trees to perform point-in-country and point-in-province queries.
See the examples folder for full code examples for downloading data, computing the label trees, and finally performing millions of point-in-country/point-in-province lookups.

### Demo server:
You can also run a local demo server on port 8000 by running the `pinpointer-server` command after installation. 
On startup, the server will download country and province data to the `data` directory and compute depth 6 label trees for both.
The server exposes two endpoints, `/lat_lon_to_country` and `/lat_lon_to_province`, which take `lat` and `lon` query arguments and return a country or province code, respectively.
If the lat/lon pair does not fall within any country, the endpoints return "-99" instead.

Here are some example requests to the server:
```
curl http://localhost:8000/lat_lon_to_country?lat=10&lon=20
# TD
curl http://localhost:8000/lat_lon_to_province?lat=10&lon=20
# TD-SA
```