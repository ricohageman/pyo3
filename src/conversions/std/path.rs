use crate::conversion::IntoPyObject;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::{ffi, FromPyObject, IntoPyObjectExt, PyAny, PyErr, PyObject, PyResult, Python};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[allow(deprecated)]
impl ToPyObject for Path {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_os_str().into_py_any(py).unwrap()
    }
}

// See osstr.rs for why there's no FromPyObject impl for &Path

impl FromPyObject<'_> for PathBuf {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        // We use os.fspath to get the underlying path as bytes or str
        let path = unsafe { ffi::PyOS_FSPath(ob.as_ptr()).assume_owned_or_err(ob.py())? };
        Ok(path.extract::<OsString>()?.into())
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for &Path {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for &Path {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static PY_PATH: GILOnceCell<PyObject> = GILOnceCell::new();
        PY_PATH
            .import(py, "pathlib", "Path")?
            .call((self.as_os_str(),), None)
    }
}

impl<'py> IntoPyObject<'py> for &&Path {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

#[allow(deprecated)]
impl ToPyObject for Cow<'_, Path> {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (**self).to_object(py)
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for Cow<'_, Path> {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        (*self).to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, Path> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, Path> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

#[allow(deprecated)]
impl ToPyObject for PathBuf {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (**self).to_object(py)
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for PathBuf {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        (*self).to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for PathBuf {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for &PathBuf {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        (**self).to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for &PathBuf {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyString, PyStringMethods};
    use crate::{IntoPyObject, IntoPyObjectExt, PyObject, Python};
    use std::borrow::Cow;
    use std::fmt::Debug;
    use std::path::{Path, PathBuf};

    #[test]
    #[cfg(not(windows))]
    fn test_non_utf8_conversion() {
        Python::with_gil(|py| {
            use std::ffi::OsStr;
            #[cfg(not(target_os = "wasi"))]
            use std::os::unix::ffi::OsStrExt;
            #[cfg(target_os = "wasi")]
            use std::os::wasi::ffi::OsStrExt;

            // this is not valid UTF-8
            let payload = &[250, 251, 252, 253, 254, 255, 0, 255];
            let path = Path::new(OsStr::from_bytes(payload));

            // do a roundtrip into Pythonland and back and compare
            let py_str = path.into_pyobject(py).unwrap();
            let path_2: PathBuf = py_str.extract().unwrap();
            assert_eq!(path, path_2);
        });
    }

    #[test]
    fn test_intopyobject_roundtrip() {
        Python::with_gil(|py| {
            fn test_roundtrip<'py, T>(py: Python<'py>, obj: T)
            where
                T: IntoPyObject<'py> + AsRef<Path> + Debug + Clone,
                T::Error: Debug,
            {
                let pyobject = obj.clone().into_bound_py_any(py).unwrap();
                let roundtripped_obj: PathBuf = pyobject.extract().unwrap();
                assert_eq!(obj.as_ref(), roundtripped_obj.as_path());
            }
            let path = Path::new("Hello\0\n🐍");
            test_roundtrip::<&Path>(py, path);
            test_roundtrip::<Cow<'_, Path>>(py, Cow::Borrowed(path));
            test_roundtrip::<Cow<'_, Path>>(py, Cow::Owned(path.to_path_buf()));
            test_roundtrip::<PathBuf>(py, path.to_path_buf());
        });
    }

    #[test]
    fn test_from_pystring() {
        Python::with_gil(|py| {
            let path = "Hello\0\n🐍";
            let pystring = PyString::new(py, path);
            let roundtrip: PathBuf = pystring.extract().unwrap();
            assert_eq!(roundtrip, Path::new(path));
        });
    }

    #[test]
    #[allow(deprecated)]
    fn test_intopy_string() {
        use crate::IntoPy;

        Python::with_gil(|py| {
            fn test_roundtrip<T>(py: Python<'_>, obj: T)
            where
                T: IntoPy<PyObject> + AsRef<Path> + Debug + Clone,
            {
                let pyobject = obj.clone().into_py(py).into_bound(py);
                let pystring = pyobject.downcast_exact::<PyString>().unwrap();
                assert_eq!(pystring.to_string_lossy(), obj.as_ref().to_string_lossy());
                let roundtripped_obj: PathBuf = pyobject.extract().unwrap();
                assert_eq!(obj.as_ref(), roundtripped_obj.as_path());
            }
            let path = Path::new("Hello\0\n🐍");
            test_roundtrip::<&Path>(py, path);
            test_roundtrip::<Cow<'_, Path>>(py, Cow::Borrowed(path));
            test_roundtrip::<Cow<'_, Path>>(py, Cow::Owned(path.to_path_buf()));
            test_roundtrip::<PathBuf>(py, path.to_path_buf());
        });
    }
}
