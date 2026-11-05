#include "DynamicPulseEngine.h"

void DynamicPulseEngine::processAudio(float currentEnergy, double deltaTime) {
    // 1. Calcolo del Flux (Il "Quando")
    double energyFlux = (currentEnergy - previousEnergy) / deltaTime;

    if (energyFlux > fluxThreshold) {
        handleNewBeat(currentEnergy, deltaTime);
    }

    previousEnergy = currentEnergy;
}

void DynamicPulseEngine::handleNewBeat(float energy, double deltaTime) {
    // 2. Calcolo della Curva (Il "Quanto")
    double detectedBPM = calculateInstantBPM(energy);
    currentBPM = (currentBPM * smoothingFactor) + (detectedBPM * (1.0 - smoothingFactor));

    // 3. Calcolo della Gerarchia (Il "Dove")
    double beatWeight = calculateBeatWeight(energy);

    // Salva il punto nella griglia
    beatGrid.push_back({lastBeatTime, currentBPM, beatWeight});
    lastBeatTime += (currentBPM / 60.0) * deltaTime; // Aggiornamento approssimativo del tempo
}

double DynamicPulseEngine::calculateInstantBPM(float energy) {
    // Trasforma l'energia in una velocità di base
    return (energy * 12.0); // Esempio: Energia 10 -> BPM 120
}

double DynamicPulseEngine::calculateBeatWeight(float energy) {
    // Determina il peso del battito (da 0.5 a 1.0)
    if (energy >= 8.5f) return 1.0;  // Downbeat forte
    if (energy >= 6.0f) return 0.75; // Battito intermedio
    return 0.5;                       // Battito leggero
}
