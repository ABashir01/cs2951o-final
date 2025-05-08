#!/bin/bash

COURSE_CARGO_BIN="/course/cs2951o/resource/cargo/bin/"
if [ -d "${COURSE_CARGO_BIN}" ]; then
  export PATH="${PATH}:${COURSE_CARGO_BIN}"
fi

cargo build --release
