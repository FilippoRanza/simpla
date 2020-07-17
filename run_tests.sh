#! /bin/bash

# run simpla_parser test before simpla (the full compiler) tests
# this can be achieved also by hand, but in this way is easier for Travis

set -e 

cd simpla_parser
cargo build --verbose --all
cargo test --verbose --all --  --nocapture

cd ..
cargo build --verbose --all
cargo test --verbose --all --  --nocapture
