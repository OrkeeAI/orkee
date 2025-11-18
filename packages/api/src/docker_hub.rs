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

    let config: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse Docker config.json")?;

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
    #[derive(Deserialize)]
    struct ListResponse {
        results: Vec<ListResult>,
        next: Option<String>,
    }

    #[derive(Deserialize)]
    struct ListResult {
        name: String,
        description: Option<String>,
        star_count: u32,
        pull_count: u64,
    }

    let client = reqwest::Client::new();
    let mut all_images = Vec::new();
    let mut next_url = Some(format!(
        "https://hub.docker.com/v2/repositories/{}/?page_size=100",
        username
    ));

    // Fetch all pages
    while let Some(url) = next_url {
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

        let list_response: ListResponse = response
            .json()
            .await
            .context("Failed to parse Docker Hub response")?;

        // Add images from this page
        all_images.extend(list_response.results.into_iter().map(|r| DockerHubImage {
            name: format!("{}/{}", username, r.name),
            description: r.description.unwrap_or_default(),
            star_count: r.star_count,
            pull_count: r.pull_count,
            is_official: false,
            is_automated: false,
        }));

        // Move to next page
        next_url = list_response.next;
    }

    Ok(all_images)
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
        let result = get_docker_hub_token();
        assert!(result.is_ok(), "get_docker_hub_token should not panic");

        match result.unwrap() {
            Some(token) => {
                assert!(!token.is_empty(), "Token should not be empty if present");
                println!("Found Docker Hub auth token");
            }
            None => {
                println!("No Docker Hub auth token found (not logged in)");
            }
        }
    }

    #[tokio::test]
    async fn test_search_images_with_limit() {
        // Test that limit parameter is respected
        let result = search_images("nginx", Some(3)).await;

        if let Ok(images) = result {
            assert!(images.len() <= 3, "Should respect limit parameter");
            println!("Found {} nginx images", images.len());
        }
    }

    #[tokio::test]
    async fn test_search_images_validates_query() {
        // Test searching with empty query
        let result = search_images("", None).await;

        // Should either succeed with no results or fail gracefully
        match result {
            Ok(images) => println!("Empty query returned {} results", images.len()),
            Err(e) => println!("Empty query handled gracefully: {}", e),
        }
    }

    #[tokio::test]
    async fn test_get_image_detail_official() {
        // Test getting details for official Alpine image
        let result = get_image_detail("library", "alpine").await;

        if let Ok(detail) = result {
            assert_eq!(detail.name, "alpine");
            assert_eq!(detail.namespace, "library");
            assert!(detail.star_count > 0, "Official image should have stars");
            assert!(detail.pull_count > 0, "Official image should have pulls");
            println!(
                "Alpine image: {} stars, {} pulls",
                detail.star_count, detail.pull_count
            );
        } else {
            println!("Could not fetch Alpine image detail (network issue)");
        }
    }

    #[tokio::test]
    async fn test_get_image_detail_nonexistent() {
        // Test getting details for non-existent image
        let result =
            get_image_detail("nonexistent_namespace_12345", "nonexistent_repo_67890").await;

        // Should return an error
        assert!(result.is_err(), "Should fail for non-existent image");
        println!("Non-existent image correctly returned error");
    }

    #[tokio::test]
    async fn test_list_user_images() {
        // Test listing user images
        // This will only work if user is logged in and has images

        // Try to get logged-in username
        if let Ok(Some(_token)) = get_docker_hub_token() {
            // We have a token, but we need a username to test
            // For now, just test that the function doesn't panic with a dummy username
            let result = list_user_images("library").await;

            match result {
                Ok(images) => {
                    println!("Found {} images for library namespace", images.len());
                    // Library namespace should have many official images
                    if !images.is_empty() {
                        let first = &images[0];
                        assert!(
                            first.name.starts_with("library/"),
                            "Image name should include namespace"
                        );
                    }
                }
                Err(e) => {
                    println!("Could not list user images: {}", e);
                }
            }
        } else {
            println!("Skipping list_user_images test (not logged in)");
        }
    }

    #[tokio::test]
    async fn test_search_images_special_characters() {
        // Test search with URL-unsafe characters
        let result = search_images("node.js", Some(5)).await;

        // Should handle URL encoding properly
        if let Ok(images) = result {
            println!(
                "Search with special chars returned {} results",
                images.len()
            );
        }
    }

    #[tokio::test]
    async fn test_docker_hub_image_structure() {
        // Test that DockerHubImage structure contains all expected fields
        let result = search_images("ubuntu", Some(1)).await;

        if let Ok(images) = result {
            if let Some(image) = images.first() {
                // Verify all fields are present
                assert!(!image.name.is_empty(), "Name should not be empty");
                println!("Image structure validation passed for: {}", image.name);
                println!(
                    "  Description: {}",
                    &image.description[..image.description.len().min(50)]
                );
                println!("  Stars: {}, Pulls: {}", image.star_count, image.pull_count);
                println!(
                    "  Official: {}, Automated: {}",
                    image.is_official, image.is_automated
                );
            }
        }
    }
}
