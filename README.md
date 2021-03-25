# pkgman - IPFS-based package manager for Linux

pkgman is a package manager written in Rust that uses IPFS to download the packages from other
users instead from a sever in an attempt to reduce server costs.

## Start a bootstrap node

`cargo run -- --bootstrap`

## Querying a package

Check whether the network contains a certain package

`cargo run -- --query <package name>`

## Downloading a package

Download a package from the network

`cargo run -- --download <package name>`

## Updating all packages

Update all packages that are in the system. It is assumed that $HOME/.cache/pkgman/PKGLIST.toml
file exists that contains a list of all packages that the system has.

`cargo run -- --update`

## Adding new packages

`cargo run -- --add <path to PKGINFO.toml>`

## Copying

Public domain
