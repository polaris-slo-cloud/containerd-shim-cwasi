#!/bin/bash

file="$1"
for (( i=1; i<=1; i++ ))
  do
    sudo ctr -n k8s.io run --rm --runtime=io.containerd.cwasi.v1 --annotation cwasi.secondary.function=true --net-host=true docker.io/keniack/func_b:latest $(echo $RANDOM) /func_b.wasm &
done


start=`date +%s.%N`

sudo ctr -n k8s.io run --rm --runtime=io.containerd.cwasi.v1 --net-host=true --env STORAGE_IP=192.168.0.213:8888 --env FUNCTIONS_NUM=1 docker.io/keniack/func_a:latest $(echo $RANDOM) /func_a.wasm func_b.wasm $file

end=`date +%s.%N`
runtime=$( echo "$end - $start" | bc -l )

echo "Duration " $runtime
sudo kill -9 $(pgrep -f cwasi)
sudo ctr -n k8s.io c rm $(sudo ctr -n k8s.io c ls -q)
