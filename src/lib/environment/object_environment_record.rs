//! # Object Records
//!
//! Each object Environment Record is associated with an object called its binding object.
//! An object Environment Record binds the set of string identifier names that directly
//! correspond to the property names of its binding object.
//! Property keys that are not strings in the form of an `IdentifierName` are not included in the set of bound identifiers.
//! More info:  [Object Records](https://tc39.github.io/ecma262/#sec-object-environment-records)

use crate::{
    environment::{
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
    },
    js::{
        property::Property,
        value::{Value, ValueData},
    },
};
use gc::Gc;
use gc_derive::{Finalize, Trace};

#[derive(Debug, Trace, Finalize, Clone)]
pub struct ObjectEnvironmentRecord {
    pub bindings: Value,
    pub with_environment: bool,
    pub outer_env: Option<Environment>,
}

impl EnvironmentRecordTrait for ObjectEnvironmentRecord {
    fn has_binding(&self, name: &str) -> bool {
        if self.bindings.has_field(name) {
            if self.with_environment {
                // TODO: implement unscopables
            }
            true
        } else {
            false
        }
    }

    fn create_mutable_binding(&mut self, name: String, deletion: bool) {
        // TODO: could save time here and not bother generating a new undefined object,
        // only for it to be replace with the real value later. We could just add the name to a Vector instead
        let bindings = &mut self.bindings;
        let uninitialized = Gc::new(ValueData::Undefined);
        let mut prop = Property::new(uninitialized);
        prop.enumerable = true;
        prop.writable = true;
        prop.configurable = deletion;
        bindings.set_prop(name, prop);
    }

    fn create_immutable_binding(&mut self, _name: String, _strict: bool) -> bool {
        true
    }

    fn initialize_binding(&mut self, name: &str, value: Value) {
        // We should never need to check if a binding has been created,
        // As all calls to create_mutable_binding are followed by initialized binding
        // The below is just a check.
        debug_assert!(self.has_binding(&name));
        self.set_mutable_binding(name, value, false)
    }

    fn set_mutable_binding(&mut self, name: &str, value: Value, strict: bool) {
        debug_assert!(value.is_object() || value.is_function());

        let bindings = &mut self.bindings;
        bindings.update_prop(name, Some(value.clone()), None, None, Some(strict));
    }

    fn get_binding_value(&self, name: &str, strict: bool) -> Value {
        if self.bindings.has_field(name) {
            self.bindings.get_field(name)
        } else {
            if strict {
                // TODO: throw error here
                // Error handling not implemented yet
            }
            Gc::new(ValueData::Undefined)
        }
    }

    fn delete_binding(&mut self, name: &str) -> bool {
        self.bindings.remove_prop(name);
        true
    }

    fn has_this_binding(&self) -> bool {
        false
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Value {
        // Object Environment Records return undefined as their
        // WithBaseObject unless their withEnvironment flag is true.
        if self.with_environment {
            return self.bindings.clone();
        }

        Gc::new(ValueData::Undefined)
    }

    fn get_outer_environment(&self) -> Option<Environment> {
        match &self.outer_env {
            Some(outer) => Some(outer.clone()),
            None => None,
        }
    }

    fn set_outer_environment(&mut self, env: Environment) {
        self.outer_env = Some(env);
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Function
    }

    fn get_global_object(&self) -> Option<Value> {
        if let Some(outer) = &self.outer_env {
            outer.borrow().get_global_object()
        } else {
            None
        }
    }
}
