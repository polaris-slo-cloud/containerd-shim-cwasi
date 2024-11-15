#!/bin/bash

# Check if the user provided a file as an argument
if [ -z "$1" ]; then
    echo "Usage: $0 <log_file>"
    exit 1
fi

# Input file containing the log data (from command-line argument)
input_file="$1"


# Initialize variables
total_transfer_duration=0
total_serialization_duration=0
total_total_duration=0
count=0
round_number=1

# Function to calculate and print the average durations for the current round
print_average() {
    if [ $count -gt 0 ]; then
        avg_transfer_duration=$(echo "$total_transfer_duration / $count" | bc -l)
        avg_serialization_duration=$(echo "$total_serialization_duration / $count" | bc -l)
        avg_total_duration=$(echo "$total_total_duration / $count" | bc -l)
        
        echo "ROUND $round_number - Average Durations:"
        echo "Transfer Duration: $avg_transfer_duration"
        echo "Serialization Duration: $avg_serialization_duration"
        echo "Total Transfer Duration: $avg_total_duration"
        echo "----------------------------"
    fi
}

# Read through the file line by line
while IFS= read -r line; do
    # Check if it's a new round
    if [[ $line == "ROUND "* ]]; then
        # Print the average for the previous round (if it exists)
        print_average
        
        # Reset counters for the new round
        total_transfer_duration=0
        total_serialization_duration=0
        total_total_duration=0
        count=0
        
        # Increment the round number
        round_number=$(echo $line | awk '{print $2}')
    
    # Check for lines containing durations
    elif [[ $line == "Transfer duration:"* ]]; then
        transfer_duration=$(echo $line | awk '{print $3}')
        total_transfer_duration=$(echo "$total_transfer_duration + $transfer_duration" | bc -l)
        count=$((count + 1))
    
    elif [[ $line == "Serialization duration:"* ]]; then
        serialization_duration=$(echo $line | awk '{print $3}')
        total_serialization_duration=$(echo "$total_serialization_duration + $serialization_duration" | bc -l)
    
    elif [[ $line == "Total Transfer duration:"* ]]; then
        total_duration=$(echo $line | awk '{print $4}')
        total_total_duration=$(echo "$total_total_duration + $total_duration" | bc -l)
    fi

done < "$input_file"

# Print the last round's average
print_average
