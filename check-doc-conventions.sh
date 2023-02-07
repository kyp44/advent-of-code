#!/bin/bash

FLAGS="-r aoc_derive/src/ src/"

# Check for cross references that are not in code font.
grep -P "///.*? \\[[^\`].+?\\][^(]" $FLAGS

# TODO Check for new comment lines with two sentences
# TODO Check for new comment lines followed by another non-empty comment line.
