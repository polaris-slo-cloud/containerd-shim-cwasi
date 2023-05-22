#!/bin/sh

cat openfaas_40M.log | grep lo | grep rx | grep -o -P '.{0,0}value.{0,12}' | grep -Eo '[0-9].{1,8}' | tr -d "\'"