#!/bin/bash

while true
do
    sleep 3
    date
    ps -p $(pidof ctr) -o %mem,rss
done