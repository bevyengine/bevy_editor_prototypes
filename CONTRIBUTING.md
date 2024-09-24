# Contributing

This repository is a low-friction, highly experimental playground used for building the first iterations of Bevy's editor.
It does not have any true users (please, don't try and use this yet!): rapid experimentation without accumulating tech debt is the name of the game.

As a result, the rules are a little different around these parts:

1. PRs are required: no pushing to `main`.
2. Only one approval (from anyone) is required before merging a PR.
3. Trustworthy and active non-maintainer contributors may be granted write permissions.

In general, we're skewing close to the design requirements of the editor itself, to avoid being bitten by problems that only manifest when working under the real constraints.
These are documented in the associated book, found in the `design-book` folder.

Work to be done is tracked in the issues; feel free to pick one up and get started!

## Working with mdbook

The Design Book is written in `mdbook`, a very simple Rust-based tool for writing long-form documents.
To install it, use `cargo install mdbook`.
To open the book, change into the `design-book` folder in your terminal, then run `mdbook serve --open`.

## Before Submitting a PR

- Ensure you are up-to-date locally with the main bevy repo, we track bevy's `main` branch as a dependecy.