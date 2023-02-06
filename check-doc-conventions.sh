#!/bin/bash

FLAGS="-r aoc_derive/src/ src/"

# Check for cross references that are not in code font.
grep -P "///.*? \\[[^\`].+?\\][^(]" $FLAGS
