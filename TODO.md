# TODO
Feel free to open PR's for any of these todos

1. Implement Providers
2. Implement Meta Providers that makes it easier to work with similar kinds of sources(?)
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
3. Either add a CLAP cli in the library (ideal for testing) or work on another cli frontend for easier examples on query NGL
4. Implement a NGL frontend, although this would likely be a diff repo.
   
