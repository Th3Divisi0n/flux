use flux_interpreter::interpret;

#[test]
fn hello_world() {
    interpret(r#"PRINT "Hello World""#).unwrap();
}

#[test]
fn variables_and_arithmetic() {
    let source = r#"
a = 10
b = 20
result = a + b
"#;
    interpret(source).unwrap();
}

#[test]
fn functions() {
    let source = r#"
DEF multiply(a, b):
    RETURN a * b

result = multiply(6, 7)
"#;
    interpret(source).unwrap();
}

#[test]
fn control_flow() {
    let source = r#"
IF TRUE:
    x = 1
ELSE:
    x = 0

FOR i IN RANGE(3):
    total = i
"#;
    interpret(source).unwrap();
}

#[test]
fn classes() {
    let source = r#"
CLASS Point:
    DEF INIT(x, y):
        self.x = x
        self.y = y

p = Point(1, 2)
"#;
    interpret(source).unwrap();
}
