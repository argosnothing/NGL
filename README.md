# NGL - Nix Global Lookup

A unified search and aggregation layer for Nix documentation.

## The Problem

Nix documentation is scattered across dozens of sources:
- [noogle.dev](https://noogle.dev) for function documentation
- [search.nixos.org](https://search.nixos.org) for packages and options
- [nixos.org](https://nixos.org) for guides and manuals
- [home-manager](https://nix-community.github.io/home-manager/) documentation
- Various community wikis and resources

The information exists, but finding it means knowing which site to check and learning multiple different interfaces.

Several projects have been built to get data individually from these sources, but they tend to be scoped to the sources to varying degrees of generality, and couple how that data is displayed. 

## The Solution

NGL provides a single search interface that:
- **Queries multiple providers** automatically
- **Normalizes heterogeneous data** into a consistent format
- **Returns aggregated results** from all sources
- **Caches locally** for fast offline access

Search once, get documentation from everywhere.

NGL is simply a library that gives code a single source for a ton of different aggregated nix documentation. 

## Status

Pattern for writing providers is done, Noogle provider is implemented (need minor cleanup on pruning data before db insert), see the [TODO](./TODO.md) for the biggest bounties.


## [Contributing!!!](./CONTRIBUTING.MD)
NGL is written to make adding your own sources a breeze, NGL just needs you to implement "just a few methods" from the Provider trait and you're off to the races!!
The goal of NGL is to be GLOBAL, so this means writing a ton of providers, so having an ergonomic codebase for rapidly writing extraction code to the database has
been paramount.   
In the future I plan on adding functionality to work with different schemas, similarly to nix-search-tv   
[Contributing!!!!](./CONTRIBUTING.MD)  


## License

TBD

NGL 
