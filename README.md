# Quote of the Day generator

[RFC 865](https://www.rfc-editor.org/rfc/rfc865)-compatible quote-of-the-day generator.

## Usage

```
cargo run <quotes-file> <port (default: 10017)>
```

Listens on both TCP and UDP, and serves one quote per 86400 seconds since Unix epoch from the provided quotes file. If 0 is selected as a port, a port is chosen randomly for both TCP and UDP (and may not be the same!).

## Quotes file

A quotes file must have one quote on each line, with two fields, the quote and its author. The fields are separated by a pipe character, so this character cannot appear in either the quote or the author. See `quotes.txt` for an example quotes file.

## License

MIT license; see LICENSE.md.
