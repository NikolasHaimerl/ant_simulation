# Ant Simulation

This project simulates ant movement and colony interactions within a defined world.

## How to Run the Simulation

To run the simulation, you will need Rust and Cargo installed.

1.  **Build the project:**

    ```bash
    cargo build --release
    ```

2.  **Run the simulation:**

    The simulation can be run with various command-line arguments:

    *   `--ants <NUMBER>`: Specifies the number of ants to simulate. (e.g., `--ants 100`)
    *   `--map <PATH_TO_MAP_FILE>`: Specifies the path to the map file. (e.g., `--map ./hiveum_map_small.txt`)

    **Example Usage:**

    To run the simulation with 50 ants using the small map:

    ```bash
    cargo run --release -- --ants 50 --map ./hiveum_map_small.txt
    ```

    To run with 100 ants using the medium map:

    ```bash
    cargo run --release -- --ants 100 --map ./hiveum_map_medium.txt
    ```

    If no arguments are provided, it will default to a certain number of ants and use `./hiveum_map_small.txt` as the map file.

    ```bash
    cargo run --release
    ```

## Parallelization Optimization

The simulation leverages the `rayon` crate for parallel processing, specifically in the `run_simulation_step` function. This optimization is applied to:

*   **Ant Movement:** Each ant's movement is processed in parallel. This is efficient because ant movements are largely independent of each other within a single simulation step, only requiring synchronized access when updating colony populations.
*   **Colony Destruction Check:** The identification of colonies that have been destroyed (i.e., those with more than one ant) is also parallelized. This allows for quick identification of colonies that need to be removed from the simulation.
*   **Adjacent Colony Connection Updates:** When a colony is destroyed, the process of updating the connections (north, east, south, west) in all *adjacent* colonies to remove references to the destroyed colony is also parallelized. This ensures that the world graph remains consistent and efficient after a colony's removal.

This simulation is primarily **CPU-bound**, meaning its performance is limited by the processing power of the CPU rather than by waiting for I/O operations (like reading from disk or network). The `rayon` crate is an excellent choice for this type of workload as it focuses on data parallelism and efficiently distributes computational tasks across available CPU cores.

Conversely, asynchronous runtimes like `tokio` are optimized for **I/O-bound** tasks, where the program spends a significant amount of time waiting for external operations to complete. Since the ant simulation involves intensive calculations and minimal I/O, `rayon` provides a more suitable and effective parallelization strategy than `tokio` would.

## Performance Benchmarks

After the simulation completes, it displays performance benchmarks for the simulation rounds. These metrics provide insight into the efficiency and consistency of each simulation step:

*   **Total simulation time:** The overall duration from the start to the end of the simulation.
*   **Median round time:** The middle value of all recorded round times. This is a robust measure of the typical time taken for a single simulation step, less affected by outliers than the average.
*   **95th percentile round time:** 95% of all simulation rounds completed within this time. This indicates the upper bound of typical performance, highlighting how often the simulation performs at its slower end.
*   **99th percentile round time:** 99% of all simulation rounds completed within this time. This metric is useful for identifying rare performance bottlenecks or spikes, showing the worst-case performance for almost all simulation steps.

These benchmarks help in understanding the simulation's performance characteristics and identifying potential areas for further optimization.