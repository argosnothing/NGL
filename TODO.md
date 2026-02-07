## Eval These for potential provider
- https://home-manager-options.extranix.com/
- nix.dev
- https://searchix.ovh/?query=int
## Look through awesome nixos https://github.com/nix-community/awesome-nix?tab=readme-ov-file
## Data storage? format? use db? A: sqlite is in our future.

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

