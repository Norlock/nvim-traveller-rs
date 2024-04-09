#!/bin/bash

cargo build --release

cd "$(dirname "$0")/lua"
ln -s ../target/release/libnvim_traveller_rs.so nvim_traveller_rs.so
