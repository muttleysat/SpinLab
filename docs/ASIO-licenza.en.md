# ASIO: licence and obligations (internal reminder)

**English** · [Italiano](ASIO-licenza.md)

Document for whoever develops and distributes SpinLab DJ. It is not part of the user manual. It summarises the «Steinberg ASIO SDK Licensing Agreement 2.0.5» (SDK 2.3.4) and the «ASIO Usage Guidelines». **It is not legal advice**: for important decisions the original text, in the SDK package, is what counts.

## Before distributing (even for free, outside your own group)

- [ ] Have Steinberg sign the agreement («Steinberg ASIO Licensing Agreement.pdf», in the SDK package). Cost: none, royalty-free (§ 4). Contact: reception@steinberg.de. The alternative is the GPL v3, which requires publishing all the code.
- [ ] If a third person or company distributes SpinLab under their own name, they must sign the agreement too (§ 2.7 and 2.8).
- [ ] Do not commit the SDK to the repository, nor put it in the installer (§ 2.3). The repository only contains the two logos.

## What must appear in the program and in the materials

| Where | What | Status |
|---|---|---|
| Window that configures ASIO (Settings → AUDIO) | «ASIO Compatible» logo + notice | done |
| «About» window (button in the Guide) | Logo + notice | done |
| Documentation (manual, in-program guide, README) | Logo and notice | done |
| Product website | Logo on every page that talks about the product or ASIO (the footer is not enough); notice | to do, when there is a website |
| Legal pages / imprint | Logo or notice | to do |
| Advertising and promotional material | Logo (where there is room) | to do |
| Packaging, if the product has a physical form | Logo | not applicable |

**Notice** (§ 3.1.k of the agreement): *ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.*
The logo guidelines (§ 15) use instead *ASIO is a registered trademark of Steinberg Media Technologies GmbH.* The white logo on a transparent background contains no text; the black version already contains the second sentence. If a single wording is needed, ask Steinberg which one to use.

The first time «ASIO» appears in a text of the product it must be written with the ® symbol.

## Prohibitions

- «ASIO» cannot be in the company name, nor in the name or logo of the product (you can write «for ASIO» or «ASIO compatible» in plain text, with the logo nearby).
- Do not use «ASIO» on merchandising (shirts, pens...), except demo versions of the product.
- Do not modify the logos: they must be used as supplied.
- Do not use the logo on products that are not ASIO compatible.

## During the life of the product

- If users exceed 1,000, the use of the trademark must be tracked by country, year and product, and provided to Steinberg on request (§ 3.1.j).
- Steinberg can terminate the agreement with 36 months' notice (6 if it proposes a new one), or immediately if a breach is not remedied within 14 days.
- Applicable law: German. Venue: Hamburg.

## Building

The SDK is only needed by whoever builds. The path is set in `src-tauri/.cargo/config.toml` (variable `CPAL_ASIO_DIR`). Version used: 2.3.4.
