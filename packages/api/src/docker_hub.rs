// ABOUTME: Docker Hub API integration for searching images and fetching metadata
// ABOUTME: Uses Docker auth token from ~/.docker/config.json for authenticated requests

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Docker Hub image search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerHubImage {
    pub name: String,
    pub description: String,
    pub star_count: u32,
    pub pull_count: u64,
    pub is_official: bool,
    pub is_automated: bool,
}

/// Docker Hub search response
#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    name: String,
    description: String,
    star_count: u32,
    pull_count: u64,
    is_official: bool,
    is_automated: bool,
}

/// Docker Hub repository detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetail {
    pub name: String,
    pub namespace: String,
    pub repository_type: String,
    pub status: i32,
    pub description: String,
    pub is_private: bool,
    pub star_count: u32,
    pub pull_count: u64,
    pub last_updated: String,
}

/// Get Docker auth token from config file
fn get_docker_hub_token() -> Result<Option<String>> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let config_path = format!("{}/.docker/config.json", home);

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let config: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse Docker config.json")?;

    // Try to get auth token for Docker Hub
    if let Some(auths) = config.get("auths").and_then(|a| a.as_object()) {
        for (server, auth) in auths {
            if server.contains("index.docker.io") {
                if let Some(auth_str) = auth.get("auth").and_then(|a| a.as_str()) {
                    return Ok(Some(auth_str.to_string()));
                }
            }
        }
    }

    Ok(None)
}

/// Search Docker Hub for images
pub async fn search_images(query: &str, limit: Option<u32>) -> Result<Vec<DockerHubImage>> {
    let limit = limit.unwrap_or(25);
    let url = format!(
        "https://hub.docker.com/v2/search/repositories/?query={}&page_size={}",
        urlencoding::encode(query),
        limit
    );

    let client = reqwest::Client::new();
    let mut request = client.get(&url);

    // Add auth if available
    if let Some(token) = get_docker_hub_token()? {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .context("Failed to send request to Docker Hub")?;

    if !response.status().is_success() {
        anyhow::bail!("Docker Hub API returned error: {}", response.status());
    }

    let search_response: SearchResponse = response
        .json()
        .await
        .context("Failed to parse Docker Hub response")?;

    Ok(search_response
        .results
        .into_iter()
        .map(|r| DockerHubImage {
            name: r.name,
            description: r.description,
            star_count: r.star_count,
            pull_count: r.pull_count,
            is_official: r.is_official,
            is_automated: r.is_automated,
        })
        .collect())
}

/// Get detailed information about a specific image
pub async fn get_image_detail(namespace: &str, repository: &str) -> Result<ImageDetail> {
    let url = format!(
        "https://hub.docker.com/v2/repositories/{}/{}",
        namespace, repository
    );

    let client = reqwest::Client::new();
    let mut request = client.get(&url);

    // Add auth if available
    if let Some(token) = get_docker_hub_token()? {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .context("Failed to send request to Docker Hub")?;

    if !response.status().is_success() {
        anyhow::bail!("Docker Hub API returned error: {}", response.status());
    }

    let detail: ImageDetail = response
        .json()
        .await
        .context("Failed to parse Docker Hub response")?;

    Ok(detail)
}

/// List images for authenticated user
pub async fn list_user_images(username: &str) -> Result<Vec<DockerHubImage>> {
    let url = format!(
        "https://hub.docker.com/v2/repositories/{}/?page_size=100",
        username
    );

    let client = reqwest::Client::new();
    let mut request = client.get(&url);

    // Add auth if available
    if let Some(token) = get_docker_hub_token()? {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .context("Failed to send request to Docker Hub")?;

    if !response.status().is_success() {
        anyhow::bail!("Docker Hub API returned error: {}", response.status());
    }

    #[derive(Deserialize)]
    struct ListResponse {
        results: Vec<ListResult>,
    }

    #[derive(Deserialize)]
    struct ListResult {
        name: String,
        description: Option<String>,
        star_count: u32,
        pull_count: u64,
    }

    let list_response: ListResponse = response
        .json()
        .await
        .context("Failed to parse Docker Hub response")?;

    Ok(list_response
        .results
        .into_iter()
        .map(|r| DockerHubImage {
            name: format!("{}/{}", username, r.name),
            description: r.description.unwrap_or_default(),
            star_count: r.star_count,
            pull_count: r.pull_count,
            is_official: false,
            is_automated: false,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_images() {
        // Test searching for a popular image
        let result = search_images("alpine", Some(5)).await;

        // This test may fail if Docker Hub is unreachable
        if let Ok(images) = result {
            assert!(!images.is_empty());
            assert!(images.iter().any(|img| img.name.contains("alpine")));
        }
    }

    #[test]
    fn test_get_docker_hub_token() {
        // Test that we can attempt to get a token without panicking
        let _ = get_docker_hub_token();
    }
}
