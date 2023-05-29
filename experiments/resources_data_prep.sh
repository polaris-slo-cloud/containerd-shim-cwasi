#!/bin/sh

file="$1"
resource="$2"
cat $file | grep $resource | grep -o -P '.{0,0}value.{0,12}' | grep -Eo '[0-9].{1,8}' | tr -d "\'"
