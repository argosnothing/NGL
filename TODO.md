# TODO
Feel free to open PR's for any of these todos


In no particular order...  
- Implement Providers (Global!!! Meaning we need as much data as possible!!!! MORE MORE MORE)
- Config file
- Modularity... Currently NGL is one crate, maybe this is how we'll do it, but one idea would be to investigate ways to decouple providers from
  NGL code they should not care about. For example, instead of a ProviderEvent wrapping a seo_orm model, wrap a publically facing Domain model that NGL will then map onto the sea_orm model. 
  - In line with this investigate ways to have NGL providers as separate plugins, crates, somehow. They would have NGL as a dependency to manage their state maybe?? For example, lets say you only care about the noogle provider. NGL currently only syncs kinds of data you care about, but the over providers and their own dependencies are in the same crate, could there be a flag we could run to say we want "everything" or we want xyz provider, and it'd only compile code that that provider/s uses?
- Implement Meta Providers that makes it easier to work with similar kinds of sources
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
- Schema configs. Having some way to rapidly write out mappings of json key -> vals to NGL data structures and generate new meta providers
  Could be very powerful for covering a ton of similar data, like blogs, etc. 
- Either add a CLAP cli in the library (ideal for testing) or work on another cli frontend for easier examples on query NGL
- Implement a NGL frontend, although this would likely be a diff repo.
   
