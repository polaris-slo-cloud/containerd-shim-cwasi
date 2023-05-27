#!/bin/sh

cat cwasi_2M.log | grep cpu| grep -o -P '.{0,0}value.{0,12}' | grep -Eo '[0-9].{1,8}' | tr -d "\'"
