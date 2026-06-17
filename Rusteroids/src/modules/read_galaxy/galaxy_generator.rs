use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Write, BufWriter};
use rand::seq::SliceRandom;
use rand::Rng;


pub fn generate_galaxy_file(num_planets: usize) -> std::io::Result<()> {
    if num_planets < 4 {
        panic!("Almeno 4 pianeti");
    }

    let output_filename = "galaxy-initialization.txt";

    let min_rand_neighbors = ((num_planets as f64) * 0.05).ceil() as usize;
    let max_rand_neighbors = ((num_planets as f64) * 0.10).ceil() as usize;

    let mut galaxy: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_planets + 1];
    let mut rng = rand::rng();

    for i in 1..=num_planets {
        let prev_neighbor = if i == 1 { num_planets } else { i - 1 };
        let next_neighbor = if i == num_planets { 1 } else { i + 1 };

        galaxy[i].insert(prev_neighbor);
        galaxy[i].insert(next_neighbor);
    }

    for i in 1..=num_planets {
        let quanti_random = rng.random_range(min_rand_neighbors..=max_rand_neighbors);

        let mut potential_candidates: Vec<usize> = (1..=num_planets)
            .filter(|&p| p != i && !galaxy[i].contains(&p))
            .collect();

        potential_candidates.shuffle(&mut rng);

        let quanti_da_prendere = quanti_random.min(potential_candidates.len());

        for v in potential_candidates.into_iter().take(quanti_da_prendere) {
            galaxy[i].insert(v);
        }
    }

    let file = File::create(output_filename)?;
    let mut writer = BufWriter::new(file);

    for i in 1..=num_planets {
        let neighbors_str: Vec<String> = galaxy[i]
            .iter()
            .map(|id| id.to_string())
            .collect();


        writeln!(writer, "{} {}", i, neighbors_str.join(" "))?;
    }

    writer.flush()?;

    println!("Nuova galassia generata in '{}'", output_filename);

    Ok(())
}