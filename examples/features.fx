# TYPE() — works for every FLUX value kind
x = 123
y = 4.5
name = "Alex"
enabled = true
items = [1, 2, 3]
player = {"name": "Alex", "level": 5}
nothing = none

PRINT TYPE(x)
PRINT TYPE(y)
PRINT TYPE(name)
PRINT TYPE(enabled)
PRINT TYPE(items)
PRINT TYPE(player)
PRINT TYPE(nothing)

IF TYPE(x) == "INTEGER":
    PRINT "x is a number"

# ASK / ANSWER — prompts on the console, auto-converts the input
ASK "What is your name?"
user_name = ANSWER

ASK "How old are you?"
age = ANSWER

PRINT "Hello, " + user_name
PRINT TYPE(age)

IF age >= 18 AND age < 100:
    PRINT "Adult"
ELSE:
    PRINT "Not an adult"


# Variables and types
name = "FLUX"
version = 1.0
active = TRUE
nothing = NONE

PRINT name
PRINT version
PRINT active
PRINT nothing

# Arrays and dictionaries
colors = ["red", "green", "blue"]
scores = {"alice": 95, "bob": 87}

PRINT colors
PRINT scores
PRINT colors[0]
PRINT scores["alice"]

# Functions
DEF factorial(n):
    IF n <= 1:
        RETURN 1
    RETURN n * factorial(n - 1)

PRINT factorial(5)

# Classes
CLASS Player:
    DEF INIT(name, level):
        self.name = name
        self.level = level
        
        DEF describe(self):
            RETURN self.name + " (level " + str(self.level) + ")"

hero = Player("Aria", 10)
PRINT hero.describe()

# Error handling
TRY:
    result = 10 / 0
EXCEPT:
    PRINT "Caught an error"

# Modules
IMPORT math
PRINT math.abs(-42)

IMPORT sys
PRINT sys.version


