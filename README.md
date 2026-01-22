<h1 align="center">
    paramspider
  <br>
</h1>

<h4 align="center">Mining URLs from dark corners of Web Archives for bug hunting, fuzzing, and further probing</h4>

<p align="center">
  <a href="#about">About</a> •
  <a href="#status">Status</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#examples">Examples</a>
</p>

## About

`paramspider` fetches archived URLs for a domain (or list of domains) from the Wayback Machine, filters out non-relevant assets by extension, and normalizes query parameters to a placeholder for fuzzing or further analysis.

## Status

This repository is a Rust rewrite and is not a drop-in source tree match for the original Python project. The CLI behavior is intended to be compatible with the original.

## Installation

Build from source with Cargo:

```sh
cargo build --release
```

Run the binary directly:

```sh
./target/release/paramspider -d example.com
```

Or install locally:

```sh
cargo install --path .
```

Docker:

```sh
docker build -t paramspider-rs .
docker run --rm paramspider-rs -d example.com
```

## Usage

```sh
paramspider -d example.com
```

## Examples

- Discover URLs for a single domain:

  ```sh
  paramspider -d example.com
  ```

- Discover URLs for multiple domains from a file:

  ```sh
  paramspider -l domains.txt
  ```

- Stream URLs to the terminal:

  ```sh
  paramspider -d example.com -s
  ```

- Set up a web request proxy:

  ```sh
  paramspider -d example.com --proxy '127.0.0.1:7890'
  ```

- Add a placeholder for URL parameter values (default: "FUZZ"):

  ```sh
  paramspider -d example.com -p '"><h1>reflection</h1>'
  ```
