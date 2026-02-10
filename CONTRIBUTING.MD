# Contributing

Early on, the [schema](./src/schema.rs) is going to be up in the air, as more providers are written, we should iron out what the important data for each kind of documentation is. Feel free to add onto the schema.


## Writing a Provider  
Providers are what connects a source of information (api, website, etc) to the NGL database.

The task of the provider is to: 
1. Decide, and declare what kinds of data it *can* reliably parse into NGLEntityData models.
2. take the raw information from the documentation source, using its own schema.
3. Use the current request to decide *what* information
3. convert the providers local schema into the databases schema for that kind of data

This is all done through implementing the Provider trait on your provider, everything else NGL will handle for you. 
