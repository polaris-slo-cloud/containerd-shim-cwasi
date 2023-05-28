#!/bin/bash

file="$1"
timestamp_array=( $(cat $file | grep -o -P "2023.{0,35}" | sort -r | grep -oE '.{0,3}.{,9}Z' | tr -d 'Z') )
my_array_length=${#timestamp_array[@]}

for (( j=0; j<${my_array_length}; j++ ));
do
  #printf "Current index %d with value %s\n" $j "${timestamp_array[$j]}"

  #end=$(date --date "${timestamp_array[$j]}" +%s.%9N)
  end="${timestamp_array[$j]}"
  j=$j+1
  #start=$(date --date "${timestamp_array[$j]}" +%s.%9N)
  start="${timestamp_array[$j]}"
  #echo "$end - $start"
  diff=$(echo "$end - $start" | bc)
  echo '0'$diff

done

