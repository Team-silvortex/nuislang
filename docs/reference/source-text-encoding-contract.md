# Source Text Encoding Contract

Nuis follows the same practical source-text rule as Rust: repository source,
configuration, scripts, and documentation use standard UTF-8.

Unicode is allowed in identifiers, string and character literals, comments,
and documentation. The project does not require ASCII-only source.

The repository contract is:

* UTF-8 without a byte-order mark
* LF line endings
* no invalid UTF-8 byte sequences
* no hidden zero-width or bidirectional control characters
* ordinary Unicode text remains valid project content

Editor defaults live in `.editorconfig`, while `.gitattributes` normalizes
tracked text to LF and keeps known binary formats out of text conversion.

Run the same validation used by the mainline gate with:

```bash
python3 scripts/check-text-encoding.py
```

The scanner only reads Git-tracked files with known text formats. Generated
artifacts and tracked binary fixtures are not decoded as source text.
