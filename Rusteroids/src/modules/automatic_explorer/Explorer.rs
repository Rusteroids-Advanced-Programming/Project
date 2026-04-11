use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use crossbeam_channel::{Receiver, Sender};

use common_game::components::resource::{
    BasicResource, BasicResourceType, ComplexResourceRequest, ComplexResourceType,
    GenericResource, ResourceType
};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};
use crate::modules::manual_explorer::explorer_ai::ExplorerAI;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ResType {
    Basic(BasicResourceType),
    Complex(ComplexResourceType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TaskType {
    Mapping,
    Hoarding,
    HubHunt,
}

#[derive(Debug, Clone, PartialEq)]
enum AIState {
    Mapping,
    Hoarding,
    HubHunting,
    Gathering,
    Returning,
    Idle,
    Fleeing,
}

#[derive(Debug, Default, Clone)]
struct MissionLog {
    map_complete: bool,
    thirty_basic_total: bool,
    five_of_each_basic: bool,
    one_of_each_complex: bool,
    network_technician: bool,
    nomad: bool,
    hub_hunter: bool,
    hoarder: bool,
    victory_lap: bool,
    grand_slam: bool,
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

    pub extraction_sites: Arc<RwLock<HashSet<ID>>>,
    pub hub_candidate: Arc<RwLock<(ID, usize)>>,
    pub hub_visits: Arc<RwLock<HashMap<ID, u32>>>,
    pub max_bag_size: Arc<RwLock<usize>>,

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
            to_orchestrator,
            from_orchestrator,
            to_planet: Arc::new(RwLock::new(None)),
            from_planet: Arc::new(RwLock::new(None)),
            neighbours: Arc::new(RwLock::new(Vec::new())),
            basic_resources: Arc::new(RwLock::new(HashSet::new())),
            combinations: Arc::new(RwLock::new(HashSet::new())),
            knowledge_base: Arc::new(RwLock::new(HashMap::new())),
            visited: Arc::new(RwLock::new(HashSet::new())),

            visited_edges: Arc::new(RwLock::new(HashSet::new())),
            blocked_edges: Arc::new(RwLock::new(HashSet::new())),
            unreachable_planets: Arc::new(RwLock::new(HashSet::new())),
            abandoned_tasks: Arc::new(RwLock::new(HashSet::new())),

            arrival_time: Arc::new(RwLock::new(SystemTime::now())),
            planet_cooldowns: Arc::new(RwLock::new(HashMap::new())),

            extraction_sites: Arc::new(RwLock::new(HashSet::new())),
            hub_candidate: Arc::new(RwLock::new((current_planet_id, 0))),
            hub_visits: Arc::new(RwLock::new(HashMap::new())),
            max_bag_size: Arc::new(RwLock::new(0)),

            state: Arc::new(RwLock::new(AIState::Mapping)),
            dfs_stack: Arc::new(RwLock::new(vec![])),
        }
    }

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

    // --- UPDATED RISK DISPLAY ---
    pub fn get_risk_display(&self) -> String {
        let elapsed = self.arrival_time.read().unwrap().elapsed().unwrap_or(Duration::from_secs(0)).as_secs();
        if elapsed < 5 {
            format!("✅ Safe ({}s)", elapsed) // 0-4
        } else if elapsed < 10 {
            format!("⚠️ Risky ({}s)", elapsed) // 5-9
        } else if elapsed < 15 {
            format!("⚡ Dangerous ({}s)", elapsed) // 10-14
        } else if elapsed < 25 {
            format!("🔴 VERY DANGEROUS!!! ({}s)", elapsed) // 15-24
        } else {
            format!("💀 EXPULSION IMMINENT ({}s)", elapsed) // 25+
        }
    }

    // --- UPDATED RISK LOGIC ---
    fn get_risk_level(&self) -> (String, bool) {
        let elapsed = self.arrival_time.read().unwrap().elapsed().unwrap_or(Duration::from_secs(0)).as_secs();
        // Return (Status String, Must Flee Boolean)
        // Must flee if elapsed >= 25
        (self.get_risk_display(), elapsed >= 25)
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

        println!("AI: Scanned Planet {}. Res: {:?}. Neigh: {:?}", current_id, basic_data, neighbors_data);

        let neighbor_count = neighbors_data.len();
        {
            let mut hub_cand = self.hub_candidate.write().unwrap();
            if neighbor_count > hub_cand.1 {
                *hub_cand = (current_id, neighbor_count);
                println!("AI [Hub Hunter]: New Hub Candidate: Planet {} ({} neighbors).", current_id, neighbor_count);
            }
            let mut visits = self.hub_visits.write().unwrap();
            *visits.entry(current_id).or_insert(0) += 1;
        }

        kb.insert(current_id, PlanetKnowledge {
            id: current_id,
            neighbors: neighbors_data,
            generates: Some(basic_data.into_iter().collect()),
            combines: Some(complex_data.into_iter().collect()),
        });
        self.visited.write().unwrap().insert(current_id);
    }

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
                    let is_cooldown = if let Some(time) = cooldowns.get(&neighbor) {
                        SystemTime::now() < *time
                    } else { false };

                    if !blocked.contains(&(current, neighbor)) && !visited_bfs.contains(&neighbor) && !is_cooldown {
                        visited_bfs.insert(neighbor);
                        came_from.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        None
    }

    fn has_resource(&self, res: ResType) -> bool {
        let bag = self.bag.read().unwrap();
        let dummy = bag.to_dummy();
        match res {
            ResType::Basic(b) => dummy.basic.get(&b).map_or(false, |&count| count > 0),
            ResType::Complex(c) => dummy.complex.get(&c).map_or(false, |&count| count > 0),
        }
    }

    fn update_hoarder_score(&self) {
        let bag = self.bag.read().unwrap().to_dummy();
        let count = bag.basic.values().sum::<usize>() + bag.complex.values().sum::<usize>();
        let mut max = self.max_bag_size.write().unwrap();
        if count > *max { *max = count; }
    }

    fn check_mission_status(&self) -> MissionLog {
        let mut status = MissionLog::default();
        let kb_size = self.knowledge_base.read().unwrap().len();
        status.map_complete = kb_size >= 8;
        let bag = self.bag.read().unwrap().to_dummy();
        let total_basic: usize = bag.basic.values().sum();
        status.thirty_basic_total = total_basic >= 30;
        let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
        let mut all_five = true;
        for b in &basics { if *bag.basic.get(b).unwrap_or(&0) < 5 { all_five = false; } }
        status.five_of_each_basic = all_five;
        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
        let mut all_complex = true;
        for c in &complexes { if *bag.complex.get(c).unwrap_or(&0) < 1 { all_complex = false; } }
        status.one_of_each_complex = all_complex;
        let kb = self.knowledge_base.read().unwrap();
        let mut total_known_edges = 0;
        for data in kb.values() { total_known_edges += data.neighbors.len(); }
        let visited_edges_count = self.visited_edges.read().unwrap().len();
        status.network_technician = visited_edges_count >= total_known_edges && total_known_edges > 0;
        status.nomad = self.extraction_sites.read().unwrap().len() >= 5;
        let (hub_id, _) = *self.hub_candidate.read().unwrap();
        let visits = self.hub_visits.read().unwrap().get(&hub_id).cloned().unwrap_or(0);
        status.hub_hunter = visits >= 3;
        status.hoarder = *self.max_bag_size.read().unwrap() >= 67;
        let pos = *self.current_planet_id.read().unwrap();
        let spawn = *self.spawn_planet_id.read().unwrap();
        let all_tasks_done = status.map_complete && status.thirty_basic_total && status.five_of_each_basic &&
            status.one_of_each_complex && status.network_technician && status.nomad &&
            status.hub_hunter && status.hoarder;
        if all_tasks_done && pos == spawn { status.victory_lap = true; }
        if all_tasks_done && status.victory_lap { status.grand_slam = true; }
        status
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
            if count >= 10 { return false; }
        }
        for _ in 1..=2 {
            let energy = self.ask_available_cells();
            if energy > 0 {
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
}

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
                if mission.grand_slam {
                    println!("AI [WIN]: GRAND SLAM! All Tasks Complete! Back at spawn.");
                    break;
                }

                let current_id = *ai.current_planet_id.read().unwrap();
                let bag_dummy = ai.bag.read().unwrap().to_dummy();
                let total_items = bag_dummy.basic.values().sum::<usize>() + bag_dummy.complex.values().sum::<usize>();
                let abandoned = ai.abandoned_tasks.read().unwrap();

                // --- RISK CHECK ---
                let (risk_status, must_flee) = ai.get_risk_level();
                let mut current_state = AIState::Idle;

                if must_flee {
                    current_state = AIState::Fleeing;
                } else {
                    let need_mapping = !mission.map_complete || !mission.network_technician;
                    let basics_saturated = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen]
                        .iter().all(|b| *bag_dummy.basic.get(b).unwrap_or(&0) >= 10);

                    if basics_saturated && !mission.hoarder {
                        current_state = AIState::Gathering;
                    } else if need_mapping && !abandoned.contains(&TaskType::Mapping) {
                        current_state = AIState::Mapping;
                    } else if !mission.one_of_each_complex {
                        current_state = AIState::Gathering;
                    } else if !mission.hoarder || !mission.thirty_basic_total || !mission.five_of_each_basic || !mission.nomad {
                        if !abandoned.contains(&TaskType::Hoarding) {
                            current_state = AIState::Hoarding;
                        } else {
                            current_state = AIState::HubHunting;
                        }
                    } else if !mission.hub_hunter && !abandoned.contains(&TaskType::HubHunt) {
                        current_state = AIState::HubHunting;
                    } else {
                        current_state = AIState::Returning;
                    }
                }

                println!("\nAI [Status]\n \
                          \tState: {:?} | Risk: {}\n \
                          \tGOALS [Map]: Map:{} | Net:{} | Nomad:{} | Hub:{}\n \
                          \tGOALS [Res]: 30+:{} | 5ea:{} | Hoard:{} | Craft:{}\n \
                          \tGOALS [End]: Home:{}\n \
                          \tSTATS: Bag Size: {}",
                         current_state,
                         risk_status,
                         if mission.map_complete { "✅" } else { "❌" },
                         if mission.network_technician { "✅" } else { "❌" },
                         if mission.nomad { "✅" } else { "❌" },
                         if mission.hub_hunter { "✅" } else { "❌" },
                         if mission.thirty_basic_total { "✅" } else { "❌" },
                         if mission.five_of_each_basic { "✅" } else { "❌" },
                         if mission.hoarder { "✅" } else { "❌" },
                         if mission.one_of_each_complex { "✅" } else { "❌" },
                         if mission.victory_lap { "✅" } else { "❌" },
                         total_items
                );

                match current_state {
                    AIState::Fleeing => {
                        println!("AI [DANGER]: Time Threshold Exceeded! Forcing Evacuation!");

                        // --- UPDATED COOLDOWN: 7 SECONDS ---
                        let cooldown_until = SystemTime::now() + Duration::from_secs(7);
                        ai.planet_cooldowns.write().unwrap().insert(current_id, cooldown_until);
                        println!("AI [Cooldown]: Planet {} is banned for 7s.", current_id);

                        let kb = ai.knowledge_base.read().unwrap();
                        let blocked = ai.blocked_edges.read().unwrap();
                        let unreachable = ai.unreachable_planets.read().unwrap();
                        let cooldowns = ai.planet_cooldowns.read().unwrap();

                        if let Some(data) = kb.get(&current_id) {
                            let safe_target = data.neighbors.iter().find(|&&n| {
                                let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                !blocked.contains(&(current_id, n)) && !unreachable.contains(&n) && !is_cd
                            });

                            if let Some(&t) = safe_target {
                                drop(kb); drop(blocked); drop(unreachable); drop(cooldowns);
                                ai.travel_request(t);
                                sleep(Duration::from_millis(1500));
                                if *ai.current_planet_id.read().unwrap() != current_id {
                                    ai.scan_current_planet();
                                } else {
                                    println!("AI [Error]: Flee failed! Planet likely dead.");
                                    ai.blocked_edges.write().unwrap().insert((current_id, t));
                                }
                            } else {
                                println!("AI [CRITICAL]: No safe place to flee! Trapped!");
                            }
                        }
                    },

                    // ... (States: Mapping, Hoarding, Hub, Return, Gathering) ...
                    // All other logic from the "Targeted Hoarding" fix is preserved below.
                    // Copying the corrected Hoarding logic to ensure no regression.

                    AIState::Hoarding => {
                        let basic_set = ai.basic_resources.read().unwrap().clone();
                        let mut extracted_any = false;
                        let mut missing_resource = None;

                        // 1. Try to extract local resources
                        if !basic_set.is_empty() {
                            for res in basic_set {
                                if ai.attempt_extract(res) {
                                    println!("AI [Hoarding]: Extracted {:?}.", res);
                                    extracted_any = true;
                                }
                                if ai.bag.read().unwrap().to_dummy().basic.values().sum::<usize>() >= 67 { break; }
                            }
                        }

                        // 2. Targeted Movement if full
                        if !extracted_any {
                            let bag = ai.bag.read().unwrap().to_dummy();
                            let basics_list = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];

                            for b in basics_list {
                                if *bag.basic.get(&b).unwrap_or(&0) < 10 { missing_resource = Some(b); break; }
                            }

                            let mut move_target = None;
                            if let Some(target_res) = missing_resource {
                                let kb = ai.knowledge_base.read().unwrap();
                                let unreachable = ai.unreachable_planets.read().unwrap();
                                let cooldowns = ai.planet_cooldowns.read().unwrap();
                                for (pid, data) in kb.iter() {
                                    let is_cd = if let Some(t) = cooldowns.get(pid) { SystemTime::now() < *t } else { false };
                                    if data.generates.as_ref().map_or(false, |g| g.contains(&target_res)) && !unreachable.contains(pid) && *pid != current_id && !is_cd {
                                        move_target = Some(*pid);
                                        break;
                                    }
                                }
                            }

                            if let Some(target_pid) = move_target {
                                println!("AI [Hoarding]: Targeting {:?} at Planet #{}", missing_resource.unwrap(), target_pid);
                                if let Some(path) = ai.find_path_to_planet(target_pid) {
                                    if let Some(&next) = path.first() {
                                        ai.travel_request(next);
                                        sleep(Duration::from_millis(1500));
                                        if *ai.current_planet_id.read().unwrap() == current_id {
                                            ai.blocked_edges.write().unwrap().insert((current_id, next));
                                        } else {
                                            ai.scan_current_planet();
                                        }
                                    }
                                } else {
                                    ai.unreachable_planets.write().unwrap().insert(target_pid);
                                }
                            } else {
                                // Fallback
                                let kb = ai.knowledge_base.read().unwrap();
                                let blocked = ai.blocked_edges.read().unwrap();
                                let cooldowns = ai.planet_cooldowns.read().unwrap();
                                if let Some(data) = kb.get(&current_id) {
                                    if let Some(&rnd) = data.neighbors.iter().find(|&&n| {
                                        let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                        !blocked.contains(&(current_id, n)) && !is_cd
                                    }) {
                                        drop(kb); drop(blocked); drop(cooldowns);
                                        ai.travel_request(rnd);
                                        sleep(Duration::from_millis(1500));
                                        ai.scan_current_planet();
                                    }
                                }
                            }
                        }
                    },

                    // ... (Other states remain same as provided in previous messages) ...
                    AIState::Mapping => {
                        // (Use corrected logic)
                        {
                            let basic = ai.basic_resources.read().unwrap().clone();
                            for res in basic {
                                let count = *ai.bag.read().unwrap().to_dummy().basic.get(&res).unwrap_or(&0);
                                if count < 10 {
                                    if !ai.attempt_extract(res) { break; }
                                }
                            }
                        }
                        let mut next_move = None;
                        {
                            let kb = ai.knowledge_base.read().unwrap();
                            if let Some(planet_data) = kb.get(&current_id) {
                                let visited = ai.visited.read().unwrap();
                                let blocked = ai.blocked_edges.read().unwrap();
                                let unreachable = ai.unreachable_planets.read().unwrap();
                                let cooldowns = ai.planet_cooldowns.read().unwrap();
                                next_move = planet_data.neighbors.iter().find(|&&n| {
                                    let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                    !visited.contains(&n) && !blocked.contains(&(current_id, n)) && !unreachable.contains(&n) && !is_cd
                                }).cloned();
                                if next_move.is_none() {
                                    let edges = ai.visited_edges.read().unwrap();
                                    next_move = planet_data.neighbors.iter().find(|&&n| {
                                        let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                        !edges.contains(&(current_id, n)) && !blocked.contains(&(current_id, n)) && !unreachable.contains(&n) && !is_cd
                                    }).cloned();
                                }
                            }
                        }
                        if let Some(next_hop) = next_move {
                            ai.dfs_stack.write().unwrap().push(current_id);
                            ai.travel_request(next_hop);
                            sleep(Duration::from_millis(1500));
                            if *ai.current_planet_id.read().unwrap() != current_id {
                                ai.scan_current_planet();
                            } else {
                                ai.blocked_edges.write().unwrap().insert((current_id, next_hop));
                            }
                        } else {
                            let should_abandon = {
                                let kb = ai.knowledge_base.read().unwrap();
                                let visited = ai.visited.read().unwrap();
                                let unreachable = ai.unreachable_planets.read().unwrap();
                                let remaining = (1..=8).filter(|id| !visited.contains(id) && !unreachable.contains(id)).count();
                                remaining == 0 && !mission.map_complete
                            };
                            if should_abandon {
                                ai.abandoned_tasks.write().unwrap().insert(TaskType::Mapping);
                                continue;
                            }
                            let mut stack = ai.dfs_stack.write().unwrap();
                            if let Some(prev) = stack.pop() {
                                drop(stack); ai.travel_request(prev); sleep(Duration::from_millis(1500));
                                if *ai.current_planet_id.read().unwrap() != current_id { ai.scan_current_planet(); }
                            } else {
                                let blocked = ai.blocked_edges.read().unwrap();
                                let cooldowns = ai.planet_cooldowns.read().unwrap();
                                let kb = ai.knowledge_base.read().unwrap();
                                if let Some(data) = kb.get(&current_id) {
                                    if let Some(&rnd) = data.neighbors.iter().find(|&&n| {
                                        let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                        !blocked.contains(&(current_id, n)) && !is_cd
                                    }) {
                                        drop(kb); drop(blocked); drop(cooldowns);
                                        ai.travel_request(rnd);
                                        sleep(Duration::from_millis(1500));
                                        ai.scan_current_planet();
                                    }
                                }
                            }
                        }
                    },

                    AIState::HubHunting => {
                        let (hub_id, _) = *ai.hub_candidate.read().unwrap();
                        let unreachable = ai.unreachable_planets.read().unwrap();
                        if unreachable.contains(&hub_id) {
                            ai.abandoned_tasks.write().unwrap().insert(TaskType::HubHunt);
                            continue;
                        }
                        if current_id == hub_id {
                            let kb = ai.knowledge_base.read().unwrap();
                            let cooldowns = ai.planet_cooldowns.read().unwrap();
                            if let Some(data) = kb.get(&current_id) {
                                if let Some(&n) = data.neighbors.iter().find(|&&n| {
                                    let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                    !is_cd
                                }) {
                                    drop(kb); drop(cooldowns); ai.travel_request(n); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                                }
                            }
                        } else {
                            let mut path_found = false;
                            let mut next_hop = 0;
                            {
                                drop(ai.knowledge_base.read().unwrap());
                                if let Some(path) = ai.find_path_to_planet(hub_id) {
                                    if let Some(&next) = path.first() { next_hop = next; path_found = true; }
                                }
                            }
                            if path_found {
                                ai.travel_request(next_hop); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                            } else {
                                ai.unreachable_planets.write().unwrap().insert(hub_id);
                            }
                        }
                    },

                    AIState::Returning => {
                        let spawn = *ai.spawn_planet_id.read().unwrap();
                        let mut next_hop = 0;
                        let mut found = false;
                        {
                            if let Some(path) = ai.find_path_to_planet(spawn) {
                                if let Some(&n) = path.first() { next_hop = n; found = true; }
                            }
                        }
                        if found {
                            ai.travel_request(next_hop); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                        } else { sleep(Duration::from_secs(2)); }
                    },

                    AIState::Gathering => {
                        // (Same logic as provided in previous successful update)
                        // ... Copy-paste the Gathering Block from the previous message ...
                        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
                        let bag = ai.bag.read().unwrap().to_dummy();
                        let mut target = None;
                        let mut failed_this_tick = HashSet::new();

                        for _ in 0..6 {
                            target = None;
                            for c in &complexes {
                                if !failed_this_tick.contains(c) && *bag.complex.get(c).unwrap_or(&0) < 1 { target = Some(*c); break; }
                            }
                            if target.is_none() {
                                for c in &complexes {
                                    if !failed_this_tick.contains(c) && *bag.complex.get(c).unwrap_or(&0) < 5 { target = Some(*c); break; }
                                }
                            }

                            if let Some(c) = target {
                                let ingredients = ai.get_ingredients(c);
                                let mut can_craft = true;
                                let mut ing_missing_basic = None;
                                for ing in ingredients {
                                    match ing {
                                        ResType::Basic(b) => { if *bag.basic.get(&b).unwrap_or(&0) < 6 { can_craft = false; ing_missing_basic = Some(b); break; } },
                                        ResType::Complex(comp) => { if *bag.complex.get(&comp).unwrap_or(&0) < 1 { can_craft = false; break; } }
                                    }
                                }

                                if can_craft {
                                    let can_craft_here = { let local = ai.combinations.read().unwrap(); local.contains(&c) };
                                    if can_craft_here {
                                        println!("AI [Gathering]: Crafting {:?}!", c);
                                        let energy = ai.ask_available_cells();
                                        if energy > 0 { ai.combine_resource(c); sleep(Duration::from_millis(500)); break; }
                                        else { sleep(Duration::from_millis(1500)); break; }
                                    } else {
                                        let mut candidates = Vec::new();
                                        {
                                            let kb = ai.knowledge_base.read().unwrap();
                                            let unreachable = ai.unreachable_planets.read().unwrap();
                                            for (pid, data) in kb.iter() {
                                                if data.combines.as_ref().map_or(false, |x| x.contains(&c)) && !unreachable.contains(pid) { candidates.push(*pid); }
                                            }
                                        }
                                        if candidates.is_empty() { failed_this_tick.insert(c); continue; }

                                        let mut found_factory_path = false;
                                        for pid in candidates {
                                            if pid == current_id { ai.scan_current_planet(); found_factory_path = true; break; }
                                            if let Some(path) = ai.find_path_to_planet(pid) {
                                                if let Some(&next) = path.first() {
                                                    ai.travel_request(next); sleep(Duration::from_millis(1500));
                                                    if *ai.current_planet_id.read().unwrap() == current_id { ai.blocked_edges.write().unwrap().insert((current_id, next)); }
                                                    else { ai.scan_current_planet(); }
                                                    found_factory_path = true; break;
                                                }
                                            } else { ai.unreachable_planets.write().unwrap().insert(pid); }
                                        }
                                        if found_factory_path { break; } else { failed_this_tick.insert(c); }
                                    }
                                } else {
                                    if let Some(b_need) = ing_missing_basic {
                                        let mut candidates = Vec::new();
                                        {
                                            let kb = ai.knowledge_base.read().unwrap();
                                            let unreachable = ai.unreachable_planets.read().unwrap();
                                            for (pid, data) in kb.iter() {
                                                if data.generates.as_ref().map_or(false, |g| g.contains(&b_need)) && !unreachable.contains(pid) { candidates.push(*pid); }
                                            }
                                        }
                                        if candidates.is_empty() { failed_this_tick.insert(c); continue; }

                                        let mut found_source = false;
                                        for pid in candidates {
                                            if pid == current_id { ai.attempt_extract(b_need); found_source = true; break; }
                                            if let Some(path) = ai.find_path_to_planet(pid) {
                                                if let Some(&next) = path.first() {
                                                    ai.travel_request(next); sleep(Duration::from_millis(1500));
                                                    if *ai.current_planet_id.read().unwrap() == current_id { ai.blocked_edges.write().unwrap().insert((current_id, next)); }
                                                    else { ai.scan_current_planet(); }
                                                    found_source = true; break;
                                                }
                                            } else { ai.unreachable_planets.write().unwrap().insert(pid); }
                                        }
                                        if found_source { break; } else { failed_this_tick.insert(c); }
                                    } else { failed_this_tick.insert(c); }
                                }
                            } else {
                                println!("AI [Gathering]: No viable recipes. Force switching to Hoarding.");
                                let kb = ai.knowledge_base.read().unwrap();
                                let blocked = ai.blocked_edges.read().unwrap();
                                let cooldowns = ai.planet_cooldowns.read().unwrap();
                                if let Some(data) = kb.get(&current_id) {
                                    if let Some(&rnd) = data.neighbors.iter().find(|&&n| {
                                        let is_cd = if let Some(t) = cooldowns.get(&n) { SystemTime::now() < *t } else { false };
                                        !blocked.contains(&(current_id, n)) && !is_cd
                                    }) {
                                        drop(kb); drop(blocked); drop(cooldowns);
                                        ai.travel_request(rnd); sleep(Duration::from_millis(1500)); ai.scan_current_planet();
                                    }
                                }
                                break;
                            }
                        }
                    },
                    AIState::Idle => sleep(Duration::from_secs(5)),
                }
            }
        });
    }

    fn move_to_planet(&self, to_planet: Option<Sender<ExplorerToPlanet>>, planet_id: ID) {
        let prev_id = *self.current_planet_id.read().unwrap();

        *self.to_planet.write().unwrap() = to_planet;
        *self.current_planet_id.write().unwrap() = planet_id;
        *self.arrival_time.write().unwrap() = SystemTime::now();

        if prev_id != planet_id && prev_id != 0 {
            self.visited_edges.write().unwrap().insert((prev_id, planet_id));
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
                        println!("AI [Bag Update]: Added {:?} | Basic: {:?} | Complex: {:?}", r_type, bag.basic, bag.complex);
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
}