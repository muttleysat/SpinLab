#include <iostream>
#include "DynamicPulseEngine.h"

int main() {
    DynamicPulseEngine engine;
    
    // Simulazione di una progressione Rock (es. Come On Eileen)
    // Ogni passo rappresenta un piccolo incremento di tempo
    float energyLevels[] = {2.0, 3.5, 8.5, 9.0, 4.0, 3.0, 7.5, 8.0}; // Picchi e valli energetiche
    double timeStep = 0.5; // Ogni "battito" simulato dura 0.5 secondi

    std::cout << "Inizio Analisi Dynamic Pulse Engine..." << std::endl;
    std::cout << "BPM Iniziale: " << engine.getCurrentBPM() << std::endl;
    std::cout << "------------------------------------" << std::endl;

    for (float energy : energyLevels) {
        engine.processAudio(energy, timeStep);
        std::cout << "Energia: " << energy 
                  << " | BPM Attuale: " << engine.getCurrentBPM() 
                  << " | Battiti Rilevati: " << engine.getBeatGrid().size() 
                  << std::endl;
    }

    return 0;
}
