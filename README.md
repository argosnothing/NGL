# NGL - Nix Global Lookup

A unified search and aggregation layer for Nix documentation.

## The Problem  
Nix documentation is scattered across dozens of sources:
- [noogle.dev](https://noogle.dev) for function documentation
- [search.nixos.org](https://search.nixos.org) for packages and options
- [nixos.org](https://nixos.org) for guides and manuals
- [home-manager](https://nix-community.github.io/home-manager/) documentation
- many, many various community wikis and resources

The information exists, but finding it means knowing which site to check and learning multiple different interfaces.

Several projects have been built to get data individually from these sources, but they tend to be scoped to the sources to varying degrees of generality, and couple how that data is displayed. 

## The Solution


NGL provides a single search interface that:
- **Queries multiple providers** automatically
- **Normalizes heterogeneous data** into a consistent format
- **Returns aggregated results** from all sources
- **Caches locally** for fast offline access

Search once, get documentation from everywhere.

NGL is **NOT** a solution providing its own frontend.  
It wants to be used in **YOUR** nix related documentation project.   
### NGL emphasizes control over  
- What *kind* of data gets synced
- What providers (sources of data) you want to deal with. ( including their dependencies for feature flags)  
All while letting you focus on your website/api/tui-app instead of document fetching, databases, caching, and whatever nonsense I had to do for the nixpkgs provider to not destroy your ram!   

This means, you don't need to worry about additional bloat from data you don't care about using, only the data you want, from the sources you want for the things you want. 

## Status

Many providers are implemented, the major missing ones are nixos-manual for our first guide providers, but we have bunches currently:
- Noogle for functions and examples
- Nixpkgs for packages
- hjem for options and examples ( web scrapped currently )
- nvf for options and examples ( web scrapped currently )

The API for responses is still up in the air, and i'm potentially relying on potential consumers of this API to tell me the data they care about for each kind that NGL offers. 

Currently the language i'm going with is markdown for formatted data, this means the current html parses will provide you markdown data.


## [Contributing!!!](./CONTRIBUTING.MD)
Still in early days, **BUT**   

NGL is written to make adding your own sources a breeze, NGL just needs you to implement "just a few methods" from the Provider trait and you're off to the races!!
The goal of NGL is to be GLOBAL, so this means writing a ton of providers, so having an ergonomic codebase for rapidly writing extraction code to the database has
been paramount.   
In the future I plan on adding functionality to work with different schemas, similarly to nix-search-tv   
[Contributing!!!!](./CONTRIBUTING.MD)  


## License

TBD

NGL 
