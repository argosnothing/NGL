# Copilot Instructions for NGL (Nix Global Lookup)

## Project Overview

NGL is a **normalization and aggregation layer** for Nix documentation. The Nix ecosystem's documentation is fractured across many sources (noogle.dev, nixos.org, home-manager docs, etc.) - the information exists but is scattered. 

**NGL's core purpose**: Provide a stable, unified API that:
1. Queries multiple provider APIs
2. Transforms heterogeneous provider data into a standardized NGL schema
3. Returns normalized responses that consumers can reliably parse
4. Allows adding/removing providers without breaking consumer code

Consumers interact with NGL's schema, not provider-specific formats.

## Core Concepts

### Providers
Data sources for Nix knowledge. Each provider has an API endpoint and custom response structure.

**Provider-Kind Relationship:**
- Most providers are kind-specific (e.g., noogle = functions only)
- Some providers may return multiple kinds of data
- Kind is determined from the provider's response structure, not assumed from provider name
- Each provider requires:
  1. Custom type definitions to parse its API response
  2. Transformation logic to convert provider schema -> NGL schema

**Candidate providers:**
- **noogle.dev**: Functions (confirmed kind-specific)
  - API: `https://noogle.dev/api/v1/data`
  - Returns complete dataset: all functions, signatures, docs, examples
  - See `../noogle-search-tv` for working implementation
- **home-manager-options.extranix.com**: Options (to evaluate)
- **nixos.org**: Guides/docs (to evaluate)
- **searchix.ovh**: (to evaluate)
- Others in TODO.md

### Data Kinds
Different categories of Nix knowledge:
- **Functions**: Nix language functions (e.g., "add" from nixpkgs)
- **Types**: Type definitions (e.g., "int" - not in nixpkgs, needs different provider)
- **Options**: NixOS/Home Manager configuration options (e.g., "services.forgejo")
- **Guides**: Setup/usage documentation (e.g., Forgejo deployment guide)
- **Packages**: Package information and metadata
- **Examples**: Real-world usage examples

### Query Flow
1. User searches for term (e.g., "add", "int", "forgejo")
2. NGL determines relevant data kinds for the query
3. Routes requests to appropriate providers
4. Composes unified response from multiple sources
5. Caches results with timestamps

### Example Queries

**"add"**
- Noogle provides function documentation
- Response includes signature, description, examples

**"int"**
- Noogle doesn't have it (builtin type, not in nixpkgs)
- Type-specialized provider supplies type information

**"forgejo"**
- NixOS manual provides setup guide
- Options provider gives `services.forgejo.*` configuration
- Package provider gives package metadata
- Unified response combines all three

### Architecture

### Query Strategy
- **First run**: Query all provider APIs and cache complete datasets locally
- **Subsequent queries**: Search against local cache only (no network calls)
- **Refresh**: Periodic background updates with `last_updated` tracking per provider

### Provider Resilience
- **Provider failures should not break NGL**
- Failed providers (API changes, parse errors, network issues) are silently omitted from results
- Debug mode: Enable verbose logging to see which providers failed and why
- Goal: Adding/removing/breaking providers doesn't impact consumer code

### Provider Requirements
- Must provide API endpoints or structured data via API (avoid HTML scraping)
- Markdown via API is acceptable (e.g., GitHub raw content)
- Each provider will need custom logic for its response structure
- Only API-accessible data sources are candidates

### Storage Strategy
- Cache entire provider datasets locally
- Optimize for fast local lookup/search
- Store heterogeneous data (different schemas per provider)
- Track metadata: provider, kind, last_updated

### Response Format

**Structure:**
```rust
pub struct NGLResponse {
    pub provider: String,
    pub matches: Vec<Match>,
}

pub struct Match {
    pub kind: DataKind,
    pub data: MatchData, // kind-specific structured data
}
```

**Example response:**
```json
[
  {
    "provider": "noogle",
    "matches": [
      {
        "kind": "Function",
        "data": { /* function-specific fields */ }
      }
    ]
  },
  {
    "provider": "nixos-org-manual",
    "matches": [
      {
        "kind": "Guide",
        "data": { 
          "title": "darwin.linux-builder",
          "content": "...",
          "embedded_examples": [...]
        }
      }
    ]
  }
]
```

**Key principles:**
- One `NGLResponse` per provider
- Each response contains multiple `matches` (results from that provider)
- Each match has a `kind` (Function, Guide, ConfigOption, etc.) and kind-specific `data`
- **No data duplication**: Examples embedded in guides stay part of guide data (placement matters)
- Consumers interact with standardized Match structure, not provider-specific formats

**DataKind categories:**
- **Function**: Nix language functions with signatures, descriptions
- **Type**: Type definitions
- **ConfigOption**: NixOS/Home Manager configuration options
- **Guide**: Documentation/tutorials with embedded examples
- **Package**: Package information and metadata
- **Example**: Standalone code examples (not part of guides)

## Technology Stack

Considering **Rust** based on existing noogle-search-tv project:
- Familiar ecosystem (reqwest, serde, rusqlite)
- Strong typing for provider response schemas
- Proven pattern: HTTP client + caching + Nix flake distribution

## Project Status

Early-stage. Current evaluation phase:

- **Provider evaluation**: Testing candidate API endpoints and data quality
- **Storage decision**: Leaning toward SQLite with FTS5 for local search
- **Provider gaps**: Identifying what data kinds need new providers (e.g., builtin types)
- **Provider schemas**: Each provider needs custom type definitions for its response structure

See [awesome-nix](https://github.com/nix-community/awesome-nix) for ecosystem resources.

## Copilot Role in This Project

**DO NOT write code implementations.** Your role is limited to:

1. **Planning and architecture**: Help design systems, evaluate approaches, discuss tradeoffs
2. **Research**: Investigate APIs, libraries, and endpoints when given URLs or topics
3. **Advisory**: Suggest better ways to implement requirements, point out potential issues
4. **Quick reference**: Provide information about best practices, patterns, and ecosystem tools

The user implements the code themselves. Your job is to accelerate their decision-making and provide technical guidance, not to do the work for them.

## Development Approach

When the user asks for help:

1. **Research first**: Check TODO.md for context on planned evaluations and open questions
2. **Provide options**: Present multiple approaches with tradeoffs rather than implementing a solution
3. **Explain reasoning**: Help the user understand why certain approaches might work better
4. **Nix ecosystem context**: Bring knowledge of Nix/NixOS patterns, common tools, and community resources

## Resources

- TODO.md tracks evaluation tasks and open questions
- [Awesome Nix](https://github.com/nix-community/awesome-nix) community resources
