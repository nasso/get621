# get621

[![build](https://github.com/nasso/get621/actions/workflows/rust.yml/badge.svg)](https://github.com/nasso/get621/actions/workflows/rust.yml)
[![Telegram](https://img.shields.io/badge/Telegram-Join%20Chat-blue.svg)](https://t.me/rs621)

Command line tool for [e621.net](https://e926.net), written in Rust.

## Features

- Regular tag searching, using any of the search options from the website.
- Pool bulk downloading.
- Post downloading.
- Parents/children posts fetching.
- Posts/pools bulk downloading.
- Unlimited result count (automatically splits into multiple API requests).
- Reverse image searching (experimental).
- Various output modes:
  - "verbose" (artist, id, tags, description...).
  - "raw" (posts are downloaded to the standard output).
  - "id" (post IDs are printed to the standard output).

_Note: there can be up to 6 tags at once. Trying to search for more will cause a
422 "Unprocessable entity" HTTP error. This is an API limitation._

## Usage

### Search for posts

#### Single post:

```sh
get621 asriel_dreemurr order:score
```

#### Multiple posts (here, 5):

```sh
get621 asriel_dreemurr order:score --limit 5
```

_Note: `--limit` can be replaced with `-l`._

#### Blacklist tags:

```sh
get621 asriel_dreemurr order:score -- -solo -chicken
```

_Note: Since the syntax to blacklist a tag uses a dash, it must be placed after
two dashes `--` to make the difference between a blacklisted tag and a command
option/flag (such as `-l` or `--limit`). As a result, anything after `--` will
be treated as a tag for the request._

### Saving posts

This will download posts to the current working directory as `<id>.<ext>`.

**_This will overwrite any file with the same name in the current working
directory, without any warning._**

#### Single post:

```sh
get621 --save asriel_dreemurr order:score
```

#### Multiple posts (here, 10):

```sh
get621 --save asriel_dreemurr order:score --limit 10
```

_Note: `--save` can be replaced with `-s`._

### Bulk saving pools

This will download posts to the current working directory as
`<pool_id>-<page>_<post_id>.<ext>`.

**_This will overwrite any file with the same name in the current working
directory, without any warning._**

```sh
get621 --pool <pool_id> --save
```

_Note: `--pool` can be replaced with `-P`._

### Reverse search images

```sh
get621 reverse path/to/image1.png /another/image2.gif ./glob/**/pattern/*.jpg
```

_Note: `-s` or `--save` can be used to download posts to the current working
directory._

## Building

1. [Install rust](https://rustup.rs) if you don't have it already.
2. Clone the repository:

   ```sh
   git clone https://github.com/nasso/get621.git
   cd get621
   ```

3. Use Cargo to build get621:

   - For debug builds:

     ```sh
     cargo build
     ```

   - For release builds:

     ```sh
     cargo build --release
     ```

_Note: Linux users will probably need to have OpenSSL installed with the
development packages to compile the `openssl-sys` crate (e.g. `libssl-dev` on
Ubuntu and `openssl-devel` on Fedora). `pkg-config` is also required when
targeting Linux._

## License

`get621` is licensed under the terms of either the MIT license or the Apache
License (Version 2.0), at your choice.

See [LICENSE-MIT] and [LICENSE-APACHE-2.0].

[license-mit]: https://github.com/nasso/get621/blob/master/LICENSE-MIT
[license-apache-2.0]:
  https://github.com/nasso/get621/blob/master/LICENSE-APACHE-2.0
