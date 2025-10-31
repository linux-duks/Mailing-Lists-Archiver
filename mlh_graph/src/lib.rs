use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::str::FromStr;

use pyo3::exceptions::PyValueError;

pub mod mlhid;
pub mod mlhtypes;
pub use mlhid::MLHID;
pub use mlhtypes::NodeType;

/// gets the MLHID for a origin type.
#[pyfunction]
fn mlhid_from_person_identification(identification: &str) -> PyResult<String> {
    Ok(MLHID::from_person_identification(identification).to_string())
}

/// gets the MLHID for a origin type.
#[pyfunction]
fn mlhid_from_origin_url(origin: &str) -> PyResult<String> {
    Ok(MLHID::from_origin_url(origin).to_string())
}

/// gets the MLHID for a str content for a specific node_type
#[pyfunction]
fn mlhid_from_content_str(node_type: &str, content: &str) -> PyResult<String> {
    match NodeType::from_str(node_type) {
        Ok(ntype) => Ok(MLHID::from_content_str(ntype, content).to_string()),
        Err(e) => Err(PyValueError::new_err(e)),
    }
}

/// Mailing-Lists Heritage Graph supporting library
#[pymodule]
fn mlh_graph(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mlhid_from_person_identification, m)?)?;
    m.add_function(wrap_pyfunction!(mlhid_from_origin_url, m)?)?;
    m.add_function(wrap_pyfunction!(mlhid_from_content_str, m)?)?;

    // Inserting to sys.modules allows importing submodules nicely from Python

    // let sys = PyModule::import(py, "sys")?;
    // let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;
    // sys_modules.set_item("mlh_graph.mlhid", m.getattr("mlhid")?)?;
    // sys_modules.set_item("mlh_graph.mlhtypes", m.getattr("mlhtypes")?)?;

    Ok(())
}
