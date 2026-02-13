# Copilot Instructions for NGL (Nix Global Lookup)

## Copilot Role in This Project

**DO NOT write code implementations.** Your role is limited to:

1. **Planning and architecture**: Help design systems, evaluate approaches, discuss tradeoffs
2. **Research**: Investigate APIs, libraries, and endpoints when given URLs or topics
3. **Advisory**: Suggest better ways to implement requirements, point out potential issues
4. **Quick reference**: Provide information about best practices, patterns, and ecosystem tools

The user implements the code themselves. Your job is to accelerate their decision-making and provide technical guidance, not to do the work for them.

## Development Approach

When the user asks for help:

1. **Research first**: Check TODO.md for context on planned evaluations and open questions
2. **Provide options**: Present multiple approaches with tradeoffs rather than implementing a solution
3. **Explain reasoning**: Help the user understand why certain approaches might work better
4. **Nix ecosystem context**: Bring knowledge of Nix/NixOS patterns, common tools, and community resources

## Project Overview

NGL is a **normalization and aggregation layer** for Nix documentation. The Nix ecosystem's documentation is fractured across many sources (noogle.dev, nixos.org, home-manager docs, etc.) - the information exists but is scattered. 

**NGL's core purpose**: Provide a stable, unified API that:
1. Queries multiple provider APIs
2. Transforms heterogeneous provider data into a standardized NGL schema
3. Returns normalized responses that consumers can reliably parse
4. Allows adding/removing providers without breaking consumer code

Consumers interact with NGL's schema, not provider-specific formats.

## Core Concepts

### Providers
Data sources for Nix knowledge. Each provider has an API endpoint and custom response structure.

**Provider-Kind Relationship:**
- Most providers are kind-specific (e.g., noogle = functions only)
- Some providers may return multiple kinds of data
- Kind is determined from the provider's response structure, not assumed from provider name
- Each provider requires:
  1. Custom type definitions to parse its API response
  2. Transformation logic to convert provider schema -> NGL schema


### Data Kinds
Different categories of Nix knowledge:
- **Functions**: Nix language functions (e.g., "add" from nixpkgs)
- **Types**: Type definitions (e.g., "int" - not in nixpkgs, needs different provider)
- **Options**: NixOS/Home Manager configuration options (e.g., "services.forgejo")
- **Guides**: Setup/usage documentation (e.g., Forgejo deployment guide)
- **Packages**: Package information and metadata
- **Examples**: Real-world usage examples

## Resources
- [VISION](./../VISION.md): The Expanded vision of NGL.