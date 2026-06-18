"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var _a;
let selectedPlanetId = null;
let visualizerIntervalId = null;
let logsIntervalId = null;
const mainMenu = document.getElementById('main-menu');
const menuInitial = document.getElementById('menu-initial');
const menuDifficulty = document.getElementById('menu-difficulty');
const gameInterface = document.getElementById('game-interface');
const btnStartGame = document.getElementById('btn-start-game');
const diffButtons = document.querySelectorAll('.btn-diff');
const planetsInput = document.getElementById('planets-count');
const gameOverScreen = document.getElementById('game-over-screen');
const btnRestart = document.getElementById('btn-restart');
function renderGalaxy(planets, explorers) {
    const container = document.getElementById('galaxy-container');
    const canvas = document.getElementById('connections-canvas');
    if (!container || !canvas)
        return;
    const width = container.clientWidth;
    const height = container.clientHeight;
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    container.querySelectorAll('.planet-wrapper').forEach(el => el.remove());
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) / 2 - 80;
    const planetPositions = new Map();
    planets.forEach((p, i) => {
        const angle = (i * 2 * Math.PI) / planets.length;
        const x = centerX + radius * Math.cos(angle);
        const y = centerY + radius * Math.sin(angle);
        planetPositions.set(p.id, { x, y });
    });
    ctx.setLineDash([]);
    ctx.strokeStyle = "rgba(52, 152, 219, 0.2)";
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 5]);
    const drawnConnections = new Set();
    planets.forEach(p => {
        const start = planetPositions.get(p.id);
        p.neighbors.forEach(neighborId => {
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
    planets.forEach(p => {
        const pos = planetPositions.get(p.id);
        const planetWrapper = document.createElement('div');
        const planet_name = p.name;
        planetWrapper.className = 'planet-wrapper';
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
        const localExplorers = explorers.filter(ex => ex.current_planet === p.id && ex.alive);
        localExplorers.forEach((ex, index) => {
            const exDiv = document.createElement('div');
            exDiv.className = 'explorer-badge';
            exDiv.innerText = `#${ex.id}`;
            exDiv.style.position = 'absolute';
            exDiv.style.bottom = `${40 + (index * 20)}px`;
            exDiv.style.left = '50%';
            exDiv.style.transform = 'translateX(-50%)';
            planetWrapper.appendChild(exDiv);
        });
        container.appendChild(planetWrapper);
    });
}
function showDetails(p, allExplorers) {
    const box = document.getElementById('planet-details');
    if (!box)
        return;
    box.classList.remove('hidden');
    document.getElementById('det-name').innerText = `${p.name} #${p.id}`;
    document.getElementById('det-type').innerText = p.planet_type;
    const stateEl = document.getElementById('det-alive');
    stateEl.innerText = p.alive ? "VIVO" : "DISTRUTTO";
    stateEl.style.color = p.alive ? "#2ecc71" : "#e74c3c";
    document.getElementById('det-energy').innerText = `${p.energy_cells} unità`;
    document.getElementById('det-rockets').innerText = p.stats.rockets.toString();
    document.getElementById('det-neighbors').innerText = p.neighbors.length > 0
        ? p.neighbors.join(', ')
        : "Nessun vicino";
    const baseContainer = document.getElementById('det-res-base');
    if (p.resources_base.length > 0) {
        baseContainer.innerHTML = p.resources_base
            .map(res => `<span class="badge base">${res}</span>`)
            .join('');
    }
    else {
        baseContainer.innerHTML = '<span style="color: #666; font-style: italic;">Nessuna</span>';
    }
    const complexContainer = document.getElementById('det-res-complex');
    if (p.resources_complex.length > 0) {
        complexContainer.innerHTML = p.resources_complex
            .map(res => `<span class="badge complex">${res}</span>`)
            .join('');
    }
    else {
        complexContainer.innerHTML = '<span style="color: #666; font-style: italic;">Nessuna</span>';
    }
    const rocketEl = document.getElementById('det-rockets-ready');
    if (p.has_rocket) {
        rocketEl.innerHTML = '<span class="status-ready">PRONTO</span>';
    }
    else {
        rocketEl.innerHTML = '<span class="status-empty">NON DISPONIBILE</span>';
    }
}
function closePlanetDetails() {
    const box = document.getElementById('planet-details');
    if (box) {
        box.classList.add('hidden');
    }
    selectedPlanetId = null;
}
(_a = document.getElementById('close-details')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', closePlanetDetails);
function updateExplorersPanel(explorers) {
    const container = document.getElementById('explorers-status-container');
    if (!container)
        return;
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
function renderInventory(bag) {
    const counts = {};
    bag.forEach(item => {
        counts[item] = (counts[item] || 0) + 1;
    });
    const presentResources = Object.keys(counts).sort();
    if (presentResources.length === 0) {
        return `<div class="no-res">Zaino vuoto</div>`;
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
function updateVisualizer() {
    return __awaiter(this, void 0, void 0, function* () {
        try {
            const response = yield fetch('/galaxy');
            if (!response.ok)
                throw new Error("Server Rust non raggiungibile");
            const data = yield response.json();
            renderGalaxy(data.planets, data.explorers);
            updateExplorersPanel(data.explorers);
            if (selectedPlanetId !== null) {
                const currentPlanet = data.planets.find(p => p.id === selectedPlanetId);
                if (currentPlanet) {
                    showDetails(currentPlanet, data.explorers);
                }
            }
            if (data.explorers.length > 0 && data.explorers.every(ex => !ex.alive)) {
                triggerGameOver();
            }
        }
        catch (error) {
            console.error("Errore nel recupero dati galassia:", error);
        }
    });
}
function updateLogs() {
    return __awaiter(this, void 0, void 0, function* () {
        try {
            const response = yield fetch('/logs');
            const logs = yield response.json();
            const content = document.getElementById('log-content');
            const scrollArea = document.getElementById('log-scroll-area');
            const isAtBottom = scrollArea.scrollHeight - scrollArea.clientHeight <= scrollArea.scrollTop + 10;
            content.innerHTML = logs
                .map(log => `<div class="log-entry">${log}</div>`)
                .join('');
            if (isAtBottom) {
                scrollArea.scrollTop = scrollArea.scrollHeight;
            }
        }
        catch (e) {
            console.error("Errore log:", e);
        }
    });
}
function triggerGameOver() {
    console.log("GAME OVER DETECTED");
    if (visualizerIntervalId)
        clearInterval(visualizerIntervalId);
    if (logsIntervalId)
        clearInterval(logsIntervalId);
    gameInterface.classList.add('hidden');
    gameOverScreen.classList.remove('hidden');
}
function startGameEngine() {
    gameOverScreen.classList.add('hidden');
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
    window.location.reload();
});
diffButtons.forEach(button => {
    button.addEventListener('click', (e) => __awaiter(void 0, void 0, void 0, function* () {
        const target = e.currentTarget;
        const difficulty = target.getAttribute('data-diff');
        let planetsCount = parseInt(planetsInput.value, 10);
        if (isNaN(planetsCount)) {
            planetsCount = 30;
        }
        else {
            if (planetsCount < 7)
                planetsCount = 7;
            if (planetsCount > 50)
                planetsCount = 50;
        }
        console.log(`Difficoltà inizializzata: ${difficulty}, Pianeti richiesti: ${planetsCount}`);
        try {
            yield fetch('/start-game', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    difficulty,
                    planets_count: planetsCount
                })
            });
        }
        catch (err) {
            console.error("Errore sincronizzazione backend:", err);
        }
        mainMenu.classList.add('hidden');
        gameInterface.classList.remove('hidden');
        startGameEngine();
    }));
});
