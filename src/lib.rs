//! Entity component system

#![deny(missing_docs)]

#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::default::Default;
use std::ops;
use std::slice;

/// Handle for an entity in the entity component system.
///
/// The internal value is the unique identifier for the entity. No two
/// entities should get the same UID during the lifetime of the ECS.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Entity {
    uid: u32,
    idx: u32,
}

/// Indexing entity index to data array.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
struct Index {
    /// Must match the uid for the entity that is querying for component.
    uid: u32,
    /// Index for this entity in the data array.
    data_idx: u32,
}

/// Operations all components must support.
pub trait AnyComponent {
    /// Remove an entity's component.
    fn remove(&mut self, e: Entity);

    /// Increment space for entities by one.
    fn reserve_entity_space(&mut self);
}

/// Storage for a single component type.
#[derive(Serialize, Deserialize)]
pub struct ComponentData<C> {
    /// Dense component data.
    data: Vec<C>,
    /// Entity idx corresponding to elements in data.
    entities: Vec<Entity>,
    /// Sparse array mapping entity indices to data values.
    entity_idx_to_data: Vec<Index>,
}

impl<C> ComponentData<C> {
    /// Construct new empty `ComponentData` instance.
    pub fn new() -> ComponentData<C> {
        ComponentData {
            data: Vec::new(),
            entities: Vec::new(),
            entity_idx_to_data: Vec::new(),
        }
    }

    /// Insert a component to an entity.
    pub fn insert(&mut self, e: Entity, comp: C) {
        debug_assert!(self.data.len() == self.entities.len());

        if self.contains(e) {
            // Component is set for entity, replace existing component.
            self.data[self.entity_idx_to_data[e.idx as usize].data_idx as usize] = comp;
        } else {
            // Add a new component.
            let data_idx = self.data.len() as u32;
            self.data.push(comp);
            self.entities.push(e);
            self.entity_idx_to_data[e.idx as usize] = Index {
                uid: e.uid,
                data_idx: data_idx,
            };
        }
    }

    /// Return whether an entity contains this component.
    #[inline(always)]
    pub fn contains(&self, e: Entity) -> bool {
        debug_assert!(e.uid != 0);
        debug_assert!((e.idx as usize) < self.entity_idx_to_data.len());

        self.entity_idx_to_data[e.idx as usize].uid == e.uid
    }

    /// Get a reference to a component only if it exists for this entity.
    #[inline(always)]
    pub fn get(&self, e: Entity) -> Option<&C> {
        if self.contains(e) {
            Some(&self.data[self.entity_idx_to_data[e.idx as usize].data_idx as usize])
        } else {
            None
        }
    }

    /// Get a mutable reference to a component only if it exists for this entity.
    #[inline(always)]
    pub fn get_mut(&mut self, e: Entity) -> Option<&mut C> {
        if self.contains(e) {
            Some(&mut self.data[self.entity_idx_to_data[e.idx as usize].data_idx as usize])
        } else {
            None
        }
    }

    /// Iterate entity ids in this component.
    pub fn ent_iter(&self) -> slice::Iter<Entity> {
        self.entities.iter()
    }

    /// Iterate elements in this component.
    pub fn iter(&self) -> slice::Iter<C> {
        self.data.iter()
    }

    /// Iterate mutable elements in this component.
    pub fn iter_mut(&mut self) -> slice::IterMut<C> {
        self.data.iter_mut()
    }
}

impl<C> ops::Index<Entity> for ComponentData<C> {
    type Output = C;

    fn index<'a>(&'a self, e: Entity) -> &'a C {
        self.get(e).unwrap()
    }
}

impl<C> ops::IndexMut<Entity> for ComponentData<C> {
    fn index_mut<'a>(&'a mut self, e: Entity) -> &'a mut C {
        self.get_mut(e).unwrap()
    }
}

impl<C> AnyComponent for ComponentData<C> {
    fn remove(&mut self, e: Entity) {
        debug_assert!(self.data.len() == self.entities.len());
        if self.contains(e) {
            let removed_index = self.entity_idx_to_data[e.idx as usize];
            self.entity_idx_to_data[e.idx as usize] = Default::default();

            // To keep the data compact, we do swap-remove with the last data item and update the
            // lookup on the moved item. If the component being removed isn't the last item in the
            // list, we need to reset the lookup value for the component that was moved.
            if removed_index.data_idx as usize != self.entities.len() - 1 {
                let last_entity = self.entities[self.entities.len() - 1];
                self.entities.swap_remove(removed_index.data_idx as usize);
                self.entity_idx_to_data[last_entity.idx as usize] = Index {
                    uid: last_entity.uid,
                    data_idx: removed_index.data_idx,
                };
            } else {
                self.entities.swap_remove(removed_index.data_idx as usize);
            }

            self.data.swap_remove(removed_index.data_idx as usize);
        }
    }

    fn reserve_entity_space(&mut self) {
        self.entity_idx_to_data.push(Default::default());
    }
}

/// Operations for the internal component store object.
pub trait Store {
    /// Perform an operation for each component container.
    fn for_each_component<F>(&mut self, f: F) where F: FnMut(&mut AnyComponent);
}

/// Generic entity component system container
///
/// Needs to be specified with the parametrized `Store` type that has struct fields for the actual
/// components. This can be done with the `Ecs!` macro.
#[derive(Serialize, Deserialize)]
pub struct Ecs<ST> {
    next_uid: u32,
    next_idx: u32,
    free_indices: Vec<u32>,
    active: ComponentData<bool>,
    store: ST,
}

impl<ST: Default + Store> Ecs<ST> {
    /// Construct a new entity component system.
    pub fn new() -> Ecs<ST> {
        Ecs {
            next_uid: 1,
            next_idx: 0,
            free_indices: Vec::new(),
            active: ComponentData::new(),
            store: Default::default(),
        }
    }

    /// Create a new empty entity.
    pub fn make(&mut self) -> Entity {
        let uid = self.next_uid;
        self.next_uid += 1;

        let idx = if let Some(idx) = self.free_indices.pop() {
            idx
        } else {
            self.next_idx += 1;
            self.store.for_each_component(|c| c.reserve_entity_space());
            self.active.reserve_entity_space();
            self.next_idx - 1
        };

        let ret = Entity {
            uid: uid,
            idx: idx,
        };
        self.active.insert(ret, true);
        ret
    }

    /// Remove an entity from the system and clear its components.
    pub fn remove(&mut self, e: Entity) {
        self.free_indices.push(e.idx);
        self.active.remove(e);
        self.store.for_each_component(|c| c.remove(e));
    }

    /// Return whether the system contains an entity.
    pub fn contains(&self, e: Entity) -> bool {
        self.active.contains(e)
    }

    /// Iterate through all the active entities.
    pub fn iter(&self) -> slice::Iter<Entity> {
        self.active.ent_iter()
    }
}

impl<ST> ops::Deref for Ecs<ST> {
    type Target = ST;

    fn deref(&self) -> &ST {
        &self.store
    }
}

impl<ST> ops::DerefMut for Ecs<ST> {
    fn deref_mut(&mut self) -> &mut ST {
        &mut self.store
    }
}

/// Entity component system builder macro.
///
/// Defines a local `Ecs` type that's parametrized with a custom component
/// store type with the component types you specify. Will also define a trait
/// `Component` which will be implemented for the component types.
#[macro_export]
macro_rules! Ecs {
    {
        // Declare the type of the (plain old data) component and the
        // identifier to use for it in the ECS.
        $($compname:ident: $comptype:ty,)+
    } => {
        mod _ecs_inner {
            // Use the enum to convert components to numbers for component bit masks etc.
            #[allow(non_camel_case_types, dead_code)]
            pub enum ComponentNum {
                $($compname,)+
            }

        }

        pub use self::_ecs_inner::ComponentNum;

        #[derive(Serialize, Deserialize)]
        pub struct _ComponentStore {
            $(pub $compname: $crate::ComponentData<$comptype>),+
        }

        impl ::std::default::Default for _ComponentStore {
            fn default() -> _ComponentStore {
                _ComponentStore {
                    $($compname: $crate::ComponentData::new()),+
                }
            }
        }

        impl $crate::Store for _ComponentStore {
            fn for_each_component<F>(&mut self, mut f: F)
                where F: FnMut(&mut $crate::AnyComponent)
            {
                $(f(&mut self.$compname as &mut $crate::AnyComponent);)+
            }
        }

        #[allow(dead_code)]
        pub fn matches_mask(ecs: &$crate::Ecs<_ComponentStore>, e: $crate::Entity, mask: u64) -> bool {
            $(if mask & (1 << ComponentNum::$compname as u8) != 0 && !ecs.$compname.contains(e) {
                return false;
            })+
            return true;
        }

        /// Common operations for ECS component value types.
        pub trait Component {
            /// Add a clone of the component value to an entity in an ECS.
            ///
            /// Can't move the component itself since we might be using this
            /// through a trait object.
            fn add_to_ecs(&self, ecs: &mut $crate::Ecs<_ComponentStore>, e: $crate::Entity);

            /// Add a clone of the component to a loadout struct.
            fn add_to_loadout(self, loadout: &mut Loadout);
        }

        $(impl Component for $comptype {
            fn add_to_ecs(&self, ecs: &mut $crate::Ecs<_ComponentStore>, e: $crate::Entity) {
                ecs.$compname.insert(e, self.clone());
            }

            fn add_to_loadout(self, loadout: &mut Loadout) {
                loadout.$compname = Some(self);
            }
        })+

        pub type Ecs = $crate::Ecs<_ComponentStore>;

        /// A straightforward representation for the complete data of an
        /// entity.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Loadout {
            $(pub $compname: Option<$comptype>),+
        }

        impl ::std::default::Default for Loadout {
            fn default() -> Loadout {
                Loadout {
                    $($compname: None),+
                }
            }
        }

        #[allow(dead_code)]
        impl Loadout {
            /// Create a new blank loadout.
            pub fn new() -> Loadout { Default::default() }

            /// Get the loadout that corresponds to an existing entity.
            pub fn get(ecs: &Ecs, e: $crate::Entity) -> Loadout {
                Loadout {
                    $($compname: ecs.$compname.get(e).map(|e| e.clone())),+
                }
            }

            /// Create a new entity in the ECS with this loadout.
            pub fn make(&self, ecs: &mut Ecs) -> $crate::Entity {
                let e = ecs.make();
                $(self.$compname.as_ref().map(|c| ecs.$compname.insert(e, c.clone()));)+
                e
            }

            /// Builder method for adding a component to this loadout.
            pub fn c<C: Component>(mut self, comp: C) -> Loadout {
                comp.add_to_loadout(&mut self);
                self
            }
        }
    }
}

/// Build a component type mask to match component iteration with.
///
/// You must have ComponentNum enum from the Ecs! macro expansion in scope
/// when using this.
#[macro_export]
macro_rules! build_mask {
    ( $($compname:ident),+ ) => {
        0u64 $(| (1u64 << ComponentNum::$compname as u8))+
    }
}
