use std::collections::{HashMap, HashSet};
use std::mem;

use bevy::ecs::component::HookContext;
use bevy::ecs::{
    component::ComponentId, prelude::*, system::IntoObserverSystem, world::DeferredWorld,
};

// -----------------------------------------------------------------------------
// Fragments and templates

/// An anchor is an identifier for a built fragment.
#[derive(Hash, Eq, PartialEq, Clone)]
pub enum Anchor {
    /// The fragment is static and using an automatic incrementing ID.
    Auto(u64),
    /// The fragment has been explicitly named.
    Named(String),
}

/// Receipts allow templates to intelligently update existing ecs structures.
#[derive(Default, Component, Clone)]
pub struct Receipt {
    /// The components it inserted.
    components: HashSet<ComponentId>,
    /// The receipts of all the children, organized by name.
    anchors: HashMap<Anchor, Entity>,
}

/// A fragment is a tree of bundles with optional names. This is typically built
/// using the [`template!`](crate::template!) macro.
pub struct Fragment {
    /// The name of the fragment, used to identify children across builds.
    pub name: Option<String>,
    /// The bundle to be inserted on the entity.
    pub bundle: BoxedBundle,
    /// The template for the children. This boils down to a type-erased
    /// `Fragment` vector.
    pub children: Template,
}

impl Fragment {
    /// Builds this fragment on the given entity.
    ///
    /// It may modify the entity itself and some or all of its children.
    /// A [`Receipt`] is stored on the entity to track what was built, enabling incremental updates.
    pub fn build(self, entity: Entity, world: &mut World) {
        // Clone the receipt for the targeted entity.
        let receipt = world
            .get::<Receipt>(entity)
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        // Build the bundle. Insert new components, replace existing ones and remove components
        // that are no longer needed.
        let components = self.bundle.inner.build(entity, world, receipt.components);

        // Build the children.
        let anchors = self.children.build(entity, world, receipt.anchors);

        // Place the new receipt onto the entity.
        world.entity_mut(entity).insert(Receipt {
            components,
            anchors,
        });
    }

    /// Builds only the nonexistent parts of this fragment on the given entity.
    ///
    /// It does not modify components of children that already exist, only initializes newly
    /// created ones.
    ///
    /// If build_entity is false it does not build the given entity, only its children.
    ///
    /// It may modify the entity itself and some or all of its children.
    /// A [`Receipt`] is stored on the entity to track what was built, enabling incremental updates.
    pub fn build_nonexistent(self, entity: Entity, world: &mut World, build_entity: bool) {
        // Clone the receipt for the targeted entity.
        let receipt = world
            .get::<Receipt>(entity)
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        let components = if build_entity {
            // Build the bundle. Insert new components, replace existing ones and remove components
            // that are no longer needed.
            self.bundle.inner.build(entity, world, receipt.components)
        } else {
            receipt.components
        };

        // Build the nonexistent children.
        let anchors = self
            .children
            .build_nonexistent(entity, world, receipt.anchors);

        // Place the new receipt onto the entity.
        world.entity_mut(entity).insert(Receipt {
            components,
            anchors,
        });
    }
}

impl Default for Fragment {
    fn default() -> Fragment {
        Fragment {
            name: None,
            bundle: BoxedBundle::from(()),
            children: Template::default(),
        }
    }
}

/// A template is a list of fragments. Each fragment in the list is expected to
/// have either a unique name, or no specific name.
pub type Template = Vec<Fragment>;

/// An extension trait for building templates. Required because [`Template`] is
/// just a type alias.
pub trait BuildTemplate {
    /// Builds the template on an entity. The fragments in the template become its children.
    fn build(
        self,
        entity: Entity,
        world: &mut World,
        current_anchors: HashMap<Anchor, Entity>,
    ) -> HashMap<Anchor, Entity>;

    /// Builds only the nonexistent parts of the template on an entity.
    ///
    /// It does not modify components of children that already exist, only initializes newly
    /// created ones.
    fn build_nonexistent(
        self,
        entity: Entity,
        world: &mut World,
        current_anchors: HashMap<Anchor, Entity>,
    ) -> HashMap<Anchor, Entity>;
}

impl BuildTemplate for Template {
    fn build(
        self,
        entity: Entity,
        world: &mut World,
        current_anchors: HashMap<Anchor, Entity>,
    ) -> HashMap<Anchor, Entity> {
        build_template_base(self, entity, world, current_anchors, false)
    }

    fn build_nonexistent(
        self,
        entity: Entity,
        world: &mut World,
        current_anchors: HashMap<Anchor, Entity>,
    ) -> HashMap<Anchor, Entity> {
        build_template_base(self, entity, world, current_anchors, true)
    }
}

fn build_template_base(
    template: Template,
    entity: Entity,
    world: &mut World,
    mut current_anchors: HashMap<Anchor, Entity>,
    build_nonexistent_only: bool,
) -> HashMap<Anchor, Entity> {
    // Get or create an entity for each fragment.
    let mut i = 0;
    let fragments: Vec<_> = template
        .into_iter()
        .map(|fragment| {
            let mut new = false;

            // Compute the anchor for this fragment, using it's name if supplied
            // or an auto-incrementing counter if not.
            let anchor = match fragment.name {
                Some(ref name) => Anchor::Named(name.clone()),
                None => {
                    let anchor = Anchor::Auto(i);
                    i += 1;
                    anchor
                }
            };

            // Find the existing child entity based on the anchor, or spawn a
            // new one.
            let entity = {
                if let Some(existing_entity) = current_anchors.remove(&anchor) {
                    existing_entity
                } else {
                    let new_entity = world.spawn_empty().id();
                    new = true;

                    new_entity
                }
            };

            // Store the fragment, it's anchor, and it's entity id.
            (fragment, anchor, entity, new)
        })
        .collect();

    // Clear any remaining orphans from the previous template. We do this
    // first (before deparenting) so that hooks still see the parent when
    // they run.
    for orphan in current_anchors.into_values() {
        world.entity_mut(orphan).despawn();
    }

    // Position the entities as children.
    let mut entity = world.entity_mut(entity);
    let child_entities: Vec<_> = fragments.iter().map(|(_, _, entity, _)| *entity).collect();
    entity.remove::<Children>();
    entity.add_children(&child_entities);

    // Build the children and produce the receipts. It's important that this
    // happens *after* the entities are positioned as children to make hooks
    // work correctly.
    fragments
        .into_iter()
        .map(|(fragment, anchor, entity, new)| {
            if build_nonexistent_only {
                fragment.build_nonexistent(entity, world, new);
            } else {
                fragment.build(entity, world);
            }

            (anchor, entity)
        })
        .collect()
}

/// An error returned when converting a template into a fragment.
pub enum TemplateIntoFragmentError {
    /// The template was empty.
    Empty,
    /// The template contained more than one fragment.
    MultipleFragments(usize),
}

impl TryInto<Fragment> for Template {
    type Error = TemplateIntoFragmentError;

    fn try_into(mut self) -> Result<Fragment, TemplateIntoFragmentError> {
        match self.len() {
            0 => Err(TemplateIntoFragmentError::Empty),
            1 => Ok(self.pop().unwrap()),
            n => Err(TemplateIntoFragmentError::MultipleFragments(n)),
        }
    }
}

// -----------------------------------------------------------------------------
// Dynamically typed bundles

/// Bundles are not dyn-compatible, which means they cannot be boxed. This
/// trait provides a dyn-compatible alternative.
pub trait ErasedBundle {
    /// Inserts a bundle on the specified entity, and removes components present
    /// in the provided hash set which are no-longer needed.
    fn build(
        self: Box<Self>,
        entity_id: Entity,
        world: &mut World,
        current_components: HashSet<ComponentId>,
    ) -> HashSet<ComponentId>;
}

impl<B> ErasedBundle for B
where
    B: Bundle,
{
    fn build(
        self: Box<Self>,
        entity_id: Entity,
        world: &mut World,
        current_components: HashSet<ComponentId>,
    ) -> HashSet<ComponentId> {
        // Collect set of component ids present in the bundle.
        let mut new_components = HashSet::new();
        B::get_component_ids(world.components(), &mut |maybe_id| {
            if let Some(id) = maybe_id {
                new_components.insert(id);
            }
        });

        // Insert the bundle.
        let mut entity = world.entity_mut(entity_id);
        entity.insert(*self);

        // Remove the components in the previous bundle but not this one.
        for component_id in current_components.difference(&new_components) {
            entity.remove_by_id(*component_id);
        }

        // Return the new set of components.
        new_components
    }
}

/// A Boxed version of a bundle, built using [`ErasedBundle`].
pub struct BoxedBundle {
    inner: Box<dyn ErasedBundle + Send + 'static>,
}

impl BoxedBundle {
    /// Creates a new boxed bundle.
    pub fn new(bundle: impl ErasedBundle + Send + Sync + 'static) -> BoxedBundle {
        BoxedBundle {
            inner: Box::new(bundle),
        }
    }
}

impl<B> From<B> for BoxedBundle
where
    B: Bundle,
{
    fn from(bundle: B) -> BoxedBundle {
        BoxedBundle {
            inner: Box::new(bundle),
        }
    }
}

// -----------------------------------------------------------------------------
// Callbacks and observers

/// This is a helper for adding observers to a `template` macro.
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{template, on};
/// # use bevy::prelude::*;
/// template!{
///     Name::new("MyEntity") => [
///         on(|trigger: Trigger<Pointer<Click>>| {
///             // Do something when "MyEntity" is clicked.
///         });
///         on(|trigger: Trigger<Pointer<Drag>>| {
///             // Do something when "MyEntity" is dragged.
///         });
///     ];
/// };
/// ```
pub fn on<E, B, M, I>(system: I) -> Callback
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    Callback::new(system)
}

/// Wrapper around an observer that makes it observe the parent.
#[derive(Component)]
#[component(on_insert = insert_callback)]
#[component(on_remove = remove_callback)]
pub struct Callback {
    observer: Option<Observer>,
}

impl Callback {
    /// Creates a new callback.
    pub fn new<E, B, M, I>(system: I) -> Callback
    where
        E: Event,
        B: Bundle,
        I: IntoObserverSystem<E, B, M>,
    {
        Callback {
            observer: Some(Observer::new(system)),
        }
    }
}

impl From<Observer> for Callback {
    fn from(observer: Observer) -> Callback {
        Callback {
            observer: Some(observer),
        }
    }
}

fn insert_callback(mut world: DeferredWorld, context: HookContext) {
    let mut callback = world.get_mut::<Callback>(context.entity).unwrap();
    let Some(mut observer) = mem::take(&mut callback.observer) else {
        return;
    };
    if let Some(parent_id) = world.get::<ChildOf>(context.entity).map(ChildOf::parent) {
        observer.watch_entity(parent_id);
    }
    let mut commands = world.commands();
    let mut entity_commands = commands.entity(context.entity);
    entity_commands.remove::<Observer>();
    entity_commands.insert(observer);
}

fn remove_callback(mut world: DeferredWorld, context: HookContext) {
    let mut commands = world.commands();
    commands.entity(context.entity).remove::<Observer>();
}

// -----------------------------------------------------------------------------
// Commands

/// An extension trait for `EntityCommands` which allows templates to be built
/// on entities.
pub trait TemplateEntityCommandsExt {
    /// Builds a fragment directly on the entity. Accepts anything that
    /// implements `TryInto<Fragment>` and does nothing on a failure. This is
    /// implemented for `Template` and is `Ok` when template has exactly one
    /// fragment.
    ///
    /// To build the fragments in a template as children of the entity, see
    /// [`build_children`](TemplateEntityCommandsExt::build_children).
    fn build<F>(&mut self, fragment: F) -> &mut Self
    where
        F: TryInto<Fragment>;

    /// Builds only the nonexistent parts of this fragment directly on entity. It does
    /// not modify components of children that already exist, only initializes newly
    /// created ones.
    ///
    /// If build_entity is false it does not build the entity, only its children.
    fn build_nonexistent<F>(&mut self, fragment: F, build_entity: bool) -> &mut Self
    where
        F: TryInto<Fragment>;

    /// Builds the fragments in the template as children of the entity. If the
    /// template is empty this will remove all children.
    ///
    /// To build a fragment directly on the entity, see
    /// [`build`](TemplateEntityCommandsExt::build).
    fn build_children(&mut self, template: Template) -> &mut Self;

    /// Builds only the nonexistent parts of template as children of the entity. It does
    /// not modify components on entities that already exist, only initializes newly created ones.
    /// If the template is empty, all current children will be removed.
    fn build_nonexistent_children(&mut self, template: Template) -> &mut Self;
}

impl TemplateEntityCommandsExt for EntityCommands<'_> {
    fn build<F>(&mut self, fragment: F) -> &mut Self
    where
        F: TryInto<Fragment>,
    {
        if let Ok(fragment) = fragment.try_into() {
            // Build the fragment.
            self.queue(|entity: EntityWorldMut| {
                fragment.build(entity.id(), entity.into_world_mut());
            })
        } else {
            self
        }
    }

    fn build_nonexistent<F>(&mut self, fragment: F, build_entity: bool) -> &mut Self
    where
        F: TryInto<Fragment>,
    {
        if let Ok(fragment) = fragment.try_into() {
            // Build the fragment.
            self.queue(move |entity: EntityWorldMut| {
                fragment.build_nonexistent(entity.id(), entity.into_world_mut(), build_entity);
            })
        } else {
            self
        }
    }

    fn build_children(&mut self, children: Template) -> &mut Self {
        self.queue(|entity: EntityWorldMut| {
            let (entity_id, world) = (entity.id(), entity.into_world_mut());

            // Access the receipt for the parent.
            let receipt = world
                .get::<Receipt>(entity_id)
                .map(ToOwned::to_owned)
                .unwrap_or_default();

            // Build the children.
            let anchors = children.build(entity_id, world, receipt.anchors);

            // Place the new receipt onto the parent.
            world
                .entity_mut(entity_id)
                .insert(Receipt { anchors, ..receipt });
        })
    }

    fn build_nonexistent_children(&mut self, children: Template) -> &mut Self {
        self.queue(|entity: EntityWorldMut| {
            let (entity_id, world) = (entity.id(), entity.into_world_mut());

            // Access the receipt for the parent.
            let receipt = world
                .get::<Receipt>(entity_id)
                .map(ToOwned::to_owned)
                .unwrap_or_default();

            // Build the nonexistent children.
            let anchors = children.build_nonexistent(entity_id, world, receipt.anchors);

            // Place the new receipt onto the parent.
            world
                .entity_mut(entity_id)
                .insert(Receipt { anchors, ..receipt });
        })
    }
}

// -----------------------------------------------------------------------------
// Macros

/// This is a declarative template macro for Bevy!
///
/// It gives you something a little like `bsn` and a little `jsx`. Like `bsn`,
/// it's a shorthand for defining ECS structures. Like `jsx` you can build
/// fragments (in this case `Template` values) at runtime and compose them using
/// normal Rust functions and syntax.
///
/// Here's an example of what it looks like:
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{template, Template};
/// # use bevy::prelude::*;
/// # let dark_mode = false;
/// # #[derive(Component)]
/// # pub struct MyMarkerComponent;
/// pub fn my_template(dark_mode: bool) -> Template {
///     // We create a new `Template` using the template macro.
///     template! {
///         (
///             // Here we define an entity with a bundle of components.
///             Text::new(""),
///             TextFont::from_font_size(28.0),
///             if dark_mode { TextColor::WHITE } else { TextColor::BLACK }
///         ) => [
///             // Here we define the entity's children.
///             TextSpan::new("Hello ");
///             TextSpan::new("World");
///             ( TextSpan::new("!"), MyMarkerComponent );
///         ];
///     }
/// }
/// ```
///
/// # Fragment
///
/// The template macro is a sequence of "fragments" and "splices" delimited by
/// semicolons. A fragment represents a single entity, with optional children. A
/// splice is a way to insert one template into the middle of another.
///
/// # Fragments
///
/// A fragment consists of three things: a name, a bundle, and a list of
/// children. They generally looks something like the following
/// ```text
/// ( $name: )? $bundle ( => [ $children ] )? ;
/// ```
/// where `(...)?` indicates an optional part.
///
/// ## Names
///
/// The first part of a fragment is an optional name (followed by a colon). A
/// name can be either static or dynamic. Static names are symbols which are
/// converted to strings at compile time. Dynamic names are blocks returning a
/// type that implements `Display`, and which are converted to strings at
/// runtime.
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{template, Template};
/// # use bevy::prelude::*;
/// fn template(dynamic_name: &str) -> Template {
///     template! {
///         Text::new("I don't have a name!");
///         static_name: Text::new("I am named `static_name`!");
///         {dynamic_name}: Text::new(format!("I am named `{}`!", dynamic_name));
///     }
/// }
/// ```
///
/// Names give fragments continuity, and used to ensure the right entity gets
/// updated when a template is built multiple times. It is usually fine to not
/// give fragments explicit names, and let the macro name them automatically.
/// See the section on "Static and Dynamic Fragments" for more information about
/// the limits of anonymous fragments and automatic naming.
///
/// ## Bundles
///
/// The only required part of a fragment is the bundle. Every fragment must have
/// a bundle, even if it is the empty bundle `()`. Specifically, the bundle
/// portion of a fragment must be a rust expression that evaluates to a type
/// implementing `Into<BoxedBundle>` (which includes every `Bundle` type). Here
/// are some examples of valid fragments with different bundles.
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{template, b};
/// # use bevy::prelude::*;
/// # #[derive(Component)]
/// # struct ComponentA;
/// # #[derive(Component)]
/// # struct ComponentB;
/// # #[derive(Component)]
/// # enum EnumComponent { A, B }
/// # let foo = true;
/// # template! {
/// // The empty bundle.
/// ();
///
/// // A code-block returning the empty bundle.
/// { () };
///
/// // A single component.
/// ComponentA;
///
/// // An if statement that switches between different values for the same component.
/// if foo { EnumComponent::A } else { EnumComponent::B };
///
/// // An if statement that switches between different bundle types.
/// if foo { b!(ComponentA) } else { b!(ComponentB) };
///
/// // A tuple-bundle.
/// (ComponentA, ComponentB);
///
/// // A code-block returning a tuple bundle.
/// { (ComponentA, ComponentB) };
/// # };
/// ```
///
/// Since these are all normal rust expressions, they have complete access to
/// the entire language. They can access local variables, use iterators, and
/// even include loops or cause side-effects. The only caveat is that each
/// fragment's bundle expression *must* evaluate to a specific type. If you want
/// to return bundles of different types, you must manually convert them to a
/// `BoxedBundle` using [`b!`](crate::b!) or [`Into<BoxedBundle>`] before
/// returning the different bundles.
///
/// ## Children
///
/// Fragments may have an optional list of children, separated from the bundle
/// by `=>` and enclosed in square brackets. The list of children is itself a
/// template, and has semantics identical to the top-level `template!{}` macro.
///
/// Every child fragment will create a child entity when built. When a template
/// is built multiple times (or applied to an existing entity) all children not
/// created by the template will are removed, and the entity children are
/// re-ordered to match the template.
///
/// # Splices
///
/// Splices allow you to insert a sub-template into the middle of another. This
/// inserts the sub-template's fragments into the other template's fragment list
/// at the splice point.
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{template, Template};
/// # use bevy::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent;
/// fn sub_template() -> Template {
///     template! {
///         one: MyComponent;
///         two: MyComponent;
///     }
/// }
///
/// let parent_template = template! {
///     parent: Text::new("parent") => [
///         // Insert both "one" and "two" as children of "parent".
///         @{ sub_template() };
///     ];
///     // Inserts both "one" and "two" as siblings of "parent".
///     @{ sub_template() };
/// };
/// ```
///
/// # Building Templates
///
/// Templates can be "built" on an entity in much the same way that normal
/// bundles can be "inserted", but building a template builds the entire
/// hierarchy, spawning new child entities when required. Template building is
/// also "nondestructive" and "incremental". Nondestructive means you can build
/// a template on an existing entity without messing up unrelated components.
/// Incremental means that, if you build two templates on the same entity, the
/// second build will undo all the stuff done by the first build so that it
/// matches up with the second (without touching other components not added by
/// the first template). For more information about this, see the
/// `TemplateEntityCommandsExt` extension trait.
///
/// # Template Functions
///
/// As indicated above, templates are most useful when built "multiple times".
/// Unfortunately, due to borrow-checker rules, `Template` is neither `Copy` nor
/// `Clone` and is consumed when built. The recommended approach is to write
/// functions which create `Template` values.
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{template, Template};
/// # use bevy::prelude::*;
/// fn template(cond: bool) -> Template {
///     template! [
///         Text::new( if cond { "true" } else { "false" } );
///     ]
/// }
/// ```
///
/// Each template function can be thought of as generating some specific version
/// of a given template. The template incrementalism rules ensure that if
/// multiple versions of the "same" template are built on the same entity, it
/// will be gracefully updated to match only the most recent.
///
/// # Static and Dynamic Fragments
///
/// In the context of a template function emerge two kinds of fragment: Static
/// and Dynamic.
///
/// Static fragments are ones which do not change *position* between invocations
/// of a template function; they always appear in the same exact spot, never
/// omitted or moved around. More formally, a fragment is static if and only if
/// the same number of (other) static fragments always appear in front of it
/// within the template.
///
/// Clearly if the fist fragment is always present in the template, then it is
/// static. Likewise if the second fragment is always present it is static too,
/// and so on and so on. This property allows the macro to able to give a fixed
/// index to each static fragment, and for this reason static fragment do not
/// need to be given names.
///
/// Dynamic fragments, by contrast, are ones which:
/// + Are conditional, or otherwise not always present in the template.
/// + Move around between invocations of the template.
///
/// Both of these can mess with automatic indexing, so dynamic fragments do need
/// to be given names. Especially in the case where fragments can move around,
/// users need to ensure that the name moves with the fragment.
///
/// For example when creating a dynamic list of fragments that may be
/// re-ordered, you should try to use some sort of stable name.
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::{Template, template};
/// # use bevy::prelude::*;
/// # struct Item { id: usize, value: usize };
/// # #[derive(Component)]
/// # struct Value(usize);
/// fn template(list: Vec<Item>) -> Template {
///     list.into_iter()
///         .map(|item| template! {
///             {item.id}: Value(item.value);
///         })
///         .flatten() // Flatten out all the templates into an iterator of fragments
///         .collect() // Collect the fragments into a single template
/// }
/// ```
///
/// # Grammar
///
/// The entire `template!` macro is defined with the following ABNF grammar
///
/// ```text
/// <template> = *( <item> )
///     <item> = ( <splice> | <fragment> ) ";"
///   <splice> = "@" <$block>                  -- where block returns `T: IntoIterator<Item = Fragment>`.
/// <fragment> = <name>? <$expr> <children>?   -- where expression returns `B: Into<BoxedBundle>`.
///     <name> = ( <$ident> | <$block> ) ":"   -- where block returns `D: Display`.
/// <children> = "=> [" <template> "]"
///   <$ident> = an opaque rust identifier
///    <$expr> = a rust expression of a given type
///   <$block> = a rust codeblock of a given type
/// ```
///
#[macro_export]
macro_rules! template {
    ($($body:tt)*) => {{
        #[allow(unused_mut)]
        let mut fragments = Vec::new();
        bevy_i_cant_believe_its_not_bsn::push_item!(fragments; $($body)*);
        fragments
    }};
}

// This pushes the next fragment in the template into the provided vector.
#[doc(hidden)]
#[macro_export]
macro_rules! push_item {
    // Empty cases.

    () => {};
    ($fragments:ident;) => {};

    // Fragments

    // Anonymous bundle expression.
    ($fragments:ident; $bundle:expr $( => [ $( $children:tt )+ ] )? ; $( $($sib:tt)+ )?) => {
        bevy_i_cant_believe_its_not_bsn::push_fragment!($fragments; { None } $bundle $( => [ $( $children )* ] )* ; $( $( $sib )* )* )
    };
    // Bundle expression with fixed name.
    ($fragments:ident; $name:ident: $bundle:expr $( => [ $( $children:tt )+ ] )? ; $( $($sib:tt)+ )?) => {
        // Stringify the name and throw it in a code-block.
        bevy_i_cant_believe_its_not_bsn::push_fragment!($fragments; { Some(stringify!($name).to_string()) } $bundle $( => [ $( $children )* ] )* ; $( $( $sib )* )* )
    };
    // Bundle expression with dynamic name.
    ($fragments:ident; $name:block: $bundle:expr $( => [ $( $children:tt )+ ] )? ; $( $($sib:tt)+ )?) => {
        bevy_i_cant_believe_its_not_bsn::push_fragment!($fragments; { Some($name.to_string()) } $bundle $( => [ $( $children )* ] )* ; $( $( $sib )* )* )
    };

    // Splices

    // A code-block returning an iterator of fragments.
    ($fragments:ident; @ $block:block ; $( $($sib:tt)+ )? ) => {
        $fragments.extend({ $block }); // Extend the fragments with the value of the block.
        $(bevy_i_cant_believe_its_not_bsn::push_item!($fragments; $($sib)*))* // Continue pushing siblings onto the current list.
    };
}

// This is called by `push_item` to actually push the fragment, once it's
// figured out what sort of pattern the fragment is using.
#[doc(hidden)]
#[macro_export]
macro_rules! push_fragment {
    ($fragments:ident; $name:block $bundle:expr $( => [ $( $children:tt )+ ] )? ; $( $($sib:tt)+ )?) => {
        let fragment = bevy_i_cant_believe_its_not_bsn::Fragment {
            name: $name,
            bundle: bevy_i_cant_believe_its_not_bsn::BoxedBundle::from($bundle),
            children: {
                #[allow(unused_mut)]
                let mut fragments = Vec::new();
                $(bevy_i_cant_believe_its_not_bsn::push_item!(fragments; $($children)*);)* // Push the first child onto a new list of children.
                fragments
            },
        };
        $fragments.push(fragment);
        $(bevy_i_cant_believe_its_not_bsn::push_item!($fragments; $($sib)*))* // Continue pushing siblings onto the current list.
    };
}

/// This macro is just a shorthand for creating a [`BoxedBundle`] from a bundle.
///
/// # Example
///
/// ```
/// # use bevy_i_cant_believe_its_not_bsn::b;
/// # use bevy::prelude::*;
/// let empty_bundle = b!();
/// let single_bundle = b!(Transform::default());
/// let multi_bundle = b!(Transform::default(), Visibility::Visible);
/// ```
#[macro_export]
macro_rules! b {
    ($( $item:expr ),* ) => {
        bevy_i_cant_believe_its_not_bsn::BoxedBundle::from( ( $( $item ),* ) )
    };
}
