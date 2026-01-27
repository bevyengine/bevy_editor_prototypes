# Architecture

Bevy's editor, like the rest of Bevy, is fundamentally designed to be modular, reusable, and robust to weird use patterns.
But at the same time, a new user's initial experience should be polished, reasonably complete and ready to jump into.

These goals are obviously in tension, and to thread the needle we need to think carefully about our project architecture, both organizationally and technically.

So far, we've agreed upon a general architecture for a MVP release:

- The Bevy Launcher is a binary application made using `bevy`, `bevy_feathers`, and `bevy_cli`
  - The launcher is the shippable binary
  - A non-programmer user will create new projects via the Launcher.
  - The launcher hooks into cli's such as `bevy_cli` and `rustup` to provide a GUI for common command line actions within bevy.

- All Bevy Engine tooling, including the Launcher and Editor, will live within the main repo, and must be updated and fixed as part of the ordinary development process.
  - As a large project, it represents a great proving ground for how changes will impact users
  - As an important part of Bevy users' workflow, it's important that we don't break it!
  - As a consequence, the Bevy editor cannot rely on any external crates that they themselves rely on `bevy`: they are not kept up to date with `main`
  - Fixing the editor before each release is much more difficult, requiring diverse expertise and a large volume of changes all at once

- Functionality that is useful without a graphical editor should be usable without the Editor
  - Project creation functionality should live in the `bevy_cli`, and be called by the Launcher
  - Asset preprocessing steps should be standalone tools whenever possible, which can then be called by the `bevy_cli` (and then the Editor)

- The foundational, reusable elements of the `bevy_editor` should be spun out into their own crates once they mature.
  - UI widgets, an undo-redo model, preferences, viewport widgets, a node graph abstraction and more are all great candidates for this approach.

- Self-contained GUI-based development tools should be self-contained `Plugin`s which can be reused by projects without requiring the whole Editor.
  - For example: an asset browser or system visualization tools
- The Editor should focus on the scene creation workflow
  - All development workflows that require live game interactions, such as running the game inside the editor window, create a exponentially harder architecture problem.
  - This is significantly simpler and offers more flexibility to users around customized workflows
  - We should still develop, ship, and promote powerful, polished tools for these use cases!

  ## Open questions

These questions are pressing, and need serious design work.

- How do we distribute the Bevy editor?
  - The Editor is built as a plugin within the user's project and we ship a binary application called `The Bevy Launcher` that sets up projects to use the Editor.
  - Explanation/Reasoning:
    - In order for the editor to display components (and other data) that a entity and by extension a BSN file, can be created with. The Editor must have access to the reflection type data. Rust currently has no built in reflection support and theres no tooling from which the editor could extract this data without being constrained by a validly compiled and running game project binary.
    - The only option that has the least risk, maintenance requirements, and time, is to have the Editor compiled together with the user's project allowing the Editor's type registry to have all the necessary data for creating and loading BSN files.
    - In the future we should look to break away from this hard requirement as if forces a specific file structure on users and isn't how editors traditionally work in game development.

- Do users need to have a local working copy of the Rust compiler to use the editor effectively?
  - Yes, everyone has to have a local working Rust compiler.

- How does the Bevy editor communicate with the Bevy game to enable effective scene editing with user-defined types?
  - The editor doesn't "communicate" with the bevy game, rather it is built with the game allowing the sharing of all reflection data in a way guaranteed for the editor to process/create all BSN for the game.

- How should undo-redo be handled?
  - TBD

## Extensions

These questions are less pressing, but deserve investigation to ensure we can support more advanced user needs.

- How do we allow users to add and remove non-trivial functionality to the editor?
  - Via the Launcher or via bevy's cargo features which the editor will be able to hook into on account of being apart of the bevy crate.
- How are preferences handled?
  - where are they stored?
    - Stored in two locations, a `bevy.toml` within each project's assets folder, and in the OS specific default location acting as Global defaults that the per project settings override.
  - how are they shared between projects?
    - The Global settings.
  - How are they shared between team members?
    - They can just commit the `bevy.toml` to their repo's
- Can the Bevy editor play nicely with a hot-patched type definitions for things like components?
  - Hot-patching isn't developed enough for the editor to work with it.
