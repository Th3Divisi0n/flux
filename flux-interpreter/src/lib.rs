use flux_ast::{Expr, Program, Stmt};
use flux_runtime::{
    apply_binary_op, apply_unary_op, default_globals, Environment, FluxResult, FunctionValue,
    NativeFunction, ObjectValue, RuntimeError, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("runtime error at line {line}: {source}")]
    Runtime {
        line: usize,
        #[source]
        source: RuntimeError,
    },

    #[error("runtime error: {0}")]
    Other(#[from] RuntimeError),
}

pub struct Interpreter {
    env: Rc<RefCell<Environment>>,
    return_value: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(default_globals())),
            return_value: None,
        }
    }

    pub fn run(&mut self, program: &Program) -> FluxResult<Value> {
        self.execute_block(&program.statements)?;
        Ok(Value::None)
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> FluxResult<()> {
        for stmt in statements {
            self.execute_stmt(stmt)?;
            if self.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> FluxResult<()> {
        match stmt {
            Stmt::Print { expr, .. } => {
                let value = self.eval_expr(expr)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Ask { message, .. } => {
                let prompt = self.eval_expr(message)?;
                print!("{prompt} ");
                io::stdout().flush().ok();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| RuntimeError::Exception {
                        message: format!("failed to read input: {e}"),
                    })?;
                let input = input.trim();

                let answer = if let Ok(i) = input.parse::<i64>() {
                    Value::Integer(i)
                } else if let Ok(f) = input.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::String(input.to_string())
                };

                if self.env.borrow().get("ANSWER").is_some() {
                    self.env.borrow_mut().assign("ANSWER", answer)?;
                } else {
                    self.env.borrow_mut().define("ANSWER", answer);
                }
                Ok(())
            }
            Stmt::Assign { name, expr, .. } => {
                let value = self.eval_expr(expr)?;
                if self.env.borrow().get(name).is_some() {
                    self.env.borrow_mut().assign(name, value)?;
                } else {
                    self.env.borrow_mut().define(name.clone(), value);
                }
                Ok(())
            }
            Stmt::MemberAssign {
                object,
                member,
                expr,
                ..
            } => {
                let target_name = if let Expr::Variable { name, .. } = object {
                    Some(name.clone())
                } else {
                    None
                };
                let mut obj = self.eval_expr(object)?;
                let value = self.eval_expr(expr)?;
                if let Value::Object(ref mut object) = obj {
                    object.fields.insert(member.clone(), value);
                    if let Some(name) = target_name {
                        self.env.borrow_mut().assign(&name, obj)?;
                    }
                    Ok(())
                } else {
                    Err(RuntimeError::TypeError {
                        message: "member assignment requires object target".to_string(),
                    })
                }
            }
            Stmt::IndexAssign {
                object,
                index,
                expr,
                ..
            } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                let value = self.eval_expr(expr)?;
                match obj {
                    Value::Array(arr) => {
                        let i = idx.as_integer()? as usize;
                        let mut arr = arr.borrow_mut();
                        if i >= arr.len() {
                            return Err(RuntimeError::IndexError {
                                message: format!("array index out of range: {i}"),
                            });
                        }
                        arr[i] = value;
                        Ok(())
                    }
                    Value::Dict(dict) => {
                        let key = idx.to_string();
                        dict.borrow_mut().insert(key, value);
                        Ok(())
                    }
                    other => Err(RuntimeError::TypeError {
                        message: format!("index assignment not supported for {}", other.type_name()),
                    }),
                }
            }
            Stmt::Expr { expr, .. } => {
                self.eval_expr(expr)?;
                Ok(())
            }
            Stmt::FunctionDef {
                name,
                params,
                body,
                ..
            } => {
                let mut param_names = Vec::new();
                let mut defaults = HashMap::new();
                for param in params {
                    param_names.push(param.name.clone());
                    if let Some(default) = &param.default {
                        defaults.insert(
                            param.name.clone(),
                            self.eval_expr(default)?,
                        );
                    }
                }

                let func = FunctionValue {
                    name: name.clone(),
                    params: param_names,
                    defaults,
                    body: body.clone(),
                    closure: Environment {
                        values: self.env.borrow().values.clone(),
                        parent: self.env.borrow().parent.clone(),
                    },
                };
                self.env.borrow_mut().define(name.clone(), Value::Function(func));
                Ok(())
            }
            Stmt::Return { expr, .. } => {
                self.return_value = Some(if let Some(expr) = expr {
                    self.eval_expr(expr)?
                } else {
                    Value::None
                });
                Err(RuntimeError::ReturnSignal)
            }
            Stmt::If {
                condition,
                then_body,
                elif_branches,
                else_body,
                ..
            } => {
                if self.eval_expr(condition)?.is_truthy() {
                    self.execute_block(then_body)
                } else {
                    for (elif_cond, elif_body) in elif_branches {
                        if self.eval_expr(elif_cond)?.is_truthy() {
                            return self.execute_block(elif_body);
                        }
                    }
                    if let Some(body) = else_body {
                        self.execute_block(body)
                    } else {
                        Ok(())
                    }
                }
            }
            Stmt::While { condition, body, .. } => {
                while self.eval_expr(condition)?.is_truthy() {
                    match self.execute_block(body) {
                        Ok(()) => {}
                        Err(RuntimeError::BreakSignal) => break,
                        Err(RuntimeError::ContinueSignal) => continue,
                        Err(RuntimeError::ReturnSignal) => return Err(RuntimeError::ReturnSignal),
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            }
            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                let iterable_value = self.eval_expr(iterable)?;
                let items = match iterable_value {
                    Value::Array(arr) => arr.borrow().clone(),
                    other => {
                        return Err(RuntimeError::TypeError {
                            message: format!("FOR loop requires iterable, got {}", other.type_name()),
                        })
                    }
                };

                for item in items {
                    if self.env.borrow().get(variable).is_some() {
                        self.env.borrow_mut().assign(variable, item)?;
                    } else {
                        self.env.borrow_mut().define(variable.clone(), item);
                    }

                    match self.execute_block(body) {
                        Ok(()) => {}
                        Err(RuntimeError::BreakSignal) => break,
                        Err(RuntimeError::ContinueSignal) => continue,
                        Err(RuntimeError::ReturnSignal) => return Err(RuntimeError::ReturnSignal),
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            }
            Stmt::Break { .. } => Err(RuntimeError::BreakSignal),
            Stmt::Continue { .. } => Err(RuntimeError::ContinueSignal),
            Stmt::Pass { .. } => Ok(()),
            Stmt::Import { module, alias, .. } => {
                let binding = alias.clone().unwrap_or_else(|| module.clone());
                let value = self.load_module(module)?;
                self.env.borrow_mut().define(binding, value);
                Ok(())
            }
            Stmt::FromImport { module, names, .. } => {
                let module_value = self.load_module(module)?;
                for (name, alias) in names {
                    let binding = alias.clone().unwrap_or_else(|| name.clone());
                    let value = self.get_module_export(&module_value, &name)?;
                    self.env.borrow_mut().define(binding, value);
                }
                Ok(())
            }
            Stmt::ClassDef { name, body, .. } => {
                let class_obj = self.build_class(name, body)?;
                self.env.borrow_mut().define(name.clone(), class_obj);
                Ok(())
            }
            Stmt::Try {
                body,
                handlers,
                finally_body,
                ..
            } => {
                let result = self.execute_block(body);

                // Control-flow signals (RETURN/BREAK/CONTINUE) are not
                // exceptions: they must pass straight through to the
                // enclosing function/loop, not be swallowed or matched
                // against EXCEPT clauses.
                let try_result: FluxResult<()> = match result {
                    Ok(()) => Ok(()),
                    Err(
                        err @ (RuntimeError::ReturnSignal
                        | RuntimeError::BreakSignal
                        | RuntimeError::ContinueSignal),
                    ) => Err(err),
                    Err(err) => {
                        let message = err.to_string();
                        let mut outcome = None;
                        for handler in handlers {
                            match self.matches_handler(handler, &message) {
                                Ok(true) => {
                                    outcome = Some(Ok(()));
                                    break;
                                }
                                Ok(false) => continue,
                                Err(handler_err) => {
                                    outcome = Some(Err(handler_err));
                                    break;
                                }
                            }
                        }
                        // No handler matched: re-raise the original error.
                        outcome.unwrap_or(Err(err))
                    }
                };

                // FINALLY always runs, even if the TRY body or an EXCEPT
                // handler raised. If FINALLY itself raises/returns, that
                // outcome takes precedence over whatever TRY/EXCEPT produced.
                let finally_result = if let Some(finally_body) = finally_body {
                    self.execute_block(finally_body)
                } else {
                    Ok(())
                };

                match finally_result {
                    Err(e) => Err(e),
                    Ok(()) => try_result,
                }
            }
            Stmt::Raise { expr, span } => {
                let message = if let Some(expr) = expr {
                    self.eval_expr(expr)?.to_string()
                } else {
                    "Exception".to_string()
                };
                Err(RuntimeError::Exception {
                    message: format!("line {}: {message}", span.start_line),
                })
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> FluxResult<Value> {
        match expr {
            Expr::Literal { value, .. } => Ok(Value::from_literal(value)),
            Expr::Variable { name, .. } => self
                .env
                .borrow()
                .get(name)
                .ok_or_else(|| RuntimeError::UndefinedVariable {
                    name: name.clone(),
                }),
            Expr::Binary { left, op, right, .. } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                apply_binary_op(&left_val, *op, &right_val)
            }
            Expr::Unary { op, operand, .. } => {
                let val = self.eval_expr(operand)?;
                apply_unary_op(*op, &val)
            }
            Expr::Call { callee, args, .. } => {
                if let Expr::Member { object, member, .. } = callee.as_ref() {
                    let obj_val = self.eval_expr(object)?;
                    if let Value::Object(ref obj) = obj_val {
                        if let Some(func) = obj.methods.get(member).cloned() {
                            let mut arg_values = vec![obj_val];
                            for arg in args {
                                arg_values.push(self.eval_expr(arg)?);
                            }
                            return self.call_function(func, arg_values);
                        }
                        if let Some(native) = obj.native_methods.get(member).cloned() {
                            let mut arg_values = vec![obj_val];
                            for arg in args {
                                arg_values.push(self.eval_expr(arg)?);
                            }
                            if let Some(arity) = native.arity {
                                if arg_values.len() != arity {
                                    return Err(RuntimeError::ArgumentError {
                                        message: format!(
                                            "{}() expects {} arguments, got {}",
                                            native.name,
                                            arity,
                                            arg_values.len()
                                        ),
                                    });
                                }
                            }
                            return (native.func)(&arg_values);
                        }
                    }
                }

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                self.call_callable(callee, arg_values)
            }
            Expr::Index { object, index, .. } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                match obj {
                    Value::Array(arr) => {
                        let i = idx.as_integer()?;
                        let arr = arr.borrow();
                        arr.get(i as usize)
                            .cloned()
                            .ok_or_else(|| RuntimeError::IndexError {
                                message: format!("array index out of range: {i}"),
                            })
                    }
                    Value::String(s) => {
                        let i = idx.as_integer()?;
                        s.chars()
                            .nth(i as usize)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or_else(|| RuntimeError::IndexError {
                                message: format!("string index out of range: {i}"),
                            })
                    }
                    other => Err(RuntimeError::TypeError {
                        message: format!("indexing not supported for {}", other.type_name()),
                    }),
                }
            }
            Expr::Member { object, member, .. } => {
                let obj = self.eval_expr(object)?;

                match obj {
                    Value::Object(object) => {
                        if let Some(value) = object.fields.get(member) {
                            return Ok(value.clone());
                        }

                        if let Some(func) = object.methods.get(member) {
                            return Ok(Value::Function(func.clone()));
                        }

                        if let Some(native) = object.native_methods.get(member) {
                            return Ok(Value::NativeFunction(native.clone()));
                        }

                        Err(RuntimeError::UndefinedProperty {
                            name: member.clone(),
                        })
                    }

                    other => Err(RuntimeError::TypeError {
                        message: format!("member access not supported for {}", other.type_name()),
                    }),
                }
            }
            Expr::Array { elements, .. } => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.eval_expr(element)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(values))))
            }
            Expr::Dict { entries, .. } => {
                let mut dict = HashMap::new();
                for (key_expr, value_expr) in entries {
                    let key = self.eval_expr(key_expr)?.to_string();
                    let value = self.eval_expr(value_expr)?;
                    dict.insert(key, value);
                }
                Ok(Value::Dict(Rc::new(RefCell::new(dict))))
            }
            Expr::Lambda { .. } => Err(RuntimeError::TypeError {
                message: "lambda expressions are not yet supported".to_string(),
            }),
        }
    }

    fn call_callable(&mut self, callee: &Expr, args: Vec<Value>) -> FluxResult<Value> {
        let callee_value = self.eval_expr(callee)?;
        match callee_value {
            Value::Function(func) => self.call_function(func, args),
            Value::NativeFunction(native) => {
                if let Some(arity) = native.arity {
                    if args.len() != arity {
                        return Err(RuntimeError::ArgumentError {
                            message: format!(
                                "{}() expects {} arguments, got {}",
                                native.name,
                                arity,
                                args.len()
                            ),
                        });
                    }
                }
                (native.func)(&args)
            }
            Value::Object(obj) if obj.class_name != "module" => {
                self.call_class_constructor(obj, args)
            }
            other => Err(RuntimeError::TypeError {
                message: format!("{} is not callable", other.type_name()),
            }),
        }
    }

    fn call_function(&mut self, func: FunctionValue, args: Vec<Value>) -> FluxResult<Value> {
        if args.len() > func.params.len() {
            return Err(RuntimeError::ArgumentError {
                message: format!(
                    "{}() expects at most {} arguments, got {}",
                    func.name,
                    func.params.len(),
                    args.len()
                ),
            });
        }

        let child_env = Rc::new(RefCell::new(Environment {
            values: func.closure.values.clone(),
            parent: func.closure.parent.clone(),
        }));

        for (i, param) in func.params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(default) = func.defaults.get(param) {
                default.clone()
            } else {
                return Err(RuntimeError::ArgumentError {
                    message: format!("missing required argument '{param}'"),
                });
            };
            child_env.borrow_mut().define(param.clone(), value);
        }

        let previous_env = self.env.clone();
        self.env = child_env;
        self.return_value = None;

        let result = self.execute_block(&func.body);

        self.env = previous_env;

        
        match result {
            Ok(()) => Ok(self.return_value.take().unwrap_or(Value::None)),
            Err(RuntimeError::ReturnSignal) => Ok(self.return_value.take().unwrap_or(Value::None)),
            Err(e) => Err(e),
        }
    }

    fn call_class_constructor(
        &mut self,
        class_template: ObjectValue,
        args: Vec<Value>,
    ) -> FluxResult<Value> {
        let instance = ObjectValue {
            class_name: class_template.class_name.clone(),
            fields: class_template.fields.clone(),
            methods: class_template.methods.clone(),
            native_methods: class_template.native_methods.clone(),
        };

        if let Some(init) = instance.methods.get("init").cloned() {
            let child_env = Rc::new(RefCell::new(Environment {
                values: init.closure.values.clone(),
                parent: init.closure.parent.clone(),
            }));

            child_env
                .borrow_mut()
                .define("self", Value::Object(instance.clone()));

            for (i, param) in init.params.iter().enumerate() {
                let value = if i < args.len() {
                    args[i].clone()
                } else if let Some(default) = init.defaults.get(param) {
                    default.clone()
                } else {
                    return Err(RuntimeError::ArgumentError {
                        message: format!("missing required argument '{param}'"),
                    });
                };

                child_env.borrow_mut().define(param.clone(), value);
            }

            let previous_env = self.env.clone();

            self.env = child_env;
            self.return_value = None;

            let result = self.execute_block(&init.body);

            let updated_instance = self.env.borrow().get("self");

            self.env = previous_env;

            match result {
                Ok(()) | Err(RuntimeError::ReturnSignal) => {
                    if let Some(Value::Object(updated)) = self.return_value.take() {
                        Ok(Value::Object(updated))
                    } else if let Some(self_val) = updated_instance {
                        Ok(self_val)
                    } else {
                        Ok(Value::Object(instance))
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(Value::Object(instance))
        }
    }
    fn build_class(&mut self, name: &str, body: &[Stmt]) -> FluxResult<Value> {
        let mut methods = HashMap::new();
        let mut fields = HashMap::new();

        for stmt in body {
            match stmt {
                Stmt::FunctionDef { name: method_name, params, body, .. } => {
                    let mut param_names = Vec::new();
                    let mut defaults = HashMap::new();
                    for param in params {
                        param_names.push(param.name.clone());
                        if let Some(default) = &param.default {
                            defaults.insert(param.name.clone(), self.eval_expr(default)?);
                        }
                    }
                    let key = method_name.to_lowercase();
                    methods.insert(
                        key,
                        FunctionValue {
                            name: method_name.clone(),
                            params: param_names,
                            defaults,
                            body: body.clone(),
                            closure: Environment {
                                values: self.env.borrow().values.clone(),
                                parent: self.env.borrow().parent.clone(),
                            },
                        },
                    );
                }
                Stmt::Assign { name: field_name, expr, .. } => {
                    fields.insert(field_name.clone(), self.eval_expr(expr)?);
                }
                _ => {}
            }
        }

        Ok(Value::Object(ObjectValue {
            class_name: name.to_string(),
            fields,
            methods,
            native_methods: HashMap::new(),
        }))
    }

    fn load_module(&mut self, module: &str) -> FluxResult<Value> {
        match module {
            "math" => Ok(self.builtin_math_module()),
            "io" => Ok(self.builtin_io_module()),
            "sys" => Ok(self.builtin_sys_module()),
            m if m.eq_ignore_ascii_case("fxwindows") => Ok(self.builtin_fxwindows_module()),
            m if m.eq_ignore_ascii_case("fxterminal") => Ok(self.builtin_fxterminal_module()),
            other => self.load_installed_module(other),
        }
    }

    /// Fallback for `IMPORT <name>` when `<name>` isn't a built-in module:
    /// loads a package installed via `fx install` (see the Phase 5 package
    /// manager) from `flux_modules/<name>/`, relative to the current
    /// working directory — the same place `fx install` copies it to.
    ///
    /// Packages follow the layout documented in
    /// `documentation/LANGUAGE_SPEC.md` §7: a `flux.toml` plus a
    /// `src/library.fx` entry point, by convention (same as `src/main.fx`
    /// for a project). That file is parsed and executed once, in a fresh
    /// environment that still has access to FLUX's built-in functions
    /// (RANGE, LEN, etc.) but not to the importing script's variables.
    /// Every name the file defines at its top level becomes an export —
    /// same shape as a built-in module, so `IMPORT Name` and
    /// `Name.thing()` behave identically whether `Name` ships with the
    /// runtime or was installed as a package.
    fn load_installed_module(&mut self, name: &str) -> FluxResult<Value> {
        let package_dir = std::path::Path::new("flux_modules").join(name);

        if !package_dir.join("flux.toml").is_file() {
            return Err(RuntimeError::Exception {
                message: format!(
                    "module '{name}' not found (not a built-in module, and no package is \
                     installed at '{}' — run `fx install {name}` first)",
                    package_dir.display()
                ),
            });
        }

        let source_path = package_dir.join("src").join("library.fx");
        let source = std::fs::read_to_string(&source_path).map_err(|e| RuntimeError::Exception {
            message: format!(
                "failed to read entry file '{}' for package '{name}': {e}",
                source_path.display()
            ),
        })?;

        let tokens = flux_lexer::lex(&source).map_err(|e| RuntimeError::Exception {
            message: format!("error loading package '{name}': {e}"),
        })?;
        let program = flux_parser::parse(tokens).map_err(|e| RuntimeError::Exception {
            message: format!("error loading package '{name}': {e}"),
        })?;

        // Fresh globals (built-ins only, no access to the importer's own
        // variables) so the package runs in isolation.
        let module_env = Rc::new(RefCell::new(Environment::with_parent(Rc::new(RefCell::new(
            default_globals(),
        )))));
        let mut module_interpreter = Interpreter {
            env: module_env.clone(),
            return_value: None,
        };
        module_interpreter.execute_block(&program.statements)?;

        let fields = module_env.borrow().values.clone();

        Ok(Value::Object(ObjectValue {
            class_name: "module".to_string(),
            fields,
            methods: HashMap::new(),
            native_methods: HashMap::new(),
        }))
    }

    fn get_module_export(&self, module: &Value, name: &str) -> FluxResult<Value> {
        if let Value::Object(obj) = module {
            if obj.class_name == "module" {
                return obj
                    .fields
                    .get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError::Exception {
                        message: format!("cannot import name '{name}'"),
                    });
            }
        }
        Err(RuntimeError::TypeError {
            message: "invalid module object".to_string(),
        })
    }

    fn builtin_math_module(&self) -> Value {
        let mut fields = HashMap::new();
        fields.insert(
            "abs".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "abs".to_string(),
                arity: Some(1),
                func: Rc::new(|args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(n.abs())),
                    Value::Float(n) => Ok(Value::Float(n.abs())),
                    other => Err(RuntimeError::TypeError {
                        message: format!("abs() not supported for {}", other.type_name()),
                    }),
                }),
            }),
        );
        fields.insert(
            "max".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "max".to_string(),
                arity: None,
                func: Rc::new(|args| {
                    if args.is_empty() {
                        return Err(RuntimeError::ArgumentError {
                            message: "max() requires at least one argument".to_string(),
                        });
                    }
                    let mut max_val = args[0].as_float()?;
                    for arg in &args[1..] {
                        max_val = max_val.max(arg.as_float()?);
                    }
                    Ok(Value::Float(max_val))
                }),
            }),
        );
        fields.insert(
            "min".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "min".to_string(),
                arity: None,
                func: Rc::new(|args| {
                    if args.is_empty() {
                        return Err(RuntimeError::ArgumentError {
                            message: "min() requires at least one argument".to_string(),
                        });
                    }
                    let mut min_val = args[0].as_float()?;
                    for arg in &args[1..] {
                        min_val = min_val.min(arg.as_float()?);
                    }
                    Ok(Value::Float(min_val))
                }),
            }),
        );

        Value::Object(ObjectValue {
            class_name: "module".to_string(),
            fields,
            methods: HashMap::new(),
            native_methods: HashMap::new(),
        })
    }

    fn builtin_io_module(&self) -> Value {
        let mut fields = HashMap::new();
        fields.insert(
            "read_file".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "read_file".to_string(),
                arity: Some(1),
                func: Rc::new(|args| {
                    let path = args[0].as_string()?;
                    std::fs::read_to_string(&path)
                        .map(Value::String)
                        .map_err(|e| RuntimeError::Exception {
                            message: format!("failed to read '{path}': {e}"),
                        })
                }),
            }),
        );
        fields.insert(
            "write_file".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "write_file".to_string(),
                arity: Some(2),
                func: Rc::new(|args| {
                    let path = args[0].as_string()?;
                    let content = args[1].to_string();
                    std::fs::write(&path, content).map(|_| Value::None).map_err(|e| {
                        RuntimeError::Exception {
                            message: format!("failed to write '{path}': {e}"),
                        }
                    })
                }),
            }),
        );

        Value::Object(ObjectValue {
            class_name: "module".to_string(),
            fields,
            methods: HashMap::new(),
            native_methods: HashMap::new(),
        })
    }

    fn builtin_sys_module(&self) -> Value {
        let mut fields = HashMap::new();
        fields.insert(
            "version".to_string(),
            Value::String("FLUX 1.0.0".to_string()),
        );
        fields.insert(
            "platform".to_string(),
            Value::String(std::env::consts::OS.to_string()),
        );

        Value::Object(ObjectValue {
            class_name: "module".to_string(),
            fields,
            methods: HashMap::new(),
            native_methods: HashMap::new(),
        })
    }

    /// The first official FLUX standard library module: FXwindows.
    ///
    /// This wires up the full object model FXwindows programs expect —
    /// `create_window`, `Button`, `Label`, `TextBox`, `Image`, `Panel`,
    /// `Slider`, `Checkbox`, plus `window.add(widget)` / `window.show()` —
    /// so FXwindows.fx scripts parse, run, and mutate real (shared, via
    /// Rc<RefCell<..>>) state today.
    ///
    /// With the default `gui` feature enabled, `show()` opens a real,
    /// interactive OS window via the `flux-gui` crate (winit + egui) and
    /// blocks until the user closes it. Build with `--no-default-features`
    /// to fall back to a text-summary placeholder instead (e.g. for a
    /// headless machine/CI box).
    fn builtin_fxwindows_module(&self) -> Value {
        let mut fields = HashMap::new();

        fields.insert(
            "create_window".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "create_window".to_string(),
                arity: Some(3),
                func: Rc::new(|args| {
                    let title = args[0].as_string()?;
                    let width = args[1].as_integer()?;
                    let height = args[2].as_integer()?;

                    let mut window_fields = HashMap::new();
                    window_fields.insert("title".to_string(), Value::String(title));
                    window_fields.insert("width".to_string(), Value::Integer(width));
                    window_fields.insert("height".to_string(), Value::Integer(height));
                    window_fields.insert(
                        "children".to_string(),
                        Value::Array(Rc::new(RefCell::new(Vec::new()))),
                    );

                    let mut native_methods = HashMap::new();
                    native_methods.insert(
                        "show".to_string(),
                        NativeFunction {
                            name: "show".to_string(),
                            arity: Some(1),
                            func: Rc::new(|args| {
                                let obj = match &args[0] {
                                    Value::Object(obj) => obj,
                                    other => {
                                        return Err(RuntimeError::TypeError {
                                            message: format!(
                                                "show() must be called on a Window, got {}",
                                                other.type_name()
                                            ),
                                        })
                                    }
                                };

                                #[cfg(feature = "gui")]
                                {
                                    let spec = Self::window_object_to_gui_spec(obj);
                                    flux_gui::show_window(spec).map_err(|e| {
                                        RuntimeError::Exception {
                                            message: format!(
                                                "failed to open FXwindows window: {e}"
                                            ),
                                        }
                                    })?;
                                }

                                #[cfg(not(feature = "gui"))]
                                {
                                    let title =
                                        obj.fields.get("title").cloned().unwrap_or(Value::None);
                                    let width =
                                        obj.fields.get("width").cloned().unwrap_or(Value::None);
                                    let height =
                                        obj.fields.get("height").cloned().unwrap_or(Value::None);
                                    let count = match obj.fields.get("children") {
                                        Some(Value::Array(arr)) => arr.borrow().len(),
                                        _ => 0,
                                    };
                                    println!(
                                        "[FXwindows] '{title}' ({width}x{height}) with {count} element(s) — built without the 'gui' feature, this is a placeholder."
                                    );
                                }

                                Ok(Value::None)
                            }),
                        },
                    );
                    native_methods.insert(
                        "add".to_string(),
                        NativeFunction {
                            name: "add".to_string(),
                            arity: Some(2),
                            func: Rc::new(|args| {
                                let obj = match &args[0] {
                                    Value::Object(obj) => obj,
                                    other => {
                                        return Err(RuntimeError::TypeError {
                                            message: format!(
                                                "add() must be called on a Window, got {}",
                                                other.type_name()
                                            ),
                                        })
                                    }
                                };
                                match obj.fields.get("children") {
                                    Some(Value::Array(children)) => {
                                        children.borrow_mut().push(args[1].clone());
                                        Ok(Value::None)
                                    }
                                    _ => Err(RuntimeError::TypeError {
                                        message: "window has no 'children' collection".to_string(),
                                    }),
                                }
                            }),
                        },
                    );

                    Ok(Value::Object(ObjectValue {
                        class_name: "Window".to_string(),
                        fields: window_fields,
                        methods: HashMap::new(),
                        native_methods,
                    }))
                }),
            }),
        );

        fields.insert(
            "Button".to_string(),
            Value::NativeFunction(Self::widget_constructor("Button", &["text", "width", "height"])),
        );
        fields.insert(
            "Label".to_string(),
            Value::NativeFunction(Self::widget_constructor("Label", &["text", "x", "y"])),
        );
        fields.insert(
            "TextBox".to_string(),
            Value::NativeFunction(Self::widget_constructor("TextBox", &["placeholder", "x", "y"])),
        );
        fields.insert(
            "Image".to_string(),
            Value::NativeFunction(Self::widget_constructor("Image", &["path", "x", "y"])),
        );
        fields.insert(
            "Panel".to_string(),
            Value::NativeFunction(Self::widget_constructor("Panel", &["x", "y", "width", "height"])),
        );
        fields.insert(
            "Slider".to_string(),
            Value::NativeFunction(Self::widget_constructor(
                "Slider",
                &["min", "max", "value", "x", "y"],
            )),
        );
        fields.insert(
            "Checkbox".to_string(),
            Value::NativeFunction(Self::widget_constructor("Checkbox", &["label", "checked", "x", "y"])),
        );

        Value::Object(ObjectValue {
            class_name: "module".to_string(),
            fields,
            methods: HashMap::new(),
            native_methods: HashMap::new(),
        })
    }

    /// Builds a native constructor for a simple FXwindows UI block: it just
    /// maps positional arguments onto named fields on a fresh object, e.g.
    /// `FXwindows.Label("hi", 10, 20)` -> Object { text: "hi", x: 10, y: 20 }.
    /// Field values (and their order) match each block's usage in the docs.
    fn widget_constructor(class_name: &'static str, field_names: &'static [&'static str]) -> NativeFunction {
        NativeFunction {
            name: class_name.to_string(),
            arity: Some(field_names.len()),
            func: Rc::new(move |args| {
                let mut fields = HashMap::new();
                for (name, value) in field_names.iter().zip(args.iter()) {
                    fields.insert((*name).to_string(), value.clone());
                }
                Ok(Value::Object(ObjectValue {
                    class_name: class_name.to_string(),
                    fields,
                    methods: HashMap::new(),
                    native_methods: HashMap::new(),
                }))
            }),
        }
    }

    /// FLUX's second standard-library module: FXterminal. Lets a script
    /// open a real, terminal-styled console window and print lines into
    /// it while the program keeps running — a `create_console(...)` call
    /// returns a `Console` object; `.show()` opens the window and, unlike
    /// `FXwindows.window.show()`, returns immediately instead of blocking,
    /// so the `.print(...)` calls that follow keep feeding it new lines.
    ///
    /// With the `gui` feature disabled (or before `.show()` is called),
    /// `.print(...)` still works — it just writes to stdout instead of a
    /// window, prefixed with the console's title, same fallback spirit as
    /// FXwindows' headless placeholder.
    fn builtin_fxterminal_module(&self) -> Value {
        let mut fields = HashMap::new();

        fields.insert(
            "create_console".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "create_console".to_string(),
                arity: Some(3),
                func: Rc::new(|args| {
                    let title = args[0].as_string()?;
                    let width = args[1].as_integer()?;
                    let height = args[2].as_integer()?;

                    let mut console_fields = HashMap::new();
                    console_fields.insert("title".to_string(), Value::String(title.clone()));
                    console_fields.insert("width".to_string(), Value::Integer(width));
                    console_fields.insert("height".to_string(), Value::Integer(height));

                    #[cfg(feature = "gui")]
                    let gui_handle = flux_gui::ConsoleHandle::default();

                    let mut native_methods = HashMap::new();

                    // show() — opens the window. Non-blocking, unlike
                    // FXwindows: the script keeps running afterward.
                    {
                        #[cfg(feature = "gui")]
                        let gui_handle = gui_handle.clone();
                        let title = title.clone();
                        native_methods.insert(
                            "show".to_string(),
                            NativeFunction {
                                name: "show".to_string(),
                                arity: Some(1),
                                func: Rc::new(move |_args| {
                                    #[cfg(feature = "gui")]
                                    {
                                        let spec = flux_gui::ConsoleSpec {
                                            title: title.clone(),
                                            width: width as f32,
                                            height: height as f32,
                                        };
                                        flux_gui::open_console(spec, gui_handle.clone()).map_err(
                                            |e| RuntimeError::Exception {
                                                message: format!(
                                                    "failed to open FXterminal console: {e}"
                                                ),
                                            },
                                        )?;
                                    }
                                    #[cfg(not(feature = "gui"))]
                                    {
                                        println!(
                                            "[FXterminal] console '{title}' opened ({width}x{height}) — built without the 'gui' feature, output below will print to stdout."
                                        );
                                    }
                                    Ok(Value::None)
                                }),
                            },
                        );
                    }

                    // print(text) — appends one line.
                    {
                        #[cfg(feature = "gui")]
                        let gui_handle = gui_handle.clone();
                        #[cfg(not(feature = "gui"))]
                        let title = title.clone();
                        native_methods.insert(
                            "print".to_string(),
                            NativeFunction {
                                name: "print".to_string(),
                                arity: Some(2),
                                func: Rc::new(move |args| {
                                    let text = args[1].to_string();
                                    #[cfg(feature = "gui")]
                                    gui_handle.print_line(text);
                                    #[cfg(not(feature = "gui"))]
                                    println!("[{title}] {text}");
                                    Ok(Value::None)
                                }),
                            },
                        );
                    }

                    // clear() — wipes everything printed so far.
                    {
                        #[cfg(feature = "gui")]
                        let gui_handle = gui_handle.clone();
                        native_methods.insert(
                            "clear".to_string(),
                            NativeFunction {
                                name: "clear".to_string(),
                                arity: Some(1),
                                func: Rc::new(move |_args| {
                                    #[cfg(feature = "gui")]
                                    gui_handle.clear();
                                    Ok(Value::None)
                                }),
                            },
                        );
                    }

                    // close() — closes the window if it's open.
                    {
                        #[cfg(feature = "gui")]
                        let gui_handle = gui_handle.clone();
                        native_methods.insert(
                            "close".to_string(),
                            NativeFunction {
                                name: "close".to_string(),
                                arity: Some(1),
                                func: Rc::new(move |_args| {
                                    #[cfg(feature = "gui")]
                                    gui_handle.close();
                                    Ok(Value::None)
                                }),
                            },
                        );
                    }

                    Ok(Value::Object(ObjectValue {
                        class_name: "Console".to_string(),
                        fields: console_fields,
                        methods: HashMap::new(),
                        native_methods,
                    }))
                }),
            }),
        );

        Value::Object(ObjectValue {
            class_name: "module".to_string(),
            fields,
            methods: HashMap::new(),
            native_methods: HashMap::new(),
        })
    }

    #[cfg(feature = "gui")]
    fn get_f32(fields: &HashMap<String, Value>, key: &str, default: f32) -> f32 {
        match fields.get(key) {
            Some(Value::Integer(n)) => *n as f32,
            Some(Value::Float(n)) => *n as f32,
            _ => default,
        }
    }

    #[cfg(feature = "gui")]
    fn get_f64(fields: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
        match fields.get(key) {
            Some(Value::Integer(n)) => *n as f64,
            Some(Value::Float(n)) => *n,
            _ => default,
        }
    }

    #[cfg(feature = "gui")]
    fn get_string(fields: &HashMap<String, Value>, key: &str, default: &str) -> String {
        match fields.get(key) {
            Some(Value::String(s)) => s.clone(),
            Some(other) => other.to_string(),
            None => default.to_string(),
        }
    }

    #[cfg(feature = "gui")]
    fn get_bool(fields: &HashMap<String, Value>, key: &str, default: bool) -> bool {
        match fields.get(key) {
            Some(Value::Boolean(b)) => *b,
            _ => default,
        }
    }

    /// Builds a `flux_gui::WindowSpec` (a plain, FLUX-agnostic description
    /// of a window and its widgets) from a `Window` object's fields and
    /// `children` array, so `show()` can hand it straight to `flux-gui`.
    #[cfg(feature = "gui")]
    fn window_object_to_gui_spec(obj: &ObjectValue) -> flux_gui::WindowSpec {
        let mut widgets = Vec::new();

        if let Some(Value::Array(children)) = obj.fields.get("children") {
            for child in children.borrow().iter() {
                if let Value::Object(w) = child {
                    let widget = match w.class_name.as_str() {
                        "Button" => Some(flux_gui::Widget::Button {
                            text: Self::get_string(&w.fields, "text", "Button"),
                            width: Self::get_f32(&w.fields, "width", 100.0),
                            height: Self::get_f32(&w.fields, "height", 30.0),
                        }),
                        "Label" => Some(flux_gui::Widget::Label {
                            text: Self::get_string(&w.fields, "text", ""),
                            x: Self::get_f32(&w.fields, "x", 10.0),
                            y: Self::get_f32(&w.fields, "y", 10.0),
                        }),
                        "TextBox" => Some(flux_gui::Widget::TextBox {
                            placeholder: Self::get_string(&w.fields, "placeholder", ""),
                            x: Self::get_f32(&w.fields, "x", 10.0),
                            y: Self::get_f32(&w.fields, "y", 10.0),
                        }),
                        "Image" => Some(flux_gui::Widget::Image {
                            path: Self::get_string(&w.fields, "path", ""),
                            x: Self::get_f32(&w.fields, "x", 10.0),
                            y: Self::get_f32(&w.fields, "y", 10.0),
                        }),
                        "Panel" => Some(flux_gui::Widget::Panel {
                            x: Self::get_f32(&w.fields, "x", 10.0),
                            y: Self::get_f32(&w.fields, "y", 10.0),
                            width: Self::get_f32(&w.fields, "width", 100.0),
                            height: Self::get_f32(&w.fields, "height", 100.0),
                        }),
                        "Slider" => Some(flux_gui::Widget::Slider {
                            min: Self::get_f64(&w.fields, "min", 0.0),
                            max: Self::get_f64(&w.fields, "max", 100.0),
                            value: Self::get_f64(&w.fields, "value", 0.0),
                            x: Self::get_f32(&w.fields, "x", 10.0),
                            y: Self::get_f32(&w.fields, "y", 10.0),
                        }),
                        "Checkbox" => Some(flux_gui::Widget::Checkbox {
                            label: Self::get_string(&w.fields, "label", ""),
                            checked: Self::get_bool(&w.fields, "checked", false),
                            x: Self::get_f32(&w.fields, "x", 10.0),
                            y: Self::get_f32(&w.fields, "y", 10.0),
                        }),
                        _ => None,
                    };
                    if let Some(widget) = widget {
                        widgets.push(widget);
                    }
                }
            }
        }

        flux_gui::WindowSpec {
            title: Self::get_string(&obj.fields, "title", "FLUX Window"),
            width: Self::get_f32(&obj.fields, "width", 800.0),
            height: Self::get_f32(&obj.fields, "height", 600.0),
            widgets,
        }
    }

    fn matches_handler(&mut self, handler: &flux_ast::ExceptHandler, message: &str) -> FluxResult<bool> {
        let type_matches = match &handler.exception_type {
            None => true,
            Some(t) => message.contains(t) || t == "Exception",
        };

        if !type_matches {
            return Ok(false);
        }

        if let Some(binding) = &handler.binding {
            self.env
                .borrow_mut()
                .define(binding.clone(), Value::String(message.to_string()));
        }

        self.execute_block(&handler.body)?;
        Ok(true)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn interpret(source: &str) -> Result<Value, String> {
    let tokens = flux_lexer::lex(source).map_err(|e| e.to_string())?;
    let program = flux_parser::parse(tokens).map_err(|e| e.to_string())?;
    let mut interpreter = Interpreter::new();
    let result = interpreter.run(&program).map_err(|e| e.to_string());

    #[cfg(feature = "gui")]
    flux_gui::wait_for_consoles();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_hello_world() {
        let result = interpret(r#"PRINT "Hello World""#);
        assert!(result.is_ok());
    }

    #[test]
    fn runs_function() {
        let source = r#"
DEF add(a, b):
    RETURN a + b

result = add(2, 3)
"#;
        let tokens = flux_lexer::lex(source).unwrap();
        let program = flux_parser::parse(tokens).unwrap();
        let mut interpreter = Interpreter::new();
        interpreter.run(&program).unwrap();
        assert_eq!(
            interpreter.env.borrow().get("result"),
            Some(Value::Integer(5))
        );
    }
}
