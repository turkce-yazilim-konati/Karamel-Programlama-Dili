use crate::types::*;
use crate::compiler::*;
use std::cell::RefCell;
use std::sync::Arc;

#[cfg(not(feature = "unittest"))]
use crate::{debug_println};


pub struct StaticStorage {
    pub index                 : usize,
    pub constants             : Vec<VmObject>,
    pub constant_size         : u8,
    pub temp_size             : u8,
    pub temp_counter          : u8,
    pub variables             : Vec<(String, u8)>,
    pub memory                : Arc<RefCell<Vec<VmObject>>>,
    pub stack                 : Arc<RefCell<Vec<VmObject>>>,
    pub total_const_variables : u8,
    pub parent_location       : Option<usize>
}

/*
### STORAGE STRUCTURE ###
-------------------------
  CONSTANT VALUES
-------------------------
  VARIABLE VALUES
-------------------------
  TEMP VALUES
-------------------------
*/
impl StaticStorage {
    pub fn new() -> StaticStorage {
        StaticStorage {
            index: 0,
            constants: Vec::new(),
            constant_size: 0,
            temp_size: 0,
            temp_counter: 0,
            total_const_variables: 0,
            memory: Arc::new(RefCell::new(Vec::new())),
            stack: Arc::new(RefCell::new(Vec::new())),
            variables: Vec::new(),
            parent_location: None
        }
    }

    pub fn build(&mut self) {
        self.constant_size = self.constants.len() as u8;
        let mut memory = self.memory.borrow_mut();
        let mut stack  = self.stack.borrow_mut();

        /* Allocate memory */
        let memory_size = self.get_constant_size() 
                        + self.get_variable_size() 
                        + self.get_temp_size();
        
        memory.reserve(memory_size.into());

        /* Move all constants informations to memory location */
        memory.append(&mut self.constants);

        /*  Allocate variable memory and update references */
        let mut index = self.get_constant_size();
        for (_, value) in self.variables.iter_mut() {
            memory.push(VmObject::convert(Arc::new(BramaPrimative::Empty)));
            *value = index;
            index += 1;
        }

        for _ in 0..self.temp_size {
            stack.push(VmObject::convert(Arc::new(BramaPrimative::Empty)));
        }
    }
    pub fn get_memory(&mut self) -> Arc<RefCell<Vec<VmObject>>> { self.memory.clone() }
    pub fn get_stack(&mut self) -> Arc<RefCell<Vec<VmObject>>> { self.stack.clone() }
    pub fn get_constant_size(&self) -> u8 { self.constant_size }
    pub fn get_variable_size(&self) -> u8 { self.variables.len() as u8 }
    pub fn get_temp_size(&self) -> u8     { self.temp_size }
    
    pub fn set_parent_location(&mut self, parent_location: usize) {
        self.parent_location = Some(parent_location);
    }
    pub fn get_parent_location(&self) -> Option<usize> {
        self.parent_location
    }
    
    pub fn set_temp_size(&mut self, value: u8) { self.temp_size = value; }

    /// add variable and constant data same time. Assign constant location to variable reference.
    /// If variable or constant data already assigned before, it will try to update variable value with constant.
    pub fn add_static_data(&mut self, name: &str, value: Arc<BramaPrimative>) {
        let variable_location = self.add_variable(name);
        let constant_location = self.add_constant(value.clone());
        
        self.variables[variable_location as usize].1 = constant_location as u8;
    }

    pub fn add_constant(&mut self, value: Arc<BramaPrimative>) -> usize {
        let constant_position = self.constants.iter().position(|x| {
            *x.deref() == *value
        });
        
        match constant_position {
            Some(position) => position,
            None => {
                self.constants.push(VmObject::convert(value));
                self.constants.len() -1
            }
        }
    }

    pub fn add_variable(&mut self, name: &str) -> u8 {
        let result = self.variables.iter().position(|(key, _)| key == name);
        match result {
            Some(location) => self.variables[location].1,
            _ => {
                self.variables.push((name.to_string(), 0));
                0
            }
        }
    }

    pub fn get_variable_location(&self, name: &str) -> Option<u8> {
        let result = self.variables.iter().position(|(key, _)| key == name);
        match result {
            Some(location) => Some(self.variables[location].1),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn get_variable_value(&self, name: &str) -> Option<Arc<BramaPrimative>> {
        match self.get_variable_location(name) {
            Some(loc) => Some(self.memory.borrow_mut()[loc as usize].deref()),
            _ => None
        }
    }

    pub fn get_constant_location(&self, value: Arc<BramaPrimative>) -> Option<u8> {
        return match self.memory.borrow().iter().position(|x| { *x.deref() == *value }) {
            Some(number) => Some(number as u8),
            _ => None
        };
    }

    pub fn get_function_constant(&self, name: String, module_path: Vec<String>, framework: String) -> Option<u8> {
        
        for (index, item) in self.memory.borrow().iter().enumerate() {
            if let BramaPrimative::Function(reference, _) = &*item.deref() {
                if reference.name        == name && 
                   reference.module_path == module_path && 
                   reference.framework   == framework {
                    return Some(index as u8);
                }
            }
        }

        None
    }

    pub fn get_class_constant(&self, name: String, _module_path: Vec<String>, _framework: String) -> Option<u8> {
        
        for (index, item) in self.memory.borrow().iter().enumerate() {
            if let BramaPrimative::Class(reference) = &*item.deref() {
                if reference.get_class_name() == name {
                    return Some(index as u8);
                }
            }
        }

        None
    }

    #[cfg(feature = "unittest")]
    pub fn dump(&self) {}

    #[cfg(not(feature = "unittest"))]
    pub fn dump(&self) {
        debug_println!("╔════════════════════════════════════════╗");
        debug_println!("║               MEMORY DUMP              ║");
        debug_println!("╠═══╦═════╦══════════════════════════════╣");

        let consts    = self.constant_size;
        let variables = self.constant_size as usize + self.variables.len();

        for (index, item) in self.memory.borrow().iter().enumerate() {
            let last_type = if consts as usize == index {
                "V"
            } else if variables as usize == index {
                "T"
            } else {
                "C"
            };

            debug_println!("║ {} ║ {:3?} ║ {:28} ║", last_type, index, format!("{:?}", *item.deref()));
        }

        debug_println!("╚═══╩═════╩══════════════════════════════╝");
        debug_println!("╔════════════════════════════════════════╗");
        debug_println!("║             VARIABLE DUMP              ║");
        debug_println!("╠════════════════════════════════════════╣");
        for (variable, value) in &self.variables {
            debug_println!("║ {:38} ║", format!("{} [{}]", variable, value));
        }
        debug_println!("╚════════════════════════════════════════╝");

        debug_println!("╔════════════════════════════════════════╗");
        debug_println!("║                  STACK                 ║");
        debug_println!("╠════════════════════════════════════════╣");
        debug_println!("║ Stack size: {:<10}                 ║", self.stack.borrow().len());
        debug_println!("╠════════════════════════════════════════╣");
        for item in self.stack.borrow().iter() {
            debug_println!("║ {:38} ║", format!("{:?}", item.deref()));
        }
        debug_println!("╚════════════════════════════════════════╝");
    }
}
