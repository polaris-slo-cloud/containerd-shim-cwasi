#!/bin/bash


sudo ctr -n k8s.io run --rm --runtime=io.containerd.cwasi.v1 --annotation cwasi.secondary.function=true --net-host=true docker.io/keniack/func_b:latest $(echo $RANDOM) /func_b.wasm &

start=`date +%s.%N`

sudo ctr -n k8s.io run --rm --runtime=io.containerd.cwasi.v1 --net-host=true docker.io/keniack/func_a:latest $(echo $RANDOM) /func_a.wasm func_b.wasm file_4M.txt

end=`date +%s.%N`
runtime=$( echo "$end - $start" | bc -l )

echo "Duration " $runtime
sudo pkill -9 -f cwasi
sudo ctr -n k8s.io c rm $(sudo ctr -n k8s.io c ls -q)
