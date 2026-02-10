# Notes, Concepts

## Purpose
NGL is first, and foremost, for aggregating *documentation* on *kinds* of data. Very important to focus on this.  
*Functions* in NGL are not meant to represent a function in the sense of a signature to that function, but losely documentation about a function.
This means a *function* kind can include documentation that also has examples, and even options if that is in its documentation.

### Example
Noogle page on a function would end up being indexed as a function, with the pages content.
It would also index each example for that function that is in that same page. This concept extends to *guides* as well, which
themselves can potentially contain documentation on a *function* if the provider is capable of consistently separating out function documentation.

## Providers are bespoke.
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
