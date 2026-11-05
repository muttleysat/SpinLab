#ifndef DYNAMIC_PULSE_ENGINE_H
#define DYNAMIC_PULSE_ENGINE_H

#include <vector>

// Rappresenta ogni battito rilevato nella griglia
struct BeatPoint {
    double timestamp;       // Momento esatto (secondi)
    double bpmAtMoment;     // Velocità in quel momento
    double weight;          // Importanza del battito (0.5 - 1.0)
};

class DynamicPulseEngine {
private:
    // --- Proprietà di Stato ---
    double currentBPM = 120.0;
    double lastBeatTime = 0.0;
    float previousEnergy = 0.0f;
    std::vector<BeatPoint> beatGrid;

    // --- Costanti del "DNA" Algoritmico (Ottimizzate per il Rock) ---
    const float fluxThreshold = 1.5f;       // Sensibilità dei picchi
    const double smoothingFactor = 0.7;     // Fluidità della curva BPM
    const float maxEnergyRange = 10.0f;     // Dinamica energetica

public:
    DynamicPulseEngine() {}

    /**
     * Analizza il flusso audio continuo.
     * @param currentEnergy Energia sonora attuale (da 0 a 10)
     * @param deltaTime Tempo trascorso dall'ultimo campione
     */
    void processAudio(float currentEnergy, double deltaTime);

    // Funzioni di supporto per la logica interna
    double calculateInstantBPM(float energy);
    double calculateBeatWeight(float energy);
    void handleNewBeat(float energy, double deltaTime);

    // Accessori
    double getCurrentBPM() const { return currentBPM; }
    const std::vector<BeatPoint>& getBeatGrid() const { return beatGrid; }
};

#endif
