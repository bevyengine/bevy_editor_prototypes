//! Tree building logic for the entity inspector.
//!
//! This module contains the logic for building tree structures from entity data,
//! including component grouping by crate and tree node creation.
//!
//! # Related Documentation
//!
//! - [`crate::events::EntityInspectorRows`] - Input data structure for tree building
//! - [`crate::reflection::extract_crate_and_type`] - Component name parsing utility
//! - [`crate::ui::TreeNode`] - Core tree node structure
//! - [`TreeNodeType`] - Enum for different node types and styling

use crate::events::{EntityInspectorRows, InspectorNodeData};
use crate::reflection::{extract_crate_and_type, extract_reflect_fields};
use crate::ui::{TreeConfig, TreeNode, TreeState};
use bevy::prelude::*;

/// Type of tree node for visual styling
#[derive(Clone, Debug, PartialEq)]
pub enum TreeNodeType {
    /// A root entity node
    Entity,
    /// A crate grouping node
    CrateGroup,
    /// A component node
    Component,
    /// A field or property node
    Field,
}

impl Default for TreeNodeType {
    fn default() -> Self {
        Self::Field
    }
}

/// A tree node component for the inspector UI
#[derive(Component, Clone, Debug)]
pub struct InspectorTreeNode {
    /// The base tree node data
    pub base: TreeNode,
    /// The specific inspector data for this node
    pub data: InspectorNodeData,
}

/// Builder for creating inspector tree structures from entity data
#[derive(Default)]
pub struct InspectorTreeBuilder {
    /// The current state of the tree, including nodes and root nodes
    pub tree_state: TreeState,
    /// Configuration for the tree view
    pub config: TreeConfig,
}

impl InspectorTreeBuilder {
    /// Creates a new tree builder with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom tree configuration
    pub fn with_config(mut self, config: TreeConfig) -> Self {
        self.config = config;
        self
    }

    /// Inserts a tree node into the builder
    pub fn insert(&mut self, id: String, node: TreeNode) {
        self.tree_state.nodes.insert(id, node);
    }

    /// Builds a tree structure from entity inspector data with component grouping by crate
    pub fn build_from_inspector_data(inspector_data: &EntityInspectorRows) -> Self {
        let mut builder = Self::new();

        info!(
            "build_from_inspector_data: Processing {} entities",
            inspector_data.rows.len()
        );

        for (entity, row) in &inspector_data.rows {
            let entity_id = format!("entity_{entity:?}");
            let entity_label = if row.name.is_empty() {
                format!("Entity {entity:?}")
            } else {
                format!("{} ({entity:?})", row.name)
            };

            info!(
                "build_from_inspector_data: Processing entity {:?} with {} components: {}",
                entity,
                row.components.len(),
                entity_label
            );

            // Group components by crate
            let mut components_by_crate = std::collections::BTreeMap::new();

            for (component_name, component_reflect) in &row.components {
                let (crate_name, type_name) = extract_crate_and_type(component_name);
                components_by_crate
                    .entry(crate_name)
                    .or_insert_with(Vec::new)
                    .push((type_name, component_name.clone(), component_reflect));
            }

            let mut entity_children = Vec::new();

            // Create crate group nodes
            for (crate_name, components) in components_by_crate {
                let crate_group_id = format!("{entity_id}_{crate_name}");
                entity_children.push(crate_group_id.clone());

                let mut crate_children = Vec::new();

                // Add components to this crate group
                for (type_name, _full_component_name, component_reflect) in components {
                    let component_node_id = format!("{crate_group_id}_{type_name}");
                    crate_children.push(component_node_id.clone());

                    // Extract fields from the component
                    let fields = extract_reflect_fields(component_reflect.as_ref());
                    let mut component_children = Vec::new();

                    // Add fields as children of the component
                    for (field_name, field_value) in fields {
                        let field_node_id = format!("{component_node_id}_field_{field_name}");
                        component_children.push(field_node_id.clone());

                        let field_node = TreeNode {
                            id: field_node_id,
                            label: format!("{field_name}: {field_value}"),
                            is_expanded: false,
                            children: Vec::new(),
                            parent: Some(component_node_id.clone()),
                            depth: 3,
                            node_type: TreeNodeType::Field,
                        };

                        builder
                            .tree_state
                            .nodes
                            .insert(field_node.id.clone(), field_node);
                    }

                    // Create component node with short type name
                    let component_node = TreeNode {
                        id: component_node_id,
                        label: type_name,
                        is_expanded: false,
                        children: component_children,
                        parent: Some(crate_group_id.clone()),
                        depth: 2,
                        node_type: TreeNodeType::Component,
                    };

                    builder
                        .tree_state
                        .nodes
                        .insert(component_node.id.clone(), component_node);
                }

                // Create crate group node
                let crate_node = TreeNode {
                    id: crate_group_id,
                    label: crate_name,
                    is_expanded: false,
                    children: crate_children,
                    parent: Some(entity_id.clone()),
                    depth: 1,
                    node_type: TreeNodeType::CrateGroup,
                };

                builder
                    .tree_state
                    .nodes
                    .insert(crate_node.id.clone(), crate_node);
            }

            // Create entity node
            let entity_node = TreeNode {
                id: entity_id.clone(),
                label: entity_label,
                is_expanded: false,
                children: entity_children,
                parent: None,
                depth: 0,
                node_type: TreeNodeType::Entity,
            };

            builder
                .tree_state
                .nodes
                .insert(entity_id.clone(), entity_node);
            builder.tree_state.root_nodes.push(entity_id);
        }

        builder
    }

    /// Creates a tree view UI from the builder's data
    pub fn build_tree_view(&self, commands: &mut Commands) -> Entity {
        crate::ui::tree::build_tree_view(commands, &self.tree_state, &self.config)
    }
}
