use flux_ast::{BinaryOp, Literal, UnaryOp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

pub type FluxResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    None,
    Array(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<String, Value>>>),
    Function(FunctionValue),
    NativeFunction(NativeFunction),
    Object(ObjectValue),
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub name: String,
    pub params: Vec<String>,
    pub defaults: HashMap<String, Value>,
    pub body: Vec<flux_ast::Stmt>,
    pub closure: Environment,
}

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: Option<usize>,
    pub func: Rc<dyn Fn(&[Value]) -> FluxResult<Value>>,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeFunction({})", self.name)
    }
}

#[derive(Clone)]
pub struct ObjectValue {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, FunctionValue>,
    /// Native (Rust-implemented) methods, e.g. for standard library objects like FXwindows widgets.
    /// Keyed the same way as `methods`; checked whenever an interpreted method isn't found.
    pub native_methods: HashMap<String, NativeFunction>,
}

impl std::fmt::Debug for ObjectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectValue")
            .field("class_name", &self.class_name)
            .field("fields", &self.fields)
            .field("methods", &self.methods)
            .field("native_methods", &self.native_methods.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.values.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> FluxResult<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
        })
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("undefined variable '{name}'")]
    UndefinedVariable { name: String },

    #[error("undefined property '{name}'")]
    UndefinedProperty { name: String },

    #[error("type error: {message}")]
    TypeError { message: String },

    #[error("argument error: {message}")]
    ArgumentError { message: String },

    #[error("index error: {message}")]
    IndexError { message: String },

    #[error("key error: {message}")]
    KeyError { message: String },

    #[error("{message}")]
    Exception { message: String },

    #[error("value error: {message}")]
    ValueError { message: String },

    #[error("return")]
    ReturnSignal,

    #[error("break")]
    BreakSignal,

    #[error("continue")]
    ContinueSignal,
}

impl Value {
    pub fn from_literal(lit: &Literal) -> Self {
        match lit {
            Literal::String(s) => Value::String(s.clone()),
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Float(n) => Value::Float(*n),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::None => Value::None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::None => false,
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::Dict(dict) => !dict.borrow().is_empty(),
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "String",
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::Boolean(_) => "Boolean",
            Value::None => "None",
            Value::Array(_) => "Array",
            Value::Dict(_) => "Dict",
            Value::Function(_) => "Function",
            Value::NativeFunction(_) => "Function",
            Value::Object(_) => "Object",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        // The closure environment is intentionally excluded: `Environment`
        // can recursively hold other environments and isn't itself a
        // meaningful thing to compare for equality. Two functions are equal
        // here if they have the same observable definition.
        self.name == other.name
            && self.params == other.params
            && self.defaults == other.defaults
            && self.body == other.body
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Rc::ptr_eq(&self.func, &other.func)
    }
}

impl PartialEq for ObjectValue {
    fn eq(&self, other: &Self) -> bool {
        self.class_name == other.class_name
            && self.fields == other.fields
            && self.methods == other.methods
            && self.native_methods == other.native_methods
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Integer(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::None => write!(f, "NONE"),
            Value::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Dict(dict) => {
                let items: Vec<String> = dict
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Function(func) => write!(f, "<function {}>", func.name),
            Value::NativeFunction(func) => write!(f, "<native function {}>", func.name),
            Value::Object(obj) => write!(f, "<{} object>", obj.class_name),
        }
    }
}

pub fn default_globals() -> Environment {
    let mut env = Environment::new();

    env.define(
        "__range__",
        Value::NativeFunction(NativeFunction {
            name: "range".to_string(),
            arity: None,
            func: Rc::new(|args| {
                let (start, end, step) = match args.len() {
                    1 => (0, args[0].as_integer()?, 1),
                    2 => (args[0].as_integer()?, args[1].as_integer()?, 1),
                    3 => (
                        args[0].as_integer()?,
                        args[1].as_integer()?,
                        args[2].as_integer()?,
                    ),
                    _ => {
                        return Err(RuntimeError::ArgumentError {
                            message: "range expects 1 to 3 arguments".to_string(),
                        })
                    }
                };

                if step == 0 {
                    return Err(RuntimeError::ValueError {
                        message: "range step cannot be zero".to_string(),
                    });
                }

                let mut values = Vec::new();
                let mut current = start;
                if step > 0 {
                    while current < end {
                        values.push(Value::Integer(current));
                        current += step;
                    }
                } else {
                    while current > end {
                        values.push(Value::Integer(current));
                        current += step;
                    }
                }
                Ok(Value::Array(Rc::new(RefCell::new(values))))
            }),
        }),
    );

    env.define(
        "len",
        Value::NativeFunction(NativeFunction {
            name: "len".to_string(),
            arity: Some(1),
            func: Rc::new(|args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::Array(arr) => Ok(Value::Integer(arr.borrow().len() as i64)),
                    Value::Dict(dict) => Ok(Value::Integer(dict.borrow().len() as i64)),
                    other => Err(RuntimeError::TypeError {
                        message: format!("len() not supported for {}", other.type_name()),
                    }),
                }
            }),
        }),
    );

    env.define(
        "str",
        Value::NativeFunction(NativeFunction {
            name: "str".to_string(),
            arity: Some(1),
            func: Rc::new(|args| Ok(Value::String(args[0].to_string()))),
        }),
    );

    env.define(
        "int",
        Value::NativeFunction(NativeFunction {
            name: "int".to_string(),
            arity: Some(1),
            func: Rc::new(|args| match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(*n)),
                Value::Float(n) => Ok(Value::Integer(*n as i64)),
                Value::String(s) => s.parse::<i64>().map(Value::Integer).map_err(|_| {
                    RuntimeError::ValueError {
                        message: format!("invalid integer literal '{s}'"),
                    }
                }),
                other => Err(RuntimeError::TypeError {
                    message: format!("int() not supported for {}", other.type_name()),
                }),
            }),
        }),
    );

    env.define(
        "float",
        Value::NativeFunction(NativeFunction {
            name: "float".to_string(),
            arity: Some(1),
            func: Rc::new(|args| match &args[0] {
                Value::Float(n) => Ok(Value::Float(*n)),
                Value::Integer(n) => Ok(Value::Float(*n as f64)),
                Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
                    RuntimeError::ValueError {
                        message: format!("invalid float literal '{s}'"),
                    }
                }),
                other => Err(RuntimeError::TypeError {
                    message: format!("float() not supported for {}", other.type_name()),
                }),
            }),
        }),
    );

    env.define(
        "type",
        Value::NativeFunction(NativeFunction {
            name: "type".to_string(),
            arity: Some(1),
            func: Rc::new(|args| Ok(Value::String(args[0].type_name().to_uppercase()))),
        }),
    );

    env
}

impl Value {
    pub fn as_integer(&self) -> FluxResult<i64> {
        match self {
            Value::Integer(n) => Ok(*n),
            Value::Float(n) => Ok(*n as i64),
            other => Err(RuntimeError::TypeError {
                message: format!("expected Integer, got {}", other.type_name()),
            }),
        }
    }

    pub fn as_float(&self) -> FluxResult<f64> {
        match self {
            Value::Float(n) => Ok(*n),
            Value::Integer(n) => Ok(*n as f64),
            other => Err(RuntimeError::TypeError {
                message: format!("expected Float, got {}", other.type_name()),
            }),
        }
    }

    pub fn as_string(&self) -> FluxResult<String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            other => Err(RuntimeError::TypeError {
                message: format!("expected String, got {}", other.type_name()),
            }),
        }
    }
}

pub fn apply_binary_op(left: &Value, op: BinaryOp, right: &Value) -> FluxResult<Value> {
    match op {
        BinaryOp::Add => add_values(left, right),
        BinaryOp::Sub => numeric_binary(left, right, |a, b| Value::Integer(a - b), |a, b| Value::Float(a - b)),
        BinaryOp::Mul => numeric_binary(left, right, |a, b| Value::Integer(a * b), |a, b| Value::Float(a * b)),
        BinaryOp::Div => numeric_binary(left, right, |a, b| Value::Float(a as f64 / b as f64), |a, b| Value::Float(a / b)),
        BinaryOp::FloorDiv => {
            if matches!((left, right), (Value::Integer(_) | Value::Float(_), Value::Integer(0))) {
                return Err(RuntimeError::ValueError {
                    message: "floor division by zero".to_string(),
                });
            }
            numeric_binary(left, right, |a, b| Value::Integer(a / b), |a, b| Value::Float((a / b).floor()))
        }
        BinaryOp::Mod => {
            if matches!((left, right), (Value::Integer(_) | Value::Float(_), Value::Integer(0))) {
                return Err(RuntimeError::ValueError {
                    message: "modulo by zero".to_string(),
                });
            }
            numeric_binary(left, right, |a, b| Value::Integer(a % b), |a, b| Value::Float(a % b))
        }
        BinaryOp::Pow => {
            let base = left.as_float()?;
            let exp = right.as_float()?;
            Ok(Value::Float(base.powf(exp)))
        }
        BinaryOp::Eq => Ok(Value::Boolean(values_equal(left, right))),
        BinaryOp::NotEq => Ok(Value::Boolean(!values_equal(left, right))),
        BinaryOp::Lt => compare_values(left, right, |a, b| a < b),
        BinaryOp::LtEq => compare_values(left, right, |a, b| a <= b),
        BinaryOp::Gt => compare_values(left, right, |a, b| a > b),
        BinaryOp::GtEq => compare_values(left, right, |a, b| a >= b),
        BinaryOp::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
        BinaryOp::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
    }
}

pub fn apply_unary_op(op: UnaryOp, operand: &Value) -> FluxResult<Value> {
    match op {
        UnaryOp::Not => Ok(Value::Boolean(!operand.is_truthy())),
        UnaryOp::Neg => match operand {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            other => Err(RuntimeError::TypeError {
                message: format!("unary '-' not supported for {}", other.type_name()),
            }),
        },
    }
}

fn add_values(left: &Value, right: &Value) -> FluxResult<Value> {
    match (left, right) {
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        (Value::String(a), other) => Ok(Value::String(format!("{a}{other}"))),
        (other, Value::String(b)) => Ok(Value::String(format!("{other}{b}"))),
        _ => numeric_binary(left, right, |a, b| Value::Integer(a + b), |a, b| Value::Float(a + b)),
    }
}

fn numeric_binary<FInt, FFloat>(
    left: &Value,
    right: &Value,
    int_op: FInt,
    float_op: FFloat,
) -> FluxResult<Value>
where
    FInt: Fn(i64, i64) -> Value,
    FFloat: Fn(f64, f64) -> Value,
{
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(int_op(*a, *b)),
        _ => Ok(float_op(left.as_float()?, right.as_float()?)),
    }
}

fn compare_values<F>(left: &Value, right: &Value, cmp: F) -> FluxResult<Value>
where
    F: Fn(f64, f64) -> bool,
{
    Ok(Value::Boolean(cmp(left.as_float()?, right.as_float()?)))
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => *a as f64 == *b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::None, Value::None) => true,
        _ => false,
    }
}
