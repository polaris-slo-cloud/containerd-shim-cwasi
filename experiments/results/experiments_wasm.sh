#!/bin/bash


# Check if a file argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <logfile>"
    exit 1
fi

# File provided as argument
LOG_FILE="$1"


# Extract timestamps from the log file
while read -r line; do
  if [[ "$line" =~ "ROUND" ]]; then
    echo "$line"
  elif [[ "$line" =~ "Start transfer" ]]; then
    start_transfer=$(echo "$line" | awk '{print $6, $7}' | sed 's/ UTC//')
    echo "Extracted Start transfer at: $start_transfer"
  elif [[ "$line" =~ "Received chunk" ]]; then
    received_chunk=$(echo "$line" | awk '{print $7}' | sed 's/Z//')
    echo "Extracted Received chunk at: $received_chunk"
  elif [[ "$line" =~ "After serialization at" ]]; then
    after_serialization=$(echo "$line" | awk '{print $4}' | sed 's/Z//')
    echo "Extracted After serialization at: $after_serialization"
    
    # Convert times to seconds since epoch for calculation
    start_transfer_time=$(date -d "$start_transfer" +%s.%N 2>/dev/null)
    received_chunk_time=$(date -d "$received_chunk" +%s.%N 2>/dev/null)
    after_serialization_time=$(date -d "$after_serialization" +%s.%N 2>/dev/null)
    
    # Debugging: Print the epoch times
    echo "Start transfer time (epoch): $start_transfer_time"
    echo "Received chunk time (epoch): $received_chunk_time"
    echo "After serialization time (epoch): $after_serialization_time"
    
    # Calculate the durations
    receive_duration=$(echo "$received_chunk_time - $start_transfer_time" | bc)
    serialization_duration=$(echo "$after_serialization_time - $received_chunk_time" | bc)
    total_duration=$(echo "$after_serialization_time - $start_transfer_time" | bc)
    
    # Print the results
    echo "Transfer duration: $receive_duration seconds"
    echo "Serialization duration: $serialization_duration seconds"
    echo "Total Transfer duration: $total_duration seconds"
    echo "----------"
  fi
done < "$LOG_FILE"
