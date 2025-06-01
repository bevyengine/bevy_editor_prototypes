# Roadmap

Making an editor is a huge project! How do we even know where to begin?

Like usual, we're starting by building something small. While we won't be adopting any of the existing prototypes directly to avoid drowning in a new sprawling code-base, please look to them aggressively for inspiration on both features and implementations.

The stages contained below are a flexible, coarse resolution roadmap to give newcomers and onlookers a sense of the overall plan and progress.
If you'd like to add items to this roadmap, open a PR and ping `@alice-i-cecile` as needed for dispute resolution.
Try to keep each stage small, simple, and easily accomplished.
We can easily add more stages, but trying to jump ahead to a complex feature risks killing all momentum.
For actionable, well-scoped items, please open issues instead.

As you complete tasks, check them off here!
The tasks don't need to be done *well* to be considered completed here: the Bevy community is very good at cleaning up loose strings as we work.

This list is not "all of the features that the editor will have": other stages will follow as these features are added.
It's best not to plan too far in advance!

## Stage 0: Hello Project

After this milestone, we have a project that Bevy developers can work on together.

- [x] make a repo
- [x] add a README
- [x] add a CONTRIBUTING.md
- [x] add a basic Bevy project
- [x] define the broad plan for what we're doing
- [x] define basic constraints for how we're building the editor
- [x] define a shared vision for what the editor is (and is not)

## Stage 1: Read-only Scene Viewer

After this milestone, we have a useful if limited tool that lets users see how scenes would be rendered and represented in Bevy.

If this work is done to a high level of polish, we can consider providing it as clearly scoped standalone tool for Bevy users to experiment with and refine.

- [x] can load scenes from disk using a native file picking widget
- [x] can display scenes in a viewer
  - [x] 2D
  - [x] 3D
- [x] infinite grid
  - [x] 2D
  - [x] 3D
- [x] simple camera controller
  - [x] 2D
  - [x] 3D
- [ ] toggleable gizmos for physical entities that aren't visible in the scene itself
  - [ ] UI outlines
  - [ ] lights
  - [ ] AABBs
  - [ ] cameras
- [x] lists entities in the scene
  - [ ] supports hierarchy via a folding tree view
- [ ] select entities
  - [ ] look up entities by name
  - [ ] from the scene using picking
  - [x] from the inspector
  - [ ] show selected entities in the inspector
  - [ ] clearly show selected entities in the world via an outline
  - [ ] one entity
  - [ ] multiple entities
- [ ] components of selected entity are shown in the inspector with component values, including components specific to the user's game
- [ ] resources can be inspected, showing their values
- [ ] loaded assets can be inspected, providing basic information about them

## Stage 2: Basic Editing Capabilities

After this milestone, the scene viewer can be used to make very simple changes to scenes, allowing it to be used as a simple level editor tool.

Additionally, the inspector can and should be spun out and shipped as a helpful first-party dev tool.

- [ ] entities can be spawned
- [ ] components can be added or removed from entities
- [ ] resources can be added or removed
- [ ] the values of components and resources can be modified
- [ ] interactive transform gizmo in the viewport that modifies all selected objects
- [ ] scenes can be saved back to disk
  - .bsn integration would be ideal, but we can read and write .ron scene files well enough to make an MVP

## Stage 3: Technical Difficulties

After this milestone, we've solved the most critical technical challenges that may force large-scale refactors and rewrites,
and have the confidence to build more features without accumulating serious technical risk.

- [ ] undo-redo abstraction
  - [ ] architecture established
  - [ ] used for all modifications
- [ ] Bevy Editor can call the bevy_cli
- [ ] game-defined components can be edited and instantiated
- [ ] game-specific rendering techniques display correctly in the editor's scene editing
- [ ] a stand-alone Bevy editor executable or launcher can be shipped as a download
- [ ] asset hot reloading works inside of the editor, changing assets displayed when the underlying file is modified
- [ ] Bevy Editor users can press "Run Game", and an appropriate binary for the game will be run in a new window
- [ ] tooltips
- [ ] hotkeys
  - [ ] standardized framework to add these
  - [ ] centralized list for users to view the hotkeys
  - [ ] hotkeys for our actions
- [ ] keyboard-based UI navigation

## Stage 4: Design Time

After this milestone, we will have the raw components needed to quickly build out new UI-based features in a consistent, polished fashion.

Additionally, these widgets will be useful for Bevy's examples, users and other dev tooling.

- [ ] mockup demonstrating what an ideal out-of-the-box 1.0 editor would look like
- [ ] clear written and visual guidelines for exactly what the "default Bevy editor theme" looks like and cares about
- [ ] Bevy Editor themed widgets (built on top of headless widgets)
  - [ ] text entry
  - [ ] numeric entry
  - [ ] file path entry
  - [ ] radio buttons
  - [ ] dropdown
  - [ ] checkboxes
  - [ ] slider
  - [ ] color selector
  - [ ] image preview
  - [ ] progress bar
  - [ ] audio playback widget
- [ ] editor-specific widgets
  - [ ] resizable panes
  - [ ] scrollable tool detail panels
  - [ ] toolbar
  - [ ] Blender-style workspaces
  - [ ] context menus
  - [ ] command palette
  - [ ] toggleable fullscreen panes
- [ ] exclusively use these widgets in the editor itself

## Stage 5: End-to-end projects

After this milestone, users will be able to download the Bevy editor and use it to get started with Bevy.

The completion of this milestone represents the first true MVP of a "Bevy editor", and the point where we should consider shipping this as an experimental alpha editor to experienced Bevy users,
and moving it to live inside of the `bevyengine/bevy` repository.

- [ ] download the Bevy editor as a standalone executable
- [ ] create a new Bevy project
  - [ ] minimal / blank
  - [ ] from template
  - [ ] basic browsing for templates
- [ ] browse and open existing Bevy projects
- [ ] launch targets/profiles
  - [ ] launch a prebuilt binary (for non-technical users)
  - [ ] cargo workspace apps and examples (for developers using the editor from the codebase)
    - [ ] set feature flags
  - [ ] option to launch with a default profile when the editor opens

### Stage 6: Customization is Accessibility

After this milestone, users will be able to customize the editor in simple ways to make it meet their needs.

- [ ] preferences
  - [ ] hotkey rebinding
  - [ ] camera controls
  - [ ] preferred language
  - [ ] hidden and visible widgets
- [ ] pane layout persists across editor invocations
- [ ] themeable UI
  - [ ] light mode / dark mode
- [ ] localization framework (first-party: English-only)

## Uncategorized work

There's a great deal of further feature work that needs to be done beyond the current milestones.
They haven't been forgotten, and are listed in no particular order here:

- [ ] theming, especially light-mode
- [ ] system stepping support
- [ ] experimental code hotpatching works inside of the editor
- [ ] editor extensions, for adding custom functionality
- [ ] a central hub for hosting (and selling?) editor extensions and assets
- [ ] performance overlays of all kinds
- [ ] system graph visualizer
  - [ ] filter systems by data access
  - [ ] filter by system set
- [ ] systems affecting the selected entity
- [ ] Tracy-powered runtime profiling
- [ ] investigate a separate process model using Bevy Remote Protocol
- [ ] investigate code hot reloading and dynamic linking
- [ ] animation graph editor
- [ ] particle effects editor
- [ ] tilemap support
- [ ] edit the abstract syntax tree (AST) of the Bevy Scene Notation (.bsn) format
- [ ] in-game dev console
- [ ] MaterialX custom material creator
- [ ] basic audio editing
- [ ] graph visualization of component values
- [ ] entity clustering with a memory usage breakdown
- [ ] generated in-editor documentation for Component/Resource/Event types
- [ ] go to definition support that opens the relevant code in your IDE of choice
- [ ] event diagnostics
- [ ] centralized design methodology ala [A4 design](https://developer.blender.org/docs/handbook/design/a4_design/)

While there are also important engine features that will unblock or improve various parts of the editor (BSN! better screen reader integration!),
this should not be tracked here: the emphasis is on user-facing goals, not any particular path.
