# TODO
Feel free to open PR's for any of these todos

1. Implement Providers
2. Data querying. NGL is going to likely contain a massive amount of data, meaning that data needs some way of being sorted, special indexing, etc.
   A NGLRequest should likely include with it some number that represents what section of data it is trying to get and it's own data limit for entries.
   Say you make a query to NGL with no filters, you are querying the entire db. This should not be done as a single response, so we would need that
   NGLRequests to have an index for what part of the response it is trying to get.
   Meaning the retrieval should only send a bit of that data. 
3. Implement Meta Providers that makes it easier to work with similar kinds of sources(?)
   Got this idea looking through nix-search-tv docu on an experimental feature they have under the experimental option.
   This could also take the form of having a kind of meta provider that reads a config file for urls that have that meta provider as it's provider
   Say we have a format that we know several sources take for the shape of their data, we would add a config file that connects to some
    ```json
     {
        provider_name: "github",
        provider_kinds: ["example"],
        provider_source: "github:argos_nothing/nixos-config"
        provider_licenses: ["some license that we should respect :)"]
     }
    ```
   that those providers can get their provider information from for each found instance. 
   would use, so for each provider in that config, we'd instantiate a new provider to deal with that data.
4. Either add a CLAP cli in the library (ideal for testing) or work on another cli frontend for easier examples on query NGL
5. Implement a NGL frontend, although this would likely be a diff repo.
   
