pub mod build_data_structs;
pub mod graph;
pub mod planet_map;
pub mod read_galaxy_file;
pub mod stats;

mod tests {
    use crate::modules::read_galaxy::build_data_structs;
    use crate::modules::read_galaxy::read_galaxy_file::read_galaxy_file;

    #[test]
    fn test_read_galaxy() {
        let expected: Vec<Vec<u32>> = vec![vec![1, 2, 3], vec![2, 1, 3], vec![3, 1, 2]];

        assert_eq!(expected, read_galaxy_file().unwrap());
    }

    #[test]
    fn test_graph() {
        let mut graph = build_data_structs::build_galaxy_graph();
        graph.remove_node(1);
        let expected = "Graph {\n  2 -> [3]\n  3 -> [2]\n}\n".to_string();
        // graph.remove_node(2);
        // let mut result = String::new();
        // for node in &graph.nodes {
        //     result = format!("{}\n{:?}\n", result, node);
        // }
        // assert_eq!("RefCell { value: Node { value: 3, adjacent_nodes: [] } }".to_string(), result.trim());
        assert_eq!(expected, format!("{:?}", graph));
    }
}
