#!/bin/bash

file="$1"
#sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true docker.io/keniack/func_b_vanilla:latest $(echo $RANDOM) /func_b_vanilla.wasm func_b.wasm file_400M.txt &

start=`date +%s.%N`

sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true --env STORAGE_IP=192.168.0.213:8888 --env FUNCB_IP=192.168.0.241 --env FUNCTIONS_NUM=1  docker.io/keniack/func_a_vanilla:latest $(echo $RANDOM) /func_a_vanilla.wasm func_b_vanilla.wasm $file

end=`date +%s.%N`
runtime=$( echo "$end - $start" | bc -l )

echo "Duration " $runtime
#sudo kill -9 $(pgrep -f wasmedge)
#sudo ctr -n k8s.io c rm $(sudo ctr -n k8s.io c ls -q)