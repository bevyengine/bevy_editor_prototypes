# Bevy Editor Prototypes

> [!WARNING]
> This project is a prototype for a shippable Bevy Editor and is still in heavy development and testing.
> Everything you see is very much a work-in-progress. Read about what we're building in the [Design Book](https://bevyengine.github.io/bevy_editor_prototypes/)! or [Figma document].
> To run just type the ```cargo run``` or ```cargo run --example``` command in the terminal of the `bevy_editor_prototypes` folder.

Want to see what we have planned? Check out [design-book/vision] for a high-level view on our goals, and [design-book/architecture] for a summary of how we think it should all fit together.

Want to help? See [design-book/roadmap] for an up-to-date summary of the work that has been done and needs to be done to advance the project to the next phase.
To avoid ballooning scope and ensure we can ship something useful and reasonably polished, please try your best to focus efforts on the next milestone that has yet to be completed, or standalone proof-of-concepts or design work for important open questions.

*IMPORTANT*:

When you open a folder with the launcher it will simply copy the template to the target location which references the editor's github repository and not your locally compiled editor. 

Instead, for local editor dev use the below example for compiling/running changes:

`cargo run --example simple_editor`


## Figma Designs

While laying out the foundations for an editor prototype in this repository, we are also experimenting with various UI designs in a public [Figma document]. If you are interested in collaborating and sharing your own designs, head over to the document and request write permissions to join the effort.

[Figma document]: https://www.figma.com/design/fkYfFPSBgnGkhbQd3HOMsL/Bevy-Editor?node-id=90-2

## License

Bevy is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

Some of the engine's code carries additional copyright notices and license terms due to their external origins.
These are generally BSD-like, but exact details vary by crate:
If the README of a crate contains a 'License' header (or similar), the additional copyright notices and license terms applicable to that crate will be listed.
The above licensing requirement still applies to contributions to those crates, and sections of those crates will carry those license terms.
The [license](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields) field of each crate will also reflect this.
For example, [`bevy_mikktspace`](./crates/bevy_mikktspace/README.md#license-agreement) has code under the Zlib license (as well as a copyright notice when choosing the MIT license).

Any assets included in this repository typically fall under different open licenses.
These will not be included in your game (unless copied in by you), and they are not distributed in the published bevy crates.
See [CREDITS.md](CREDITS.md) for the details of the licenses of those files.

### Your contributions

*Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.*
