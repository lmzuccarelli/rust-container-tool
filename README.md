## Overview

This is a simple POC that copies dockerv2 images (from a registry) to disk

## Use Case

A typical use case could be:
- Work with catalog i.e registry.redhat.io/redhat/redhat-operator-index:v4.11
- Copy to local directory (Catalog and related images)
- Make changes locally
- Push to a registry

## POC 

I used a simple approach - Occam's razor

- A scientific and philosophical rule that entities should not be multiplied unnecessarily (KISS)
- Worked with a v2 images for the POC
- Add the catalog and relevant images in a catalog later


## Usage

Clone this repo

Ensure that you have the correct permissions set in the $XDG_RUNTIME_DIR/containers/auth.json file

Execute the following to copy from a registry

```bash
mkdir -p working-dir/rhopi/blobs/sha256
cargo build 
cargo run
```

