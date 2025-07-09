# Add Entity Inspector with Component Grouping and Event-Driven Updates

## Objective

Bevy currently lacks a comprehensive, user-friendly entity inspector for runtime debugging and development. Developers need a way to:

- **Inspect entities and components** in real-time during development
- **Understand component organization** by seeing which crates contribute which components
- **Connect to remote applications** for debugging without modifying the target application
- **Navigate large entity hierarchies** efficiently with expandable tree views
- **See only relevant data** with clear visual indicators of what can be expanded

The `bevy_entity_inspector` crate provides a first-party entity inspector with modern UX patterns, designed specifically for Bevy's component model and reflection system.

## Solution

### Core Architecture

The inspector uses an **event-driven architecture** that replaces traditional polling-based systems:

- **`InspectorEvent` system**: Granular events for entity/component add/remove/update operations
- **Efficient change detection**: Hash-based diffing for remote data, key-based comparison for local data  
- **Minimal UI updates**: Only rebuilds tree when actual changes occur
- **State preservation**: Expansion and selection states maintained during rebuilds

### Component Grouping by Crate

Components are automatically organized by their crate origin for better comprehension:

```
Entity (42)
├── bevy_transform
│   ├── Transform (expandable - has field data)
│   │   ├── translation: Vec3(0.0, 0.0, 0.0) 
│   │   ├── rotation: Quat(0.0, 0.0, 0.0, 1.0)
│   │   └── scale: Vec3(1.0, 1.0, 1.0)
│   └── GlobalTransform
├── bevy_render
│   ├── Visibility (dimmed - no expandable data)
│   └── ViewVisibility (dimmed - no expandable data)
└── my_game
    └── Player
        ├── health: f32(100.0)
        └── level: u32(1)
```

### Visual Design System

- **Color-coded hierarchy**: Different colors for entities, crate groups, components, and fields
- **Expandability indicators**: Reduced opacity for components without field data
- **Professional disclosure triangles**: Clear expand/collapse indicators  
- **Responsive design**: Scrollable interface with hover effects

### Remote Inspection Support

- **`bevy_remote` integration**: Connect to any Bevy app with remote plugin enabled
- **Efficient networking**: Only transfers data when actual changes occur
- **Reflection-based**: Works with any component that implements `Reflect` and `ReflectDeserialize`
- **Graceful degradation**: Components without reflection support still appear in tree

### Key Features

- **Event-driven updates** for optimal performance
- **Component grouping by crate** for better organization  
- **Remote inspection** via `bevy_remote` (optional feature)
- **Tree-based UI** with expand/collapse functionality
- **Reflection support** for automatic component introspection
- **Modern theming** with dark/light theme support
- **Change tracking** with efficient diff algorithms
- **State persistence** during UI rebuilds

## Testing

### Basic Inspector Example
```bash
# Run basic inspector (empty until data source configured)
cargo run --example inspector -p bevy_entity_inspector
```

### Remote Inspection Testing  
```bash
# Terminal 1: Start target application with remote plugin
cargo run --example cube_server -p bevy_entity_inspector --features remote

# Terminal 2: Start inspector connected to remote app  
cargo run --example inspector -p bevy_entity_inspector --features remote
```


**Component Coverage Verified:**
- ✅ **Transform hierarchy**: `Transform`, `GlobalTransform`, `TransformTreeChanged`
- ✅ **Rendering components**: `Mesh3d`, `MeshMaterial3d`, `RenderEntity`, `SyncToRenderWorld`  
- ✅ **Lighting components**: `DirectionalLight`, `PointLight`, `SpotLight`, `ShadowSettings`
- ✅ **Camera components**: `Camera`, `Camera3d`, `CameraRenderGraph`, `Projection`
- ✅ **Visibility components**: `Visibility`, `ViewVisibility`, `InheritedVisibility`
- ✅ **Custom components**: User-defined components with reflection support

**Performance Testing:**
- ✅ **Large entity counts**: Tested with 1000+ entities in remote mode
- ✅ **Component field expansion**: Deep component field hierarchies
- ✅ **Real-time updates**: Live component value changes during gameplay
- ✅ **Network efficiency**: Minimal bandwidth usage during remote inspection

**UI/UX Validation:**
- ✅ **Expandability indicators**: Clear visual feedback for expandable vs non-expandable components
- ✅ **Crate grouping**: Proper organization of Bevy engine vs user components  
- ✅ **Tree navigation**: Smooth expand/collapse with state preservation
- ✅ **Performance**: Responsive UI even with large component trees

The inspector provides a professional debugging experience that scales from simple scenes to complex 3D applications with extensive lighting setups.
