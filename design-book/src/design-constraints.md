# Design Constraints

To help ensure that we're building an editor that's both useful to end users and feasible to maintain,
we've developed a set of design constraints and goals. Please try to follow them when making and reviewing changes!
As we're developing, initial prototypes may bend or break the rules: just be mindful of how we can move towards the final desirable end goal.

Below each bullet point, motivation is given.

## Must

- The Bevy editor must be a Bevy application.
  - This is essential both for dogfooding and to ease development: Bevy contributors all know how to write Bevy!
- The Bevy editor must run on Windows, Mac and Linux.
  - These desktop platforms are the key targets, as they are used by professionals for in-depth creative work. Web is a stretch goal, mobile and console need substantially more design.

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
  - When multiple competing standards exist (or the standards are bad), we should offer preset control schemes that map directly to familiar tools.
- The Bevy editor should be stable, and recover from crashes in the game or the editor as gracefully as possible.
  - Losing work sucks a lot!
- Iteration times should be kept fast, using whatever means make sense for a given workflow.
  - Compiling Rust is slow, and more iterations speeds up work and leads to a better result.
- Version-controllable: assets and settings should be feasible to check into version control and be relatively robust to merge conflicts.
  - This makes sharing settings across teams easier, and reduces confusing and wasteful problems with consolodating changes.
- Integrate nicely with other related Bevy initiatives, namely dev tools and the Bevy CLI.
  - We should avoid duplicating work and stepping on each others toes.

## Should not

- The editor is not an asset or code creation tool (certainly not in the default configuration). If an established tool does a good job authoring content, focus on importing its output, not reinventing Blender, VSCode, Asesprite, and Reaper.  
  - We have limited resources, and this isn't a good use of them. Artists and professionals already know and prefer specialized tools.
  - Instead, the Bevy Editor is a scene creation tool with powerful Bevy-specific debugging capabilities, with limited asset-tweaking capabilities for quick and simple tasks.
- The Bevy Editor should not have a native look-and-feel, and should not use native menus.
  - This is a poor use of resources, leads to inconsistent aesthetics, and makes it harder to test and teach across platforms. Instead, focus on the useful behavior and design conventions.
  - File picking widgets are an exception: these are non-intrusive, good crates exist and are not central to the user experience.
- End users should not have to install or update Rust.
  - This is a relatively challenging task for non-technical users, and should be handled transparently by the installer if it's absolutely needed.

## Must not

- Users must not have to write Rust code or use a terminal to perform core workflows of the editor.
  - Gamedev teams have critical non-technical members (artists and designers) that need to be able to use the editor.
- 3rd party UI solutions are forbidden: `bevy_ui` and abstractions on top of it only.
  - Dogfooding and improving `bevy_ui` is a major goal. If something sucks, fix it upstream!
- No scripting languages.
  - This targets a very niche audience: Rust is safer and easier to write than C++ and pure artists won't write Lua either. Prefer GUI controls, and then defer to node-based logic for highly complex art/design workflows.
- The Bevy Editor is not a plugin for a third-party editor like Blender or LDTK. While tools like [Blenvy](https://github.com/kaosat-dev/Blenvy) can be very productive for a team making a specific game, they're not a replacement for the Bevy Editor.
  - The existence of a standalone Bevy Editor tool (even with relatively limited capabilities) is valuable for helping new users get comfortable with the idea of making games in Bevy and become a functional member of a team.
  - Each of these tools focuses on a single style of game: primarily 2D vs 3D.
  - Being able to see your scene *as it appears in the game* is a vital requirement for effective scene creation: third party tools will not have the appropriate level of integration and come with their own subtly different rendering engines.
  - These tools have their own specialized learning curve, and a huge quantity of unrelated functionality. This comes at a cost in terms of performance, learning curves and usability.
  - The editor components (cameras, transform gizmos, UI elements, inspector and more) are incredibly valuable for the Bevy ecosystem as a whole and will be reused in other tooling and Bevy projects.
  - While some functionality (primarily level editing) is shared, much of the more specialized functionality (the system graph visualizer and system stepping for example) is Bevy specific and requires tight integration.
  - Coupling Bevy's development to an external tool makes project management and the developer learning curve much more painful, and introduces a large amount of external risk as the projects we depend on can make serious breaking changes or go defunct without warning.

## Open questions

Some questions aren't decided yet, and are highly controversial!

- Should the Bevy Editor live in the Bevy repository (tracking `main`) or its own (tracking `latest`)?
  - Arguments for tracking `main`
    - There's no need to duplicate project management and CI processes.
    - New changes to Bevy can be used immediately.
    - Emergent editor needs can be addressed and tested in a single PR, without needing to wait a cycle.
    - The editor serves as a great stress test for all of Bevy's systems.
    - The editor will never require painful, dedicated migration work, as it must be fixed in each PR.
    - The editor can be used to aid feature development and testing of Bevy itself.
    - Users tracking `main` will have a functional, if unstable editor for their games.
    - Pulling in 3rd party Bevy dependencies is a large maintenance burden and risk.
    - Functionality that's essential to the editor is often essential to end-users, and should be upstreamed anyways.
  - Arguments for tracking `latest`
    - Project management is easier: separate PR queues, issues, and permissions
    - We can use third-party Bevy dependencies, avoiding reinventing the wheel and exploding scope.
    - Updating the editor during each stabilization period is a great way to stress test the migration guide.
    - Users can use unstable versions of the editor in their real projects.
