# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Language Preference**: Always communicate in Japanese (日本語) when interacting with users. All documentation, comments, and code annotations should also be written in Japanese.

## Guidelines

@docs/guidelines.md

## Project Architecture

This is a Tauri desktop application built with:
- **Frontend**: SvelteKit with TypeScript and Vite
- **Backend**: Rust (Tauri v2)
- **Adapter**: Static adapter for SSG (no SSR due to Tauri constraints)

### Key Structure
- `src/` - SvelteKit frontend application
- `src-tauri/` - Rust backend with Tauri configuration
- `src-tauri/src/lib.rs` - Main Tauri application logic
- `src-tauri/tauri.conf.json` - Tauri app configuration
- `svelte.config.js` - Uses adapter-static for prerendering

## Development Commands

### Frontend Development
- `yarn dev` - Start development server (runs SvelteKit dev server on localhost:1420)
- `yarn build` - Build frontend for production
- `yarn preview` - Preview production build

### Type Checking
- `yarn check` - Run Svelte type checking
- `yarn check:watch` - Run type checking in watch mode

### Tauri Development
- `yarn tauri dev` - Start Tauri development mode (builds and runs desktop app)
- `yarn tauri build` - Build production desktop application

## Important Notes

- The app uses static adapter because Tauri doesn't support SSR
- Development server runs on port 1420 as configured in tauri.conf.json
- Rust backend uses library crate pattern with `tauri_app_with_cc_lib` name
- Frontend build output goes to `build/` directory for Tauri consumption