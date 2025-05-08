#!/bin/bash
set -e

# build archive
rm -f archive.zip
zip -r archive.zip \
  input src \
  requirements.txt \
  compile.sh run.sh runAll.sh \
  report.pdf \
  team.txt \
  results.log
