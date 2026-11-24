//! The ids of the arenas. Each is its own type, so a schema's id is never taken for a type's
//! (G6): the arenas hold every node, and an edge is an id, never a pointer.

/// Defines an id: a `u32` index into one arena, `Copy`, ordered, printed as its number.
macro_rules! id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            /// The arena's first entry.
            pub const FIRST: Self = Self(0);

            /// The id of the arena's `index`th entry; `None` past `u32::MAX` entries.
            #[must_use]
            pub fn new(index: usize) -> Option<Self> {
                u32::try_from(index).ok().map(Self)
            }

            /// The entry's index in its arena.
            #[must_use]
            pub fn index(self) -> usize {
                self.0 as usize
            }
        }
    };
}

id!(
    /// A document of a [`crate::Documents`] set, in the order it was added.
    DocId
);
id!(
    /// A JSON node of one document's tree.
    NodeId
);
id!(
    /// A type of the type graph.
    TypeId
);
