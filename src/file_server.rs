use anyhow::{anyhow, bail, Context, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
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

    fn map(mut self, route: &str, fs_path: &str, is_directory: bool) -> Result<Self> {
        let route = route.strip_suffix('/').unwrap_or(route);
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

    fn get_file(file_path: PathBuf) -> Result<PathBuf> {
        if !file_path.exists() {
            bail!("file not found: {}", file_path.display());
        }

        if !file_path.is_file() {
            bail!("not a file: {}", file_path.display());
        }

        Ok(file_path)
    }

    pub fn handle_file_access(&self, file: &str) -> Result<PathBuf> {
        let file_path = self
            .mount_points
            .values()
            .filter(|mp| !mp.is_directory)
            .find(|mp| mp.route == file)
            .map(|mp| mp.fs_path.clone());

        if let Some(file_path) = file_path {
            return FileServer::get_file(file_path);
        }

        let dir_mount_point = self
            .mount_points
            .values()
            .filter(|mp| mp.is_directory)
            .find(|mp| file.starts_with(&mp.route));

        if let Some(dir_mount_point) = dir_mount_point {
            let file_name = file
                .strip_prefix(&dir_mount_point.route)
                .with_context(|| format!("file should have prefix: {}", dir_mount_point.route))?;

            let safe_file_name = match Path::new(file_name).file_name() {
                Some(filename) => Ok(filename.to_owned()),
                None => Err(anyhow!("invalid file name: {file}")),
            }?;

            let file_path = dir_mount_point.fs_path.join(safe_file_name);
            return Self::get_file(file_path);
        }

        bail!("failed to map file: {file}")
    }
}

#[cfg(test)]
mod tests {
    use super::FileServer;

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
}
