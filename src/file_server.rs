use anyhow::{bail, Context, Result};
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MountPoint {
    pub route: String,
    pub fs_path: PathBuf,
    pub is_directory: bool,
}

#[derive(Debug)]
pub struct FileServer {
    pub mount_points: HashMap<String, MountPoint>,
}

impl Default for FileServer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileServer {
    pub fn new() -> Self {
        Self {
            mount_points: HashMap::new(),
        }
    }

    fn is_safe_relative_subpath(path: &Path) -> bool {
        !path.is_absolute() && path.components().all(|comp| comp != Component::ParentDir)
    }

    fn map(mut self, route: &str, fs_path: &str, is_directory: bool) -> Result<Self> {
        let route = route.trim_matches('/');

        let mount_point = MountPoint {
            route: route.to_owned(),
            fs_path: PathBuf::from(fs_path),
            is_directory,
        };

        if let Some(existing_mp) = self.mount_points.get(route) {
            bail!(
                "{route} has already been mapped to: {}",
                existing_mp.fs_path.display()
            );
        }

        self.mount_points.insert(route.to_owned(), mount_point);
        Ok(self)
    }

    pub fn map_dir(self, route: &str, dir_path: &str) -> Result<Self> {
        self.map(route, dir_path, true)
    }

    pub fn map_file(self, route: &str, file_path: &str) -> Result<Self> {
        self.map(route, file_path, false)
    }

    fn get_file_path(&self, file: &str) -> Result<PathBuf> {
        let file = file.trim_matches('/');
        if !Self::is_safe_relative_subpath(Path::new(file)) {
            bail!("file location is not safe: {file}");
        }

        let file_path = self
            .mount_points
            .values()
            .filter(|mp| !mp.is_directory)
            .find(|mp| mp.route == file)
            .map(|mp| mp.fs_path.clone());

        if let Some(file_path) = file_path {
            return Ok(file_path);
        }

        let dir_mount_point = self
            .mount_points
            .values()
            .filter(|mp| mp.is_directory)
            .find(|mp| file.starts_with(&mp.route));

        if let Some(dir_mount_point) = dir_mount_point {
            let file_name = file
                .strip_prefix(&dir_mount_point.route)
                .with_context(|| format!("file should have prefix: {}", dir_mount_point.route))?
                .trim_matches('/');

            return Ok(dir_mount_point.fs_path.join(file_name));
        }

        bail!("failed to get file path: {file}")
    }

    fn validate_file_exists(file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            bail!("file not found: {}", file_path.display());
        }

        if !file_path.is_file() {
            bail!("not a file: {}", file_path.display());
        }

        Ok(())
    }

    pub fn handle_file_access(&self, file: &str) -> Result<PathBuf> {
        let file_path = self.get_file_path(file)?;
        Self::validate_file_exists(&file_path)?;
        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::FileServer;

    fn get_dummy_file_server() -> FileServer {
        FileServer::new()
            .map_file("/favicon.ico", "assets/favicon.ico")
            .unwrap()
            .map_dir("static", "assets/")
            .unwrap()
    }

    #[test]
    fn test_map_dir_ok() {
        let fs = FileServer::new().map_dir("/static", "./relative/static");

        assert!(fs.is_ok());
    }

    #[test]
    fn test_map_dir_twice_err() {
        let fs = FileServer::new()
            .map_dir("/static", "/home/foo/absolute/static")
            .unwrap()
            .map_dir("/static", "relative/static");

        assert!(fs.is_err());
    }

    #[test]
    fn test_map_file_ok() {
        let fs =
            FileServer::new().map_file("/favicon.ico", "/home/foo/absolute/static/favicon.ico");

        assert!(fs.is_ok());
    }

    #[test]
    fn test_map_file_twice_err() {
        let fs = FileServer::new()
            .map_file("/favicon.ico", "/home/foo/absolute/static/favicon.ico")
            .unwrap()
            .map_file("/favicon.ico", "relative/static/favicon.ico");

        assert!(fs.is_err());
    }

    #[test]
    fn test_get_file_path_file_map_ok() {
        let fs = get_dummy_file_server();
        let actual_path = fs.get_file_path("/favicon.ico").unwrap();
        assert_eq!(PathBuf::from("assets/favicon.ico"), actual_path)
    }

    #[test]
    fn test_get_file_path_file_map_err() {
        let fs = get_dummy_file_server();
        let res = fs.get_file_path("/not-valid.txt");
        assert!(res.is_err());
    }

    #[test]
    fn test_get_file_path_dir_map_ok() {
        let fs = get_dummy_file_server();
        let actual_path = fs.get_file_path("/static/dog.png").unwrap();
        assert_eq!(PathBuf::from("assets/dog.png"), actual_path)
    }

    #[test]
    fn test_get_file_path_dir_map_upward_traversal_err() {
        let fs = get_dummy_file_server();
        let res = fs.get_file_path("/static/../dog.png");
        assert!(res.is_err());
    }

    #[test]
    fn test_get_file_path_dir_map_nesting_ok() {
        let fs = get_dummy_file_server();
        let actual_path = fs.get_file_path("/static/animals/snake.gif").unwrap();
        assert_eq!(PathBuf::from("assets/animals/snake.gif"), actual_path)
    }

    #[test]
    fn test_get_file_path_dir_map_nesting2_ok() {
        let fs = get_dummy_file_server();
        let actual_path = fs.get_file_path("static/animals/birds/dove.jpeg/").unwrap();
        assert_eq!(PathBuf::from("assets/animals/birds/dove.jpeg"), actual_path)
    }
}
