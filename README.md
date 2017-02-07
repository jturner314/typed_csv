# typed_csv

[![Build Status](https://travis-ci.org/jturner314/typed_csv.svg?branch=master)](https://travis-ci.org/jturner314/typed_csv)

This crate provides wrappers for the reader and writer in the [`csv`][csv]
crate that provide checking of the CSV headers (when reading) and automatically
writing the CSV headers (when writing) according to the field names in the
record type.

## Documentation

Run `cargo doc --open` in this repository.

## Simple examples

The reader does type-based decoding for each record in the CSV data. It checks
that the headers match the field names in the record type. The reader can
optionally reorder columns to match headers to field names, treat headers as
case-insensitive, or use an arbitrary predicate to match headers to field
names.

```rust
extern crate rustc_serialize;
extern crate typed_csv;

#[derive(RustcDecodable)]
struct Record {
    count: usize,
    animal: String,
    description: String,
}

fn main() {
    let data = "\
count,animal,description
7,penguin,happy
10,cheetah,fast
4,armadillo,armored
9,platypus,unique
7,mouse,small
";

    let rdr = typed_csv::Reader::from_string(data);
    for row in rdr.decode() {
        let Record { animal, description, count } = row.unwrap();
        println!("{}, {}: {}", animal, description, count);
    }
}
```

The writer automatically writes a header row according to the field names in
the record type.

```rust
extern crate rustc_serialize;
extern crate typed_csv;

#[derive(RustcEncodable)]
struct Record {
    count: usize,
    animal: &'static str,
    description: &'static str,
}

fn main() {
    let records = vec![
        Record { count: 7, animal: "penguin", description: "happy" },
        Record { count: 10, animal: "cheetah", description: "fast" },
        Record { count: 4, animal: "armadillo", description: "armored" },
        Record { count: 9, animal: "platypus", description: "unique" },
    ];

    let mut wtr = typed_csv::Writer::from_memory();
    for record in records.into_iter() {
        wtr.encode(record).unwrap();
    }

    assert_eq!(wtr.as_string(), "\
count,animal,description
7,penguin,happy
10,cheetah,fast
4,armadillo,armored
9,platypus,unique
");
}
```

## Issues

This crate needs more tests, and it probably has a few bugs.
Please [file an issue](https://github.com/jturner314/typed_csv/issues/new) if
you find any bugs.

## License

Significant portions of this crate are closely based on code from
the [`csv`][csv] crate, which is dual-licensed under the Unlicense and MIT
licenses. Many thanks to [BurntSushi](http://burntsushi.net/) (Andrew Gallant)
for creating such a fast and featureful CSV crate!

This crate is similarly dual-licensed under
the [Unlicense](https://unlicense.org/) and MIT licenses.
See [COPYING](COPYING) for more information.

[csv]: https://github.com/BurntSushi/rust-csv
