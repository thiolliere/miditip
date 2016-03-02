#!/bin/sh

./init.sh

cargo run -- client 1 2 0.0.0.0:8080 &
cargo run -- client 6 0 0.0.0.0:8080 &

sleep 4

./kill.sh
