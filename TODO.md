## Eval These for potential provider
- https://home-manager-options.extranix.com/
- nix.dev
- https://searchix.ovh/?query=int
## Look through awesome nixos https://github.com/nix-community/awesome-nix?tab=readme-ov-file
## Data storage? format? use db? A: sqlite is in our future.

# Notes, Concepts

## Purpose
NGL is first, and formost, for aggregating *documentation* on *kinds* of data. Very important to focus on this.  
*Functions* in NGL are not meant to represent a function in the sense of a signature to that function, but losely documentation about a function.
This means a *function* kind can include documentation that also has examples, and even options if that is in its documentation.

### Example
Noogle page on a function would end up being indexed as a function, with the pages content.
It would also index each example for that function that is in that same page. This concept extends to *guides* as well, which
themselves can potentially contain documentation on a *function* if the provider is capable of consistently separating out function documentation.

## Provider are bespoke.
Good nix documentation is spread out in different formats, different sites, different apis, a provider is a module in NGL that manages those sources, this includes all logic
that takes that documentation and atomizes it in a way the NGL can index and make sense of, in any method possible. If a provider fails to update the db when it needs to update, it should
do so in a way that lets the rest of the application function.

## Providers are built understanding the limitations of their sources
Because data is structured different on different sites, a provider needs to be bespoke in how it handles that data. 
The logic will naturally be *tailored* for that source. Even with this in mind It's unlikely a provider will be able work with every *kind* of data in a consistent manner.

For example, nixpkgs manual as many guides about packages, and some of those packages talk about functions and use functions, but the structure of the github that i'd likely be using to source that data, doesn't provide any method to accurately capture where a function is being described.
For noogle this isn't the case, as noogle's very purpose is to be a search engine for nix functions defined in nixpkgs, so we would consider the *provider*, *noogle* to supply the kinds: *functions* and *examples*, but not other ones such as *guides*, like we can do with the nixpkgs manual.

so the tl;dr every source will have its own challenges for accurately atomizing its data into chunks of different kinds. There might even be sources that fail to do every kind all toghether, in which they would not be a candidate for having a provider in NGL.

## Elevator pitch
NGL is a pragmatic solution to Nix documentation fragmentation. It's not trying to be a central source for all your documentation in the literal sense, because that documentation already exists, but it's to allow applications to utilize that documentation through a singular normalized interface for various purposes. 


## Focus on
Noogle is the idea first provider to implement as it exposes a singular endpoint, i have experience from my last project,
and the data is regular json that is fairly simple to map.  
1. Noogle! 
2. Get the pull from the singular endpoint! This will require mapping structs onto json ( provider level structs to receive direct )
3. convert into domain data (like in noogle-search, but with a structure that can be generalized to other function providers).
4. mssql? orm?? some way to map domain data to and from a db in some clean/efficient way
5. the db is our cache, why did i make two folders for that??
6. The big question. We have guides, that have examples, but guides and examples should be different *kinds*
  * Do we:
    1. Simply duplicate the data. Parse the entire guide, include examples ( code blocks ), and also reparse those same guides for examples isolated.
    2. Create some way that a guide can reference the example so we avoid duplicating data. ( storage efficient ) worse querying time.
7. Data retrieval to produce NGLResponses
8. loop back on the greater schema, what things have we learned, there will likely be major changes that need to happen or have already happened by this point.
9. Next provider.

