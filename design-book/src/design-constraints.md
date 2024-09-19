# Design Constraints

To help ensure that we're building an editor that's both useful to end users and feasible to maintain,
we've developed a set of design constraints and goals. Please try to follow them when making and reviewing changes!
As we're developing, initial prototypes may bend or break the rules: just be mindful of how we can move towards the final desirable end goal.

Below each bullet point, motivation is given.

## Must

- The editor must target the latest stable Bevy release.
  - This eases project management by separating the work across two repositories, and ensures that we aggressively test each Bevy release's migration path.
- The Bevy editor must be a Bevy application.
  - This is essential both for dogfooding and to ease development: Bevy contributors all know how to write Bevy!

## Should

- Reusable code (camera controllers, inspectors, UI widgets...) should be cleanly separated into dedicated, documented libraries.
  - This keeps our repository clean, and allows users to steal our work to use in their own projects and tooling.
- Widgets, styles, colors and interaction paradigms should be consistent across the editor.
  - This avoids duplication, makes changes easier and helps give the editor a cohesive look.
- The editor should be distributed as an executable, either directly from the Bevy website or via Steam.
  - This makes it easier for non-technical users to install.
- The editor should be configurable, and easily adapted to both different genres and specific games.
  - Every game needs a different set of features. We need to allow teams to expose or extend what they care about, and hide what they don't.
- Industry-standard conventions for controls, UX and so on should be followed unless there's a good reason not to.
  - Learning a new tool is hard enough: please don't make it any harder.
- The Bevy editor should be stable, and recover from crashes in the game or the editor as gracefully as possible.
  - Losing work sucks a lot!
- Iteration times should be kept fast, using whatever means make sense for a given workflow.
  - Compiling Rust is slow, and more iterations speeds up work and leads to a better result.
- Version-controllable: assets and settings should be feasible to check into version control and be relatively robust to merge conflicts.
  - This makes sharing settings across teams easier, and reduces confusing and wasteful problems with consolodating changes.

## Should not

- 3rd party Bevy dependencies should be used sparingly, and only with the consent of the maintainer.
  - Bevy's ecosystem is one of its strengths, and we should lean on it, but being incorporated into the editor is a huge maintenance burden.
- The editor is not an asset or code creation tool (certainly not in the default configuration). If an established tool does a good job authoring content, focus on importing its output, not reinventing Blender, VSCode Asesprite and Reaper.
  - We have limited resources, and this isn't a good use of them. Artists and professionals already know and prefer specialized tools.
- The Bevy editor should not have a native look-and-feel.
  - This is a poor use of resources, and makes it harder to test across platforms. Instead, focus on the useful behavior and design conventions.
- End users should not have to install or update Rust.
  - This is a relatively challenging task for non-technical users, and should be handled transparently by the installer if it's absolutely needed.

## Must not

- Users must not have to write Rust code or use a terminal to perform core workflows of the editor.
  - Gamedev teams have critical non-technical members (artists and designers) that need to be able to use the editor.
- 3rd party UI solutions are forbidden: `bevy_ui` and abstractions on top of it only.
  - Dogfooding and improving `bevy_ui` is a major goal. If something sucks, fix it upstream!
- No scripting languages.
  - This targets a very niche audience: Rust is safer and easier to write than C++ and pure artists won't write Lua either. Prefer GUI controls, and then defer to node-based logic for highly complex art/design workflows.
