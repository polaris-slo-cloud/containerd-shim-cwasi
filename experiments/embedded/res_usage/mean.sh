#!/bin/bash

# Define the log file
log_file="output.txt"

# Process the log file and calculate the mean for each metric and subsystem
awk -F", " '
{
    # Initialize variables
    metric = ""; subsystem = ""; value = 0;

    # Extract the metric, subsystem, and value fields
    for (i=1; i<=NF; i++) {
        if ($i ~ /metric=/) {
            # Extract the full metric name
            split($i, m, "'\''")
            metric = m[2]  # Capture the full metric name
	    gsub(",", "", metric)  # Remove any trailing commas from the metric
        }
        if ($i ~ /subsystem=/) {
            # Extract the subsystem name
            split($i, s, "'\''")
            subsystem = s[2]
        }
        if ($i ~ /value=/) {
            # Extract the value
            split($i, v, "'\''")
            value = v[2]
        }
    }

    # Create a unique key based on subsystem and metric
    key = subsystem "_" metric

    # Sum the values and keep track of count for each key
    sum[key] += value
    count[key]++

    # Track the order of the first appearance of each subsystem
    if (!(subsystem in seen)) {
        subsystems_ordered[order++] = subsystem
        seen[subsystem] = 1
    }
}
END {
    # Print the results in the order the subsystems first appeared
    for (i = 0; i < order; i++) {
        subsystem = subsystems_ordered[i]
        printf "Subsystem: %s\n", subsystem
        for (key in sum) {
            split(key, parts, "_")
            if (parts[1] == subsystem) {
                metric = parts[2]  # Get the full metric name
                mean = sum[key] / count[key]
                printf "  %s mean: %.f\n", key, mean
            }
        }
        printf "\n"
    }
}
' "$log_file"