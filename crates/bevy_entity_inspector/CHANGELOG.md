# Changelog

## [Unreleased] - Event-Driven Inspector Refactor

### Added
- **Event-driven architecture**: Replaced hash-based change detection with granular `InspectorEvent` system
- **Component grouping**: Components are now organized by crate name in the tree UI (e.g., "bevy_transform", "my_game")
- **Visual styling system**: Different colors for entities, crate groups, components, and fields with reduced opacity for non-expandable items
- **Professional UI organization**: Tree structure similar to other ECS inspectors like Flecs
- **Efficient change tracking**: `EntityInspectorRows` now tracks added/removed/updated entities separately
- **Hash-based remote change detection**: Remote polling uses content hashing for efficient change detection
- **Component name simplification**: Components now display as "crate::Type" instead of full module paths
- **Public utilities**: Added `extract_crate_and_type()` function for component name processing
- **Node type system**: Added `TreeNodeType` enum for visual styling of different tree nodes
- **Comprehensive documentation**: Added detailed rustdoc for all public APIs
- **Performance optimizations**: UI only rebuilds when actual changes occur

### Changed
- **BREAKING**: `EntityInspectorRow` now includes `data_hash` field for change detection
- **BREAKING**: Component names in remote inspection now use crate-qualified format
- **BREAKING**: Tree structure now includes crate grouping level between entities and components
- **BREAKING**: `TreeNode` now includes `node_type` field for visual styling
- **Remote polling**: Improved to handle components without `ReflectDeserialize` support
- **Tree rebuilding**: Now preserves expansion states and only updates when necessary
- **Event handling**: Centralized in `handle_inspector_events` system with batched processing
- **Visual organization**: Components grouped by crate for better understanding of system ownership

### Fixed
- **Runtime panic**: Fixed `TreeState` resource initialization in `TreePlugin`
- **Initial population**: Remote data now properly detected as "added" on first load
- **UI flickering**: Eliminated unnecessary tree rebuilds through event-driven updates
- **Memory efficiency**: Reduced redundant data structures and improved change detection

### Performance Improvements
- **Reduced UI updates**: Only rebuilds tree when actual entity/component changes occur
- **Efficient change detection**: Hash-based comparison for remote data, key-based for local
- **Batched event processing**: Multiple changes processed in single update cycle
- **Preserved UI state**: Expansion and selection states maintained during rebuilds

### Documentation
- Added comprehensive module-level documentation
- Documented all public structs, enums, and functions
- Added usage examples and performance notes
- Created detailed change detection and event system documentation
