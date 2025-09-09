# Orkee Cloud Package

This package provides optional cloud synchronization capabilities for Orkee.

## Overview

The cloud package enables:
- Automatic project backups
- Multi-device synchronization  
- Access your projects from anywhere
- Secure cloud storage

## Requirements

Cloud features require an Orkee Cloud account. Visit [orkee.ai](https://orkee.ai) to learn more.

## Building

```bash
# Build Orkee with cloud features enabled
cargo build --features cloud

# The cloud features are disabled by default
cargo build  # Cloud features NOT included
```

## Usage

Once built with cloud features and authenticated:

```bash
orkee cloud enable   # Enable cloud sync
orkee cloud status   # Check sync status
orkee cloud sync     # Manual sync
```

## License

Part of the Orkee project. See root LICENSE file for details.