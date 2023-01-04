# hotsauce

Regex matching on a Iterator over u8.
This uses the regex-automata crate, see that for limitations.

## Limitations

This crate can at the moment not find matches at the end of the iterator.
This only matters for empty matches.

## Verification Recommendation

It is recommended to always use [cargo-crev](https://github.com/crev-dev/cargo-crev)
to verify the trustworthiness of each of your dependencies, including this one.
