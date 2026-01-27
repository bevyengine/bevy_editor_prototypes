# Architecture

Bevy's editor, like the rest of Bevy, is fundamentally designed to be modular, reusable, and robust to weird use patterns.
But at the same time, a new user's initial experience should be polished, reasonably complete and ready to jump into.

These goals are obviously in tension, and to thread the needle we need to think carefully about our project architecture, both organizationally and technically.

So far, we've agreed upon the following architecture:

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
- the editor will not try and inspect the running process: instead, users will be encouraged to reuse modular dev tools from the editor, `bevy_dev_tools` and the broader ecosystem by compiling them into their own project under a per-project `dev_tools` feature flag
  - we should still develop, ship and promote powerful, polished tools for these use cases!
  - this is significantly simpler and offers more flexibility to users around customized workflows

  ## Open questions

These questions are pressing, and need serious design work.

- how do we distribute the Bevy editor?
  - do users need to have a local working copy of the Rust compiler to use the editor effectively?
- how does the Bevy editor communicate with the Bevy game to enable effective scene editing with user-defined types?
- how should undo-redo be handled?

## Extensions

These questions are less pressing, but deserve investigation to ensure we can support more advanced user needs.

- do we need a launcher?
  - are users able to use versions of the Bevy editor that are newer (or older) than their current project?
- how do we allow users to add and remove non-trivial functionality to the editor?
  - for now they can just fork things,
- how are preferences handled?
  - where are they stored?
  - how are they shared between projects?
  - how are they shared between team members?
- can the Bevy editor play nicely with a hotpatched type definitions for things like components?
