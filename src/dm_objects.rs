use std::sync::Arc;
use dm::ast::FormatTreePath;
use dm::constants::{Constant, Pop};
use dm::objtree::{NodeIndex, ObjectTree, TypeProc, TypeRef, TypeVar};
use pyo3::iter::IterNextOutput::Return;
use pyo3::prelude::*;
use pyo3::types::*;

#[pyclass]
pub struct DmVariable {
    #[pyo3(get)]
    pub name: String,
    pub value: Option<Constant>,
}

impl DmVariable {
    pub fn new(name: String, var: &TypeVar) -> Self {
        return DmVariable {
            name: name,
            value: var.value.constant.clone()
        }
    }

    fn constant_to_pyobject(&self, py: Python, var: &Constant) -> PyResult<Py<PyAny>> {
        return match var {
            Constant::List(l) => self.list_to_pyobject(py, l),
            Constant::Float(f) => Ok(f.into_py(py)),
            Constant::Resource(s) => Ok(s.as_ref().into_py(py)),
            Constant::String(s) => Ok(s.as_ref().into_py(py)),
            _ => Ok(format!("{}", var).into_py(py))
        };
    }

    fn list_to_pyobject(&self, py: Python, list: &Box<[(Constant, Option<Constant>)]>) -> PyResult<Py<PyAny>> {
        let mut dict = PyDict::new(py);

        for (key, value) in list.as_ref() {
            let key = self.constant_to_pyobject(py, key)?;

            let value = if let Some(value) = value {
                self.constant_to_pyobject(py, value)?
            } else {
                py.None().into()
            };

            dict.set_item(key, value)?;
        }

        return Ok(dict.into_py(py));
    }
}

#[pymethods]
impl DmVariable {
    #[getter]
    fn has_value(&self) -> bool {
        return self.value.is_some();
    }

    fn value(&self, py: Python) -> PyResult<Py<PyAny>> {
        if let Some(var) = &self.value {
            return self.constant_to_pyobject(py, var);
        } else {
            return Ok(py.None().into());
        }
    }

    fn value_repr(&self) -> PyResult<String> {
        if let Some(var) = &self.value {
            return Ok(format!("{}", var));
        } else {
            return Ok("Null".to_string());
        }
    }

    fn value_is_num(&self) -> PyResult<bool> {
        if let Some(var) = &self.value {
            match var {
                Constant::Float(_) => Ok(true),
                _ => Ok(false)
            }
        } else {
            return Err(pyo3::exceptions::PyException::new_err("Variable has no value.".to_owned()));
        }
    }

    fn value_is_string(&self) -> PyResult<bool> {
        if let Some(var) = &self.value {
            match var {
                Constant::String(_) => Ok(true),
                _ => Ok(false)
            }
        } else {
            return Err(pyo3::exceptions::PyException::new_err("Variable has no value.".to_owned()));
        }
    }

    fn value_is_resource_literal(&self) -> PyResult<bool> {
        if let Some(var) = &self.value {
            match var {
                Constant::Resource(_) => Ok(true),
                _ => Ok(false)
            }
        } else {
            return Err(pyo3::exceptions::PyException::new_err("Variable has no value.".to_owned()));
        }
    }

    fn value_is_list(&self) -> PyResult<bool> {
        if let Some(var) = &self.value {
            match var {
                Constant::List(_) => Ok(true),
                _ => Ok(false)
            }
        } else {
            return Err(pyo3::exceptions::PyException::new_err("Variable has no value.".to_owned()));
        }
    }

    fn value_is_null(&self) -> PyResult<bool> {
        if let Some(var) = &self.value {
            match var {
                Constant::Null(_) => Ok(true),
                _ => Ok(false)
            }
        } else {
            return Err(pyo3::exceptions::PyException::new_err("Variable has no value.".to_owned()));
        }
    }
}

#[pyclass]
pub struct DmProc {
    #[pyo3(get)]
    pub name: String
}

impl DmProc {
    pub fn new(name: String, proc: &TypeProc) -> Self {
        return DmProc {
            name: name,
        };
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DmObject {
    #[pyo3(get)]
    pub path: String,

    vars: Py<PyDict>,
    procs: Py<PyDict>,

    object_tree: Arc<ObjectTree>,
    node_index: NodeIndex,

    caches_set: bool,
}

impl DmObject {
    pub fn new(py: &Python, object_tree: Arc<ObjectTree>, type_ref: TypeRef) -> Self {
        let dm_object = DmObject{
            path: type_ref.path.to_owned(),
            vars: PyDict::new(*py).into(),
            procs: PyDict::new(*py).into(),
            object_tree: object_tree,
            node_index: type_ref.index(),
            caches_set: false
        };

        return dm_object;
    }

    pub fn is_root_node(&self) -> bool {
        return self.path.is_empty();
    }

    pub fn get_type_ref(&self) -> TypeRef {
        self.object_tree.expect(self.path.as_str())
    }

    fn ensure_type_cache_exists(& mut self, py: Python) {
        if self.caches_set {
            return;
        }

        self.caches_set = true;

        let vars_list = self.vars.as_ref(py);
        let procs_list = self.procs.as_ref(py);
        let type_ref = self.get_type_ref();

        let mut f = |current_type_ref: TypeRef| {
            for (name, var) in &current_type_ref.vars {
                let already_present = vars_list.contains(name).unwrap_or(false);

                if !already_present {
                    let var = DmVariable::new(name.to_string(), var);
                    let var = Py::new(py, var).unwrap();
                    vars_list.set_item(name, var).unwrap();
                }
            }

            for (name, proc) in &current_type_ref.procs {
                let already_present = procs_list.contains(name).unwrap_or(false);

                if !already_present {
                    let proc = DmProc::new(name.to_string(), proc);
                    let proc = Py::new(py, proc).unwrap();
                    procs_list.set_item(name, proc).unwrap();
                }
            }
        };

        type_ref.visit_parent_types(&mut f);
    }
}

#[pymethods]
impl DmObject {
    #[getter(is_root_node)]
    fn is_root_node_py(&self) -> PyResult<bool> {
        return Ok(self.is_root_node());
    }

    #[getter(vars)]
    fn get_vars(& mut self, py: Python) -> PyResult<&Py<PyDict>> {
        self.ensure_type_cache_exists(py);

        Ok(&self.vars)
    }

    #[getter(procs)]
    fn get_procs(& mut self, py: Python) -> PyResult<&Py<PyDict>> {
        self.ensure_type_cache_exists(py);

        Ok(&self.procs)
    }

    fn overrides_variable(& mut self, py: Python, name: String) -> PyResult<bool> {
        self.ensure_type_cache_exists(py);

        let type_ref = self.get_type_ref();

        if self.vars.as_ref(py).contains(&name).unwrap_or(false) {
            return Ok(type_ref.get().vars.contains_key(&name));
        } else {
            return Err(pyo3::exceptions::PyException::new_err("Type does not contain such a variable."));
        }
    }
}
