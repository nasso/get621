# get621
Command line tool for [e621.net](https://e926.net), purely written in Rust.

## Features

* Regular tag searching, using any of the search options from the website.
* Pool searching.
* Post(s) downlading.
* Parents/children posts fetching.
* Posts/pools bulk downloading.
* Unlimited result count (by-passes the API's limit with multiple requests).
* Reverse image searching (using [iqdb.harry.lu](http://iqdb.harry.lu)).
* Various output modes:
    - "verbose" (artist, id, tags, description...).
    - "raw" (posts are downloaded to the standard output).
    - "json" (posts are printed as a JSON array to the standard output).
    - "id" (post IDs are printed to the standard output).

_Note: there can be up to 6 tags at once. Trying to search for more will cause a
    422 "Unprocessable entity" HTTP error. This is an API limitation._

## Usage

### Search for posts

#### Single post:
```sh
get621 asriel_dreemurr order:score rating:s
```

#### Multiple posts (here, 5):
```sh
get621 asriel_dreemurr order:score rating:s --limit 5
```

_Note: `--limit` can be replaced with `-l`._

#### Blacklist tags:
```sh
get621 asriel_dreemurr order:score rating:s -- -solo -chicken
```

_Note: Since the syntax to blacklist a tag uses a dash, it must be placed after
    two dashes `--` to make the difference between a blacklisted tag and a
    command option/flag (such as `-l` or `--limit`). As a result, anything after
    `--` will be treated as a tag for the request._

### Saving posts

This will download posts to the current working directory as `<id>.<ext>`.

***This will overwrite any file with the same name in the same folder,
    without warning.***

#### Single post:
```sh
get621 --save asriel_dreemurr order:score rating:s
```

#### Multiple posts (here, 10):
```sh
get621 --save asriel_dreemurr order:score rating:s --limit 10
```

_Note: `--save` can be replaced with `-s`._

### Bulk saving pools

This will download posts to the current working directory as
    `<pool_id>-<page>_<post_id>.<ext>`.

***This will overwrite any file with the same name in the same folder,
    without warning.***

```sh
get621 --pool <pool_id> --save
```

_Note: `--pool` can be replaced with `-P`._

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
    development packages to compile the `openssl-sys` crate (e.g. `libssl-dev`
    on Ubuntu and `openssl-devel` on Fedora). `pkg-config` is also required when
    targeting Linux._


## Changelog

### v1.2.0

* Added: `reverse` sub-command to perform reverse image search (using
    iqdb.harry.lu).
* Added: `-o, --output <mode>` option to specify the output format: either `id`,
    `json`, `raw` or `verbose`.
* Changed: `--` isn't needed anymore when specifying tags. As a result, negative
    tags (e.g. `-chicken`) must be specified after `--`.
* Removed: The following flags: `-j, --json, -o, --output, -v, --verbose`.

### v1.1.0

- Complete rewrite in the Rust programming language.

#### Differences with version 1.0.0
* If tags are supplied, they MUST be placed after every flag and option (if any)
PLUS an argument separator `--`.
* When there's no result, nothing is printed on the standard output (instead of
"Post not found." or equivalent in version 1.0.0).
* If the requested limit is above the hard limit of the API (320 as of writing),
multiple requests will be done until enough posts are gathered. Note that other
flags can act on how many posts are actually returned; the only guarantee is
that at most `limit` posts will be returned.
* The program operates only on e621 (instead of e926 without some compile-time
flag). An "opt-in" NSFW filter is planned for future versions.
* `--verbose` doesn't show tags as "typed" categories anymore.
* `--verbose` formats dates as: YYYY-mm-dd HH:MM:SS.f
* Any other difference I either forgot or accidentally introduced when rewriting
this tool.

### v1.0.0

* Initial release (written in C++).
