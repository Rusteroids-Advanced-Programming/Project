# Galaxy Exploration Simulation

A concurrent, message-driven simulation in which autonomous explorer agents
traverse a graph of planets, gather and craft resources, and complete
exploration objectives while a central orchestrator manages the world and
injects hazards over time.

## Overview

The system is built around three kinds of actors, each running on its own
thread and communicating exclusively through [crossbeam](https://docs.rs/crossbeam-channel)
channels:

- **Orchestrator** - the central controller. It owns the galaxy topology, the
  per-planet statistics, every communication channel, and the main tick loop
  that periodically spawns environmental events.
- **Planets** - nodes in the galaxy graph. Each holds resources and energy
  cells, responds to explorer requests, and can be destroyed by hazards.
- **Explorers** - autonomous agents. Each runs a message loop driven by the
  orchestrator and a separate decision loop that pursues its assigned tasks
  (crafting and exploration).

No shared state is mutated directly across threads; all coordination happens
through request/response message passing, with `RwLock` used only for state
owned by a single actor.

## Architecture

### Actors and threads

For every explorer the orchestrator spawns three threads:

1. The explorer's **message loop** (`Explorer::run`), which reacts to
   orchestrator commands (start, reset, kill, move, resource requests).
2. The explorer's **decision loop** (`Explorer::handle_explorer`), which runs
   the agent's own strategy.
3. An **`ExplorerListener`** on the orchestrator side, which consumes messages
   the explorer emits (neighbour requests, travel requests, results) and
   updates global state accordingly.

### Communication protocols

Three protocol families define the message types exchanged between actors:

| Protocol | Direction | Purpose |
|----------|-----------|---------|
| `orchestrator_explorer` | Orchestrator ↔ Explorer | Lifecycle, movement, resource and topology queries |
| `planet_explorer` | Explorer ↔ Planet | Resource generation, combination, neighbour and energy queries |
| `orchestrator_planet` | Orchestrator ↔ Planet | Explorer hand-off, hazards, internal state |

Movement between planets is a three-step handshake coordinated by the
orchestrator: the destination planet registers the incoming explorer, the
origin planet releases it, and the explorer updates its active planet channel.

### World model

The galaxy is an undirected graph of planets. The orchestrator holds the
authoritative topology (`galaxy_graph`) and a `StatsMap` recording, per planet,
whether it is alive and how many hazards and rocket launches it has seen.

A configurable **difficulty** level controls how often hazards occur:

| Difficulty | Hazard ratio |
|------------|--------------|
| Easy       | 0.05 |
| Medium     | 0.10 |
| Hard       | 0.30 |

On each tick the orchestrator selects a random live planet and sends it either
an asteroid or a sunray. Asteroids can destroy a planet; an explorer that lands
on (or departs from) a destroyed planet is marked dead.

## Explorers

Each explorer keeps a private **`ExplorerMap`**: the subset of the galaxy it has
personally discovered, including per-planet info, the known sub-graph, and the
set of edges it has traversed. The map is updated every time the explorer lands
on a planet and queries its neighbours.

Explorers pursue two objectives in sequence:

1. **Crafting** (`CraftAllTask`) - craft every complex resource at least once.
   While this task is unfinished, the shared resolver (`resolve_task`) drives
   movement, steering the explorer toward planets that can craft missing items
   or extract missing raw materials.
2. **Exploration** - once crafting is complete, the explorer switches to a
   dedicated routing loop that pursues its exploration target.

### Variants

| Explorer | Exploration objective                          | Default target     |
|----------|------------------------------------------------|--------------------|
| `Explorer1` | Visit n times a number of **distinct planets** | 80% of all planets |
| `Explorer2` | Traverse a number of **distinct edges**        | 60% of all edges   |

`Explorer1` routes toward unvisited planets using a breadth-first search over
its known map, returning the first hop on the shortest path to the nearest
unvisited (or under-visited) planet. This lets it cross already-explored
territory to reach a distant target rather than giving up when all immediate
neighbours are exhausted. When no unvisited planet is reachable through the
known graph, its task is marked `Uncompletable`.

New explorer variants are added by implementing the `Explorer`, `AIHandlers`,
and `ExplorerBehaviour` traits, and registering them in `initialize_explorers`.

## Resources and crafting

Two resource categories exist:

- **Basic**: Carbon, Hydrogen, Oxygen, Silicon (extracted from planets).
- **Complex**: Diamond, Water, Robot, Life, Dolphin, AIPartner (crafted from
  pairs of ingredients).

Recipes:

| Result | Ingredients |
|--------|-------------|
| Diamond   | Carbon + Carbon |
| Water     | Hydrogen + Oxygen |
| Robot     | Silicon + Life |
| Life      | Water + Carbon |
| Dolphin   | Water + Life |
| AIPartner | Robot + Diamond |

The recipe resolver computes a **shopping list** recursively: given a target
complex resource, it determines which basic resources are still missing and, in
a separate pass, the ordered list of intermediate complex resources that must be
crafted first. Items already held in the bag are reused, each unit only once.

## Tasks

All tasks implement a common `Task` trait exposing their state and progress.
A task moves through three states:

- `Pending` - in progress.
- `Finished` - objective met; this is terminal.
- `Uncompletable` - the objective can no longer be reached (for example, the
  explorer is isolated with no reachable unexplored territory).

State transitions are monotonic: a finished or uncompletable task is never
reset back to pending.

Tasks fall into two groups:

- **Common tasks** - shared across explorers and defined under `common_tasks`,
  for example `CraftAllTask`, which every explorer uses to craft all complex
  resources.
- **Personal tasks** - specific to a single explorer variant and defined under
  that explorer's own module, for example `TotalPlanetsVisitedTask`
  (`Explorer1`) and `TotalEdgesVisitedTask` (`Explorer2`).

## Logging

The orchestrator maintains two logs:

- A bounded circular buffer of raw string messages for recent activity.
- A growing list of **structured log events** (`LogEvent`), each describing a
  participant, an event type, a channel, and a payload, suitable for downstream
  inspection or visualization.


## Building and running

The project is a standard Cargo workspace and depends on a `common_game`
crate for shared component, protocol, and logging definitions.

```sh
cargo build
cargo run
```
