use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_galaxy_file() -> Result<Vec<Vec<u32>>, Box<dyn Error>> {
    let path = "./galaxy-initialization.txt";
    let file = File::open(path);
    let mut result = Vec::new();

    match file {
        Err(e) => Err(Box::new(e)),
        Ok(file) => {
            let reader = BufReader::new(file);

            for line in reader.lines() {
                match line {
                    Err(e) => {
                        return Err(Box::new(e));
                    }
                    Ok(line) => {
                        let parts: Vec<u32> = line
                            .as_str()
                            .trim()
                            .split_whitespace()
                            .map(|s| s.parse::<u32>().unwrap())
                            .collect();
                        result.push(parts);
                    }
                }
            }
            Ok(result)
        }
    }
}
