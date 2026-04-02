interface PlanetStatsDTO {
    asteroids: number;
    sunrays: number;
    rockets: number;
}

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

interface ExplorerDataDTO {
    id: number;
    current_planet: number;
    bag: string[];
    alive: boolean;
}

interface GalaxyResponse {
    planets: PlanetDataDTO[];
    explorers: ExplorerDataDTO[];
}


let selectedPlanetId: number | null = null;

function renderGalaxy(planets: PlanetDataDTO[], explorers: ExplorerDataDTO[]) {
    const container = document.getElementById('galaxy-container');
    if (!container) return;

    container.innerHTML = '';

    planets.forEach(p => {
        
        const planetWrapper = document.createElement('div');
        planetWrapper.className = 'planet-wrapper';
        planetWrapper.style.position = 'relative'; 

        
        const img = document.createElement('img');
        if (p.alive) {
            img.src = 'media/planet.gif';
            img.className = 'planet-sprite';
        } else {
            img.src = 'media/dead_planet.gif';
            img.className = 'planet-sprite dead-animation';
        }

        if (selectedPlanetId === p.id) {
            img.style.border = "2px solid #e74c3c";
            img.style.borderRadius = "50%";
        }

        img.onclick = () => {
            selectedPlanetId = p.id;
            
            showDetails(p, explorers);
            renderGalaxy(planets, explorers);
        };

        planetWrapper.appendChild(img);
        
        const localExplorers = explorers.filter(ex => ex.current_planet === p.id);

        localExplorers.forEach((ex, index) => {
            const exDiv = document.createElement('div');
            exDiv.className = 'explorer-badge';
            exDiv.innerText = `#${ex.id}`;
            exDiv.style.position = 'absolute';
            exDiv.style.top = `${index * 20}px`;
            exDiv.style.left = '50%';
            exDiv.style.transform = 'translateX(-50%)';

            planetWrapper.appendChild(exDiv);
        });

        container.appendChild(planetWrapper);
    });
}

function showDetails(p: PlanetDataDTO, allExplorers: ExplorerDataDTO[]) {
    const box = document.getElementById('planet-details');
    if (!box) return;

    box.classList.remove('hidden');
    
    document.getElementById('det-name')!.innerText = p.name;
    document.getElementById('det-type')!.innerText = p.planet_type;
    
    const stateEl = document.getElementById('det-alive')!;
    stateEl.innerText = p.alive ? "VIVO" : "DISTRUTTO";
    stateEl.style.color = p.alive ? "#2ecc71" : "#e74c3c";

    document.getElementById('det-energy')!.innerText = `${p.energy_cells} unità`;
    document.getElementById('det-rockets')!.innerText = p.stats.rockets.toString();
    document.getElementById('det-neighbors')!.innerText = p.neighbors.length > 0
        ? p.neighbors.join(', ')
        : "Nessun vicino";


    const baseContainer = document.getElementById('det-res-base')!;
    if (p.resources_base.length > 0) {
        baseContainer.innerHTML = p.resources_base
            .map(res => `<span class="badge base">${res}</span>`)
            .join('');
    } else {
        baseContainer.innerHTML = '<span style="color: #666; font-style: italic;">Nessuna</span>';
    }


    const complexContainer = document.getElementById('det-res-complex')!;
    if (p.resources_complex.length > 0) {
        complexContainer.innerHTML = p.resources_complex
            .map(res => `<span class="badge complex">${res}</span>`)
            .join('');
    } else {
        complexContainer.innerHTML = '<span style="color: #666; font-style: italic;">Nessuna</span>';
    }

    const rocketEl = document.getElementById('det-rockets-ready')!;

    if (p.has_rocket) {
        rocketEl.innerHTML = '<span class="status-ready">PRONTO</span>';
    } else {
        rocketEl.innerHTML = '<span class="status-empty">NON DISPONIBILE</span>';
    }
    const explorerSection = document.getElementById('det-explorers-list')!;
    const localExplorers = allExplorers.filter(ex => ex.current_planet === p.id);

    if (localExplorers.length > 0) {
        explorerSection.innerHTML = localExplorers.map(ex => `
            <div class="explorer-detail-card">
                <div class="explorer-header">Explorer #${ex.id}</div>
                <div class="explorer-bag">
                    <strong>Bag:</strong> ${ex.bag.length > 0 ? ex.bag.join(', ') : 'Vuota'}
                </div>
            </div>
        `).join('');
    } else {
        explorerSection.innerHTML = '<p class="no-data">Nessun explorer su questo pianeta</p>';
    }
}


async function updateVisualizer() {
    try {
        const response = await fetch('/galaxy');
        if (!response.ok) throw new Error("Server Rust non raggiungibile");
        
        const data: GalaxyResponse = await response.json();
        
        renderGalaxy(data.planets, data.explorers);

        
        if (selectedPlanetId !== null) {
            const currentPlanet = data.planets.find(p => p.id === selectedPlanetId);
            if (currentPlanet) {
                
                showDetails(currentPlanet, data.explorers);
            }
        }
    } catch (error) {
        console.error("Errore nel recupero dati galassia:", error);
    }
}

async function updateLogs() {
    try {
        const response = await fetch('/logs');
        const logs: string[] = await response.json();
        const content = document.getElementById('log-content')!;

        
        content.innerHTML = logs.reverse()
            .map(log => `<div class="log-entry">${log}</div>`)
            .join('');

    } catch (e) { console.error("Errore log:", e); }
}

setInterval(updateLogs, 500); 

setInterval(updateVisualizer, 1000);

updateVisualizer();