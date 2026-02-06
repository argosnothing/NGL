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

## The Solution

NGL provides a single search interface that:
- **Queries multiple providers** automatically
- **Normalizes heterogeneous data** into a consistent format
- **Returns aggregated results** from all sources
- **Caches locally** for fast offline access

Search once, get documentation from everywhere.

## Status

Early development. Architecture and provider evaluation phase.

See `.github/copilot-instructions.md` for detailed design documentation.

## Example

When you search "mkEnableOption":
- Noogle provides function signature and examples
- NixOS options shows where it's used
- Guides show practical applications

All in one normalized response.

## License

TBD

NGL 
