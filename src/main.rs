use ant_simulation::simulation::run_simulation_step;
use ant_simulation::world::World;
use clap::Parser;
use std::time::Instant;

/// Command-line arguments for the ant simulation.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of ants to simulate.
    #[arg(short, long)]
    ants: usize,

    /// Path to the map file defining the world layout.
    #[arg(short, long, default_value = "./hiveum_map_small.txt")]
    map: String,
}

/// Initializes the world and ants, then simulates a fixed number of steps.
/// Measures and prints performance statistics for each simulation round.
fn main() {
    // Parse command-line arguments.
    let args = Args::parse();

    let mut world = World::new(&args.map).unwrap();
    let mut ants = world.initialize_ants(args.ants);
    println!("{} colonies loaded.", world.colonies.len());

    // ASSUMPTION: At start two ants cannot spawn in the same colony, this means that there have to be at least as many colonies as ants.
    assert!(world.colonies.len() >= args.ants, "Not enough colonies to place all ants. There are only {} colonies, but {} ants were requested.", world.colonies.len(), args.ants);
    let simulation_start = Instant::now();
    let mut round_times = Vec::new();

    // Run the simulation for a maximum of 10,000 rounds.
    for _ in 0..10000 {
        // If all colonies are destroyed, stop the simulation early.
        if world.colonies.is_empty() {
            break;
        }

        let round_start = Instant::now();
        run_simulation_step(&mut world, &mut ants);
        round_times.push(round_start.elapsed().as_micros() as u64);
    }

    // Calculate the total simulation duration.
    let simulation_duration = simulation_start.elapsed();

    // Print the final state of the world, showing remaining colonies and their connections.
    println!("\nSimulation finished. Remaining world:");
    for (name, colony_arc) in &world.colonies {
        let colony = colony_arc.read().unwrap();
        let mut connections = Vec::new();
        if let Some(c) = colony.north.clone() {
            connections.push(format!("north={}", c.read().unwrap().name));
        }
        if let Some(c) = colony.east.clone() {
            connections.push(format!("east={}", c.read().unwrap().name));
        }
        if let Some(c) = colony.south.clone() {
            connections.push(format!("south={}", c.read().unwrap().name));
        }
        if let Some(c) = colony.west.clone() {
            connections.push(format!("west={}", c.read().unwrap().name));
        }
        println!("{} {}", name, connections.join(" "));
    }

    // Sort round times to calculate percentiles.
    round_times.sort();
    println!("\nPerformance measurements:");
    println!("Total simulation time: {:?}", simulation_duration);
    println!(
        "Median round time: {} µs",
        calculate_percentile(&round_times, 50.0)
    );
    println!(
        "95th percentile round time: {} µs",
        calculate_percentile(&round_times, 95.0)
    );
    println!(
        "99th percentile round time: {} µs",
        calculate_percentile(&round_times, 99.0)
    );
}

/// Calculates the specified percentile of a sorted list of `u64` values.
///
/// # Arguments
/// * `sorted_values` - A slice of `u64` values, which must be sorted in ascending order.
/// * `percentile` - The desired percentile to calculate (e.g., 50.0 for median, 95.0 for 95th percentile).
///
/// # Returns
/// The `u64` value at the calculated percentile. Returns 0 if `sorted_values` is empty.
fn calculate_percentile(sorted_values: &[u64], percentile: f64) -> u64 {
    if sorted_values.is_empty() {
        return 0;
    }

    let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index]
}
