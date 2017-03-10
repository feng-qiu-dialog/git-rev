#!/bin/sh

TARGET=target/release/git-rev

rm -f $TARGET
cargo build --release
ls -lh $TARGET
strip --strip-all $TARGET
ls -lh $TARGET