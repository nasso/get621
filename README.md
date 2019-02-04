# get621
E621/926 command line tool.

Version 1.1.0 is a complete rewrite in the Rust programming language.

## Differences with version 1.0.0
- If tags are supplied, they MUST be placed after every flag and option (if any)
PLUS an argument separator `--`.
- When there's no result, nothing is printed on the standard output (instead of "Post not found." or
equivalent in version 1.0.0).
- If the requested limit is above the hard limit of the API (320 as of writing), multiple requests
will be done until enough posts are gathered. Note that other flags can act on how many posts are
actually returned; the only guarantee is that at most `limit` posts will be returned.
- The program operates only on e621 (instead of e926 without some compile-time flag). An "opt-in"
NSFW filter is planned for future versions.
- `--verbose` doesn't show tags as "typed" categories anymore.
- `--verbose` formats dates as: YYYY-mm-dd HH:MM:SS.f
- Any other difference I either forgot or accidentally introduced when rewriting this tool.
