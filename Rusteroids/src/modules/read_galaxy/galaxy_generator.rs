use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Procedurally generates a topological galaxy graph file ensuring a backbone ring layout augmented with random chord links.
pub fn generate_galaxy_file(num_planets: usize) -> std::io::Result<()> {
    // Structural sanity check to ensure the topological algorithm has enough nodes to build the network ring
    if num_planets < 7 {
        panic!("At least 7 planets");
    }

    let output_filename = "galaxy-initialization.txt";

    // Compute dynamic density bounds for the additional random graph paths based on total planet count
    let min_rand_neighbors = ((num_planets as f64) * 0.05).ceil() as usize;
    let max_rand_neighbors = ((num_planets as f64) * 0.10).ceil() as usize;

    let mut galaxy: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_planets + 1];
    let mut rng = rand::rng();

    // Backbone Pass: Enforce global network connectivity by stitching all nodes into a bidirectional ring loop
    for i in 1..=num_planets {
        let prev_neighbor = if i == 1 { num_planets } else { i - 1 };
        let next_neighbor = if i == num_planets { 1 } else { i + 1 };

        galaxy[i].insert(prev_neighbor);
        galaxy[i].insert(next_neighbor);
    }

    // Chordal Pass: Inject random shortcuts to convert the simple ring into a more complex small-world topological graph
    for i in 1..=num_planets {
        let neighbors_num = rng.random_range(min_rand_neighbors..=max_rand_neighbors);

        // Filter candidates to exclude self-loops and nodes that are already linked as immediate backbone neighbors
        let mut potential_candidates: Vec<usize> = (1..=num_planets)
            .filter(|&p| p != i && !galaxy[i].contains(&p))
            .collect();

        potential_candidates.shuffle(&mut rng);

        let neighbors_chosen = neighbors_num.min(potential_candidates.len());

        for v in potential_candidates.into_iter().take(neighbors_chosen) {
            galaxy[i].insert(v);
        }
    }

    // Serialization Pass: Write out the generated topology map using buffered IO for higher performance
    let file = File::create(output_filename)?;
    let mut writer = BufWriter::new(file);

    for i in 1..=num_planets {
        let neighbors_str: Vec<String> = galaxy[i].iter().map(|id| id.to_string()).collect();

        // Write format: "[Planet_ID] [Space-separated list of neighboring Planet_IDs]"
        writeln!(writer, "{} {}", i, neighbors_str.join(" "))?;
    }

    writer.flush()?;

    Ok(())
}
