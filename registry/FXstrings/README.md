# FXstrings

A small string-utility library, written entirely in FLUX — no native code
at all. It exists mainly as a working example package for the Phase 5
package manager: everything about it (its `flux.toml`, its layout, the
way `fx install` handles it) is exactly what a real third-party package
looks like.

```bash
fx install FXstrings
```

```flux
IMPORT FXstrings

PRINT FXstrings.reverse("flux")          # "xulf"
PRINT FXstrings.is_palindrome("level")   # TRUE
PRINT FXstrings.repeat("ab", 3)          # "ababab"
```

## API

| Function | Description |
|---|---|
| `reverse(s)` | Returns `s` reversed |
| `is_palindrome(s)` | `TRUE` if `s` reads the same forwards and backwards |
| `repeat(s, times)` | Returns `s` concatenated with itself `times` times |

See [`src/library.fx`](src/library.fx) for the implementation.
