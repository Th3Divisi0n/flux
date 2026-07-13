# First, install the example package this demo imports:
#   fx install FXstrings
#   fx run examples/package_manager_demo.fx

IMPORT FXstrings

PRINT FXstrings.reverse("flux")
PRINT FXstrings.is_palindrome("level")
PRINT FXstrings.is_palindrome("flux")
PRINT FXstrings.repeat("ab", 3)
