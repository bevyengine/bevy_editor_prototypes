use core::time::Duration;

use std::path::Path;

use bevy::{
    asset::{AssetPath, UntypedAssetId},
    platform::collections::HashMap,
    prelude::*,
};

/// Cache entry for a preview image at a specific resolution.
#[derive(Clone, Debug)]
pub struct PreviewCacheEntry {
    /// The preview image handle.
    pub image_handle: Handle<Image>,
    /// The asset ID that this preview is for.
    pub asset_id: UntypedAssetId,
    /// Resolution in pixels (max dimension).
    pub resolution: u32,
    /// Timestamp for cache invalidation.
    pub timestamp: Duration,
}

/// Cache for preview images to avoid re-rendering unchanged assets.
#[derive(Resource, Default)]
pub struct PreviewCache {
    /// Maps asset path with resolution suffix to cache entry.
    path_cache: HashMap<AssetPath<'static>, PreviewCacheEntry>,
    /// Maps asset ID to all resolutions for efficient lookup and cleanup.
    id_to_paths: HashMap<UntypedAssetId, Vec<AssetPath<'static>>>,
}

impl PreviewCache {
    /// Creates a new empty cache.
    pub fn new() -> Self {
        Self {
            path_cache: HashMap::new(),
            id_to_paths: HashMap::new(),
        }
    }

    /// Generates a cache path with resolution suffix.
    /// Format: "path/to/image_64x64.png"
    fn cache_path_for_resolution<'a>(path: &AssetPath<'a>, resolution: u32) -> AssetPath<'static> {
        let path_buf = path.path();
        let parent = path_buf.parent();
        let file_name = path_buf.file_name().and_then(|n| n.to_str());

        if let Some(file_name) = file_name {
            if let Some(dot_pos) = file_name.rfind('.') {
                let name_without_ext = &file_name[..dot_pos];
                let extension = &file_name[dot_pos..];
                let new_file_name = format!(
                    "{}_{}x{}{}",
                    name_without_ext, resolution, resolution, extension
                );

                if let Some(parent) = parent {
                    AssetPath::from(parent.join(new_file_name))
                } else {
                    AssetPath::from(new_file_name)
                }
            } else {
                let new_file_name = format!("{}_{}x{}", file_name, resolution, resolution);
                if let Some(parent) = parent {
                    AssetPath::from(parent.join(new_file_name))
                } else {
                    AssetPath::from(new_file_name)
                }
            }
        } else {
            let path_str = path.path().to_string_lossy();
            AssetPath::from(format!("{}_{}x{}", path_str, resolution, resolution))
        }
    }

    /// Extracts the original path and resolution from a cache path.
    fn parse_cache_path(path: &AssetPath<'static>) -> Option<(AssetPath<'static>, u32)> {
        let path_buf = path.path();
        let parent = path_buf.parent();
        let file_name = path_buf.file_name().and_then(|n| n.to_str())?;

        let (name_without_res, resolution) = Self::extract_resolution_from_filename(file_name)?;

        let original_path = if let Some(parent) = parent {
            parent.join(&name_without_res)
        } else {
            Path::new(&name_without_res).to_path_buf()
        };

        Some((AssetPath::from(original_path), resolution))
    }

    /// Extracts resolution from a filename.
    fn extract_resolution_from_filename(file_name: &str) -> Option<(String, u32)> {
        if let Some(dot_pos) = file_name.rfind('.') {
            let name_without_ext = &file_name[..dot_pos];
            let extension = &file_name[dot_pos..];

            if let Some(underscore_pos) = name_without_ext.rfind('_') {
                if let Some(resolution) =
                    Self::parse_resolution_suffix(&name_without_ext[underscore_pos + 1..])
                {
                    let original_name =
                        format!("{}{}", &name_without_ext[..underscore_pos], extension);
                    return Some((original_name, resolution));
                }
            }
        } else {
            if let Some(underscore_pos) = file_name.rfind('_') {
                if let Some(resolution) =
                    Self::parse_resolution_suffix(&file_name[underscore_pos + 1..])
                {
                    let original_name = file_name[..underscore_pos].to_string();
                    return Some((original_name, resolution));
                }
            }
        }

        None
    }

    /// Parses a resolution suffix like "64x64" into a u32.
    fn parse_resolution_suffix(suffix: &str) -> Option<u32> {
        if let Some(x_pos) = suffix.find('x') {
            let width_str = &suffix[..x_pos];
            let height_str = &suffix[x_pos + 1..];

            if let (Ok(width), Ok(height)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
                if width == height && height_str.chars().all(|c| c.is_ascii_digit()) {
                    return Some(width);
                }
            }
        }
        None
    }

    /// Gets a cached preview by asset path and resolution.
    /// Returns highest resolution if resolution is None.
    pub fn get_by_path<'a>(
        &self,
        path: &AssetPath<'a>,
        resolution: Option<u32>,
    ) -> Option<&PreviewCacheEntry> {
        let owned_path: AssetPath<'static> = path.clone().into_owned();

        if let Some(res) = resolution {
            let cache_path = Self::cache_path_for_resolution(&owned_path, res);
            self.path_cache.get(&cache_path)
        } else {
            self.find_highest_resolution_for_path(&owned_path)
        }
    }

    /// Finds the highest resolution entry for a given path.
    fn find_highest_resolution_for_path(
        &self,
        path: &AssetPath<'static>,
    ) -> Option<&PreviewCacheEntry> {
        let mut best_entry: Option<&PreviewCacheEntry> = None;
        let mut best_resolution = 0u32;

        for (cache_path, entry) in &self.path_cache {
            if let Some((original_path, res)) = Self::parse_cache_path(cache_path) {
                if original_path.path() == path.path() && res > best_resolution {
                    best_resolution = res;
                    best_entry = Some(entry);
                }
            }
        }

        best_entry
    }

    /// Gets all cached previews for an asset path, sorted by resolution.
    pub fn get_all_by_path<'a>(&self, path: &AssetPath<'a>) -> Vec<&PreviewCacheEntry> {
        let owned_path: AssetPath<'static> = path.clone().into_owned();
        let mut entries = Vec::new();

        for (cache_path, entry) in &self.path_cache {
            if let Some((original_path, _)) = Self::parse_cache_path(cache_path) {
                if original_path.path() == owned_path.path() {
                    entries.push(entry);
                }
            }
        }

        entries.sort_by_key(|e| e.resolution);
        entries
    }

    /// Gets a cached preview by asset ID and resolution.
    /// Returns highest resolution if resolution is None.
    pub fn get_by_id(
        &self,
        asset_id: UntypedAssetId,
        resolution: Option<u32>,
    ) -> Option<&PreviewCacheEntry> {
        let paths = self.id_to_paths.get(&asset_id)?;

        if let Some(res) = resolution {
            for path in paths {
                if let Some(entry) = self.path_cache.get(path) {
                    if entry.resolution == res {
                        return Some(entry);
                    }
                }
            }
            None
        } else {
            let mut best_entry: Option<&PreviewCacheEntry> = None;
            let mut best_resolution = 0u32;

            for path in paths {
                if let Some(entry) = self.path_cache.get(path) {
                    if entry.resolution > best_resolution {
                        best_resolution = entry.resolution;
                        best_entry = Some(entry);
                    }
                }
            }

            best_entry
        }
    }

    /// Inserts a preview into the cache for a specific resolution.
    pub fn insert<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        asset_id: impl Into<UntypedAssetId>,
        resolution: u32,
        image_handle: Handle<Image>,
        timestamp: Duration,
    ) {
        let path: AssetPath<'static> = path.into().into_owned();
        let asset_id = asset_id.into();
        let cache_path = Self::cache_path_for_resolution(&path, resolution);

        let entry = PreviewCacheEntry {
            image_handle,
            asset_id,
            resolution,
            timestamp,
        };

        self.path_cache.insert(cache_path.clone(), entry);

        let paths = self.id_to_paths.entry(asset_id).or_insert_with(Vec::new);
        if !paths.contains(&cache_path) {
            paths.push(cache_path);
        }
    }

    /// Removes a preview from the cache by path and resolution.
    /// Removes all resolutions if resolution is None.
    pub fn remove_by_path<'a>(
        &mut self,
        path: &AssetPath<'a>,
        resolution: Option<u32>,
    ) -> Option<PreviewCacheEntry> {
        let owned_path: AssetPath<'static> = path.clone().into_owned();

        if let Some(res) = resolution {
            let cache_path = Self::cache_path_for_resolution(&owned_path, res);
            if let Some(entry) = self.path_cache.remove(&cache_path) {
                self.remove_path_from_id_mapping(&entry.asset_id, &cache_path);
                Some(entry)
            } else {
                None
            }
        } else {
            self.remove_all_resolutions_for_path(&owned_path)
        }
    }

    /// Removes all cache entries for a given path.
    fn remove_all_resolutions_for_path(
        &mut self,
        path: &AssetPath<'static>,
    ) -> Option<PreviewCacheEntry> {
        let mut removed_entry: Option<PreviewCacheEntry> = None;
        let mut asset_ids_to_check = Vec::new();

        let cache_paths: Vec<AssetPath<'static>> = self
            .path_cache
            .keys()
            .filter_map(|cache_path| {
                if let Some((original_path, _)) = Self::parse_cache_path(cache_path) {
                    if original_path.path() == path.path() {
                        return Some(cache_path.clone());
                    }
                }
                None
            })
            .collect();

        for cache_path in cache_paths {
            if let Some(entry) = self.path_cache.remove(&cache_path) {
                asset_ids_to_check.push(entry.asset_id);
                if removed_entry.is_none() {
                    removed_entry = Some(entry);
                }
            }
        }

        for asset_id in asset_ids_to_check {
            self.cleanup_id_mapping(&asset_id, path);
        }

        removed_entry
    }

    /// Removes a path from an asset ID's path list.
    fn remove_path_from_id_mapping(
        &mut self,
        asset_id: &UntypedAssetId,
        cache_path: &AssetPath<'static>,
    ) {
        if let Some(paths) = self.id_to_paths.get_mut(asset_id) {
            paths.retain(|p| p != cache_path);
            if paths.is_empty() {
                self.id_to_paths.remove(asset_id);
            }
        }
    }

    /// Cleans up ID mapping for a removed asset.
    fn cleanup_id_mapping(
        &mut self,
        asset_id: &UntypedAssetId,
        original_path: &AssetPath<'static>,
    ) {
        if let Some(paths) = self.id_to_paths.get_mut(asset_id) {
            paths.retain(|cache_path| {
                if let Some((parsed_path, _)) = Self::parse_cache_path(cache_path) {
                    parsed_path.path() != original_path.path()
                } else {
                    true
                }
            });
            if paths.is_empty() {
                self.id_to_paths.remove(asset_id);
            }
        }
    }

    /// Removes a preview from the cache by asset ID and resolution.
    /// Removes all resolutions if resolution is None.
    pub fn remove_by_id(
        &mut self,
        asset_id: UntypedAssetId,
        resolution: Option<u32>,
    ) -> Option<PreviewCacheEntry> {
        let paths = self.id_to_paths.get(&asset_id)?.clone();

        if let Some(res) = resolution {
            for path in &paths {
                if let Some(entry) = self.path_cache.get(path) {
                    if entry.resolution == res {
                        let cache_path = path.clone();
                        if let Some(removed) = self.path_cache.remove(&cache_path) {
                            self.remove_path_from_id_mapping(&asset_id, &cache_path);
                            return Some(removed);
                        }
                    }
                }
            }
            None
        } else {
            let mut removed_entry: Option<PreviewCacheEntry> = None;

            for path in &paths {
                if let Some(entry) = self.path_cache.remove(path) {
                    if removed_entry.is_none() {
                        removed_entry = Some(entry);
                    }
                }
            }

            self.id_to_paths.remove(&asset_id);
            removed_entry
        }
    }

    /// Clears all cached previews.
    pub fn clear(&mut self) {
        self.path_cache.clear();
        self.id_to_paths.clear();
    }

    /// Returns the number of cached preview entries.
    pub fn len(&self) -> usize {
        self.path_cache.len()
    }

    /// Checks if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.path_cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use bevy::{
        asset::AssetPlugin,
        image::Image,
        prelude::*,
        render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        },
    };
    use tempfile::TempDir;

    use super::*;

    fn create_test_image(
        images: &mut Assets<Image>,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Handle<Image> {
        let pixel_data: Vec<u8> = (0..(width * height))
            .flat_map(|_| color.iter().copied())
            .collect();

        let image = Image::new_fill(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &pixel_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        images.add(image)
    }

    #[test]
    fn test_cache_path_parsing() {
        let path: AssetPath<'static> = "test.png".into();
        let cache_path = PreviewCache::cache_path_for_resolution(&path, 64);
        assert_eq!(cache_path.path().to_string_lossy(), "test_64x64.png");

        let parsed = PreviewCache::parse_cache_path(&cache_path);
        assert!(parsed.is_some());
        let (original, res) = parsed.unwrap();
        assert_eq!(original.path().to_string_lossy(), "test.png");
        assert_eq!(res, 64);

        // Test with path
        let path2: AssetPath<'static> = "path/to/image.jpg".into();
        let cache_path2 = PreviewCache::cache_path_for_resolution(&path2, 256);
        assert_eq!(
            cache_path2.path().file_name().unwrap().to_string_lossy(),
            "image_256x256.jpg"
        );

        let parsed2 = PreviewCache::parse_cache_path(&cache_path2);
        assert!(parsed2.is_some());
        let (original2, res2) = parsed2.unwrap();
        assert_eq!(
            original2.path().file_name().unwrap().to_string_lossy(),
            "image.jpg"
        );
        assert_eq!(res2, 256);
    }

    #[test]
    fn test_preview_cache_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let assets_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: assets_dir.display().to_string(),
                ..Default::default()
            })
            .init_resource::<PreviewCache>()
            .init_asset::<Image>();

        // Create test images
        let (handle1, handle2, asset_id1, asset_id2) =
            app.world_mut()
                .resource_scope(|_world, mut images: Mut<Assets<Image>>| {
                    let h1 = create_test_image(&mut images, 64, 64, [255, 0, 0, 255]);
                    let h2 = create_test_image(&mut images, 64, 64, [0, 255, 0, 255]);
                    let id1 = h1.id().untyped();
                    let id2 = h2.id().untyped();
                    (h1, h2, id1, id2)
                });

        let path1: AssetPath<'static> = "test1.png".into();
        let path2: AssetPath<'static> = "test2.png".into();

        let mut cache = app.world_mut().resource_mut::<PreviewCache>();

        // Test insertion and query by path
        cache.insert(
            &path1,
            asset_id1,
            256,
            handle1.clone(),
            Duration::from_secs(100),
        );
        cache.insert(
            &path2,
            asset_id2,
            256,
            handle2.clone(),
            Duration::from_secs(200),
        );

        assert_eq!(cache.len(), 2, "Cache should contain 2 entries");
        assert!(!cache.is_empty(), "Cache should not be empty");

        let entry1 = cache.get_by_path(&path1, None).unwrap();
        assert_eq!(entry1.asset_id, asset_id1, "Entry should match asset ID");
        assert_eq!(
            entry1.timestamp,
            Duration::from_secs(100),
            "Entry should match timestamp"
        );
        assert_eq!(
            entry1.resolution, 256,
            "Entry should have correct resolution"
        );

        // Test query by ID
        let entry1_by_id = cache.get_by_id(asset_id1, None).unwrap();
        assert_eq!(
            entry1_by_id.image_handle.id(),
            handle1.id(),
            "Entry by ID should match"
        );

        // Test consistency between path and ID queries
        let entry2_by_path = cache.get_by_path(&path2, None).unwrap();
        let entry2_by_id = cache.get_by_id(asset_id2, None).unwrap();
        assert_eq!(
            entry2_by_path.image_handle.id(),
            entry2_by_id.image_handle.id(),
            "Path and ID queries should return same entry"
        );

        // Test multi-resolution
        let handle1_64 = handle1.clone();
        cache.insert(&path1, asset_id1, 64, handle1_64, Duration::from_secs(150));
        assert_eq!(cache.len(), 3, "Cache should have 3 entries now");

        let entry1_64 = cache.get_by_path(&path1, Some(64)).unwrap();
        assert_eq!(entry1_64.resolution, 64, "64px entry should exist");
        let entry1_256 = cache.get_by_path(&path1, Some(256)).unwrap();
        assert_eq!(entry1_256.resolution, 256, "256px entry should exist");
        // None should return highest resolution
        let entry1_default = cache.get_by_path(&path1, None).unwrap();
        assert_eq!(
            entry1_default.resolution, 256,
            "None should return highest resolution"
        );

        // Test get_all_by_path
        let all_entries = cache.get_all_by_path(&path1);
        assert_eq!(all_entries.len(), 2, "Should have 2 resolutions for path1");
        assert_eq!(all_entries[0].resolution, 64);
        assert_eq!(all_entries[1].resolution, 256);

        // Test removal by path with specific resolution
        cache.remove_by_path(&path1, Some(64));
        assert_eq!(cache.len(), 2, "Cache should have 2 entries after removal");
        assert!(
            cache.get_by_path(&path1, Some(64)).is_none(),
            "64px entry should be removed"
        );
        assert!(
            cache.get_by_path(&path1, Some(256)).is_some(),
            "256px entry should still exist"
        );

        // Test removal by path (all resolutions)
        let removed = cache.remove_by_path(&path1, None);
        assert!(removed.is_some(), "Should have removed entry");
        assert_eq!(cache.len(), 1, "Cache should have 1 entry after removal");
        assert!(
            cache.get_by_path(&path1, None).is_none(),
            "Path should be removed"
        );
        assert!(
            cache.get_by_id(asset_id1, None).is_none(),
            "ID should be removed"
        );

        // Test removal by ID
        let removed2 = cache.remove_by_id(asset_id2, None);
        assert!(removed2.is_some(), "Should have removed entry");
        assert_eq!(cache.len(), 0, "Cache should be empty");
        assert!(cache.is_empty(), "Cache should be empty");

        // Test duplicate insertion (should overwrite)
        let handle1_clone = handle1.clone();
        let handle1_clone2 = handle1.clone();
        cache.insert(
            &path1,
            asset_id1,
            256,
            handle1_clone,
            Duration::from_secs(100),
        );
        cache.insert(
            &path1,
            asset_id1,
            256,
            handle1_clone2,
            Duration::from_secs(300),
        ); // Overwrite
        assert_eq!(
            cache.len(),
            1,
            "Cache should have 1 entry after duplicate insert"
        );
        assert_eq!(
            cache.get_by_path(&path1, Some(256)).unwrap().timestamp,
            Duration::from_secs(300),
            "Entry should be updated"
        );

        // Test clearing
        cache.clear();
        assert_eq!(cache.len(), 0, "Cache should be empty after clear");
        assert!(cache.is_empty(), "Cache should be empty after clear");
    }
}
