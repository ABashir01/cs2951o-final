#!/bin/bash
set -e

# build archive
rm -f archive.zip
zip -r archive.zip \
  input src Cargo.toml Cargo.lock \
  requirements.txt \
  compile.sh run.sh runAll.sh \
  report.pdf \
  team.txt \
  results.log \
  presentation.pdf
