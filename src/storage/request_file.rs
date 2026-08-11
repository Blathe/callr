use std::fs;
use std::path::Path;

use crate::model::request::RequestFile;
use crate::storage::error::StorageError;

pub fn load_request(path: &Path) -> Result<RequestFile, StorageError> {
    let content = fs::read_to_string(path)?;
    let request = toml::from_str(&content)?;
    Ok(request)
}

pub fn save_request(path: &Path, request: &RequestFile) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(request)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::request::KeyValue;

    #[test]
    fn serialized_toml_places_scalar_fields_before_tables() {
        // TOML requires root-level scalars (url, method, ...) to appear
        // before table headers ([meta], [auth], [body]) — anything after a
        // table header is parsed as belonging to that table instead. This
        // guards against ever regressing to field-declaration order, which
        // would silently produce unloadable files.
        let mut request = RequestFile::scratch_named("Get User");
        request.url = "https://httpbin.org/get".to_string();
        let content = toml::to_string_pretty(&request).unwrap();
        let url_pos = content.find("url =").expect("url field present");
        let meta_pos = content.find("[meta]").expect("meta table present");
        assert!(url_pos < meta_pos, "scalar fields must precede table headers:\n{content}");
    }

    #[test]
    fn round_trip_preserves_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("get_user.toml");

        let mut request = RequestFile::scratch_named("Get User");
        request.url = "https://httpbin.org/get".to_string();
        request.headers.push(KeyValue {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
            enabled: true,
        });

        save_request(&path, &request).unwrap();
        let loaded = load_request(&path).unwrap();

        assert_eq!(loaded.meta.name, request.meta.name);
        assert_eq!(loaded.url, request.url);
        assert_eq!(loaded.headers.len(), 1);
        assert_eq!(loaded.headers[0].name, "Accept");
    }

    #[test]
    fn sample_collection_fixtures_are_valid() {
        // Regression guard: caught a real bug where hand-written example
        // fixtures had table headers before scalar fields, which TOML
        // silently absorbs into the wrong table instead of erroring clearly.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fixtures = ["collections/users/get_user.toml", "collections/users/get_json.toml"];
        for rel in fixtures {
            let path = std::path::Path::new(manifest_dir).join(rel);
            if !path.exists() {
                continue;
            }
            load_request(&path).unwrap_or_else(|err| panic!("{} failed to load: {err}", path.display()));
        }
    }

    #[test]
    fn resaving_a_loaded_request_produces_an_identical_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("get_user.toml");

        let mut request = RequestFile::scratch_named("Get User");
        request.url = "https://httpbin.org/get".to_string();
        save_request(&path, &request).unwrap();

        let first_write = fs::read_to_string(&path).unwrap();
        let loaded = load_request(&path).unwrap();
        save_request(&path, &loaded).unwrap();
        let second_write = fs::read_to_string(&path).unwrap();

        assert_eq!(
            first_write, second_write,
            "loading and resaving a request should not change the file"
        );
    }
}
