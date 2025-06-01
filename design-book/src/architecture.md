# Architecture

Bevy's editor, like the rest of Bevy, is fundamentally designed to be modular, reusable, and robust to weird use patterns.
But at the same time, a new user's initial experience should be polished, reasonably complete and ready to jump into.

These goals are obviously in tension, and to thread the needle we need to think carefully about our project architecture, both organizationally and technically.
The agreed-upon architecture is summarized below, with future extensions and critical open questions listed below.

- the Bevy editor is a binary application made using `bevy`, and `bevy_ui`
- the Bevy editor will live in the main Bevy repo, and must be updated and fixed as part of the ordinary development process
  - as a large binary project, it represents a great proving ground for how changes will impact users
  - as an important part of Bevy users' workflow, it's important that we don't break it!
  - as a consequence, the Bevy editor cannot rely on any crates that themselves rely on `bevy`: they are not kept up to date with `main`
  - fixing the editor before each release is much more difficult, requiring diverse expertise and a large volume of changes all at once
  - note: until we have an MVP: the editor work is done outside of the main repo, and tracks `main` on an as-updated basis
- functionality that is useful without a graphical editor should be usable without the editor
  - project creation functionality should live in the `bevy_cli`, and be called by the editor
  - asset preprocessing steps should be standalone tools whenever possible, which can then be called by the `bevy_cli` (and then the editor)
- the foundational, reusable elements of the `bevy_editor` should be spun out into their own crates once they mature, but are best prototyped inside of the code for the specific binary application
  - UI widgets, an undo-redo model, preferences, viewport widgets, a node graph abstraction and more all great candidates for this approach
- self-contained GUI-based development tools should be self-contained `Plugin`s which can be reused by projects without requiring the Bevy editor binary
  - for example: an asset browser, entity inspector or system visualization tools

  ## Open questions

- how do we distribute the Bevy editor?
  - do users need to have a local working copy of the Rust compiler to use the editor effectively?
- how does the Bevy editor communicate with the Bevy game?
  - during scene editing?
  - while the game is running?
- should the Bevy Editor be able to inspect users games, or should the users simply add modular dev tools to their own project?
  - we can always add a "run project" button that calls `cargo run`, but do we want to be fancier than that?

## Extensions

- do we need a launcher?
  - are users able to use versions of the Bevy editor that are newer (or older) than their current project?
- how do we allow users to add and remove non-trivial functionality to the editor?
  - for now they can just fork things,
- how are preferences handled?
  - where are they stored?
  - how are they shared between projects?
  - how are they shared between team members?
