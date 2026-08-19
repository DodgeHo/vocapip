# ADR-0001: Offline-first Windows MVP constraints

- Status: Accepted
- Date: 2026-08-18

## Context

VocaPip is a Windows 10/11 x64 desktop pet and micro-learning application. The MVP must remain fully useful with networking blocked and must not initiate HTTP, WebSocket, telemetry, update, dictionary, or AI traffic at runtime.

## Decision

Use Tauri 2, React, TypeScript, and Rust. Keep learner data in a local SQLite database controlled by Rust commands. Use `ts-fsrs` only to calculate scheduling state; XP and T1-T5 adaptation remain separate motivation and content-selection systems.

The webview receives only narrow Tauri capabilities. Do not ship shell, HTTP, updater, global filesystem, or arbitrary SQL permissions. Local import/export is mediated by validated Rust commands and explicit native file dialogs.

Language packs are versioned, declarative resources with `sourceLanguage`, `targetLanguage`, `explanationLanguage`, `packVersion`, and `compatibilityVersion`. Records and fields fail validation when their provenance or license is not on the approved allowlist.

## Consequences

- A Rust command layer is more code than exposing a generic SQL plugin but has a smaller permission surface.
- Local development uses Vite's loopback server; production application behavior must be tested from the bundled assets with networking blocked.
- Content, source code, pet artwork, and optional audio are licensed and documented independently.
