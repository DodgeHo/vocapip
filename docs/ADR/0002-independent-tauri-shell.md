# ADR-0002: Implement an independent Tauri shell instead of forking OpenPet

- Status: Accepted
- Date: 2026-08-18

## Evidence reviewed

OpenPet was cloned and reviewed at immutable commit `0675f4932a41d66d1b1fdbc6ddd94c46d6bd0ccd`, tagged `v0.1.6`, from `https://github.com/X-T-E-R/OpenPet.git`.

It is a GPL-3.0-or-later Tauri 2 + React + TypeScript + Rust application and demonstrates useful transparent window and tray behavior. The reviewed version also starts a local HTTP service and contains `reqwest`, release/update checks against GitHub, downloadable assets, external links, and broad process/dialog capabilities. It does not provide VocaPip's SQLite migrations, FSRS learning model, XP system, content queue, or autostart behavior. The included Nia sprites and icons do not have a separately auditable asset license in the reviewed tree.

## Decision

Implement an independent Tauri 2 application shell. Reproduce only public desktop-window behavior and use project-authored CSS artwork. Do not copy OpenPet code, network services, or unaudited media.

VocaPip remains licensed GPL-3.0-or-later as directed by the PRD and project authorization. The OpenPet audit and exact commit remain recorded because they informed this irreversible base decision, but OpenPet is not represented as a code or asset dependency.

## Rationale

Removing OpenPet's network and plugin surface while adding the complete learning and persistence architecture would be a larger and riskier transformation than a narrow Tauri shell. The independent implementation also avoids redistributing assets whose rights could not be verified.

## Consequences

- Window, tray, single-instance, and reminder behavior require direct implementation and testing.
- Project-authored pet visuals are intentionally simple so media production cannot block the learning loop.
- OpenPet attribution is retained in research notices, while its source is not vendored or bundled.
