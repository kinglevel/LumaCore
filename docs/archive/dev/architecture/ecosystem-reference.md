---
title: Ecosystem Reference
category: docs
---
# GDTF/MVR Ecosystem Reference

Reference guide for the external tools, libraries, and patterns in the GDTF/MVR
ecosystem. Useful context when designing and building LumaCore.

**Source:** Consolidated from ecosystem research conducted 2026-03-10.

## Architecture Patterns

The ecosystem follows a **hub-and-spoke** model with modular composition:

- **Standards-driven:** All tools anchor to DIN SPEC 15800 (GDTF) and DIN SPEC 15801 (MVR)
- **Modular composition:** Each tool handles one concern (parsing, creation, visualization, proxying) rather than monolithic stacks
- **Ecosystem aggregation:** Tools cross-reference and build on each other (e.g., gdtf-creator uses GDTF-Share data, Perastage consumes MVR files)

### Conceptual Layers

| Layer | Purpose | Examples |
|---|---|---|
| Specification | Canonical standards documents | DIN SPEC 15800, DIN SPEC 15801 |
| Reference/Knowledge | Guides, crawled sites, fixture databases | GDTF-Share (10,439+ fixtures), crawled manufacturer docs |
| Library | Parsing and manipulation libraries | libMVRgdtf (C++), python-mvr (Python), gdtf crate (Rust) |
| Application | User-facing tools | gdtf-creator, Perastage, LX-Net |
| Analysis/Compatibility | Legacy format research, edge case handling | Depence-dfix, format conversion tools |

### Key Abstractions

- **GDTF** -- fixture type definition (geometry, DMX layout, physical properties)
- **MVR** -- scene/venue container (fixture placement, layers, grouping)
- **Fixture Library** -- indexed collection of GDTF files with metadata
- **DMX Channel Definition** -- attribute-to-channel mapping with modes
- **3D Geometry Representation** -- hierarchical model trees for visualization

## Technology Stacks

### Languages

| Language | Version | Role in Ecosystem |
|---|---|---|
| TypeScript | 5.9.2 | Web applications, API proxies |
| C++ | C++20 | Desktop apps, core parsing libraries |
| Python | 3.8+ | MVR manipulation, scripting |
| Rust | Edition 2024 | Modular control systems, high-performance parsing |

### Frameworks and Runtimes

| Framework | Used For |
|---|---|
| React 19 | Web UI (fixture browsers, patch editors) |
| Convex BaaS | Real-time backend storage |
| wxWidgets | Desktop GUI (cross-platform) |
| Three.js | 3D fixture visualization |
| Express | HTTP API servers |
| Cloudflare Workers | Auth proxies, edge functions |

### Key Dependencies

| Dependency | Purpose |
|---|---|
| libMVRgdtf | C++ GDTF/MVR parsing (reference implementation) |
| python-mvr | Python MVR file manipulation |
| gdtf crate | Rust GDTF parsing |
| jszip | ZIP handling for .gdtf/.mvr containers |
| fast-xml-parser | XML parsing for GDTF/MVR content |
| zod | Runtime schema validation |
| AI SDKs (Claude, Gemini) | LLM-assisted fixture creation |

## Coding Conventions

Patterns observed across ecosystem projects:

### Naming and Style

| Context | Convention |
|---|---|
| React components | PascalCase `.tsx` |
| Route files | lowercase `.tsx` |
| Utility modules | camelCase `.ts` |
| Indentation | 2 spaces (TypeScript/web), 4 spaces (C++/Python) |
| TypeScript mode | Strict, with path aliases (`~/` maps to `app/`) |

### Error Handling

- `try-catch` with `instanceof` type narrowing (TypeScript projects)
- Zod schema validation at API boundaries for runtime safety
- XML parsing errors treated as non-fatal with fallback defaults

### Logging

- Custom tagged logger with colored output
- Format: `[TAG] message` with severity-based coloring
- Session-scoped log files with timestamps

### Styling

- Tailwind CSS with OKLCH color system for perceptual uniformity
- Component-level styling, no global CSS beyond resets and theme tokens

## Integrations and Protocols

### APIs

| API | Purpose |
|---|---|
| GDTF-Share | Fixture library (10,439+ fixtures), manufacturer uploads |
| Convex.dev | Real-time document storage and sync |
| GitHub Actions | CI/CD pipelines |

### Network Protocols

| Protocol | Role |
|---|---|
| DMX512 | Physical fixture control (512 channels per universe) |
| sACN (E1.31) | Streaming ACN -- DMX over IP (multicast) |
| Art-Net | DMX over IP (broadcast/unicast) |
| MVR-xchange | Live MVR scene sharing between applications |

### Data Formats

| Format | Description |
|---|---|
| GDTF (.gdtf) | ZIP container with XML description + 3D models + wheels |
| MVR (.mvr) | ZIP container with XML scene description + embedded GDTFs |
| DMX512 | 512-byte universe frames |
| XML | Internal structure of both GDTF and MVR containers |

## Testing Patterns

### Current State

Testing across the ecosystem is largely **manual and integration-based**:

- TypeScript type checking (`typecheck` scripts) as a first line of defense
- Zod schema validation provides runtime contract enforcement
- Browser-based manual testing for web applications
- No widespread unit test suites observed in ecosystem tools

### Risk Areas

These areas are under-tested and prone to breakage:

- **Session handling** -- state management across reconnects and timeouts
- **XML parsing edge cases** -- malformed GDTF/MVR files from various manufacturers
- **Cache race conditions** -- concurrent access to fixture library caches
- **Protocol interop** -- sACN/Art-Net/MVR-xchange message ordering and timing

### Recommended Test Priorities

1. XML parsing with manufacturer-sourced fixture files (real-world malformed data)
2. Protocol message serialization round-trips
3. ZIP container integrity (nested GDTFs in MVR files)
4. Schema validation boundary cases

## Known Concerns

### Repository and Data Issues

| Concern | Detail |
|---|---|
| Large private cache | ~24 GB firmware archive in private repos |
| No submodule tracking | External repos stored as copies, not git submodules |
| Stale crawled data | Static crawled site snapshots with no refresh mechanism |
| No link verification | External references not validated for freshness or availability |

### Coverage Gaps

- Incomplete protocol coverage -- not all sACN/Art-Net/MVR-xchange features represented
- Terminology confusion in specs (e.g., "blinder" vs "strobe" classification overlap)
- Legacy format support is research-only (Depence-dfix), no production converters

## Ecosystem Map

### External Repositories

| Repository | Language | Role |
|---|---|---|
| **zeevonk** | Rust | Code reference for idiomatic Rust lighting control patterns |
| **spec** | -- | Canonical DIN SPEC 15800/15801 standards documents |
| **libMVRgdtf** | C++ | Reference GDTF/MVR parser; useful for edge case discovery |
| **gdtf-creator** | TypeScript | AI-assisted GDTF file generation tool |
| **Perastage** | TypeScript | MVR patch workflow and visualization |
| **LX-Net** | TypeScript | GDTF-Share API proxy with caching layer |
| **Depence-dfix** | -- | Legacy format research and conversion analysis |

### Planning Workspace Structure

```
planning/
  external_repos/     # Ecosystem tool sources
    libMVRgdtf/       #   C++ parser reference
    python-mvr/       #   Python MVR manipulation
    zeevonk/          #   Rust patterns reference
    gdtf-creator/     #   AI fixture creation
    Perastage/        #   MVR/patch workflows
    spec/             #   Standards documents
    tools/            #   Misc ecosystem utilities
  private_repos/      # Internal/archived projects
    console-archive/  #   Legacy console code
    LX-Net/           #   GDTF-Share proxy
    Depence-dfix/     #   Format research
  references/         # Curated documentation
  crawled_sites/      # Static site snapshots
```

The planning workspace serves as a knowledge base -- not a build target. It provides
patterns, edge cases, and prior art to inform design decisions.
