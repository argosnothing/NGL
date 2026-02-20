# Nix Global Lookup (NGL)

A unified search and aggregation layer for Nix documentation.

https://github.com/user-attachments/assets/e4e666a5-ca3d-4cb4-b3aa-5d53fbf80bbe

---

## Overview

Nix documentation is spread across many different sites and tools. Finding information often requires knowing **where** to look and learning multiple search interfaces.

NGL provides a single API and query interface that aggregates documentation from multiple sources and returns it in a consistent format.

---

## The Problem

Documentation is fragmented:

* https://noogle.dev — function documentation
* https://search.nixos.org — packages and options
* https://nixos.org — manuals and guides
* https://nix-community.github.io/home-manager — Home Manager documentation
* community wikis and project docs

Several tools exist to query individual sources, but they are tightly coupled to their data source and presentation.

---

## The Solution

NGL pulls, normalizes, caches, and queries documentation from multiple providers.

Two core concepts:

### Kinds

Kinds describe the **type of documentation** returned:

* `packages`
* `functions`
* `options`
* `guides`
* `examples`

A single query can return results across multiple kinds.

---

### Providers

Providers describe **where the data comes from**.

Queries can be filtered by provider. NGL only syncs data from providers relevant to the query, keeping indexing minimal and fast.

Examples:

* `noogle`
* `nixpkgs`
* `home-manager`
* `plasma-manager`
* `hjem`
* `nvf`

---

## Example Query

```bash
cargo run -- \
  --providers nixpkgs,noogle,hjem,nvf \
  --kinds function,example,package \
  lib.optional
```

On a fresh install, this will:

1. Pull documentation from the requested providers
2. Cache the data locally
3. Query for `lib.optional`

---

## Example Response (trimmed)

```json
[
  {
    "provider_name": "noogle",
    "matches": [
      {
        "data": {
          "Function": {
            "name": "lib.optional",
            "signature": "optional :: bool -> a -> [a]",
            "source_url": "https://noogle.dev/f/lib/optional",
            "aliases": ["lib.lists.optional"]
          }
        }
      }
    ]
  }
]
```

Results are grouped by provider, but share a consistent structure so consumers can remain source-agnostic.

---

## Current Providers

### Native

* Noogle → functions, examples
* Nixpkgs → packages
* Home Manager → options, examples
* Plasma Manager → options, examples

### Template-based (scraped)

* hjem
* nvf

Template providers allow adding new sources by editing:

```
templates.json
```

---

## Guides (Planned)

Guide support requires modeling relationships between guides and subguides.

Planned behavior:

* index guide and subguide titles
* avoid full-text indexing entire manuals
* expose structured navigation

---

## Data Format

Parsed HTML sources are converted to **Markdown** for consistent formatting.

---

## Use Cases

NGL is designed as a foundation layer for:

* CLI search tools
* TUIs
* editor integrations
* language server features
* documentation browsers
* custom nix tooling

Example: a Noogle-style TUI that only queries function providers without indexing nixpkgs.

---

## Contributing

NGL is designed to make adding providers straightforward.

Implement the required methods from the `Provider` trait and the data can be ingested and indexed.

The goal is to support the entire Nix ecosystem.

See: [contributing](./CONTRIBUTING.md)
---

## License

TBD

---

<sub>connect all the dots</sub>
