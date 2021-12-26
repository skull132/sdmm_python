extern crate dreammaker as dm;

use pyo3::prelude::*;

mod dm_objects;
mod dm_object_tree;

// To read: https://pyo3.rs/v0.10.1/class.html
// hook-up example: https://github.com/SpaiR/StrongDMM/blob/master/src/sdmmparser/src/main.rs

#[pymodule]
/// A Python module implemented in Rust.
fn sdmm_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<dm_objects::DmObject>()?;
    m.add_class::<dm_objects::DmProc>()?;
    m.add_class::<dm_objects::DmVariable>()?;
    m.add_class::<dm_object_tree::DmObjectTree>()?;

    Ok(())
}