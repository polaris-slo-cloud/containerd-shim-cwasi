#!/bin/sh

file="$1"

cat $file | grep throughput | grep -o -P '.[0-9]*\.[0-9]*'