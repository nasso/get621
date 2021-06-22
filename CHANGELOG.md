# Changelog

## v1.3.0

### Added

- Command line argument to change the server URL.

### Fixed

- Support for the latest version of the API.

### Removed

- JSON output mode.

## v1.2.2

### Changed

- Updated license and the versions of dependencies.

## v1.2.1

### Added

- `--direct-save` flag to the `reverse` command. It tells get621 to directly
  downloads posts from e621 without requesting other post information, thus
  bypassing slower API requests.

## v1.2.0

### Added

- `reverse` sub-command to perform reverse image search (using iqdb.harry.lu).
- `-o, --output <mode>` option to specify the output format: either `id`,
  `json`, `raw` or `verbose`.

### Changed

- `--` isn't needed anymore when specifying tags. As a result, negative tags
  (e.g. `-chicken`) must be specified after `--`.

### Removed

- The following flags: `-j, --json, -o, --output, -v, --verbose`.

## v1.1.0

### Added

- If the requested limit is above the hard limit of the API (320 as of writing),
  multiple requests will be done until enough posts are gathered. Note that
  other flags can act on how many posts are actually returned; the only
  guarantee is that at most `limit` posts will be returned.

### Changed

- Complete rewrite in Rust.
- If tags are supplied, they MUST be placed after every flag and option (if any)
  PLUS an argument separator `--`.
- When there's no result, nothing is printed on the standard output (instead of
  "Post not found." or equivalent in version 1.0.0).
- The program operates only on e621 (instead of e926 without some compile-time
  flag). An "opt-in" NSFW filter is planned for future versions.
- `--verbose` formats dates as: YYYY-mm-dd HH:MM:SS.f
- Any other difference I either forgot or accidentally introduced when rewriting
  this tool.

### Removed

- `--verbose` doesn't show tags as "typed" categories anymore.

## v1.0.0

- Initial version written in C++.
