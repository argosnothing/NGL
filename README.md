# Nix Global Lookup 
A unified search and aggregation layer for Nix documentation.  


https://github.com/user-attachments/assets/e4e666a5-ca3d-4cb4-b3aa-5d53fbf80bbe


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

NGL is a library that both pulls, and caches nix documentation data from many different sources.
Two key concepts of ngl are `kinds` of data and `providers` of that data. 
`kinds` are `packages`, `functions`, `options`, `guides` and `examples` and form the kind of data NGL will give you when you query it.
In NGL a single query can potentially give you matches from various kinds of documentation if you need it to. 
`providers` are ngl's way of splitting sources of information, and similarly to `kinds` you can make queries filtered by the `providers` you care about. 
One key concept of NGL is if you make a query for a specific kind of data, or from a specific provider, NGL will only ever manage and sync from those sources.
So while it is a unified lookup, it is also lazy in what data it manages. If you want to write a noogle.dev tui, you don't care about indexing nixpkgs, after all. 

Here is an example NGL call:
`cargo run -- --providers nixpkgs,noogle,hjem,nvf --kinds function,example,package lib.optional`
On a fresh install with no database this with asynchronously pull data from those providers, with those kinds of data, and then query for lib.optional. 
In the cli it returns something like this:
```
[
  {
    "provider_name": "noogle",
    "matches": [
      {
        "data": {
          "Function": {
            "name": "lib.optional",
            "signature": "optional :: bool -> a -> [a]\n",
            "content": {
              "Markdown": "\nReturn a singleton list or an empty list, depending on a boolean\nvalue.  Useful when building lists with optional elements\n(e.g. `++ optional (system == \"i686-linux\") firefox`).\n\n# Inputs\n\n`cond`\n\n: 1\\. Function argument\n\n`elem`\n\n: 2\\. Function argument\n\n# Type\n\n```\noptional :: bool -> a -> [a]\n```\n\n# Examples\n:::{.example}\n## `lib.lists.optional` usage example\n\n```nix\noptional true \"foo\"\n=> [ \"foo\" ]\noptional false \"foo\"\n=> [ ]\n```\n\n:::\n"
            },
            "source_url": "https://noogle.dev/f/lib/optional",
            "source_code_url": "https://github.com/NixOS/nixpkgs/blob/dfcc8a7bfb5b581331aeb110204076188636c7a2//nix/store/20nxy7dhnm964yl154v1vmgblchqmxwm-source/lib/default.nix#L280",
            "aliases": [
              "lib.lists.optional"
            ]
          }
        }
      },
    ]
  },
  {
    "provider_name": "hjem",
    "matches": [{
      //... match data and so on
    }]
  }
]
```
Notice the shape of the data and how its organized by provider. The idea of NGL is that to the consumer of NGL the source of the data should be agnostic to the format you receive that data in.
In otherwords NGL's goal is to give you a single api for interacting with data that might be formatted differently in the source.

## Status

Many providers are implemented, the major missing ones are nixos-manual for our first guide providers, but we have bunches currently:
- Noogle for functions and examples
- Nixpkgs for packages
- home manager for options and examples
- plasma manager for options and examples
- hjem for options and examples ( web scrapped currently )
- nvf for options and examples ( web scrapped currently )
- Two template providers that generate the 4 providers above  
  Any page that looks like the data from these last 4 sources can be added simply by
  modifying [templates.json](./templates.json)

- TODO: Guides will require a rework, as we need to express the relationship between a guide and its subguide, I don't want fts5 indexing on the entire content of a guide, but simply the guide, and subguide titles. 
The API for responses is still up in the air, and i'm currently relying on potential consumers of this API to tell me the data they care about for each kind that NGL offers. 

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

<sub>connect all the dots</sub>
