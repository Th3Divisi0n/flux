# FXstrings — small string utilities, written in FLUX itself.
#
# This is the seed package for FLUX's Phase 5 package manager: a plain
# .fx library with no native/Rust code at all, installed the same way any
# third-party package would be. `fx install FXstrings` copies this whole
# directory into flux_modules/FXstrings/, and `IMPORT FXstrings` runs
# this file once and exports every top-level DEF below.

DEF reverse(s):
    result = ""
    i = len(s) - 1
    WHILE i >= 0:
        result = result + s[i]
        i = i - 1
    RETURN result

DEF is_palindrome(s):
    RETURN s == reverse(s)

DEF repeat(s, times):
    result = ""
    FOR i IN RANGE(times):
        result = result + s
    RETURN result
