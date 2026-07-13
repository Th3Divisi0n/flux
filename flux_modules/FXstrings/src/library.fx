# FXstrings — small string utilities, written in FLUX itself.

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