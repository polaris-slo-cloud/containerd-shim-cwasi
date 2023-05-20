#!/bin/bash


#sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true docker.io/keniack/func_b_vanilla:latest $(echo $RANDOM) /func_b_vanilla.wasm func_b.wasm file_400M.txt &

start=`date +%s.%N`

sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true --env STORAGE_IP=192.168.0.38 --env FUNCB_IP=192.168.0.40 docker.io/keniack/func_a_vanilla:latest $(echo $RANDOM) /func_a_vanilla.wasm func_b_vanilla.wasm file_4M.txt

end=`date +%s.%N`
runtime=$( echo "$end - $start" | bc -l )

echo "Duration " $runtime
#sudo kill -9 $(pgrep -f wasmedge)
#sudo ctr -n k8s.io c rm $(sudo ctr -n k8s.io c ls -q)
