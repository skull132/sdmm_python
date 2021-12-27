use std::sync::Arc;
use dm::Context;
use dm::objtree::ObjectTree;
use pyo3::prelude::*;
use pyo3::{Py, PyResult, Python};
use pyo3::types::PyList;
use crate::dm_objects::DmObject;

#[pyclass]
pub struct DmObjectTree {
    objtree: Arc<ObjectTree>,
}

#[pymethods]
impl DmObjectTree {
    #[new]
    fn new(dme_path: String) -> PyResult<Self> {
        let objtree =
            Context::default()
                .parse_environment(dme_path.as_ref())
                .map_err(|e| pyo3::exceptions::PyException::new_err(e.description().to_owned()))?;

        return Ok(DmObjectTree{
            objtree: Arc::new(objtree),
        });
    }

    #[getter(source_node)]
    fn get_source_node(&self, py: Python) -> PyResult<Py<DmObject>> {
        let root_ref = self.objtree.root();
        let dm_object = DmObject::new(&py, self.objtree.clone(), root_ref);

        return Py::new(py, dm_object);
    }

    fn get_path(&self, py: Python, path: String) -> PyResult<Option<Py<DmObject>>> {
        if let Some(object) = self.objtree.find(path.as_str()) {
            let object = DmObject::new(&py, self.objtree.clone(), object);
            return Ok(Some(Py::new(py, object)?))
        } else {
            return Err(
                pyo3::exceptions::PyException::new_err(
                    format!("Path not found: {}", path)))
        }
    }

    fn get_immediate_subtypes(&self, py: Python, path: String) -> PyResult<Py<PyList>> {
        if let Some(type_ref) = self.objtree.find(path.as_str()) {
            let mut list = PyList::empty(py);

            for child in type_ref.children() {
                if child.parent_type().unwrap() == type_ref {
                    let dm_object = DmObject::new(&py, self.objtree.clone(), child);
                    let dm_object = Py::new(py, dm_object)?;
                    list.append(dm_object)?;
                }
            }

            return Ok(list.into_py(py));
        } else {
            return Err(
                pyo3::exceptions::PyException::new_err(
                    format!("Path not found: {}", path)));
        }
    }

    fn get_all_subtypes(&self, py: Python, path: String) -> PyResult<Py<PyList>> {
        if let Some(type_ref) = self.objtree.find(path.as_str()) {
            let mut list = PyList::empty(py);

            for child in type_ref.children() {
                let dm_object = DmObject::new(&py, self.objtree.clone(), child);
                let dm_object = Py::new(py, dm_object)?;
                list.append(dm_object)?;
            }

            return Ok(list.into_py(py));
        } else {
            return Err(
                pyo3::exceptions::PyException::new_err(
                    format!("Path not found: {}", path)));
        }
    }
}
