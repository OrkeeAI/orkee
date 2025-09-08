use super::{
    AuthToken, CloudError, CloudProvider, CloudResult, DownloadOptions,
    ListOptions, SnapshotId, SnapshotInfo, SnapshotMetadata, StorageUsage, UploadOptions,
};
use crate::auth::CloudCredentials;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::types::{Object, Tag};
use aws_sdk_s3::Client;
use bytes::Bytes;
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

/// AWS S3 cloud provider implementation
#[derive(Debug, Clone)]
pub struct S3Provider {
    bucket: String,
    region: String,
    endpoint: Option<String>,
    path_prefix: String,
}

impl S3Provider {
    /// Create a new S3 provider
    pub fn new(bucket: String, region: String) -> Self {
        Self {
            bucket,
            region,
            endpoint: None,
            path_prefix: "orkee-snapshots/".to_string(),
        }
    }

    /// Create S3 provider with custom endpoint (for S3-compatible services)
    pub fn new_with_endpoint(bucket: String, region: String, endpoint: String) -> Self {
        Self {
            bucket,
            region,
            endpoint: Some(endpoint),
            path_prefix: "orkee-snapshots/".to_string(),
        }
    }

    /// Create Cloudflare R2 provider
    pub fn new_r2(bucket: String, account_id: String) -> Self {
        Self {
            bucket,
            region: "auto".to_string(),
            endpoint: Some(format!("https://{}.r2.cloudflarestorage.com", account_id)),
            path_prefix: "orkee-snapshots/".to_string(),
        }
    }

    /// Set path prefix for snapshots
    pub fn with_path_prefix(mut self, prefix: String) -> Self {
        self.path_prefix = prefix;
        self
    }

    /// Create AWS S3 client from credentials
    async fn create_client(&self, credentials: &CloudCredentials) -> CloudResult<Client> {
        let aws_creds = match credentials {
            CloudCredentials::AwsCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                region: _,
            } => Credentials::new(
                access_key_id,
                secret_access_key,
                session_token.clone(),
                None,
                "orkee-cloud-sync",
            ),
            _ => {
                return Err(CloudError::Authentication(
                    "Invalid credentials for S3 provider".to_string(),
                ))
            }
        };

        let mut config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(self.region.clone()))
            .credentials_provider(aws_creds);

        if let Some(endpoint) = &self.endpoint {
            config_builder = config_builder.endpoint_url(endpoint);
        }

        let config = config_builder.load().await;
        Ok(Client::new(&config))
    }

    /// Generate S3 object key for snapshot
    fn get_object_key(&self, snapshot_id: &SnapshotId) -> String {
        format!("{}{}.snapshot", self.path_prefix, snapshot_id.0)
    }

    /// Parse snapshot ID from S3 object key
    fn parse_snapshot_id(&self, key: &str) -> Option<SnapshotId> {
        if let Some(filename) = key.strip_prefix(&self.path_prefix) {
            if let Some(id) = filename.strip_suffix(".snapshot") {
                return Some(SnapshotId(id.to_string()));
            }
        }
        None
    }

    /// Convert S3 object to SnapshotInfo
    async fn object_to_snapshot_info(
        &self,
        client: &Client,
        object: &Object,
    ) -> CloudResult<Option<SnapshotInfo>> {
        let key = object.key().unwrap_or_default();
        let snapshot_id = match self.parse_snapshot_id(key) {
            Some(id) => id,
            None => return Ok(None),
        };

        // Get object metadata to extract custom metadata
        let head_result = client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CloudError::Provider(format!("Failed to get object metadata: {}", e)))?;

        let metadata_str = head_result
            .metadata()
            .and_then(|m| m.get("orkee-metadata"))
            .ok_or_else(|| CloudError::InvalidMetadata)?;

        let metadata: SnapshotMetadata = serde_json::from_str(metadata_str)
            .map_err(|e| CloudError::Serialization(e))?;

        let last_accessed = object
            .last_modified()
            .and_then(|dt| {
                let timestamp = dt.as_secs_f64();
                let secs = timestamp as i64;
                let nsecs = ((timestamp - secs as f64) * 1_000_000_000.0) as u32;
                Utc.timestamp_opt(secs, nsecs).single()
            });

        Ok(Some(SnapshotInfo {
            id: snapshot_id,
            metadata,
            storage_path: key.to_string(),
            last_accessed,
            etag: head_result.e_tag().map(|s| s.to_string()),
        }))
    }

    /// Apply retry logic for S3 operations
    async fn with_retry<F, T, Fut>(&self, operation: F) -> CloudResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = CloudResult<T>>,
    {
        use backoff::{future::retry, ExponentialBackoff};

        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(std::time::Duration::from_secs(30)),
            ..Default::default()
        };

        retry(backoff, || async {
            match operation().await {
                Ok(result) => Ok(result),
                Err(CloudError::Network(_)) => Err(backoff::Error::transient(CloudError::Network(
                    "Network error, retrying".to_string(),
                ))),
                Err(e) => Err(backoff::Error::permanent(e)),
            }
        })
        .await
        .map_err(|e| CloudError::Provider(format!("Retry failed: {:?}", e)))
    }
}

#[async_trait]
impl CloudProvider for S3Provider {
    fn provider_name(&self) -> &'static str {
        "aws-s3"
    }

    async fn authenticate(&self, credentials: &CloudCredentials) -> CloudResult<AuthToken> {
        let client = self.create_client(credentials).await?;

        // Test authentication by listing bucket
        client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| CloudError::Authentication(format!("S3 authentication failed: {}", e)))?;

        // S3 uses long-lived credentials, so we create a token that represents successful auth
        Ok(AuthToken {
            token: "s3-authenticated".to_string(),
            expires_at: None,
            token_type: "S3Credentials".to_string(),
            scope: None,
        })
    }

    async fn test_connection(&self, _token: &AuthToken) -> CloudResult<bool> {
        // For S3, we already tested the connection during authentication
        Ok(true)
    }

    async fn upload_snapshot(
        &self,
        _token: &AuthToken,
        data: &[u8],
        metadata: SnapshotMetadata,
        options: UploadOptions,
    ) -> CloudResult<SnapshotId> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;
        let key = self.get_object_key(&metadata.id);

        // Serialize metadata for storage
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| CloudError::Serialization(e))?;

        // Prepare S3 metadata
        let mut s3_metadata = HashMap::new();
        s3_metadata.insert("orkee-metadata".to_string(), metadata_json);
        s3_metadata.insert("orkee-version".to_string(), metadata.version.to_string());
        s3_metadata.insert("orkee-project-count".to_string(), metadata.project_count.to_string());

        // Add custom metadata from options
        for (k, v) in &options.metadata {
            s3_metadata.insert(format!("orkee-custom-{}", k), v.clone());
        }

        // Prepare tags
        let mut tags = Vec::new();
        tags.push(Tag::builder().key("orkee-snapshot").value("true").build()?);
        tags.push(Tag::builder().key("orkee-version").value(metadata.version.to_string()).build()?);
        
        for (k, v) in &options.tags {
            tags.push(Tag::builder().key(k).value(v).build()?);
        }

        // Prepare data for closure
        let data_bytes = Bytes::from(data.to_vec());
        let bucket = self.bucket.clone();
        let storage_class_opt = options.storage_class.clone();
        
        self.with_retry(|| {
            let data_bytes = data_bytes.clone();
            let bucket = bucket.clone();
            let key = key.clone();
            let s3_metadata = s3_metadata.clone();
            let storage_class_opt = storage_class_opt.clone();
            let client = client.clone();
            
            async move {
                let mut put_object = client
                    .put_object()
                    .bucket(&bucket)
                    .key(&key)
                    .body(data_bytes.into())
                    .content_type("application/octet-stream")
                    .set_metadata(Some(s3_metadata));

                // Add storage class if specified
                if let Some(storage_class) = &storage_class_opt {
                    use aws_sdk_s3::types::StorageClass;
                    let sc = match storage_class.as_str() {
                        "STANDARD_IA" => StorageClass::StandardIa,
                        "ONEZONE_IA" => StorageClass::OnezoneIa,
                        "GLACIER" => StorageClass::Glacier,
                        "DEEP_ARCHIVE" => StorageClass::DeepArchive,
                        _ => StorageClass::Standard,
                    };
                    put_object = put_object.storage_class(sc);
                }

                put_object
                    .send()
                    .await
                    .map_err(|e| CloudError::Provider(format!("Failed to upload snapshot: {}", e)))
            }
        })
        .await?;

        // Apply tags separately (S3 limitation)
        if !tags.is_empty() {
            let tag_set = aws_sdk_s3::types::Tagging::builder()
                .set_tag_set(Some(tags))
                .build()?;

            self.with_retry(|| async {
                client
                    .put_object_tagging()
                    .bucket(&self.bucket)
                    .key(&key)
                    .tagging(tag_set.clone())
                    .send()
                    .await
                    .map_err(|e| CloudError::Provider(format!("Failed to apply tags: {}", e)))
            })
            .await?;
        }

        Ok(metadata.id)
    }

    async fn download_snapshot(
        &self,
        _token: &AuthToken,
        id: &SnapshotId,
        options: DownloadOptions,
    ) -> CloudResult<Vec<u8>> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;
        let key = self.get_object_key(id);

        let mut get_object = client
            .get_object()
            .bucket(&self.bucket)
            .key(&key);

        // Apply range if specified
        if let Some((start, end)) = options.byte_range {
            get_object = get_object.range(format!("bytes={}-{}", start, end));
        }

        // Apply conditional headers
        if let Some(etag) = &options.if_match {
            get_object = get_object.if_match(etag);
        }
        if let Some(etag) = &options.if_none_match {
            get_object = get_object.if_none_match(etag);
        }

        let result = self.with_retry(|| async {
            get_object
                .clone()
                .send()
                .await
                .map_err(|e| {
                    if e.to_string().contains("NoSuchKey") {
                        CloudError::SnapshotNotFound(id.0.clone())
                    } else {
                        CloudError::Provider(format!("Failed to download snapshot: {}", e))
                    }
                })
        })
        .await?;

        let body = result.body.collect().await
            .map_err(|e| CloudError::Network(format!("Failed to read response body: {}", e)))?;

        Ok(body.into_bytes().to_vec())
    }

    async fn list_snapshots(
        &self,
        _token: &AuthToken,
        options: ListOptions,
    ) -> CloudResult<Vec<SnapshotInfo>> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;

        let prefix = options.prefix
            .map(|p| format!("{}{}", self.path_prefix, p))
            .unwrap_or_else(|| self.path_prefix.clone());

        let mut list_objects = client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix);

        if let Some(max_results) = options.max_results {
            list_objects = list_objects.max_keys(max_results as i32);
        }

        let result = self.with_retry(|| async {
            list_objects
                .clone()
                .send()
                .await
                .map_err(|e| CloudError::Provider(format!("Failed to list snapshots: {}", e)))
        })
        .await?;

        let mut snapshots = Vec::new();
        
        for object in result.contents() {
            if let Some(info) = self.object_to_snapshot_info(&client, object).await? {
                    // Apply date filters
                    if let Some(after) = options.created_after {
                        if info.metadata.created_at < after {
                            continue;
                        }
                    }
                    if let Some(before) = options.created_before {
                        if info.metadata.created_at > before {
                            continue;
                        }
                    }

                    // Apply tag filters
                    if !options.tags.is_empty() {
                        let tags_match = self.check_tags_match(&client, &info.storage_path, &options.tags).await?;
                        if !tags_match {
                            continue;
                        }
                    }

                    snapshots.push(info);
                }
            }

        // Sort by creation date (newest first)
        snapshots.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));

        Ok(snapshots)
    }

    async fn get_snapshot_info(
        &self,
        _token: &AuthToken,
        id: &SnapshotId,
    ) -> CloudResult<SnapshotInfo> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;
        let key = self.get_object_key(id);

        let result = self.with_retry(|| async {
            client
                .head_object()
                .bucket(&self.bucket)
                .key(&key)
                .send()
                .await
                .map_err(|e| {
                    if e.to_string().contains("NoSuchKey") {
                        CloudError::SnapshotNotFound(id.0.clone())
                    } else {
                        CloudError::Provider(format!("Failed to get snapshot info: {}", e))
                    }
                })
        })
        .await?;

        let metadata_str = result
            .metadata()
            .and_then(|m| m.get("orkee-metadata"))
            .ok_or_else(|| CloudError::InvalidMetadata)?;

        let metadata: SnapshotMetadata = serde_json::from_str(metadata_str)
            .map_err(|e| CloudError::Serialization(e))?;

        let last_accessed = result.last_modified()
            .and_then(|dt| {
                let timestamp = dt.as_secs_f64();
                let secs = timestamp as i64;
                let nsecs = ((timestamp - secs as f64) * 1_000_000_000.0) as u32;
                Utc.timestamp_opt(secs, nsecs).single()
            });

        Ok(SnapshotInfo {
            id: id.clone(),
            metadata,
            storage_path: key,
            last_accessed,
            etag: result.e_tag().map(|s| s.to_string()),
        })
    }

    async fn delete_snapshot(&self, _token: &AuthToken, id: &SnapshotId) -> CloudResult<()> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;
        let key = self.get_object_key(id);

        self.with_retry(|| async {
            client
                .delete_object()
                .bucket(&self.bucket)
                .key(&key)
                .send()
                .await
                .map_err(|e| CloudError::Provider(format!("Failed to delete snapshot: {}", e)))
        })
        .await?;

        Ok(())
    }

    async fn get_storage_usage(&self, _token: &AuthToken) -> CloudResult<StorageUsage> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;

        let result = self.with_retry(|| async {
            client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&self.path_prefix)
                .send()
                .await
                .map_err(|e| CloudError::Provider(format!("Failed to get storage usage: {}", e)))
        })
        .await?;

        let mut total_size = 0u64;
        let mut snapshot_count = 0usize;
        let mut oldest: Option<DateTime<Utc>> = None;
        let mut newest: Option<DateTime<Utc>> = None;

        for object in result.contents() {
            if let Some(key) = object.key() {
                    if self.parse_snapshot_id(key).is_some() {
                        snapshot_count += 1;
                        total_size += object.size().unwrap_or(0) as u64;

                        if let Some(modified) = object.last_modified() {
                            let timestamp = modified.as_secs_f64();
                            let secs = timestamp as i64;
                            let nsecs = ((timestamp - secs as f64) * 1_000_000_000.0) as u32;
                            
                            if let Some(dt) = Utc.timestamp_opt(secs, nsecs).single() {
                                match oldest {
                                    None => oldest = Some(dt),
                                    Some(current_oldest) => {
                                        if dt < current_oldest {
                                            oldest = Some(dt);
                                        }
                                    }
                                }

                                match newest {
                                    None => newest = Some(dt),
                                    Some(current_newest) => {
                                        if dt > current_newest {
                                            newest = Some(dt);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

        Ok(StorageUsage {
            total_size_bytes: total_size,
            snapshot_count,
            oldest_snapshot: oldest,
            newest_snapshot: newest,
            quota_bytes: None, // S3 doesn't have explicit quotas
            available_bytes: None,
        })
    }

    async fn snapshot_exists(&self, _token: &AuthToken, id: &SnapshotId) -> CloudResult<bool> {
        let credentials = CloudCredentials::AwsCredentials {
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            region: self.region.clone(),
        };

        let client = self.create_client(&credentials).await?;
        let key = self.get_object_key(id);

        match client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(CloudError::Provider(format!("Failed to check snapshot existence: {}", e)))
                }
            }
        }
    }
}

impl S3Provider {
    /// Check if object tags match the filter
    async fn check_tags_match(
        &self,
        client: &Client,
        key: &str,
        filter_tags: &HashMap<String, String>,
    ) -> CloudResult<bool> {
        let result = client
            .get_object_tagging()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CloudError::Provider(format!("Failed to get object tags: {}", e)))?;

        for (filter_key, filter_value) in filter_tags {
            let found = result.tag_set().iter().any(|tag| {
                    tag.key() == filter_key &&
                    tag.value() == filter_value
                });
            if !found {
                return Ok(false);
            }
        }

        Ok(true)
    }
}