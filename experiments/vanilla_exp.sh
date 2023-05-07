#!/bin/bash


sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true docker.io/keniack/func_b_vanilla:latest $(echo $RANDOM) /func_b_vanilla.wasm func_b.wasm file_4M.txt &

start=`date +%s.%N`

sudo ctr -n k8s.io run --rm --runtime=io.containerd.wasmedge.v1 --net-host=true docker.io/keniack/func_a_vanilla:0.0.1 $(echo $RANDOM) /func_a_vanilla.wasm func_b_vanilla.wasm file_4M.txt

end=`date +%s.%N`
runtime=$( echo "$end - $start" | bc -l )

echo "Duration " $runtime
sudo pkill -9 -f wasmedge /dev/null 2>&1
sudo ctr -n k8s.io c rm $(sudo ctr -n k8s.io c ls -q) /dev/null 2>&1
