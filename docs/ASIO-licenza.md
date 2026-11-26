# ASIO: licenza e obblighi (promemoria interno)

[English](ASIO-licenza.en.md) · **Italiano**

Documento per chi sviluppa e distribuisce SpinLab DJ. Non fa parte del manuale per gli utenti. Riassume l'accordo «Steinberg ASIO SDK Licensing Agreement 2.0.5» (SDK 2.3.4) e le «ASIO Usage Guidelines». **Non è un parere legale**: per le decisioni importanti fa fede il testo originale, nel pacchetto dell'SDK.

## Prima di distribuire (anche gratis, fuori dal proprio gruppo)

- [ ] Far firmare a Steinberg l'accordo («Steinberg ASIO Licensing Agreement.pdf», nel pacchetto SDK). Costo: nessuno, senza royalty (§ 4). Contatto: reception@steinberg.de. L'alternativa è la GPL v3, che obbliga a pubblicare tutto il codice.
- [ ] Se una terza persona o società distribuisce SpinLab con il proprio nome, deve firmare anch'essa l'accordo (§ 2.7 e 2.8).
- [ ] Non committare l'SDK nel repository, né metterlo nell'installer (§ 2.3). Nel repository ci sono solo i due loghi.

## Cosa deve apparire nel programma e nei materiali

| Dove | Cosa | Stato |
|---|---|---|
| Finestra che configura l'ASIO (Impostazioni → AUDIO) | Logo «ASIO Compatible» + dicitura | fatto |
| Finestra «Informazioni» (pulsante nella Guida) | Logo + dicitura | fatto |
| Documentazione (manuale, guida nel programma, README) | Logo e dicitura | fatto |
| Sito web del prodotto | Logo su ogni pagina che parla del prodotto o di ASIO (non basta il piè di pagina); dicitura | da fare, quando ci sarà il sito |
| Pagine legali / imprint | Logo o dicitura | da fare |
| Pubblicità e materiale promozionale | Logo (dove c'è spazio) | da fare |
| Confezione, se il prodotto ha una forma fisica | Logo | non applicabile |

**Dicitura** (§ 3.1.k dell'accordo): *ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.*
Le linee guida del logo (§ 15) usano invece *ASIO is a registered trademark of Steinberg Media Technologies GmbH.* Il logo bianco su sfondo trasparente non contiene testo; la versione nera contiene già la seconda frase. Se serve una sola formula, chiedere a Steinberg quale usare.

La prima volta che «ASIO» compare in un testo del prodotto va scritto con il simbolo ®.

## Divieti

- «ASIO» non può stare nel nome dell'azienda, né nel nome o nel logo del prodotto (si può scrivere «per ASIO» o «compatibile con ASIO» in testo normale, con il logo vicino).
- Non usare «ASIO» su merchandising (maglie, penne...), eccetto le versioni demo del prodotto.
- Non modificare i loghi: vanno usati così come forniti.
- Non usare il logo su prodotti non compatibili con ASIO.

## Durante la vita del prodotto

- Se gli utenti superano 1.000, bisogna tenere traccia dell'uso del marchio per paese, anno e prodotto, e fornirla a Steinberg su richiesta (§ 3.1.j).
- Steinberg può chiudere l'accordo con 36 mesi di preavviso (6 se ne propone uno nuovo), o subito se non si rimedia a una violazione entro 14 giorni.
- Legge applicabile: tedesca. Foro: Amburgo.

## Compilazione

L'SDK serve solo a chi compila. Il percorso è impostato in `src-tauri/.cargo/config.toml` (variabile `CPAL_ASIO_DIR`). Versione usata: 2.3.4.
