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
