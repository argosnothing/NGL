# Contributing
Please see the [example provider](./src/providers/example/mod.rs) for a full breakdown of the application lifecycle around making a provider

## Dev environment:  
Nix develop is important, as I provide an env variable for nixpkgs if you end up compiling with that feature. It is by default compiled out on build as it can take time to cache, although i've taken some pains to make it memory efficientish.

Currently the only way to trigger a sync is by sending in a request with the specified types and providers, with None defaulting to all on both. 
cargo run lib.optional 
after you run nix develop should give you a good example of the shape of the schema

Early on, the [schema](./src/schema.rs) is going to be up in the air, as more providers are written, we should iron out what the important data for each kind of documentation is. Feel free to add onto the schema.

## Writing a Provider  
Providers are what connects a source of information (api, website, etc) to the NGL database.

By implementing the `Provider` trait, you will have access to a method that lets you put events in a sync to insert into the NGL database. 

The task of the provider is to: 
1. Decide, and declare what kinds of data it *can* reliably parse into NGLEntityData models.
2. take the raw information from the documentation source, using its own schema.
3. convert the providers local schema into the databases schema for that kind of data

After implementing the provider you will need to add a new instance of the provider to the providers vec in the [registry](./src/registry.rs)

This is all done through implementing the Provider trait on your provider, everything else NGL will handle for you, the NGL provider trait uses async_traits, so i recommend just copying the basic provider template.
