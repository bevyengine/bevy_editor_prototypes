# Vision

Bevy's fabled editor has been the subject of much debate and frustration over the years,
but to actually come together and build it, the team needs to align on a vision for its future.
What is an editor? Why are we building it? What will change? Why do people think that this will push Bevy from a mere "framework" to a "real game engine"?

Bevy's current strengths are simple but compelling:

- delightful ECS-based architecture: fast, flexible and frankly pleasant
- absurdly flexible: remove anything you want, and grab great new things from the ecosystem
- Rust all the way down, making refactoring, building, testing, distribution and improving things upstream painless

In the context of an "editor" though, the current "code-only" approach has several critical limitations that make it infeasible for all but the smallest and most technical of commercial gamedev teams:

- artists (illustrators, animators, sound effects, music) and designers (gameplay, systems, level) are entirely reliant on more technical team members to integrate and test their work in the game
- authoring game-specific content (custom materials, levels, designed content like items) requires custom tooling
- Bevy developers are left to piece together their own testing and debugging workflow
- Rust's compile times make some iteration-heavy tasks (like adjusting UI or tweaking lighting) painfully slow

The Bevy Editor isn't designed to supplant Rust, or to be an all-in-one game making app with everything from a tilemap editor to an IDE baked in.
Instead, it is a place to manage projects, an empowering frontend for artists, a scaffold for the custom tools each game needs, and a home for the visual tools needed to debug and test Bevy games.
Users will author their assets in code in their favorite applications, import it into the Bevy editor to perform any Bevy- or game-specific modifications and then use the editor to rapidly tweak, test and debug their games.

Now, let's build this together!
