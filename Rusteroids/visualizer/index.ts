/**
 * Data Transfer Object representing astronomical statistics tracker per planet.
 */
interface PlanetStatsDTO {
    asteroids: number;
    sunrays: number;
    rockets: number;
}

/**
 * Data Transfer Object containing full structural and dynamic state properties of a Planet.
 */
interface PlanetDataDTO {
    id: number;
    name: string;
    planet_type: string;
    alive: boolean;
    energy_cells: number;
    resources_base: string[];
    resources_complex: string[];
    neighbors: number[];
    stats: PlanetStatsDTO;
    has_rocket: boolean;
}

/**
 * Data Transfer Object containing full operational state properties of an Explorer agent.
 */
interface ExplorerDataDTO {
    id: number;
    current_planet: number;
    bag: string[];
    alive: boolean;
}

/**
 * High-level API response contract matching the backend Rust visualization structure.
 */
interface GalaxyResponse {
    planets: PlanetDataDTO[];
    explorers: ExplorerDataDTO[];
    game_won: boolean;
    winner_id: number | null;
}

// Global runtime execution tracking variables
let selectedPlanetId: number | null = null;
let visualizerIntervalId: number | null = null;
let logsIntervalId: number | null = null;

// HTML Interface Component Binding Elements
const mainMenu = document.getElementById('main-menu')!;
const menuInitial = document.getElementById('menu-initial')!;
const menuDifficulty = document.getElementById('menu-difficulty')!;
const gameInterface = document.getElementById('game-interface')!;
const btnStartGame = document.getElementById('btn-start-game')!;
const diffButtons = document.querySelectorAll('.btn-diff');
const planetsInput = document.getElementById('planets-count') as HTMLInputElement;

const gameOverScreen = document.getElementById('game-over-screen')!;
const btnRestart = document.getElementById('btn-restart')!;

const gameVictoryScreen = document.getElementById('game-victory-screen')!;
const btnRestartVictory = document.getElementById('btn-restart-victory')!;

/**
 * Coordinates and renders the entire circular star map galaxy grid, line links, and dynamic agent badges onto the viewport.
 * * @param planets Collection of all existing celestial nodes in the current simulation session.
 * @param explorers Collection of all operational or deceased explorer entities.
 */
function renderGalaxy(planets: PlanetDataDTO[], explorers: ExplorerDataDTO[]) {
    const container = document.getElementById('galaxy-container');
    const canvas = document.getElementById('connections-canvas') as HTMLCanvasElement;
    if (!container || !canvas) return;

    // Dynamically match resolution space to content layout wrapper bounds
    const width = container.clientWidth;
    const height = container.clientHeight;
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d')!;

    // Wipe out historical planet runtime wrapper elements to prevent leaks on repaint loops
    container.querySelectorAll('.planet-wrapper').forEach(el => el.remove());

    // Calculate absolute screen viewport anchor coordinates for perfect circular node distribution
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) / 2 - 80;

    const planetPositions = new Map<number, {x: number, y: number}>();
    planets.forEach((p, i) => {
        // Uniform distribution of nodes by mapping fractional circle step to radians
        const angle = (i * 2 * Math.PI) / planets.length;
        const x = centerX + radius * Math.cos(angle);
        const y = centerY + radius * Math.sin(angle);
        planetPositions.set(p.id, {x, y});
    });

    // Draw hyperlane connections between nodes
    ctx.setLineDash([]);
    ctx.strokeStyle = "rgba(52, 152, 219, 0.2)";
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 5]);

    const drawnConnections = new Set<string>();

    planets.forEach(p => {
        const start = planetPositions.get(p.id)!;
        p.neighbors.forEach(neighborId => {
            // Sort node keys to ensure links from A->B and B->A generate identical unique signatures
            const connectionKey = [p.id, neighborId].sort().join('-');

            if (!drawnConnections.has(connectionKey)) {
                const end = planetPositions.get(neighborId);
                if (end) {
                    ctx.beginPath();
                    ctx.moveTo(start.x, start.y);
                    ctx.lineTo(end.x, end.y);
                    ctx.stroke();
                    drawnConnections.add(connectionKey);
                }
            }
        });
    });
    ctx.setLineDash([]);

    // Append and structure dynamic DOM elements representing independent planets
    planets.forEach(p => {
        const pos = planetPositions.get(p.id)!;
        const planetWrapper = document.createElement('div');
        const planet_name = p.name;
        planetWrapper.className = 'planet-wrapper';

        // Anchor wrapper centered exactly over calculated geometric target coordinates
        planetWrapper.style.left = `${pos.x}px`;
        planetWrapper.style.top = `${pos.y}px`;
        planetWrapper.style.transform = 'translate(-50%, -50%)';

        const img = document.createElement('img');
        const fileName = planet_name.toLowerCase();
        img.src = p.alive ? `media/${fileName}.gif` : 'media/dead_planet.gif';
        img.className = `planet-sprite ${p.alive ? '' : 'dead-animation'}`;

        img.onerror = () => {
            img.src = 'media/planet.gif';
        };

        // Attach distinctive visual indicator borders to emphasize the selected node
        if (selectedPlanetId === p.id) {
            img.style.boxShadow = "0 0 20px #e74c3c";
            img.style.border = "2px solid #e74c3c";
            img.style.borderRadius = "50%";
        }

        img.onclick = () => {
            selectedPlanetId = p.id;
            showDetails(p, explorers);
            renderGalaxy(planets, explorers);
        };

        planetWrapper.appendChild(img);

        // Filter and stack UI status labels for all living explorers currently localized on this planet
        const localExplorers = explorers.filter(ex => ex.current_planet === p.id && ex.alive);
        localExplorers.forEach((ex, index) => {
            const exDiv = document.createElement('div');
            exDiv.className = 'explorer-badge';
            exDiv.innerText = `#${ex.id}`;
            exDiv.style.position = 'absolute';
            // Compute stacked positional vertical offset to elegantly overlay multiple explorer tags
            exDiv.style.bottom = `${40 + (index * 20)}px`;
            exDiv.style.left = '50%';
            exDiv.style.transform = 'translateX(-50%)';
            planetWrapper.appendChild(exDiv);
        });

        container.appendChild(planetWrapper);
    });
}

/**
 * Updates and opens the inspector control panel highlighting metrics of a specified target planet.
 * * @param p Selected planet reference mapping object.
 * @param allExplorers List of full explorer instances for state validation.
 */
function showDetails(p: PlanetDataDTO, allExplorers: ExplorerDataDTO[]) {
    const box = document.getElementById('planet-details');
    if (!box) return;

    box.classList.remove('hidden');

    document.getElementById('det-name')!.innerText = `${p.name} #${p.id}`;
    document.getElementById('det-type')!.innerText = p.planet_type;

    const stateEl = document.getElementById('det-alive')!;
    stateEl.innerText = p.alive ? "ALIVE" : "DESTROYED";
    stateEl.style.color = p.alive ? "#2ecc71" : "#e74c3c";

    document.getElementById('det-energy')!.innerText = `${p.energy_cells} units`;
    document.getElementById('det-rockets')!.innerText = p.stats.rockets.toString();
    document.getElementById('det-neighbors')!.innerText = p.neighbors.length > 0
        ? p.neighbors.join(', ')
        : "No neighbors";

    const baseContainer = document.getElementById('det-res-base')!;
    if (p.resources_base.length > 0) {
        baseContainer.innerHTML = p.resources_base
            .map(res => `<span class="badge base">${res}</span>`)
            .join('');
    } else {
        baseContainer.innerHTML = '<span style="color: #666; font-style: italic;">None</span>';
    }

    const complexContainer = document.getElementById('det-res-complex')!;
    if (p.resources_complex.length > 0) {
        complexContainer.innerHTML = p.resources_complex
            .map(res => `<span class="badge complex">${res}</span>`)
            .join('');
    } else {
        complexContainer.innerHTML = '<span style="color: #666; font-style: italic;">None</span>';
    }

    const rocketEl = document.getElementById('det-rockets-ready')!;
    if (p.has_rocket) {
        rocketEl.innerHTML = '<span class="status-ready">READY</span>';
    } else {
        rocketEl.innerHTML = '<span class="status-empty">NOT AVAILABLE</span>';
    }
}

/**
 * Closes the inspector sidebar window pane and deselects tracking.
 */
function closePlanetDetails() {
    const box = document.getElementById('planet-details');
    if (box) {
        box.classList.add('hidden');
    }
    selectedPlanetId = null;
}

document.getElementById('close-details')?.addEventListener('click', closePlanetDetails);

/**
 * Re-renders the global status card dashboards monitoring each inventory stack and life sign.
 * * @param explorers Complete sequence array tracking each target explorer agent.
 */
function updateExplorersPanel(explorers: ExplorerDataDTO[]) {
    const container = document.getElementById('explorers-status-container');
    if (!container) return;

    container.innerHTML = '';

    explorers.forEach(ex => {
        const card = document.createElement('div');
        card.className = `explorer-card ${!ex.alive ? 'dead' : ''}`;

        const statusText = ex.alive ? "ALIVE" : "DEAD";
        card.innerHTML = `
            <h4>
                <span>Explorer #${ex.id}</span>
                <span style="color: ${ex.alive ? '#2ecc71' : '#e74c3c'}">${statusText}</span>
            </h4>
            <div> Pianeta: ${ex.current_planet}</div>
            <div class="inventory-list">
                <strong>Inventory:</strong>
                ${renderInventory(ex.bag)}
            </div>
        `;

        container.appendChild(card);
    });
}

/**
 * Quantifies string collection components, aggregating and returning neat formatted HTML rows.
 * * @param bag Flat list string descriptors indicating raw contents.
 */
function renderInventory(bag: string[]): string {
    const counts: { [key: string]: number } = {};
    bag.forEach(item => {
        counts[item] = (counts[item] || 0) + 1;
    });

    const presentResources = Object.keys(counts).sort();
    if (presentResources.length === 0) {
        return `<div class="no-res">Empty Bag</div>`;
    }

    return presentResources
        .map(res => {
            return `
                <div class="res-row">
                    <span>${res}:</span>
                    <span class="res-count-positive">${counts[res]}</span>
                </div>
            `;
        })
        .join('');
}

/**
 * Queries the main state handler endpoint to receive the latest snapshot and route match outcomes.
 */
async function updateVisualizer() {
    try {
        const response = await fetch('/galaxy');
        if (!response.ok) throw new Error("Server is unreachable");

        const data: GalaxyResponse = await response.json();

        renderGalaxy(data.planets, data.explorers);
        updateExplorersPanel(data.explorers);

        if (selectedPlanetId !== null) {
            const currentPlanet = data.planets.find(p => p.id === selectedPlanetId);
            if (currentPlanet) {
                showDetails(currentPlanet, data.explorers);
            }
        }

        if (data.game_won) {
            triggerGameVictory(data.winner_id);
            return;
        }

        // Trigger termination screens if all generated task workers lose structural health
        if (data.explorers.length > 0 && data.explorers.every(ex => !ex.alive)) {
            triggerGameOver();
        }
    } catch (error) {
        console.error("Fetching galaxy data error: ", error);
    }
}

/**
 * Requests raw streaming log traces, keeping container offsets pinned down to the scroll footer.
 */
async function updateLogs() {
    try {
        const response = await fetch('/logs');
        const logs: string[] = await response.json();

        const content = document.getElementById('log-content')!;
        const scrollArea = document.getElementById('log-scroll-area')!;

        // Check if user scroll buffer alignment sits within range thresholds of container base
        const isAtBottom = scrollArea.scrollHeight - scrollArea.clientHeight <= scrollArea.scrollTop + 10;

        content.innerHTML = logs
            .map(log => `<div class="log-entry">${log}</div>`)
            .join('');

        if (isAtBottom) {
            scrollArea.scrollTop = scrollArea.scrollHeight;
        }
    } catch (e) { console.error("Error log:", e); }
}

/**
 * Freezes interface refreshing loops and targets state view adjustments towards loss layouts.
 */
function triggerGameOver() {
    console.log("GAME OVER DETECTED");

    if (visualizerIntervalId) clearInterval(visualizerIntervalId);
    if (logsIntervalId) clearInterval(logsIntervalId);

    gameInterface.classList.add('hidden');
    gameOverScreen.classList.remove('hidden');
}

/**
 * Freezes engine synchronization and displays winner details on the victory view layout.
 * * @param winnerId Numerical identity key matching the winning explorer thread.
 */
function triggerGameVictory(winnerId: number | null) {
    console.log(`VICTORY for Explorer #${winnerId}!`);

    if (visualizerIntervalId) clearInterval(visualizerIntervalId);
    if (logsIntervalId) clearInterval(logsIntervalId);

    const winnerMessageEl = document.getElementById('victory-message');
    if (winnerMessageEl) {
        if (winnerId !== null) {
            winnerMessageEl.innerHTML = `Explorer <strong style="color: #00ffcc; text-shadow: 0 0 10px rgba(0,255,204,0.8);">#${winnerId}</strong> completed all the tasks successfully!`;
        }
    }

    gameInterface.classList.add('hidden');
    gameVictoryScreen.classList.remove('hidden');
}

/**
 * Initializes visual refresh triggers and spins up window async intervals.
 */
function startGameEngine() {
    gameOverScreen.classList.add('hidden');
    gameVictoryScreen.classList.add('hidden');

    updateVisualizer();
    updateLogs();

    logsIntervalId = window.setInterval(updateLogs, 500);
    visualizerIntervalId = window.setInterval(updateVisualizer, 1000);
}

btnStartGame.addEventListener('click', () => {
    menuInitial.classList.add('hidden');
    menuDifficulty.classList.remove('hidden');
});

btnRestart.addEventListener('click', () => {
    gameOverScreen.classList.add('hidden');

    mainMenu.classList.remove('hidden');
    menuInitial.classList.remove('hidden');
    menuDifficulty.classList.add('hidden');

    selectedPlanetId = null;
    visualizerIntervalId = null;
    logsIntervalId = null;
});

btnRestartVictory.addEventListener('click', () => {
    gameVictoryScreen.classList.add('hidden');

    mainMenu.classList.remove('hidden');
    menuInitial.classList.remove('hidden');
    menuDifficulty.classList.add('hidden');

    selectedPlanetId = null;
    visualizerIntervalId = null;
    logsIntervalId = null;
});

// Configure event mappings for level selections, launching backend configuration updates
diffButtons.forEach(button => {
    button.addEventListener('click', async (e) => {
        const target = e.currentTarget as HTMLButtonElement;
        const difficulty = target.getAttribute('data-diff')!;

        let planetsCount = parseInt(planetsInput.value, 10);

        // Enforce safe parsing bounds checking before payload transmission
        if (isNaN(planetsCount)) {
            planetsCount = 30;
        } else {
            if (planetsCount < 7) planetsCount = 7;
            if (planetsCount > 50) planetsCount = 50;
        }

        console.log(`Difficulty chosen: ${difficulty}, Planets requested: ${planetsCount}`);

        try {
            await fetch('/start-game', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    difficulty,
                    planets_count: planetsCount
                })
            });
        } catch (err) {
            console.error("Backend sync error:", err);
        }

        mainMenu.classList.add('hidden');
        gameInterface.classList.remove('hidden');

        startGameEngine();
    });
});