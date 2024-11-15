#!/bin/bash

# Log file path
LOG_FILE="inter-node_iris.txt"

# Extract timestamps from the log file
while read -r line; do
  if [[ "$line" =~ "ROUND" ]]; then
    echo "$line"
  elif [[ "$line" =~ "Call external func at" ]]; then
    call_external_func=$(echo "$line" | awk '{print $5, $6}' | sed 's/ UTC//')
    echo "Extracted Call external func at: $call_external_func"
  elif [[ "$line" =~ "Server is listening on port" ]]; then
    server_listening=$(echo "$line" | awk '{print $8, $9}' | sed 's/ UTC//')
    echo "Extracted Server is listening on port at: $server_listening"
  elif [[ "$line" =~ "New client connected at" ]]; then
    client_connected=$(echo "$line" | awk '{print $5, $6}' | sed 's/ UTC//')
    echo "Extracted New client connected at: $client_connected"
  elif [[ "$line" =~ "start transfer at" ]]; then
    startafter=$(echo "$line" | awk '{print $4, $5}' | sed 's/ UTC//')
    echo "Extracted start transfer at: $startafter"
  elif [[ "$line" =~ "Received" ]]; then
    received=$(echo "$line" | awk '{print $5, $6}' | sed 's/ UTC//')
    echo "Received at: $received"
  elif [[ "$line" =~ "After serialization" ]]; then
    after_serialization=$(echo "$line" | awk '{print $4}' | sed 's/Z//')
    echo "Extracted After serialization at: $after_serialization"
    
    # Convert times to seconds since epoch for calculation
#    start_time=$(date -d "$call_external_func" +%s.%N 2>/dev/null)
    startafter_time=$(date -d "$startafter" +%s.%N 2>/dev/null)
 #   server_listening_time=$(date -d "$server_listening" +%s.%N 2>/dev/null)
 #   client_connected_time=$(date -d "$client_connected" +%s.%N 2>/dev/null)
    serial_time=$(date -d "$after_serialization" +%s.%N 2>/dev/null)
    received_time=$(date -d "$received" +%s.%N 2>/dev/null)
    
    # Debugging: Print the epoch times
#    echo "Start time (epoch): $start_time"
#    echo "Server listening time (epoch): $server_listening_time"
    echo "Start transfer time (epoch): $startafter_time"
#    echo "Client connected time (epoch): $client_connected_time"
    echo "Received (epoch): $received_time"
    echo "After serial time (epoch): $serial_time"
    
    # Calculate total time and idle time
    total_duration=$(echo "$serial_time - $startafter_time" | bc)
#    idle_duration=$(echo "$client_connected_time - $server_listening_time" | bc)
    transfer_duration=$(echo "$received_time - $startafter_time" | bc)
    serial_duration=$(echo "$serial_time - $received_time" | bc)
    
    # Calculate the effective duration (subtracting idle time)
#    effective_duration=$(echo "$total_duration - $idle_duration" | bc)
#    serial_duration=$(echo "$args_read_time - $wasmb_start_time" | bc)
    
    # Print the result
#   echo "Idle time: $idle_duration seconds"
    echo "Total duration: $total_duration seconds"
    echo "Serialization duration: $serial_duration seconds"
    echo "Transfer time: $transfer_duration seconds"
#    echo "Wasm Serialization: $serial_duration seconds"
    echo "-----" 
  fi
done < "$LOG_FILE"
