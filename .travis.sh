#!/bin/sh

echo "cargo build $FEATURES"
cargo build --verbose  $FEATURES
echo "cargo test $FEATURES"
cargo test --verbose $FEATURES
