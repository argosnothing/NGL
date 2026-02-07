# NGL Architecture Discussion - Session Summary

## Project Vision

NGL (Nix Global Lookup) is a **normalization and aggregation layer** for scattered Nix documentation.

**Core Purpose:**
- Query multiple provider APIs (noogle.dev, home-manager docs, nixos.org, etc.)
- Transform heterogeneous provider data into standardized NGL schema
- Return normalized responses consumers can reliably parse
- Allow adding/removing providers without breaking consumer code

**Key Principle:** Provider-agnostic format. Consumers never see provider-specific structures.

## Architecture Decisions

### Query Flow
1. **First run:** Query all provider APIs, cache complete datasets locally
2. **Subsequent queries:** Search local cache only (no network calls)
3. **Periodic refresh:** Background updates with `last_updated` tracking per provider

### Provider Design
- **Interface/Trait Pattern:** Each provider implements common interface
  - How to fetch data
  - How to transform response → NGL schema
  - Metadata (name, TTL, etc.)

- **Provider-Kind Relationship:**
  - Most providers are kind-specific (e.g., noogle = functions only)
  - Some may return multiple kinds
  - Kind determined from provider's response structure, not provider name
  
- **Provider Requirements:**
  - Must provide API endpoints (no web scraping)
  - Custom type definitions to parse API response
  - Transformation logic: provider schema → NGL schema

### Resilience Strategy
- **Provider failures must not break NGL**
- Failed providers silently omitted from results
- **Debug mode:** Show which providers failed and why (via tracing logs)
- Normal mode: Clean consumer experience

### Storage Strategy
- Cache entire provider datasets locally
- SQLite with FTS5 for optimized local search
- Store heterogeneous data (different schemas per provider)
- Track: provider, kind, last_updated

### Response Format

NGL returns normalized schema (provider-agnostic):

```json
[
  {
    "provider": "noogle",
    "data": {
      "Guide": null,
      "Example Code": "...",
      "Options": null
    }
  },
  {
    "provider": "another-provider",
    "data": {
      "Guide": "...",
      "Example Code": null,
      "Options": [...]
    }
  }
]
```

**NGL Schema fields (TBD):**
- Guide (documentation/tutorials)
- Example Code (usage examples)
- Options (configuration options)
- Others based on provider evaluation

## Example Query Scenarios

**"add"**
- Noogle provides function documentation
- Response includes signature, description, examples

**"int"**
- Noogle doesn't have it (builtin type, not in nixpkgs)
- Type-specialized provider supplies type information

**"forgejo"**
- NixOS manual: setup guide
- Options provider: `services.forgejo.*` configuration
- Package provider: package metadata
- All combined in normalized response

## Technology Stack

### Language: Rust
- Already familiar (noogle-search-tv project)
- Strong typing for schema transformations
- Proven patterns: HTTP + caching + Nix flakes

### Core Dependencies
From noogle-search-tv:
- `reqwest` - HTTP client
- `serde`/`serde_json` - JSON parsing
- `anyhow` - error handling
- `chrono` - timestamps
- `clap` - CLI args

New for NGL:
- `rusqlite` - SQLite with FTS5 (full-text search)
- `tracing` + `tracing-subscriber` - debug logging

### Known Provider: Noogle
- **API:** `https://noogle.dev/api/v1/data`
- **Returns:** Complete dataset (all functions at once)
- **Structure:** Functions + builtin types + repo metadata
- **Rich metadata:** Signatures, positions, examples, aliases
- **Reference:** `../noogle-search-tv/src/data.rs` for response types

## Data Kinds

Different categories of Nix knowledge:
- **Functions:** Nix language functions (e.g., "add" from nixpkgs)
- **Types:** Type definitions (e.g., "int")
- **Options:** NixOS/Home Manager config options
- **Guides:** Setup/usage documentation
- **Packages:** Package information and metadata
- **Examples:** Real-world usage examples

## Open Questions / To Evaluate

1. **NGL Schema:** What specific fields should it have?
2. **Provider candidates:**
   - home-manager-options.extranix.com
   - nix.dev
   - searchix.ovh
   - Others from awesome-nix
3. **Provider interface methods:** Exact trait definition
4. **Async vs blocking:** Multi-provider fetching strategy

## Repository Status

- `flake.nix` created (Rust dev environment)
- `.envrc` created (direnv integration)
- `.gitignore` created
- `.github/copilot-instructions.md` created with full context
- Ready for implementation phase

## Example Extraction Strategy

**For providers that return guides with embedded code examples (e.g., markdown docs):**

- **Dual storage approach**: Accept small duplication for query flexibility
  1. Store guide with full content (examples embedded in markdown)
  2. Extract and store separate Example matches with source metadata
  
**Example match structure:**
```rust
{
  "kind": "Example",
  "data": {
    "code": "nix run nixpkgs#darwin.linux-builder",
    "language": "ShellSession",
    "source": {
      "kind": "Guide",
      "title": "darwin.linux-builder",
      "provider": "nixos-org-manual",
      // section/heading context
    }
  }
}
```

**Benefits:**
- `ngl --kind example --term forgejo` returns just examples with origin traced
- Searching guides returns full context (placement preserved)
- FTS5 indexes both, fast search either way
- Small duplication (example text) traded for query flexibility

## User's Role Preference

**Copilot should NOT write code implementations.** Role is:
- Planning and architecture
- Research (APIs, libraries, endpoints)
- Advisory (suggest approaches, discuss tradeoffs)
- Quick reference (best practices, patterns)

User implements code themselves.
