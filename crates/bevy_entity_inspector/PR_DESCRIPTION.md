# Pull Request: Refactor Entity Inspector with Event-Driven System and Component Grouping

## Summary

This PR refactors the Bevy entity inspector from a hash-based polling system to an efficient event-driven architecture with organized component grouping by crate. The changes eliminate UI flickering, reduce unnecessary rebuilds, and provide a much better user experience similar to professional ECS inspectors.

## Key Changes

### 🎯 Event-Driven Architecture
- **New `InspectorEvent` enum**: Provides granular change types (entity added/removed/updated, component changes)
- **Centralized event handling**: Single `handle_inspector_events` system processes all changes
- **Efficient batching**: Multiple changes processed in single update cycle

### 📊 Improved Change Detection
- **Hash-based remote detection**: Uses content hashing to detect actual changes in remote data
- **Granular tracking**: Separate vectors for added/removed/updated entities
- **Initial population handling**: Properly detects first-time data load as "added" entities

### 🎨 Better Component Display
- **Simplified naming**: Components show as "crate::Type" instead of full module paths
- **Clear differentiation**: Easy to distinguish Bevy (`bevy_*`) vs custom components
- **Graceful degradation**: Components without reflection support still appear in tree

### 📦 Component Grouping by Crate
- **Automatic organization**: Components grouped under their crate names (e.g., "bevy_transform", "my_game")  
- **Professional UI**: Similar to ECS inspectors like Flecs with hierarchical component organization
- **Visual styling**: Different colors for entities, crate groups, components, and fields with reduced opacity for non-expandable items
- **Better understanding**: Clear separation shows which systems contribute components to entities

### ⚡ Performance Optimizations
- **Reduced UI rebuilds**: Only updates when actual changes occur
- **Preserved state**: Expansion and selection states maintained during rebuilds
- **Memory efficiency**: Eliminated redundant hash calculations and data structures

## Breaking Changes

1. **`EntityInspectorRow`**: Added `data_hash: Option<u64>` field
2. **Component naming**: Remote components now use "crate::Type" format

## Testing

- [x] Remote inspection works with cube_server example
- [x] Change detection properly identifies entity modifications
- [x] UI no longer flickers during updates
- [x] Component names display correctly with crate prefixes
- [x] Tree state preservation during rebuilds
- [x] Error handling for network and reflection issues

## Performance Impact

### Before
- Hash calculated for entire entity set every frame
- UI rebuilt whenever hash changed (including false positives)
- Frequent flickering and unnecessary work

### After
- Events only emitted when actual changes detected
- UI rebuilt only when necessary
- Smooth, responsive experience

## Migration Guide

For users with custom `EntityInspectorRow` creation:

```rust
// Before
EntityInspectorRow {
    name: "Entity Name".to_string(),
    components: component_map,
}

// After  
EntityInspectorRow {
    name: "Entity Name".to_string(),
    components: component_map,
    data_hash: None, // or Some(calculated_hash) for remote data
}
```

## Future Enhancements

This architecture enables future optimizations:
- True partial tree updates for individual entities
- Component-level change detection and updates
- More efficient network protocols
- Real-time entity monitoring

## Files Changed

- `src/lib.rs`: Core event system and change tracking
- `src/remote.rs`: Remote polling with hash-based change detection  
- `src/ui/tree.rs`: TreeState resource initialization
- `CHANGELOG.md`: Comprehensive change documentation

---

**Type**: Enhancement  
**Breaking**: Yes (minor API changes)  
**Performance**: Significant improvement  
**Documentation**: Comprehensive rustdoc added
