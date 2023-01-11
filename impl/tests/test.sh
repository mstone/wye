#!/bin/sh
cargo expand --ugly --test 04-let > foo.rs
mkdir -p vrrr
mv *.rs vrrr
cat prelude vrrr/foo.rs > foo.rs
cargo test
