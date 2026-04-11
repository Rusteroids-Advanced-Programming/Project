use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use crossbeam_channel::{Receiver, Sender};
use rand::seq::SliceRandom;
use rand::{rng};

use common_game::components::resource::{
    BasicResource, BasicResourceType, ComplexResourceRequest, ComplexResourceType,
    GenericResource, ResourceType
};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use rand::prelude::IndexedRandom;
use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};
use crate::modules::manual_explorer::explorer_ai::ExplorerAI;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ExplorerGoal { Normal, Secret }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ResType { Basic(BasicResourceType), Complex(ComplexResourceType) }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TaskType { Mapping, Hoarding, HubHunt }

#[derive(Debug, Clone, PartialEq)]
enum AIState { Mapping, Hoarding, HubHunting, Gathering, Returning, Idle, Fleeing }

#[derive(Debug, Default, Clone)]
struct MissionLog {
    map_complete: bool, thirty_basic_total: bool, five_of_each_basic: bool, one_of_each_complex: bool,
    network_technician: bool, nomad: bool, hub_hunter: bool, hoarder: bool, victory_lap: bool,
    surveyor: bool, efficiency_expert: bool, localvore: bool, scavenger: bool,
    speedrunner: bool, backpacker: bool, daredevil: bool, monopolist: bool, industrialist: bool,
    bizarre_explorer: bool, grand_slam: bool,
}

#[derive(Debug, Clone)]
pub struct PlanetKnowledge {
    pub id: ID,
    pub neighbors: Vec<ID>,
    pub generates: Option<Vec<BasicResourceType>>,
    pub combines: Option<Vec<ComplexResourceType>>,
}

#[derive(Clone)]
pub struct SmartExplorer {
    pub explorer_id: ID,
    pub current_planet_id: Arc<RwLock<ID>>,
    pub spawn_planet_id: Arc<RwLock<ID>>,
    pub bag: Arc<RwLock<BagType>>,
    pub target_goal: Arc<RwLock<ExplorerGoal>>,
    pub to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
    pub from_orchestrator: Receiver<OrchestratorToExplorer>,
    pub to_planet: Arc<RwLock<Option<Sender<ExplorerToPlanet>>>>,
    pub from_planet: Arc<RwLock<Option<Receiver<PlanetToExplorer>>>>,
    pub neighbours: Arc<RwLock<Vec<ID>>>,
    pub basic_resources: Arc<RwLock<HashSet<BasicResourceType>>>,
    pub combinations: Arc<RwLock<HashSet<ComplexResourceType>>>,
    pub knowledge_base: Arc<RwLock<HashMap<ID, PlanetKnowledge>>>,
    pub visited: Arc<RwLock<HashSet<ID>>>,
    pub visited_edges: Arc<RwLock<HashSet<(ID, ID)>>>,
    pub blocked_edges: Arc<RwLock<HashSet<(ID, ID)>>>,
    pub unreachable_planets: Arc<RwLock<HashSet<ID>>>,
    abandoned_tasks: Arc<RwLock<HashSet<TaskType>>>,
    pub arrival_time: Arc<RwLock<SystemTime>>,
    pub planet_cooldowns: Arc<RwLock<HashMap<ID, SystemTime>>>,
    pub total_visits: Arc<RwLock<u32>>,
    pub total_steps: Arc<RwLock<u32>>,
    pub planet_visit_counts: Arc<RwLock<HashMap<ID, u32>>>,
    pub last_visit_time: Arc<RwLock<HashMap<ID, SystemTime>>>,
    pub crafted_history: Arc<RwLock<HashMap<ID, Vec<ComplexResourceType>>>>,
    pub triggered_danger: Arc<RwLock<bool>>,
    pub daredevil_event: Arc<RwLock<bool>>,
    pub localvore_event: Arc<RwLock<bool>>,
    pub efficiency_achieved: Arc<RwLock<bool>>,
    pub extraction_sites: Arc<RwLock<HashSet<ID>>>,
    pub hub_candidate: Arc<RwLock<(ID, usize)>>,
    pub hub_visits: Arc<RwLock<HashMap<ID, u32>>>,
    pub max_bag_size: Arc<RwLock<usize>>,
    pub recent_path: Arc<RwLock<VecDeque<ID>>>,
    state: Arc<RwLock<AIState>>,
    dfs_stack: Arc<RwLock<Vec<ID>>>,
}

impl SmartExplorer {
    pub fn new(explorer_id: ID, current_planet_id: ID, from_orchestrator: Receiver<OrchestratorToExplorer>, to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>) -> Self {
        Self {
            explorer_id,
            current_planet_id: Arc::new(RwLock::new(current_planet_id)),
            spawn_planet_id: Arc::new(RwLock::new(current_planet_id)),
            bag: Arc::new(RwLock::new(BagType::new())),
            target_goal: Arc::new(RwLock::new(ExplorerGoal::Normal)),
            to_orchestrator, from_orchestrator,
            to_planet: Arc::new(RwLock::new(None)), from_planet: Arc::new(RwLock::new(None)),
            neighbours: Arc::new(RwLock::new(Vec::new())), basic_resources: Arc::new(RwLock::new(HashSet::new())),
            combinations: Arc::new(RwLock::new(HashSet::new())), knowledge_base: Arc::new(RwLock::new(HashMap::new())),
            visited: Arc::new(RwLock::new(HashSet::new())), visited_edges: Arc::new(RwLock::new(HashSet::new())),
            blocked_edges: Arc::new(RwLock::new(HashSet::new())), unreachable_planets: Arc::new(RwLock::new(HashSet::new())),
            abandoned_tasks: Arc::new(RwLock::new(HashSet::new())), arrival_time: Arc::new(RwLock::new(SystemTime::now())),
            planet_cooldowns: Arc::new(RwLock::new(HashMap::new())), total_visits: Arc::new(RwLock::new(1)),
            total_steps: Arc::new(RwLock::new(0)), planet_visit_counts: Arc::new(RwLock::new(HashMap::new())),
            last_visit_time: Arc::new(RwLock::new(HashMap::new())), crafted_history: Arc::new(RwLock::new(HashMap::new())),
            triggered_danger: Arc::new(RwLock::new(false)), daredevil_event: Arc::new(RwLock::new(false)),
            localvore_event: Arc::new(RwLock::new(false)), efficiency_achieved: Arc::new(RwLock::new(false)),
            extraction_sites: Arc::new(RwLock::new(HashSet::new())), hub_candidate: Arc::new(RwLock::new((current_planet_id, 0))),
            hub_visits: Arc::new(RwLock::new(HashMap::new())), max_bag_size: Arc::new(RwLock::new(0)),
            recent_path: Arc::new(RwLock::new(VecDeque::with_capacity(8))), state: Arc::new(RwLock::new(AIState::Mapping)),
            dfs_stack: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn set_goal(&self, goal: ExplorerGoal) { *self.target_goal.write().unwrap() = goal; }

    pub fn run(&self) -> Result<(), String> {
        println!("SmartExplorer #{} initialized.", self.explorer_id);
        loop {
            match self.from_orchestrator.recv() {
                Ok(msg) => match msg {
                    OrchestratorToExplorer::StartExplorerAI => self.start_ai(),
                    OrchestratorToExplorer::ResetExplorerAI => self.reset_ai(),
                    OrchestratorToExplorer::KillExplorer => { self.kill(); return Ok(()); },
                    OrchestratorToExplorer::MoveToPlanet {sender_to_new_planet, planet_id} => self.move_to_planet(sender_to_new_planet, planet_id),
                    OrchestratorToExplorer::NeighborsResponse {neighbors} => self.set_neighbours(neighbors),
                    OrchestratorToExplorer::CurrentPlanetRequest => self.get_current_planet(),
                    OrchestratorToExplorer::BagContentRequest => self.get_bag(),
                    _ => {}
                },
                Err(_) => return Err("Orchestrator Disconnected".to_string()),
            }
        }
    }

    fn update_hoarder_score(&self) {
        let bag = self.bag.read().unwrap().to_dummy();
        let count = bag.basic.values().sum::<usize>() + bag.complex.values().sum::<usize>();
        let mut max = self.max_bag_size.write().unwrap();
        if count > *max { *max = count; }
    }

    pub fn get_risk_display(&self) -> String {
        let elapsed = self.arrival_time.read().unwrap().elapsed().unwrap_or(Duration::from_secs(0)).as_secs();
        if elapsed < 5 { format!("✅ Safe ({}s)", elapsed) }
        else if elapsed < 10 { format!("⚠️ Risky ({}s)", elapsed) }
        else if elapsed < 15 { format!("⚡ Dangerous ({}s)", elapsed) }
        else if elapsed < 25 { format!("🔴 VERY DANGEROUS!!! ({}s)", elapsed) }
        else { format!("💀 EXPULSION IMMINENT ({}s)", elapsed) }
    }

    fn get_risk_level(&self) -> (String, bool) {
        let elapsed = self.arrival_time.read().unwrap().elapsed().unwrap_or(Duration::from_secs(0)).as_secs();
        if elapsed >= 10 { *self.triggered_danger.write().unwrap() = true; }
        (self.get_risk_display(), elapsed >= 25)
    }

    fn is_secret_possible(&self) -> bool {
        let kb = self.knowledge_base.read().unwrap();
        let unreachable = self.unreachable_planets.read().unwrap();
        let mut partner_factories = Vec::new();
        for (pid, data) in kb.iter() {
            if data.combines.as_ref().map_or(false, |c| c.contains(&ComplexResourceType::AIPartner)) {
                partner_factories.push(*pid);
            }
        }
        for pid in partner_factories {
            if unreachable.contains(&pid) { return false; }
        }
        true
    }

    fn get_reachable_subgraph(&self) -> HashSet<ID> {
        let start_node = *self.current_planet_id.read().unwrap();
        let kb = self.knowledge_base.read().unwrap();
        let unreachable = self.unreachable_planets.read().unwrap();
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node);
        reachable.insert(start_node);
        while let Some(curr) = queue.pop_front() {
            if let Some(data) = kb.get(&curr) {
                for &n in &data.neighbors {
                    if !unreachable.contains(&n) && !reachable.contains(&n) {
                        reachable.insert(n);
                        queue.push_back(n);
                    }
                }
            }
        }
        reachable
    }

    pub fn get_mission_progress_strings(&self) -> HashMap<String, String> {
        let mut output = HashMap::new();
        let bag = self.bag.read().unwrap().to_dummy();
        let kb = self.knowledge_base.read().unwrap();
        let visits = self.planet_visit_counts.read().unwrap();
        let kb_len = kb.len();
        output.insert("Map".to_string(), if kb_len >= 8 { "✅".to_string() } else { format!("{}%", (kb_len as f32 / 8.0 * 100.0) as u32) });
        let mut total_edges = 0;
        for data in kb.values() { total_edges += data.neighbors.len(); }
        let visited_edges = self.visited_edges.read().unwrap().len();
        output.insert("Net".to_string(), if visited_edges >= total_edges && total_edges > 0 { "✅".to_string() } else if total_edges > 0 { format!("{}%", (visited_edges as f32 / total_edges as f32 * 100.0) as u32) } else { "0%".to_string() });
        let sites = self.extraction_sites.read().unwrap().len();
        output.insert("Nomad".to_string(), if sites >= 5 { "✅".to_string() } else { format!("{}/5", sites) });
        let (hub_id, _) = *self.hub_candidate.read().unwrap();
        let hub_v = *self.hub_visits.read().unwrap().get(&hub_id).unwrap_or(&0);
        output.insert("Hub".to_string(), if hub_v >= 3 { "✅".to_string() } else { format!("{}/3", hub_v) });
        let total_b: usize = bag.basic.values().sum();
        output.insert("30+".to_string(), if total_b >= 30 { "✅".to_string() } else { format!("{}/30", total_b) });
        let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
        let min_b = basics.iter().map(|b| *bag.basic.get(b).unwrap_or(&0)).min().unwrap_or(0);
        output.insert("5ea".to_string(), if min_b >= 5 { "✅".to_string() } else { format!("Min {}/5", min_b) });
        let max_bag = *self.max_bag_size.read().unwrap();
        output.insert("Hoard".to_string(), if max_bag >= 67 { "✅".to_string() } else { format!("{}/67", max_bag) });
        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
        let crafted_count = complexes.iter().filter(|c| *bag.complex.get(c).unwrap_or(&0) >= 1).count();
        output.insert("Craft".to_string(), if crafted_count == 6 { "✅".to_string() } else { format!("{}/6", crafted_count) });
        let planets_3x = visits.values().filter(|&&v| v >= 3).count();
        output.insert("Survey".to_string(), if kb_len >= 8 && planets_3x >= 8 { "✅".to_string() } else { format!("{}/8", planets_3x) });

        // --- FIXED: Harder Efficiency (70) ---
        let steps = *self.total_steps.read().unwrap();
        let eff_done = *self.efficiency_achieved.read().unwrap();
        if eff_done { output.insert("Eff".to_string(), "✅".to_string()); }
        else if steps > 70 { output.insert("Eff".to_string(), "❌".to_string()); }
        else { output.insert("Eff".to_string(), format!("{}s", steps)); }

        let needed = (kb_len as f32 * 0.8).ceil() as usize;
        output.insert("Scav".to_string(), if sites >= needed && needed > 0 { "✅".to_string() } else { format!("{}/{}", sites, needed) });
        output.insert("Loc".to_string(), if *self.localvore_event.read().unwrap() { "✅".to_string() } else { "Pending".to_string() });
        output.insert("Speed".to_string(), if *self.triggered_danger.read().unwrap() { "❌".to_string() } else { "Active".to_string() });
        output.insert("Dare".to_string(), if *self.daredevil_event.read().unwrap() { "✅".to_string() } else { "Pending".to_string() });

        let saturated_count = basics.iter().filter(|&b| *bag.basic.get(b).unwrap_or(&0) >= 15).count();
        output.insert("Mono".to_string(), if saturated_count == 4 { "✅".to_string() } else { format!("{}/4", saturated_count) });
        output
    }

    fn check_mission_status(&self) -> MissionLog {
        let mut status = MissionLog::default();
        let bag = self.bag.read().unwrap().to_dummy();
        let kb_size = self.knowledge_base.read().unwrap().len();
        status.map_complete = kb_size >= 8;
        status.thirty_basic_total = bag.basic.values().sum::<usize>() >= 30;
        let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
        status.five_of_each_basic = basics.iter().all(|b| *bag.basic.get(b).unwrap_or(&0) >= 5);
        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
        status.one_of_each_complex = complexes.iter().all(|c| *bag.complex.get(c).unwrap_or(&0) >= 1);
        let visited_edges_count = self.visited_edges.read().unwrap().len();
        let mut total_known_edges = 0;
        for data in self.knowledge_base.read().unwrap().values() { total_known_edges += data.neighbors.len(); }
        status.network_technician = visited_edges_count >= total_known_edges && total_known_edges > 0;
        status.nomad = self.extraction_sites.read().unwrap().len() >= 5;
        let (hub_id, _) = *self.hub_candidate.read().unwrap();
        status.hub_hunter = *self.hub_visits.read().unwrap().get(&hub_id).unwrap_or(&0) >= 3;
        status.hoarder = *self.max_bag_size.read().unwrap() >= 67;
        let pos = *self.current_planet_id.read().unwrap();
        let spawn = *self.spawn_planet_id.read().unwrap();
        let all_tasks_original = status.map_complete && status.thirty_basic_total && status.five_of_each_basic && status.one_of_each_complex && status.network_technician && status.nomad && status.hub_hunter && status.hoarder;
        if all_tasks_original && pos == spawn { status.victory_lap = true; }

        status.monopolist = basics.iter().all(|b| *bag.basic.get(b).unwrap_or(&0) >= 15);

        let visits = self.planet_visit_counts.read().unwrap();
        status.surveyor = kb_size >= 8 && visits.values().filter(|&&v| v >= 3).count() >= 8;
        let steps = *self.total_steps.read().unwrap();
        if *self.efficiency_achieved.read().unwrap() {
            status.efficiency_expert = true;
        } else {
            if status.one_of_each_complex && steps <= 70 {
                *self.efficiency_achieved.write().unwrap() = true;
                status.efficiency_expert = true;
            } else if steps > 70 {
                status.efficiency_expert = false;
            }
        }
        status.speedrunner = !*self.triggered_danger.read().unwrap();
        let needed_scav = (kb_size as f32 * 0.8).ceil() as usize;
        status.scavenger = self.extraction_sites.read().unwrap().len() >= needed_scav && needed_scav > 0;
        status.backpacker = bag.basic.values().sum::<usize>() == 0;
        status.daredevil = *self.daredevil_event.read().unwrap();
        status.localvore = *self.localvore_event.read().unwrap();
        let hist = self.crafted_history.read().unwrap();
        let mut all_factories_used = true;
        let kb = self.knowledge_base.read().unwrap();
        for (pid, data) in kb.iter() {
            if data.combines.as_ref().map_or(false, |c| !c.is_empty()) {
                if !hist.contains_key(pid) { all_factories_used = false; break; }
            }
        }
        status.industrialist = all_factories_used && kb_size >= 8;
        let prereqs = all_tasks_original && status.monopolist && status.scavenger && status.industrialist;
        let mut bizarre_condition = true;
        if prereqs {
            for (pid, data) in kb.iter() {
                if data.combines.as_ref().map_or(false, |c| c.contains(&ComplexResourceType::AIPartner)) {
                    let count = hist.get(pid).map_or(0, |list| list.iter().filter(|&&x| x == ComplexResourceType::AIPartner).count());
                    if count < 2 { bizarre_condition = false; break; }
                }
            }
        } else { bizarre_condition = false; }
        status.bizarre_explorer = bizarre_condition;
        if all_tasks_original && status.victory_lap && status.surveyor && status.efficiency_expert && status.localvore {
            status.grand_slam = true;
        }
        status
    }

    fn scan_current_planet(&self) {
        let current_id = *self.current_planet_id.read().unwrap();
        self.ask_for_neighbours();
        self.ask_supported_resources();
        self.ask_combinations();
        sleep(Duration::from_millis(500));
        let mut kb = self.knowledge_base.write().unwrap();
        let neighbors_data = self.neighbours.read().unwrap().clone();
        let basic_data = self.basic_resources.read().unwrap().clone();
        let complex_data = self.combinations.read().unwrap().clone();

        let now = SystemTime::now();
        self.last_visit_time.write().unwrap().insert(current_id, now);

        {
            let mut visits = self.planet_visit_counts.write().unwrap();
            *visits.entry(current_id).or_insert(0) += 1;
        }
        let neighbor_count = neighbors_data.len();
        {
            let mut hub_cand = self.hub_candidate.write().unwrap();
            if neighbor_count > hub_cand.1 {
                *hub_cand = (current_id, neighbor_count);
            }
            let mut visits = self.hub_visits.write().unwrap();
            *visits.entry(current_id).or_insert(0) += 1;
        }
        kb.insert(current_id, PlanetKnowledge {
            id: current_id, neighbors: neighbors_data, generates: Some(basic_data.into_iter().collect()), combines: Some(complex_data.into_iter().collect()),
        });
        self.visited.write().unwrap().insert(current_id);
    }

    // ... helpers ...
    fn find_path_to_planet(&self, target_id: ID) -> Option<Vec<ID>> {
        let start_node = *self.current_planet_id.read().unwrap();
        if start_node == target_id { return Some(vec![]); }
        let knowledge = self.knowledge_base.read().unwrap();
        let blocked = self.blocked_edges.read().unwrap();
        let cooldowns = self.planet_cooldowns.read().unwrap();
        let mut queue = VecDeque::new();
        let mut visited_bfs = HashSet::new();
        let mut came_from: HashMap<ID, ID> = HashMap::new();
        queue.push_back(start_node);
        visited_bfs.insert(start_node);
        while let Some(current) = queue.pop_front() {
            if current == target_id {
                let mut path = Vec::new();
                let mut curr = target_id;
                while curr != start_node {
                    path.push(curr);
                    curr = *came_from.get(&curr).unwrap();
                }
                path.reverse();
                return Some(path);
            }
            if let Some(planet_data) = knowledge.get(&current) {
                for &neighbor in &planet_data.neighbors {
                    let is_cd = if let Some(time) = cooldowns.get(&neighbor) { SystemTime::now() < *time } else { false };
                    if !blocked.contains(&(current, neighbor)) && !visited_bfs.contains(&neighbor) && !is_cd {
                        visited_bfs.insert(neighbor);
                        came_from.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        None
    }

    fn get_ingredients(&self, target: ComplexResourceType) -> Vec<ResType> {
        match target {
            ComplexResourceType::Water => vec![ResType::Basic(BasicResourceType::Hydrogen), ResType::Basic(BasicResourceType::Oxygen)],
            ComplexResourceType::Diamond => vec![ResType::Basic(BasicResourceType::Carbon), ResType::Basic(BasicResourceType::Carbon)],
            ComplexResourceType::Life => vec![ResType::Basic(BasicResourceType::Carbon), ResType::Complex(ComplexResourceType::Water)],
            ComplexResourceType::Robot => vec![ResType::Basic(BasicResourceType::Silicon), ResType::Complex(ComplexResourceType::Life)],
            ComplexResourceType::Dolphin => vec![ResType::Complex(ComplexResourceType::Water), ResType::Complex(ComplexResourceType::Life)],
            ComplexResourceType::AIPartner => vec![ResType::Complex(ComplexResourceType::Robot), ResType::Complex(ComplexResourceType::Diamond)],
        }
    }

    fn attempt_extract(&self, res: BasicResourceType) -> bool {
        {
            let bag = self.bag.read().unwrap().to_dummy();
            let count = *bag.basic.get(&res).unwrap_or(&0);
            if count >= 15 { return false; }
        }
        for _ in 1..=2 {
            let energy = self.ask_available_cells();
            if energy > 0 {
                let elapsed = self.arrival_time.read().unwrap().elapsed().unwrap_or(Duration::from_secs(0)).as_secs();
                if elapsed >= 10 { *self.daredevil_event.write().unwrap() = true; }
                self.generate_resource(res);
                sleep(Duration::from_millis(200));
                let current_id = *self.current_planet_id.read().unwrap();
                self.extraction_sites.write().unwrap().insert(current_id);
                self.update_hoarder_score();
                return true;
            } else { sleep(Duration::from_millis(500)); }
        }
        false
    }

    fn has_resource(&self, res: ResType) -> bool {
        let bag = self.bag.read().unwrap();
        let dummy = bag.to_dummy();
        match res {
            ResType::Basic(b) => dummy.basic.get(&b).map_or(false, |&count| count > 0),
            ResType::Complex(c) => dummy.complex.get(&c).map_or(false, |&count| count > 0),
        }
    }

    fn generate_resource(&self, to_generate: BasicResourceType) {
        let to = self.to_planet.read().unwrap();
        let from = self.from_planet.read().unwrap();
        if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
            if tx.send(ExplorerToPlanet::GenerateResourceRequest {explorer_id: self.explorer_id, resource: to_generate}).is_ok() {
                if let Ok(PlanetToExplorer::GenerateResourceResponse {resource}) = rx.recv_timeout(Duration::from_millis(1000)) {
                    if let Some(r) = resource {
                        let r_type = r.get_type();
                        self.bag.write().unwrap().add_basic_resource(r);
                        println!("AI [Extraction]: Successfully extracted {:?} from Planet!", r_type);
                    }
                }
            }
        }
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::GenerateResourceResponse {explorer_id: self.explorer_id, generated: Ok(())});
    }

    fn combine_resource(&self, to_generate: ComplexResourceType) {
        let req_opt = self.get_complex_resource_request(to_generate);
        if let Some(req) = req_opt {
            let to = self.to_planet.read().unwrap();
            let from = self.from_planet.read().unwrap();
            if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
                if tx.send(ExplorerToPlanet::CombineResourceRequest {explorer_id: self.explorer_id, msg: req}).is_ok() {
                    match rx.recv_timeout(Duration::from_millis(1000)) {
                        Ok(PlanetToExplorer::CombineResourceResponse{complex_response}) => {
                            if let Ok(r) = complex_response {
                                let r_type = r.get_type();
                                self.bag.write().unwrap().add_complex_resource(r);
                                let current_id = *self.current_planet_id.read().unwrap();
                                let mut hist = self.crafted_history.write().unwrap();
                                hist.entry(current_id).or_insert(Vec::new()).push(r_type);
                                println!("AI [Bag Update]: Crafted {:?} | New Count: {}", r_type,
                                         self.bag.read().unwrap().to_dummy().complex.get(&r_type).unwrap_or(&0));
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::CombineResourceResponse {explorer_id: self.explorer_id, generated: Ok(())});
    }
}

// --- TRAIT IMPLEMENTATION ---
impl ExplorerAI for SmartExplorer {
    fn start_ai(&self) {
        self.to_orchestrator.send(ExplorerToOrchestrator::StartExplorerAIResult {explorer_id: self.explorer_id}).unwrap();
        let ai = self.clone();

        thread::spawn(move || {
            sleep(Duration::from_millis(1000));
            ai.scan_current_planet();

            loop {
                sleep(Duration::from_millis(500));
                let mission = ai.check_mission_status();
                if mission.grand_slam && *ai.target_goal.read().unwrap() == ExplorerGoal::Normal { break; }
                if mission.bizarre_explorer { break; }

                if *ai.target_goal.read().unwrap() == ExplorerGoal::Secret {
                    if !ai.is_secret_possible() { *ai.target_goal.write().unwrap() = ExplorerGoal::Normal; }
                }

                let current_id = *ai.current_planet_id.read().unwrap();
                let bag_dummy = ai.bag.read().unwrap().to_dummy();
                let abandoned = ai.abandoned_tasks.read().unwrap();
                let (_, must_flee) = ai.get_risk_level();
                let mut current_state = AIState::Idle;
                let goal = *ai.target_goal.read().unwrap();

                if must_flee {
                    current_state = AIState::Fleeing;
                } else {
                    let need_mapping = !mission.map_complete || !mission.network_technician;
                    let basics_saturated = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen]
                        .iter().all(|b| *bag_dummy.basic.get(b).unwrap_or(&0) >= 15);

                    // --- FORCE GATHERING IF BAG IS SATURATED ---
                    let needs_complex_for_badge = !mission.one_of_each_complex;
                    let needs_hoard_density = !mission.hoarder;
                    let needs_secret_crafts = goal == ExplorerGoal::Secret && !mission.bizarre_explorer;

                    let must_craft = basics_saturated && (needs_complex_for_badge || needs_hoard_density || needs_secret_crafts);

                    // --- STRICT VICTORY CONDITIONS (STOP GRINDING) ---
                    let true_ending_ready = mission.map_complete && mission.hoarder && mission.one_of_each_complex && mission.surveyor && mission.network_technician;
                    let efficiency_failed = *ai.total_steps.read().unwrap() > 70; // Hard cap

                    if goal == ExplorerGoal::Normal && true_ending_ready {
                        current_state = AIState::Returning;
                    }
                    else if goal == ExplorerGoal::Secret && true_ending_ready && mission.monopolist && mission.scavenger && mission.industrialist && mission.bizarre_explorer {
                        current_state = AIState::Returning;
                    }
                    // Fallback: If Secret Mode failed because we ran out of time/resources, just end it.
                    else if goal == ExplorerGoal::Secret && efficiency_failed && true_ending_ready {
                        current_state = AIState::Returning;
                    }
                    else if must_craft {
                        current_state = AIState::Gathering;
                    } else {
                        let reachable = ai.get_reachable_subgraph();
                        let visited = ai.visited.read().unwrap();
                        let all_reachable_visited = reachable.iter().all(|n| visited.contains(n));

                        // --- OPTIMIZED PRIORITY: Efficiency > Map > Survey > Secret > Home ---
                        if !mission.one_of_each_complex {
                            // 1. Efficiency First (1 of each)
                            current_state = AIState::Gathering;
                        } else if need_mapping && !abandoned.contains(&TaskType::Mapping) {
                            // 2. Map & Net
                            current_state = AIState::Mapping;
                        } else if !mission.surveyor {
                            // 3. Survey (3x visits)
                            current_state = AIState::HubHunting;
                        } else if goal == ExplorerGoal::Secret && !mission.bizarre_explorer {
                            // 4. Secret Task (Waifus)
                            current_state = AIState::Gathering;
                        } else if all_reachable_visited && need_mapping {
                            // Fallback for islands
                            current_state = AIState::Hoarding;
                        } else if !mission.hoarder || !mission.thirty_basic_total || !mission.five_of_each_basic || !mission.nomad {
                            if !abandoned.contains(&TaskType::Hoarding) { current_state = AIState::Hoarding; }
                            else { current_state = AIState::HubHunting; }
                        } else {
                            // --- STOP GRINDING ---
                            current_state = AIState::Returning;
                        }
                    }
                }

                match current_state {
                    AIState::Fleeing => {
                        let cooldown_until = SystemTime::now() + Duration::from_secs(7);
                        ai.planet_cooldowns.write().unwrap().insert(current_id, cooldown_until);
                        let kb = ai.knowledge_base.read().unwrap();
                        let blocked = ai.blocked_edges.read().unwrap();
                        let unreachable = ai.unreachable_planets.read().unwrap();
                        let cooldowns = ai.planet_cooldowns.read().unwrap();
                        if let Some(data) = kb.get(&current_id) {
                            let mut rng = rng();
                            let candidates: Vec<ID> = data.neighbors.iter()
                                .cloned()
                                .filter(|n| !blocked.contains(&(current_id, *n)) && !unreachable.contains(n))
                                .collect();

                            if let Some(t) = candidates.choose(&mut rng) {
                                drop(kb); drop(blocked); drop(unreachable); drop(cooldowns);
                                ai.travel_request(*t); sleep(Duration::from_millis(1500));
                                if *ai.current_planet_id.read().unwrap() != current_id { ai.scan_current_planet(); }
                                else { ai.blocked_edges.write().unwrap().insert((current_id, *t)); }
                            }
                        }
                    },
                    AIState::Mapping => {
                        {
                            let basic = ai.basic_resources.read().unwrap().clone();
                            for res in basic {
                                let count = *ai.bag.read().unwrap().to_dummy().basic.get(&res).unwrap_or(&0);
                                if count < 15 { if !ai.attempt_extract(res) { break; } }
                            }
                        }
                        let mut next_move = None;
                        {
                            let kb = ai.knowledge_base.read().unwrap();
                            if let Some(planet_data) = kb.get(&current_id) {
                                let visited = ai.visited.read().unwrap();
                                let visited_edges = ai.visited_edges.read().unwrap();
                                let blocked = ai.blocked_edges.read().unwrap();
                                let unreachable = ai.unreachable_planets.read().unwrap();
                                let cooldowns = ai.planet_cooldowns.read().unwrap();

                                let mut best_staleness = Duration::new(0, 0);
                                let mut stale_target = None;
                                let last_visits = ai.last_visit_time.read().unwrap().clone();

                                next_move = planet_data.neighbors.iter().find(|&&n| {
                                    let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                    !visited.contains(&n) && !blocked.contains(&(current_id, n)) && !unreachable.contains(&n) && !is_cd
                                }).cloned();

                                if next_move.is_none() {
                                    next_move = planet_data.neighbors.iter().find(|&&n| {
                                        let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                        let edge_visited = visited_edges.contains(&(current_id, n)) || visited_edges.contains(&(n, current_id));
                                        !edge_visited && !blocked.contains(&(current_id, n)) && !unreachable.contains(&n) && !is_cd
                                    }).cloned();
                                }

                                if next_move.is_none() {
                                    for &n in &planet_data.neighbors {
                                        let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                        if !blocked.contains(&(current_id, n)) && !unreachable.contains(&n) && !is_cd {
                                            let staleness = if let Some(last_time) = last_visits.get(&n) {
                                                last_time.elapsed().unwrap_or(Duration::new(0, 0))
                                            } else { Duration::new(u64::MAX, 0) };

                                            if staleness > best_staleness {
                                                best_staleness = staleness;
                                                stale_target = Some(n);
                                            }
                                        }
                                    }
                                    next_move = stale_target;
                                }
                            }
                        }
                        if let Some(next_hop) = next_move {
                            ai.travel_request(next_hop); sleep(Duration::from_millis(1500));
                            if *ai.current_planet_id.read().unwrap() != current_id { ai.scan_current_planet(); }
                            else { ai.blocked_edges.write().unwrap().insert((current_id, next_hop)); }
                        } else {
                            let kb = ai.knowledge_base.read().unwrap();
                            if let Some(data) = kb.get(&current_id) {
                                let mut rng = rng();
                                let candidates: Vec<ID> = data.neighbors.iter()
                                    .cloned()
                                    .filter(|n| !ai.blocked_edges.read().unwrap().contains(&(current_id, *n)))
                                    .collect();

                                if let Some(rnd) = candidates.choose(&mut rng) {
                                    drop(kb);
                                    ai.travel_request(*rnd); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                                }
                            }
                        }
                    },
                    AIState::Hoarding => {
                        let basic_set = ai.basic_resources.read().unwrap().clone();
                        let mut extracted_any = false;
                        if !basic_set.is_empty() {
                            for res in basic_set {
                                if ai.attempt_extract(res) { extracted_any = true; }
                                if ai.bag.read().unwrap().to_dummy().basic.values().sum::<usize>() >= 67 { break; }
                            }
                        }
                        if !extracted_any {
                            let bag = ai.bag.read().unwrap().to_dummy();
                            let basics_list = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
                            let mut missing_resource = None;
                            for b in basics_list {
                                if *bag.basic.get(&b).unwrap_or(&0) < 15 { missing_resource = Some(b); break; }
                            }
                            let mut move_target = None;
                            if let Some(target_res) = missing_resource {
                                let kb = ai.knowledge_base.read().unwrap();
                                let unreachable = ai.unreachable_planets.read().unwrap();
                                for (pid, data) in kb.iter() {
                                    let is_cd = if let Some(t) = ai.planet_cooldowns.read().unwrap().get(pid) { SystemTime::now() < *t } else { false };
                                    if data.generates.as_ref().map_or(false, |g| g.contains(&target_res)) && !unreachable.contains(pid) && *pid != current_id && !is_cd {
                                        move_target = Some(*pid); break;
                                    }
                                }
                            }
                            if let Some(target_pid) = move_target {
                                if let Some(path) = ai.find_path_to_planet(target_pid) {
                                    if let Some(&next) = path.first() {
                                        ai.travel_request(next); sleep(Duration::from_millis(1500));
                                        if *ai.current_planet_id.read().unwrap() == current_id { ai.blocked_edges.write().unwrap().insert((current_id, next)); }
                                        else { ai.scan_current_planet(); }
                                    }
                                }
                            } else {
                                let kb = ai.knowledge_base.read().unwrap();
                                if let Some(data) = kb.get(&current_id) {
                                    let mut rng = rng();
                                    let candidates: Vec<ID> = data.neighbors.iter().cloned().collect();
                                    if let Some(rnd) = candidates.choose(&mut rng) {
                                        drop(kb);
                                        ai.travel_request(*rnd); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                                    }
                                }
                            }
                        }
                    },

                    AIState::HubHunting => {
                        let last_visits = ai.last_visit_time.read().unwrap().clone();
                        let kb = ai.knowledge_base.read().unwrap();
                        let cooldowns = ai.planet_cooldowns.read().unwrap();

                        let mut best_move = None;
                        let mut max_staleness = Duration::new(0, 0);

                        if let Some(data) = kb.get(&current_id) {
                            for &n in &data.neighbors {
                                let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                if !is_cd {
                                    let staleness = if let Some(last_time) = last_visits.get(&n) {
                                        last_time.elapsed().unwrap_or(Duration::new(0, 0))
                                    } else {
                                        Duration::new(u64::MAX, 0)
                                    };

                                    if staleness > max_staleness {
                                        max_staleness = staleness;
                                        best_move = Some(n);
                                    }
                                }
                            }
                        }

                        if let Some(target) = best_move {
                            drop(kb); drop(cooldowns);
                            ai.travel_request(target); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                        } else {
                            let blocked = ai.blocked_edges.read().unwrap();
                            if let Some(data) = kb.get(&current_id) {
                                let mut rng = rng();
                                let candidates: Vec<ID> = data.neighbors.iter()
                                    .cloned()
                                    .filter(|n| !blocked.contains(&(current_id, *n)))
                                    .collect();

                                if let Some(rnd) = candidates.choose(&mut rng) {
                                    drop(kb); drop(blocked);
                                    ai.travel_request(*rnd); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                                }
                            }
                        }
                    },

                    AIState::Returning => {
                        let spawn = *ai.spawn_planet_id.read().unwrap();
                        if let Some(path) = ai.find_path_to_planet(spawn) {
                            if let Some(&next) = path.first() {
                                ai.travel_request(next); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                            }
                        }
                    },
                    AIState::Gathering => {
                        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
                        let bag = ai.bag.read().unwrap().to_dummy();
                        let mut target = None;

                        // --- UNLIMITED CRAFTING FOR SECRET TASK ---
                        if *ai.target_goal.read().unwrap() == ExplorerGoal::Secret {
                            let kb = ai.knowledge_base.read().unwrap();
                            let hist = ai.crafted_history.read().unwrap();
                            for (pid, data) in kb.iter() {
                                if data.combines.as_ref().map_or(false, |c| c.contains(&ComplexResourceType::AIPartner)) {
                                    let count = hist.get(pid).map_or(0, |list| list.iter().filter(|&&x| x == ComplexResourceType::AIPartner).count());
                                    if count < 2 {
                                        target = Some(ComplexResourceType::AIPartner);
                                        break;
                                    }
                                }
                            }
                        }

                        // Default Gathering Logic: ONLY GET 1 IF IN NORMAL MODE AND SATISFIED
                        if target.is_none() {
                            for c in &complexes { if *bag.complex.get(c).unwrap_or(&0) < 1 { target = Some(*c); break; } }
                            // REMOVED "MAX 5" LOGIC for default behavior to prevent obsession.
                            // We only craft more if forced by saturation.
                            if target.is_none() {
                                let basics_saturated = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen]
                                    .iter().all(|b| *bag.basic.get(b).unwrap_or(&0) >= 15);
                                if basics_saturated {
                                    // Just pick whatever is easiest to free space
                                    for c in &complexes { if *bag.complex.get(c).unwrap_or(&0) < 5 { target = Some(*c); break; } }
                                }
                            }
                        }

                        // --- RECURSIVE DEPENDENCY RESOLUTION ---
                        if let Some(mut current_target) = target {
                            loop {
                                let ingredients = ai.get_ingredients(current_target);
                                let mut missing_complex = None;
                                for ing in &ingredients {
                                    if let ResType::Complex(c) = ing {
                                        if *bag.complex.get(c).unwrap_or(&0) < 1 {
                                            missing_complex = Some(*c);
                                            break;
                                        }
                                    }
                                }
                                if let Some(missing) = missing_complex {
                                    current_target = missing;
                                } else {
                                    target = Some(current_target);
                                    break;
                                }
                            }
                        }

                        if let Some(c) = target {
                            let ingredients = ai.get_ingredients(c);
                            let mut can_craft = true;
                            for ing in &ingredients {
                                match ing {
                                    ResType::Basic(b) => { if *bag.basic.get(&b).unwrap_or(&0) < 6 { can_craft = false; break; } },
                                    ResType::Complex(comp) => { if *bag.complex.get(&comp).unwrap_or(&0) < 1 { can_craft = false; break; } }
                                }
                            }
                            if can_craft {
                                let mut candidates = Vec::new();
                                {
                                    let kb = ai.knowledge_base.read().unwrap();
                                    for (pid, data) in kb.iter() {
                                        if data.combines.as_ref().map_or(false, |x| x.contains(&c)) {
                                            // Prioritize factory that needs 2x AIPartner if that is the goal
                                            if *ai.target_goal.read().unwrap() == ExplorerGoal::Secret && c == ComplexResourceType::AIPartner {
                                                let hist = ai.crafted_history.read().unwrap();
                                                let count = hist.get(pid).map_or(0, |list| list.iter().filter(|&&x| x == ComplexResourceType::AIPartner).count());
                                                if count < 2 { candidates.push(*pid); }
                                            } else {
                                                candidates.push(*pid);
                                            }
                                        }
                                    }
                                }
                                let mut moved = false;
                                for pid in candidates {
                                    if pid == current_id {
                                        let energy = ai.ask_available_cells();
                                        if energy > 0 { ai.combine_resource(c); sleep(Duration::from_millis(500)); }
                                        else { sleep(Duration::from_millis(1500)); }
                                        moved = true;
                                        break;
                                    }
                                    if let Some(path) = ai.find_path_to_planet(pid) {
                                        if let Some(&next) = path.first() {
                                            ai.travel_request(next); sleep(Duration::from_millis(1500));
                                            if *ai.current_planet_id.read().unwrap() == current_id { ai.blocked_edges.write().unwrap().insert((current_id, next)); }
                                            else { ai.scan_current_planet(); }
                                            moved = true;
                                            break;
                                        }
                                    }
                                }
                                if !moved {
                                    // Fallback: Random move if no path to factory
                                    let kb = ai.knowledge_base.read().unwrap();
                                    if let Some(data) = kb.get(&current_id) {
                                        let mut rng = rng();
                                        if let Some(rnd) = data.neighbors.choose(&mut rng) {
                                            let rnd_copy = *rnd;
                                            drop(kb);
                                            ai.travel_request(rnd_copy);
                                            sleep(Duration::from_millis(1500));
                                            ai.scan_current_planet();
                                        }
                                    }
                                }
                            } else {
                                if let Some(b_need) = ingredients.iter().find_map(|i| if let ResType::Basic(b) = i { if *bag.basic.get(b).unwrap_or(&0) < 6 { Some(*b) } else { None } } else { None }) {
                                    let mut candidates = Vec::new();
                                    {
                                        let kb = ai.knowledge_base.read().unwrap();
                                        for (pid, data) in kb.iter() {
                                            if data.generates.as_ref().map_or(false, |g| g.contains(&b_need)) { candidates.push(*pid); }
                                        }
                                    }
                                    let mut moved = false;
                                    for pid in candidates {
                                        if pid == current_id { ai.attempt_extract(b_need); moved = true; break; }
                                        if let Some(path) = ai.find_path_to_planet(pid) {
                                            if let Some(&next) = path.first() {
                                                ai.travel_request(next); sleep(Duration::from_millis(1500));
                                                if *ai.current_planet_id.read().unwrap() == current_id { ai.blocked_edges.write().unwrap().insert((current_id, next)); }
                                                else { ai.scan_current_planet(); }
                                                moved = true;
                                                break;
                                            }
                                        }
                                    }
                                    if !moved {
                                        // Fallback: Random move if stuck
                                        let kb = ai.knowledge_base.read().unwrap();
                                        if let Some(data) = kb.get(&current_id) {
                                            let mut rng = rng();
                                            let next_planet = data.neighbors.choose(&mut rng).cloned();
                                            drop(kb);
                                            if let Some(rnd) = next_planet {
                                                ai.travel_request(rnd);
                                                sleep(Duration::from_millis(1500));
                                                ai.scan_current_planet();
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Fallback: Random move if no target
                            let kb = ai.knowledge_base.read().unwrap();
                            if let Some(data) = kb.get(&current_id) {
                                let mut rng = rng();
                                if let Some(rnd) = data.neighbors.choose(&mut rng) {
                                    let rnd_copy = *rnd;
                                    drop(kb);
                                    ai.travel_request(rnd_copy);
                                    sleep(Duration::from_millis(1500));
                                    ai.scan_current_planet();
                                }
                            }
                        }
                    },
                    AIState::Idle => sleep(Duration::from_secs(5)),
                }
            }
        });
    }

    fn reset_ai(&self) { self.to_orchestrator.send(ExplorerToOrchestrator::ResetExplorerAIResult {explorer_id: self.explorer_id}).unwrap(); }
    fn kill(&self) { self.to_orchestrator.send(ExplorerToOrchestrator::KillExplorerResult {explorer_id: self.explorer_id}).unwrap(); }
    fn get_current_planet(&self) {
        let id = *self.current_planet_id.read().unwrap();
        self.to_orchestrator.send(ExplorerToOrchestrator::CurrentPlanetResult {explorer_id: self.explorer_id, planet_id: id}).unwrap();
    }
    fn give_supported_resources(&self) { }
    fn ask_supported_resources(&self) {
        let to = self.to_planet.read().unwrap();
        let from = self.from_planet.read().unwrap();
        if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
            let _ = tx.send(ExplorerToPlanet::SupportedResourceRequest {explorer_id: self.explorer_id});
            if let Ok(PlanetToExplorer::SupportedResourceResponse{resource_list}) = rx.recv_timeout(Duration::from_millis(500)) {
                *self.basic_resources.write().unwrap() = resource_list;
            }
        }
    }
    fn give_combinations(&mut self) { }
    fn ask_combinations(&self) {
        let to = self.to_planet.read().unwrap();
        let from = self.from_planet.read().unwrap();
        if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
            let _ = tx.send(ExplorerToPlanet::SupportedCombinationRequest {explorer_id: self.explorer_id});
            if let Ok(PlanetToExplorer::SupportedCombinationResponse{combination_list}) = rx.recv_timeout(Duration::from_millis(500)) {
                *self.combinations.write().unwrap() = combination_list;
            }
        }
    }
    fn get_bag(&self) {
        let bag = self.bag.read().unwrap().to_dummy();
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::BagContentResponse {explorer_id: self.explorer_id, bag_content: bag});
    }
    fn ask_for_neighbours(&self) {
        let id = *self.current_planet_id.read().unwrap();
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::NeighborsRequest {explorer_id: self.explorer_id, current_planet_id: id});
    }
    fn set_neighbours(&self, neighbors: Vec<ID>) {
        *self.neighbours.write().unwrap() = neighbors;
    }
    fn travel_request(&self , dst: u32) {
        let curr = *self.current_planet_id.read().unwrap();
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::TravelToPlanetRequest {explorer_id: self.explorer_id, current_planet_id: curr, dst_planet_id: dst});
    }
    fn ask_available_cells(&self) -> u32 {
        let to = self.to_planet.read().unwrap();
        let from = self.from_planet.read().unwrap();
        if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
            if tx.send(ExplorerToPlanet::AvailableEnergyCellRequest {explorer_id: self.explorer_id}).is_ok() {
                if let Ok(PlanetToExplorer::AvailableEnergyCellResponse { available_cells }) = rx.recv_timeout(Duration::from_millis(500)) {
                    return available_cells;
                }
            }
        }
        0
    }
    fn get_complex_resource_request(&self, complex_resource_type: ComplexResourceType) -> Option<ComplexResourceRequest> {
        let mut guard = self.bag.write().unwrap();
        match complex_resource_type {
            ComplexResourceType::AIPartner => guard.get_complex_ingredients(ComplexResourceType::Diamond, ComplexResourceType::Robot).map(|(l, r)| ComplexResourceRequest::AIPartner(r.to_robot().unwrap(), l.to_diamond().unwrap())),
            ComplexResourceType::Robot => guard.get_diff_type_ingredients(BasicResourceType::Silicon, ComplexResourceType::Life).map(|(l, r)| ComplexResourceRequest::Robot(l.to_silicon().unwrap(), r.to_life().unwrap())),
            ComplexResourceType::Diamond => guard.get_basic_ingredients(BasicResourceType::Carbon, BasicResourceType::Carbon).map(|(l, r)| ComplexResourceRequest::Diamond(l.to_carbon().unwrap(), r.to_carbon().unwrap())),
            ComplexResourceType::Water => guard.get_basic_ingredients(BasicResourceType::Hydrogen, BasicResourceType::Oxygen).map(|(l, r)| ComplexResourceRequest::Water(l.to_hydrogen().unwrap(), r.to_oxygen().unwrap())),
            ComplexResourceType::Life => guard.get_diff_type_ingredients(BasicResourceType::Carbon, ComplexResourceType::Water).map(|(l, r)| ComplexResourceRequest::Life(r.to_water().unwrap(), l.to_carbon().unwrap())),
            ComplexResourceType::Dolphin => guard.get_complex_ingredients(ComplexResourceType::Water, ComplexResourceType::Life).map(|(l, r)| ComplexResourceRequest::Dolphin(l.to_water().unwrap(), r.to_life().unwrap())),
        }
    }
    fn move_to_planet(&self, to_planet: Option<Sender<ExplorerToPlanet>>, planet_id: ID) {
        let prev_id = *self.current_planet_id.read().unwrap();
        *self.to_planet.write().unwrap() = to_planet;
        *self.current_planet_id.write().unwrap() = planet_id;
        *self.arrival_time.write().unwrap() = SystemTime::now();
        *self.total_visits.write().unwrap() += 1;
        if prev_id != planet_id && prev_id != 0 {
            self.visited_edges.write().unwrap().insert((prev_id, planet_id));
            *self.total_steps.write().unwrap() += 1;
        }
        println!("SmartExplorer #{} arrived at Planet #{}", self.explorer_id, planet_id);
        self.to_orchestrator.send(ExplorerToOrchestrator::MovedToPlanetResult {explorer_id: self.explorer_id, planet_id }).unwrap();
        self.neighbours.write().unwrap().clear();
    }
    fn generate_resource(&self, to_generate: BasicResourceType) {
        let to = self.to_planet.read().unwrap();
        let from = self.from_planet.read().unwrap();
        if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
            if tx.send(ExplorerToPlanet::GenerateResourceRequest {explorer_id: self.explorer_id, resource: to_generate}).is_ok() {
                if let Ok(PlanetToExplorer::GenerateResourceResponse {resource}) = rx.recv_timeout(Duration::from_millis(1000)) {
                    if let Some(r) = resource {
                        let r_type = r.get_type();
                        self.bag.write().unwrap().add_basic_resource(r);
                        let bag = self.bag.read().unwrap().to_dummy();
                        // Quiet mode
                    }
                }
            }
        }
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::GenerateResourceResponse {explorer_id: self.explorer_id, generated: Ok(())});
    }
    fn combine_resource(&self, to_generate: ComplexResourceType) {
        let req_opt = self.get_complex_resource_request(to_generate);
        if let Some(req) = req_opt {
            let to = self.to_planet.read().unwrap();
            let from = self.from_planet.read().unwrap();
            if let (Some(tx), Some(rx)) = (to.as_ref(), from.as_ref()) {
                if tx.send(ExplorerToPlanet::CombineResourceRequest {explorer_id: self.explorer_id, msg: req}).is_ok() {
                    match rx.recv_timeout(Duration::from_millis(1000)) {
                        Ok(PlanetToExplorer::CombineResourceResponse{complex_response}) => {
                            match complex_response {
                                Ok(r) => {
                                    let r_type = r.get_type();
                                    self.bag.write().unwrap().add_complex_resource(r);
                                    let bag = self.bag.read().unwrap().to_dummy();
                                    println!("AI [Bag Update]: Crafted {:?} | Basic: {:?} | Complex: {:?}", r_type, bag.basic, bag.complex);

                                    let current_id = *self.current_planet_id.read().unwrap();
                                    let mut hist = self.crafted_history.write().unwrap();
                                    hist.entry(current_id).or_insert(Vec::new()).push(r_type);
                                },
                                Err(e) => println!("AI [Error]: Planet refused craft: {:?}", e),
                            }
                        },
                        Ok(_) => println!("AI [Error]: Unexpected msg from planet."),
                        Err(e) => println!("AI [Error]: Planet timeout during craft: {:?}", e),
                    }
                }
            }
        }
        let _ = self.to_orchestrator.send(ExplorerToOrchestrator::CombineResourceResponse {explorer_id: self.explorer_id, generated: Ok(())});
    }
}