ASK = "First Number"
fn = ANSWER
WHILE TYPE(fn) == "STRING":
    PRINT "Please enter numbers, not text."
    ASK = "First Number"
    fn = ANSWER
ASK = "Second Number"
sn = ANSWER
WHILE TYPE(sn) == "STRING":
    PRINT "Please enter numbers, not text."
    ASK = "Second Number"
    sn = ANSWER
ASK = "Operator? (ASMD)"
op = ANSWER
WHILE op != "A" AND op != "S" AND op != "M" AND op != "D":
    PRINT "Unknown Operator, Please try again."
    ASK = "Operator? (ASMD)"
    op = ANSWER
IF op == "A":
    PRINT fn+sn
ELIF op == "S":
    PRINT fn-sn
ELIF op == "M":
    PRINT fn*sn
ELIF op == "D":
    IF sn == 0:
        PRINT "Dividing by 0 causing a crash. Aborting..."
    ELSE:
        PRINT fn/sn