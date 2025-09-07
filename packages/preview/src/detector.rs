use crate::types::{
    Framework, PackageManager, PreviewError, PreviewResult, ProjectDetectionResult, ProjectType,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::debug;

/// Project detector for analyzing project structure and determining configuration
pub struct ProjectDetector;

impl ProjectDetector {
    /// Detect project type and configuration from a project root directory
    pub async fn detect_project<P: AsRef<Path>>(
        project_root: P,
    ) -> PreviewResult<ProjectDetectionResult> {
        let project_root = project_root.as_ref();

        debug!("Detecting project type in: {}", project_root.display());

        // Check if path exists
        if !project_root.exists() {
            return Err(PreviewError::DetectionFailed {
                reason: format!("Project path does not exist: {}", project_root.display()),
            });
        }

        // Check for Node.js project first
        if let Some(result) = Self::detect_nodejs_project(project_root).await? {
            return Ok(result);
        }

        // Check for Python project
        if let Some(result) = Self::detect_python_project(project_root).await? {
            return Ok(result);
        }

        // Check for static HTML project
        if let Some(result) = Self::detect_static_project(project_root).await? {
            return Ok(result);
        }

        // Default to unknown project type
        Ok(ProjectDetectionResult {
            project_type: ProjectType::Unknown,
            framework: None,
            package_manager: PackageManager::Npm,
            has_lock_file: false,
            dev_command: "echo 'Unknown project type'".to_string(),
            port: 3000,
            scripts: None,
        })
    }

    /// Detect Node.js project by checking for package.json
    async fn detect_nodejs_project(
        project_root: &Path,
    ) -> PreviewResult<Option<ProjectDetectionResult>> {
        let package_json_path = project_root.join("package.json");

        if !package_json_path.exists() {
            return Ok(None);
        }

        debug!("Found package.json, analyzing Node.js project");

        let package_json_content = fs::read_to_string(&package_json_path)?;
        let package_json: Value = serde_json::from_str(&package_json_content)
            .map_err(|e| PreviewError::DetectionFailed {
                reason: format!("Invalid package.json: {}", e),
            })?;

        // Extract scripts
        let scripts = package_json
            .get("scripts")
            .and_then(|s| s.as_object())
            .map(|scripts_obj| {
                scripts_obj
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect::<HashMap<String, String>>()
            });

        // Detect framework and project type
        let (project_type, framework) = Self::detect_framework(&package_json)?;

        // Check if this Node.js project has suitable dev scripts
        let has_suitable_scripts = Self::has_suitable_dev_scripts(&scripts);
        
        // If no suitable scripts and there's an index.html, let static detection handle it
        if !has_suitable_scripts && project_root.join("index.html").exists() {
            debug!("Node.js project without suitable dev scripts, checking for static alternative");
            return Ok(None);
        }

        // Determine dev command
        let dev_command = Self::determine_dev_command(&scripts, project_type);

        // Detect package manager
        let package_manager = Self::detect_package_manager(project_root)?;

        // Check for lock files
        let has_lock_file = Self::has_lock_file(project_root, package_manager)?;

        // Determine default port
        let port = Self::determine_default_port(project_type, &scripts);

        Ok(Some(ProjectDetectionResult {
            project_type,
            framework,
            package_manager,
            has_lock_file,
            dev_command,
            port,
            scripts,
        }))
    }

    /// Detect Python project by checking for requirements.txt or pyproject.toml
    async fn detect_python_project(
        project_root: &Path,
    ) -> PreviewResult<Option<ProjectDetectionResult>> {
        let requirements_path = project_root.join("requirements.txt");
        let pyproject_path = project_root.join("pyproject.toml");

        if !requirements_path.exists() && !pyproject_path.exists() {
            return Ok(None);
        }

        debug!("Found Python project indicators");

        Ok(Some(ProjectDetectionResult {
            project_type: ProjectType::Python,
            framework: Some(Framework {
                name: "Python".to_string(),
                version: None,
            }),
            package_manager: PackageManager::Npm, // Not applicable for Python
            has_lock_file: false,
            dev_command: "python3 -m http.server 8000".to_string(),
            port: 8000,
            scripts: None,
        }))
    }

    /// Detect static HTML project by checking for index.html
    async fn detect_static_project(
        project_root: &Path,
    ) -> PreviewResult<Option<ProjectDetectionResult>> {
        let index_path = project_root.join("index.html");

        if !index_path.exists() {
            return Ok(None);
        }

        debug!("Found index.html, treating as static project");

        Ok(Some(ProjectDetectionResult {
            project_type: ProjectType::Static,
            framework: Some(Framework {
                name: "Static HTML".to_string(),
                version: None,
            }),
            package_manager: PackageManager::Npm, // Not applicable for static
            has_lock_file: false,
            dev_command: "static-server".to_string(), // Will be handled by our static server
            port: 3000,
            scripts: None,
        }))
    }

    /// Detect framework from package.json dependencies
    fn detect_framework(package_json: &Value) -> PreviewResult<(ProjectType, Option<Framework>)> {
        let dependencies = package_json.get("dependencies").unwrap_or(&Value::Null);
        let dev_dependencies = package_json
            .get("devDependencies")
            .unwrap_or(&Value::Null);

        // Check for Next.js
        if Self::has_dependency(dependencies, "next")
            || Self::has_dependency(dev_dependencies, "next")
        {
            let version = Self::get_dependency_version(dependencies, "next")
                .or_else(|| Self::get_dependency_version(dev_dependencies, "next"));

            return Ok((
                ProjectType::Nextjs,
                Some(Framework {
                    name: "Next.js".to_string(),
                    version,
                }),
            ));
        }

        // Check for React
        if Self::has_dependency(dependencies, "react")
            || Self::has_dependency(dev_dependencies, "react")
        {
            let version = Self::get_dependency_version(dependencies, "react")
                .or_else(|| Self::get_dependency_version(dev_dependencies, "react"));

            return Ok((
                ProjectType::React,
                Some(Framework {
                    name: "React".to_string(),
                    version,
                }),
            ));
        }

        // Check for Vue
        if Self::has_dependency(dependencies, "vue") || Self::has_dependency(dev_dependencies, "vue")
        {
            let version = Self::get_dependency_version(dependencies, "vue")
                .or_else(|| Self::get_dependency_version(dev_dependencies, "vue"));

            return Ok((
                ProjectType::Vue,
                Some(Framework {
                    name: "Vue.js".to_string(),
                    version,
                }),
            ));
        }

        // Default to Node.js
        Ok((
            ProjectType::Node,
            Some(Framework {
                name: "Node.js".to_string(),
                version: None,
            }),
        ))
    }

    /// Check if a dependency exists in the dependencies object
    fn has_dependency(deps: &Value, dep_name: &str) -> bool {
        deps.as_object()
            .map(|obj| obj.contains_key(dep_name))
            .unwrap_or(false)
    }

    /// Get version of a dependency
    fn get_dependency_version(deps: &Value, dep_name: &str) -> Option<String> {
        deps.as_object()
            .and_then(|obj| obj.get(dep_name))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Check if the project has suitable development scripts
    fn has_suitable_dev_scripts(scripts: &Option<HashMap<String, String>>) -> bool {
        if let Some(scripts_map) = scripts {
            let dev_scripts = ["dev", "start", "serve"];
            return dev_scripts.iter().any(|script| scripts_map.contains_key(*script));
        }
        false
    }

    /// Determine the development command based on project type and available scripts
    fn determine_dev_command(
        scripts: &Option<HashMap<String, String>>,
        project_type: ProjectType,
    ) -> String {
        if let Some(scripts_map) = scripts {
            // Check for common dev script names in order of preference
            let dev_scripts = ["dev", "start", "serve"];

            for script_name in &dev_scripts {
                if scripts_map.get(*script_name).is_some() {
                    return format!("npm run {}", script_name);
                }
            }
        }

        // Fallback based on project type
        match project_type {
            ProjectType::Nextjs => "npm run dev".to_string(),
            ProjectType::React => "npm start".to_string(),
            ProjectType::Vue => "npm run serve".to_string(),
            ProjectType::Node => "npm start".to_string(),
            ProjectType::Python => "python3 -m http.server 8000".to_string(),
            ProjectType::Static => "static-server".to_string(),
            ProjectType::Unknown => "npm start".to_string(),
        }
    }

    /// Detect package manager by checking for lock files
    fn detect_package_manager(project_root: &Path) -> PreviewResult<PackageManager> {
        if project_root.join("bun.lockb").exists() {
            return Ok(PackageManager::Bun);
        }

        if project_root.join("pnpm-lock.yaml").exists() {
            return Ok(PackageManager::Pnpm);
        }

        if project_root.join("yarn.lock").exists() {
            return Ok(PackageManager::Yarn);
        }

        if project_root.join("package-lock.json").exists() {
            return Ok(PackageManager::Npm);
        }

        // Default to npm if no lock file is found
        Ok(PackageManager::Npm)
    }

    /// Check if the project has a lock file
    fn has_lock_file(project_root: &Path, package_manager: PackageManager) -> PreviewResult<bool> {
        let lock_file = match package_manager {
            PackageManager::Npm => "package-lock.json",
            PackageManager::Yarn => "yarn.lock",
            PackageManager::Pnpm => "pnpm-lock.yaml",
            PackageManager::Bun => "bun.lockb",
        };

        Ok(project_root.join(lock_file).exists())
    }

    /// Determine default port based on project type and scripts
    fn determine_default_port(
        project_type: ProjectType,
        scripts: &Option<HashMap<String, String>>,
    ) -> u16 {
        // Try to extract port from scripts if available
        if let Some(scripts_map) = scripts {
            for (_, script) in scripts_map {
                if let Some(port) = Self::extract_port_from_command(script) {
                    return port;
                }
            }
        }

        // Default ports by project type
        match project_type {
            ProjectType::Nextjs => 3000,
            ProjectType::React => 3000,
            ProjectType::Vue => 8080,
            ProjectType::Node => 3000,
            ProjectType::Python => 8000,
            ProjectType::Static => 3000,
            ProjectType::Unknown => 3000,
        }
    }

    /// Extract port number from a command string
    fn extract_port_from_command(command: &str) -> Option<u16> {
        // Simple regex patterns to extract port numbers from common command formats
        let patterns = [
            r"--port\s+(\d+)",
            r"-p\s+(\d+)",
            r"PORT=(\d+)",
            r":(\d+)",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(command) {
                    if let Some(port_match) = captures.get(1) {
                        if let Ok(port) = port_match.as_str().parse::<u16>() {
                            return Some(port);
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_detect_nextjs_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create a package.json with Next.js dependency
        let package_json = serde_json::json!({
            "name": "test-project",
            "scripts": {
                "dev": "next dev",
                "build": "next build"
            },
            "dependencies": {
                "next": "^14.0.0",
                "react": "^18.0.0"
            }
        });

        fs::write(
            project_root.join("package.json"),
            package_json.to_string(),
        )
        .unwrap();

        let result = ProjectDetector::detect_project(project_root)
            .await
            .unwrap();

        assert_eq!(result.project_type, ProjectType::Nextjs);
        assert_eq!(result.framework.as_ref().unwrap().name, "Next.js");
        assert_eq!(result.dev_command, "npm run dev");
        assert_eq!(result.port, 3000);
    }

    #[tokio::test]
    async fn test_detect_static_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create an index.html file
        fs::write(
            project_root.join("index.html"),
            "<html><body>Hello World</body></html>",
        )
        .unwrap();

        let result = ProjectDetector::detect_project(project_root)
            .await
            .unwrap();

        assert_eq!(result.project_type, ProjectType::Static);
        assert_eq!(result.framework.as_ref().unwrap().name, "Static HTML");
        assert_eq!(result.port, 3000);
    }

    #[tokio::test]
    async fn test_detect_python_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create a requirements.txt file
        fs::write(project_root.join("requirements.txt"), "flask==2.0.0\n").unwrap();

        let result = ProjectDetector::detect_project(project_root)
            .await
            .unwrap();

        assert_eq!(result.project_type, ProjectType::Python);
        assert_eq!(result.framework.as_ref().unwrap().name, "Python");
        assert_eq!(result.port, 8000);
    }
}