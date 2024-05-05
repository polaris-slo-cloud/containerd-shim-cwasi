struct CommModeSelector {
    threshold_one: f32,  // First threshold for communication frequency
    threshold_two: f32,  // Second threshold for communication frequency
    priority_max: f32,   // Maximum priority
    priority_min: f32,   // Minimum priority
    adjustment_factor: f32,  // Adjustment factor for priority calculation
    weight_latency: f32,     // Weight for latency in speed calculation
    weight_throughput: f32,  // Weight for throughput in speed calculation
    weight_cpu_usage: f32,   // Weight for CPU usage in speed calculation
    weight_memory_usage: f32,// Weight for memory usage in speed calculation
    weight_bandwidth_usage: f32, // Weight for bandwidth usage in speed calculation
    data_volume: f32,        // Volume of data to be transferred
    execution_time: f32,     // Execution time of the function
}

impl CommModeSelector {
    // Constructor to initialize the struct with default values
    fn new() -> Self {
        let threshold_one = 10.0;
        let threshold_two = 30.0;
        let priority_max = 3.0;
        let priority_min = 1.0;
        let adjustment_factor = (priority_max - priority_min) / (threshold_two - threshold_one);

        CommModeSelector {
            threshold_one,
            threshold_two,
            priority_max,
            priority_min,
            adjustment_factor,
            weight_latency: 0.2,
            weight_throughput: 0.3,
            weight_cpu_usage: 0.25,
            weight_memory_usage: 0.15,
            weight_bandwidth_usage: 0.1,
            data_volume: 500.0, // Example data volume
            execution_time: 0.5, // Example execution time
        }
    }

    // Calculate the priority based on the frequency of requests per hour
    fn calculate_priority(&self, frequency: f32) -> f32 {
        let excess_frequency = frequency.max(self.threshold_one) - self.threshold_one;
        self.priority_max - (self.adjustment_factor * excess_frequency).max(0.0).min(self.priority_max - self.priority_min)
    }

    // Calculate the communication speed based on weights and resource metrics
    fn calculate_speed(&self, speed_latency: f32, speed_throughput: f32, speed_cpu_usage: f32, speed_memory_usage: f32, speed_bandwidth_usage: f32) -> f32 {
        self.weight_latency * speed_latency +
            self.weight_throughput * speed_throughput +
            self.weight_cpu_usage * speed_cpu_usage +
            self.weight_memory_usage * speed_memory_usage +
            self.weight_bandwidth_usage * speed_bandwidth_usage
    }

    // Compute communication time for a given communication mode
    fn communication_time(&self, mode: u8, frequency: f32) -> f32 {
        let (host_same, local_snapshot, speed_latency, speed_throughput, speed_cpu_usage, speed_memory_usage, speed_bandwidth_usage) = match mode {
            1 => (1, 1, 100.0, 50.0, 20.0, 10.0, 5.0),  // Metrics for embedded communication on the same host
            2 => (1, 0, 80.0, 40.0, 15.0, 8.0, 4.0),    // Metrics for local host communication via IPC
            3 => (0, 0, 60.0, 30.0, 10.0, 5.0, 2.5),    // Metrics for remote connection
            _ => unreachable!(),
        };
        let cold_start_time = if mode == 1 { 0.0 } else { 1.0 }; // Simplified cold start time assumption
        let speed_index = self.calculate_speed(speed_latency, speed_throughput, speed_cpu_usage, speed_memory_usage, speed_bandwidth_usage);
        let priority = self.calculate_priority(frequency);
        ((1 - host_same * local_snapshot) as f32 * cold_start_time + self.data_volume / speed_index + self.execution_time) * priority
    }

    fn find_best_commode(&self, request_frequency:f32)-> u8 {

        //let request_frequency = 20.0; // Example frequency of requests per hour

        let mut best_time = f32::MAX;
        let mut best_mode = 0;

        // Calculate the communication time for each mode and find the best one
        for mode in 1..=3 {
            let time = self.communication_time(mode, request_frequency);
            println!("Communication time for mode {}: {:.2} seconds", mode, time);

            if time < best_time {
                best_time = time;
                best_mode = mode;
            }
        }

        println!("Recommended communication mode: {} with a time of {:.2} seconds", best_mode, best_time);
        return best_mode;
    }
}