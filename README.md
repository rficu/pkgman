# pkgman - IPFS-based package manager for Linux

pkgman is an IPFS-based package manager that uses public key cryptography to sign packages, such that
they are safe to download from the network, and to slowly propagate trust in the network by signing other
maintainers' public keys. The main idea is to reduce server costs by hosting the packages on IPFS
while providing the end-user the certainty that the package is safe to use.

Please read [this blogpost](https://vizardy.net/blog/ipfs_package_management.html) about this project if you're interested in more details

## Usage of pkgman

`pkgman` is a tool that is used to download signed packages that are hosted on IPFS. This methods
provides security as the default case is that a user must trust only one public and it reduces
server costs significantly as all the packages are distributed from other people instead of from a
central server.

### Initialize

Create `~/.config/pkgman/KEYRING.toml` with the initial node's information and an empty
`PKGLIST.toml` file.

`./pkgman --init`

### Update keyring

Fetch the latest keyring from the network, i.e., all the nodes that are considered trusted and
who's signatures can be considered valid when packages are verified.

`./pkgman --update-keyring`

### Start pkgman in service mode

If you wish to contribute to the network by replying to keyring and package queries, you can run
the pgkman in service mode

`./pkgman --daemon`

### Querying a package

Check whether the network contains a certain package

`./pkgman --query <package name>`

### Downloading a package

Download a package from the network

`./pkgman --download <package name>`

### Updating all packages

Update all packages that are in the system. It is assumed that $HOME/.cache/pkgman/PKGLIST.toml
file exists that contains a list of all packages that the system has.

`./pkgman -- --update`

## Usage of pkgmain

`pkgmain` is a tool for maintainers that add new signed packages to the network and allow new
nodes to become maintainers by distributing their public keys and names in keyring updates.

## Adding new packages

This either updates the version that is currently available or adds a new package based on what 
`~/.config/pkgman/PKGLIST.toml` contains.

```
./pkgmain
    --update-package
    --name clang
    --version "11.1.0"
    --path /usr/bin/clang
    --pkcs8 /home/rficu/.config/pkgman/pkcs8
```

## Adding new maintainers

```
./pkgmain
    --update-keyring
    --name rficu
    --email "rficu@email.com"
    --public-key "3c2PgNisX4vOumXAYVETS1aDKLHYEuhKSo7i1xnwr2Y="
    --pkcs8 /home/rficu/.config/pkgman/pkcs8
```

## Copying

Public domain
