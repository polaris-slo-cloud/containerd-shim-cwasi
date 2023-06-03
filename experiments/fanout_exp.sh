#!/bin/bash

index="$1"

#sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true docker.io/keniack/func_b_fanout:latest $(echo $RANDOM) /func_b_fanout.wasm func_b.wasm file_400M.txt &

start=`date +%s.%N`

sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true --env STORAGE_IP=192.168.0.213:8888 --env FUNCB_URL=192.168.0.241:1234/hello --env FUNCTIONS_NUM=$index  docker.io/keniack/func_a_fanout:latest $(echo $RANDOM) /func_a_fanout.wasm func_b_vanilla.wasm file_2M.txt

end=`date +%s.%N`
runtime=$( echo "$end - $start" | bc -l )

echo "Duration " $runtime
#sudo kill -9 $(pgrep -f wasmedge)
#sudo ctr -n k8s.io c rm $(sudo ctr -n k8s.io c ls -q)