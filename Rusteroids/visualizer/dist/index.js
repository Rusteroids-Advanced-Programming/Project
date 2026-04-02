"use strict";
// --- Interfacce ---
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
// Variabile globale per seguire quale pianeta stiamo guardando
let selectedPlanetId = null;
/**
 * Rendering della Galassia: Disegna i pianeti in fila
 */
function renderGalaxy(planets, explorers) {
    const container = document.getElementById('galaxy-container');
    if (!container)
        return;
    container.innerHTML = '';
    planets.forEach(p => {
        // 1. Creiamo un wrapper per il pianeta e i suoi occupanti
        const planetWrapper = document.createElement('div');
        planetWrapper.className = 'planet-wrapper';
        planetWrapper.style.position = 'relative'; // Necessario per posizionare gli explorer internamente
        // 2. Logica dell'immagine (la tua logica esistente)
        const img = document.createElement('img');
        if (p.alive) {
            img.src = 'media/planet.gif';
            img.className = 'planet-sprite';
        }
        else {
            img.src = 'media/dead_planet.gif';
            img.className = 'planet-sprite dead-animation';
        }
        if (selectedPlanetId === p.id) {
            img.style.border = "2px solid #e74c3c";
            img.style.borderRadius = "50%";
        }
        img.onclick = () => {
            selectedPlanetId = p.id;
            // Qui dovrai passare anche gli explorer a showDetails se vuoi vedere la loro Bag
            showDetails(p, explorers);
            renderGalaxy(planets, explorers);
        };
        planetWrapper.appendChild(img);
        // 3. AGGIUNTA EXPLORER: Filtriamo chi si trova su questo pianeta
        const localExplorers = explorers.filter(ex => ex.current_planet === p.id);
        localExplorers.forEach((ex, index) => {
            const exDiv = document.createElement('div');
            exDiv.className = 'explorer-badge';
            exDiv.innerText = `🚀 #${ex.id}`;
            // Posizionamento sfalsato se ce n'è più di uno
            exDiv.style.position = 'absolute';
            exDiv.style.top = `${index * 20}px`;
            exDiv.style.left = '50%';
            exDiv.style.transform = 'translateX(-50%)';
            planetWrapper.appendChild(exDiv);
        });
        container.appendChild(planetWrapper);
    });
}
/**
 * Aggiorna il pannello laterale includendo le risorse
 */
function showDetails(p, allExplorers) {
    const box = document.getElementById('planet-details');
    if (!box)
        return;
    box.classList.remove('hidden');
    // Dati base
    document.getElementById('det-name').innerText = p.name;
    document.getElementById('det-type').innerText = p.planet_type;
    // Stato Vita
    const stateEl = document.getElementById('det-alive');
    stateEl.innerText = p.alive ? "VIVO" : "DISTRUTTO";
    stateEl.style.color = p.alive ? "#2ecc71" : "#e74c3c";
    document.getElementById('det-energy').innerText = `${p.energy_cells} unità`;
    document.getElementById('det-rockets').innerText = p.stats.rockets.toString();
    document.getElementById('det-neighbors').innerText = p.neighbors.length > 0
        ? p.neighbors.join(', ')
        : "Nessun vicino";
    // --- NUOVA LOGICA PER LE RISORSE ---
    // Svuotiamo e ripopoliamo le Risorse Base (Generator)
    const baseContainer = document.getElementById('det-res-base');
    if (p.resources_base.length > 0) {
        baseContainer.innerHTML = p.resources_base
            .map(res => `<span class="badge base">${res}</span>`)
            .join('');
    }
    else {
        baseContainer.innerHTML = '<span style="color: #666; font-style: italic;">Nessuna</span>';
    }
    // Svuotiamo e ripopoliamo le Risorse Complesse (Combinator)
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
        rocketEl.innerHTML = '<span class="status-ready">PRONTO 🚀</span>';
    }
    else {
        rocketEl.innerHTML = '<span class="status-empty">NON DISPONIBILE</span>';
    }
    const explorerSection = document.getElementById('det-explorers-list');
    const localExplorers = allExplorers.filter(ex => ex.current_planet === p.id);
    if (localExplorers.length > 0) {
        explorerSection.innerHTML = localExplorers.map(ex => `
            <div class="explorer-detail-card">
                <div class="explorer-header">🚀 Explorer #${ex.id}</div>
                <div class="explorer-bag">
                    <strong>Bag:</strong> ${ex.bag.length > 0 ? ex.bag.join(', ') : 'Vuota'}
                </div>
            </div>
        `).join('');
    }
    else {
        explorerSection.innerHTML = '<p class="no-data">Nessun explorer su questo pianeta.</p>';
    }
}
/**
 * Funzione principale di Fetching
 */
function updateVisualizer() {
    return __awaiter(this, void 0, void 0, function* () {
        try {
            const response = yield fetch('/galaxy');
            if (!response.ok)
                throw new Error("Server Rust non raggiungibile");
            // Riceviamo l'oggetto completo dal server Rust
            const data = yield response.json();
            // 1. Renderizziamo la galassia passando ENTRAMBI gli array
            renderGalaxy(data.planets, data.explorers);
            // 2. Aggiornamento dettagli se un pianeta è selezionato
            if (selectedPlanetId !== null) {
                const currentPlanet = data.planets.find(p => p.id === selectedPlanetId);
                if (currentPlanet) {
                    // Passiamo anche gli explorer a showDetails per vedere chi c'è sopra
                    showDetails(currentPlanet, data.explorers);
                }
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
            // Visualizziamo i log (invertiti per averli dal più recente)
            content.innerHTML = logs.reverse()
                .map(log => `<div class="log-entry">${log}</div>`)
                .join('');
        }
        catch (e) {
            console.error("Errore log:", e);
        }
    });
}
// Aggiungi l'update dei log al tuo loop esistente
setInterval(updateLogs, 500); // Più veloce della galassia per reattività
// Avvio del loop ogni secondo
setInterval(updateVisualizer, 1000);
// Primo caricamento immediato
updateVisualizer();
