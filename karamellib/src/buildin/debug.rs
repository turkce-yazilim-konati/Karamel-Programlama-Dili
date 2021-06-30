use crate::buildin::{Module, Class};
use crate::compiler::function::{FunctionReference, NativeCall, NativeCallResult};
use crate::compiler::function::FunctionParameter;
use crate::compiler::value::EMPTY_OBJECT;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct DebugModule {
    methods: RefCell<HashMap<String, Rc<FunctionReference>>>,
    path: Vec<String>
}

impl Module for DebugModule {
    fn get_module_name(&self) -> String {
        "hataayıklama".to_string()
    }

    fn get_path(&self) -> &Vec<String> {
        &self.path
    }

    fn get_method(&self, name: &str) -> Option<Rc<FunctionReference>> {
        self.methods.borrow().get(name).map(|method| method.clone())
    }

    fn get_module(&self, _: &str) -> Option<Rc<dyn Module>> {
        None
    }

    fn get_methods(&self) -> Vec<(&String, Rc<FunctionReference>)> {
        match self.methods.try_borrow() {
            Ok(methods) => methods.iter().map(|(key, value)| (key, value.clone())).collect::<Vec<(&String, Rc<FunctionReference>)>>(),
            Err(_) => Vec::new()
        }
    }

    fn get_modules(&self) -> HashMap<String, Rc<dyn Module>> {
        HashMap::new()
    }

    fn get_classes(&self) -> Vec<Rc<dyn Class>> {
        Vec::new()
    }
}

impl DebugModule  {
    pub fn new() -> Rc<DebugModule> {
        let module = DebugModule {
            methods: RefCell::new(HashMap::new()),
            path: vec!["hataayıklama".to_string()]
        };

        let rc_module = Rc::new(module);
        rc_module.methods.borrow_mut().insert("doğrula".to_string(), FunctionReference::native_function(Self::assert as NativeCall, "doğrula".to_string(), rc_module.clone()));
        rc_module.clone()
    }

    pub fn assert(parameter: FunctionParameter) -> NativeCallResult {
        match parameter.length() {
            1 => {
                match parameter.iter().next().unwrap().deref().is_true() {
                    false => Err("Assert failed".to_string()),
                    true  => Ok(EMPTY_OBJECT)
                }
            },
            2 => {
                let mut iter = parameter.iter();
                let left = iter.next().unwrap().deref();
                let right = iter.next().unwrap().deref();
                match left == right {
                    false => Err(format!("Assert failed (left: {:?}, right: {:?})", left, right)),
                    true  => Ok(EMPTY_OBJECT)
                }
            },
            _ => Err("Assert failed".to_string())
        }
    }
}