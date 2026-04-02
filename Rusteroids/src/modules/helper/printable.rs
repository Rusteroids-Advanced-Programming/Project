use common_game::components::planet::Planet;

pub trait Printable {
    fn print(&self);
    fn to_string(&self) -> String;
}

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
        if self.state().can_have_rocket() {
            result.push_str(format!("   Has rocket: {}\n", self.state().has_rocket()).as_str());
        }
        result.push_str("}\n");
        result
    }
}
