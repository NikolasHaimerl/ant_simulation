use crate::world::{Ant, Colony, World};
use rand::seq::IndexedRandom;
use rayon;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;

/// Executes a single step of the ant simulation.
///
/// In this step, each ant attempts to move to an adjacent colony.
/// Colonies with more than one ant are destroyed, and their ants are removed from the simulation.
///
/// # Arguments
/// * `world` - A mutable reference to the `World` containing all colonies.
/// * `ants` - A mutable vector of `Arc<RwLock<Ant>>` representing all active ants in the simulation.
pub fn run_simulation_step(world: &mut World, ants: &mut Vec<Arc<RwLock<Ant>>>) {
    // Simulate ant movement in parallel.
    ants.par_iter().for_each(|ant| {
        let current_colony = ant.read().unwrap().location.clone();

        // Determine all possible adjacent colonies the ant can move to.
        let possible_moves: Vec<Arc<RwLock<Colony>>> = {
            let mut moves = Vec::new();
            if let Some(north_arc) = current_colony.read().unwrap().north.clone() {
                moves.push(north_arc);
            }
            if let Some(east_arc) = current_colony.read().unwrap().east.clone() {
                moves.push(east_arc);
            }
            if let Some(south_arc) = current_colony.read().unwrap().south.clone() {
                moves.push(south_arc);
            }
            if let Some(west_arc) = current_colony.read().unwrap().west.clone() {
                moves.push(west_arc);
            }
            moves
        };

        // If there are valid moves, the ant randomly selects one and moves.
        if !possible_moves.is_empty() {
            let next_colony_arc = {
                let mut rng = rand::rng();
                possible_moves.choose(&mut rng).unwrap().clone()
            };

            // Atomically remove the ant from its current colony and add it to the new one.
            // This order is crucial to prevent deadlocks and maintain data consistency.
            {
                let mut current_colony_write = current_colony.write().unwrap();
                current_colony_write.ants.retain(|a| !Arc::ptr_eq(a, &ant));
            }

            {
                let mut next_colony_write = next_colony_arc.write().unwrap();
                next_colony_write.ants.push(ant.clone());
            }

            // Update the ant's internal location reference.
            ant.write().unwrap().location = next_colony_arc.clone();
        }
    });

    // After all ants have moved, check for colonies that have been destroyed.
    // A colony is destroyed if it contains more than one ant.
    let results = world
        .colonies
        .par_iter()
        .map(|(name, colony_arc)| {
            let ants = &colony_arc.read().unwrap().ants;
            if ants.len() > 1 {
                println!(
                    "{} has been destroyed by ants {}",
                    name,
                    ants.iter()
                        .map(|ant| ant.read().unwrap().id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let removable_ants = ants
                    .iter()
                    .map(|ant| ant.read().unwrap().id)
                    .collect::<Vec<_>>();
                Some((name.clone(), removable_ants))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Process the results of destroyed colonies: remove them from the world and their ants from the simulation.
    for removable in results {
        if let Some((name, removable_ants)) = removable {
            if let Some(removed_colony) = world.colonies.remove(&name) {
                // Remove the colony and update connections in adjacent colonies.
                world
                    .colonies
                    .values()
                    .collect::<Vec<_>>()
                    .par_iter()
                    .for_each(|colony| {
                        let mut colony_write = colony.write().unwrap();

                        if let Some(ref north) = colony_write.north {
                            if Arc::ptr_eq(north, &removed_colony) {
                                colony_write.north = None;
                            }
                        }
                        if let Some(ref east) = colony_write.east {
                            if Arc::ptr_eq(east, &removed_colony) {
                                colony_write.east = None;
                            }
                        }
                        if let Some(ref south) = colony_write.south {
                            if Arc::ptr_eq(south, &removed_colony) {
                                colony_write.south = None;
                            }
                        }
                        if let Some(ref west) = colony_write.west {
                            if Arc::ptr_eq(west, &removed_colony) {
                                colony_write.west = None;
                            }
                        }
                    });
            }
            ants.retain(|x| !removable_ants.contains(&x.read().unwrap().id));
        }
    }
}
