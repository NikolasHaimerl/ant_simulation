use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Represents an individual ant in the simulation.
#[derive(Clone)]
pub struct Ant {
    /// Unique identifier for the ant.
    pub id: usize,
    /// The current colony where the ant is located.
    pub location: Arc<RwLock<Colony>>,
}

impl fmt::Debug for Ant {
    /// Implements custom debug formatting for `Ant`.
    /// This provides a concise representation of an ant, primarily showing its ID.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ant").field("id", &self.id).finish()
    }
}

/// Represents a colony in the simulation world.
pub struct Colony {
    pub name: String,
    pub north: Option<Arc<RwLock<Colony>>>,
    pub east: Option<Arc<RwLock<Colony>>>,
    pub south: Option<Arc<RwLock<Colony>>>,
    pub west: Option<Arc<RwLock<Colony>>>,
    pub ants: Vec<Arc<RwLock<Ant>>>,
}

impl fmt::Debug for Colony {
    /// Implements custom debug formatting for `Colony`.
    /// This provides a concise representation of a colony, showing its name and the ants it contains.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Colony")
            .field("name", &self.name)
            .field("ants", &self.ants)
            .finish()
    }
}

/// Represents the entire simulation world, managing all colonies and their interactions.
pub struct World {
    pub colonies: HashMap<String, Arc<RwLock<Colony>>>,
}

impl World {
    /// Creates a new `World` instance by parsing a map file.
    ///
    /// This constructor initializes the world's structure, including colonies and their connections,
    /// based on the provided map file. It's the entry point for setting up the simulation environment.
    ///
    /// # Arguments
    /// * `map_path` - The path to the map file that defines the world's layout.
    ///
    /// # Returns
    /// A `Result` containing the new `World` instance or an `io::Error` if parsing fails.
    pub fn new(map_path: &str) -> io::Result<Self> {
        let colonies = Self::parse_map(map_path)?;
        Ok(World { colonies })
    }

    /// Parses the map file to construct the colonies and their connections.
    ///
    /// This private helper function reads the map file line by line. It first creates all colonies
    /// and then, in a second pass, establishes the directional connections between them.
    /// This two-pass approach is necessary because connections might refer to colonies not yet created.
    ///
    /// # Arguments
    /// * `file_path` - The path to the map file.
    ///
    /// # Returns
    /// A `Result` containing a `HashMap` of colony names to `Arc<RwLock<Colony>>` or an `io::Error`.
    fn parse_map(file_path: &str) -> io::Result<HashMap<String, Arc<RwLock<Colony>>>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut colonies = HashMap::new();
        let mut connections = Vec::new();

        // First pass: Create all colonies and collect connection information.
        // This ensures all colony objects exist before attempting to link them.
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            let colony_name = parts[0].to_string();

            let colony = Arc::new(RwLock::new(Colony {
                name: colony_name.clone(),
                north: None,
                east: None,
                south: None,
                west: None,
                ants: Vec::new(),
            }));
            colonies.insert(colony_name.clone(), colony);
            for &part in &parts[1..] {
                let connection_parts: Vec<&str> = part.split('=').collect();
                let direction = connection_parts[0];
                let target_colony_name = connection_parts[1].to_string();
                connections.push((
                    colony_name.clone(),
                    direction.to_string(),
                    target_colony_name,
                ));
            }
        }

        // Second pass: Establish connections between colonies.
        // Now that all colonies are created, we can safely link them.
        for (from_colony_name, direction, to_colony_name) in connections {
            if let Some(from_colony_arc) = colonies.get(&from_colony_name) {
                if let Some(to_colony_arc) = colonies.get(&to_colony_name) {
                    let mut from_colony_write = from_colony_arc.write().unwrap();
                    match direction.as_str() {
                        "north" => from_colony_write.north = Some(to_colony_arc.clone()),
                        "east" => from_colony_write.east = Some(to_colony_arc.clone()),
                        "south" => from_colony_write.south = Some(to_colony_arc.clone()),
                        "west" => from_colony_write.west = Some(to_colony_arc.clone()),
                        _ => (),
                    }
                }
            }
        }
        Ok(colonies)
    }

    /// Initializes a specified number of ants and places them in randomly chosen colonies.
    ///
    /// This function ensures that each ant is initially placed in a unique colony if the number
    /// of ants is less than or equal to the total number of available colonies. This prevents
    /// immediate colony destruction at the start of the simulation due to multiple ants in one place.
    ///
    /// # Arguments
    /// * `num_ants` - The total number of ants to create and place in the world.
    ///
    /// # Returns
    /// A vector of `Arc<RwLock<Ant>>` representing the newly initialized ants.
    pub fn initialize_ants(&mut self, num_ants: usize) -> Vec<Arc<RwLock<Ant>>> {
        let mut rng = rand::rng();
        let colony_names: Vec<String> = self.colonies.keys().cloned().collect();

        // Randomly select colonies to place ants in. `choose_multiple` ensures unique selections.
        let chosen_colonies = colony_names
            .choose_multiple(&mut rng, num_ants)
            .cloned()
            .collect::<Vec<_>>();

        let mut ants = Vec::new();
        for (i, colony_name) in chosen_colonies.iter().enumerate() {
            if let Some(colony_arc) = self.colonies.get(colony_name) {
                let mut colony = colony_arc.write().unwrap();

                let ant = Arc::new(RwLock::new(Ant {
                    id: i,
                    location: colony_arc.clone(),
                }));

                colony.ants.push(ant.clone());
                ants.push(ant);
            }
        }
        ants
    }
}
