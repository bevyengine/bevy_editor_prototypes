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

## Stage 0: Hello Project

- [x] make a repo
- [x] add a README
- [x] add a CONTRIBUTING.md
- [x] add a basic Bevy project
- [x] define the broad plan for what we're doing
- [x] define basic constraints for how we're building the editor
- [x] define a shared vision for what the editor is (and is not)

## Stage 1: Technically an Editor

- [ ] can load scenes from disk
- [ ] can display scenes in a viewer
  - [ ] 2D
  - [ ] 3D
- [ ] simple camera controller
  - [ ] 2D
  - [ ] 3D
- [ ] gizmos can be toggled
  - [ ] UI outlines
  - [ ] lights
  - [ ] AABBs
- [ ] the list of entities and their components is shown in the inspector
  - [ ] supports hierarchy via a folding tree view
- [ ] existing component values can be modified using a graphical interface
- [ ] scenes can be saved back to disk

## Stage 2: Fundamentals of Editor-Game Interaction

- [ ] users can press a button in the editor, launching the game from the very beginning
- [ ] users can press a button in the editor, and their game will run, loading the currently active scene
- [ ] components can be added and removed, including components specific to the user's game
- [ ] entities can be spawned and despawned via a GUI
- [ ] entity hierarchies can be spawned and despawned via a GUI
- [ ] game-specific rendering techniques display correctly in the editor's scene editing

## Stage 3: UI/UX Foundations

- [ ] tooltips
- [ ] rebindable hotkeys
- [ ] configurable camera controls
- [ ] basic widgets
  - [ ] text entry
  - [ ] numeric entry
  - [ ] radio buttons
  - [ ] dropdown
  - [ ] checkboxes
  - [ ] scrollable lists
- [ ] resizable panes
- [ ] scene object picking: click on objects in the scene to select them in the inspector
- [ ] entities can be looked up by name in the inspector
