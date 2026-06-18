use common_game::components::planet::Planet;

/// Trait for types that can be rendered as a human-readable string and printed
/// to stdout. Mainly used for debugging and logging game entities whose default
/// `Debug` output is too verbose or not formatted nicely enough.
pub trait Printable {
    fn print(&self);
    fn to_string(&self) -> String;
}

/// Pretty-prints a `Planet` showing its id, type, generators/combinators and
/// rocket-related state. The rocket presence line is only included when the
/// planet is actually allowed to host one.
impl Printable for Planet {
    fn print(&self) {
        println!("{}", self.to_string());
    }
    fn to_string(&self) -> String {
        let mut result = "Planet ".to_string();
        result.push_str(format!("{} {{\n", self.id()).as_str());
        result.push_str(format!("   Type: {:?}\n", self.planet_type()).as_str());
        result.push_str(format!("   Basic Resources: {:?}\n", self.generator()).as_str());
        result.push_str(format!("   Complex Resources: {:?}\n", self.combinator()).as_str());
        result.push_str(format!("   Num Energy Cells: {}\n", self.state().cells_count()).as_str());
        result
            .push_str(format!("   Rockets enabled: {}\n", self.state().can_have_rocket()).as_str());
        // Only meaningful when the planet supports rockets, otherwise omitted to avoid noise.
        if self.state().can_have_rocket() {
            result.push_str(format!("   Has rocket: {}\n", self.state().has_rocket()).as_str());
        }
        result.push_str("}\n");
        result
    }
}
