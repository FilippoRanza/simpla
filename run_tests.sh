#! /bin/bash

# run simpla_parser test before simpla (the full compiler) tests
# this can be achieved also by hand, but in this way is easier for Travis

set -e 

function test_inner_crate() {
    cd "$1"
    cargo build --verbose --all
    cargo test --verbose --all --  --nocapture
    cd ..
}

test_inner_crate 'extract_line_error'

test_inner_crate 'simpla_parser'

cargo build --verbose --all
cargo test --verbose --all --  --nocapture
