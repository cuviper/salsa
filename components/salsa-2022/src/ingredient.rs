use crate::{
    cycle::CycleRecoveryStrategy, key::DependencyIndex, runtime::local_state::QueryOrigin,
    DatabaseKeyIndex, Id,
};

use super::Revision;

/// "Ingredients" are the bits of data that are stored within the database to make salsa work.
/// Each jar will define some number of ingredients that it requires.
/// Each use salsa macro (e.g., `#[salsa::tracked]`, `#[salsa::interned]`) adds one or more ingredients to the jar struct
/// that together are used to create the salsa concept.
/// For example, a tracked struct defines a [`crate::interned::InternedIngredient`] to store its identity
/// plus [`crate::function::FunctionIngredient`] values to store its fields.
/// The exact ingredients are determined by [`IngredientsFor`](`crate::storage::IngredientsFor`) implementations generated by the macro.
pub trait Ingredient<DB: ?Sized> {
    /// If this ingredient is a participant in a cycle, what is its cycle recovery strategy?
    /// (Really only relevant to [`crate::function::FunctionIngredient`],
    /// since only function ingredients push themselves onto the active query stack.)
    fn cycle_recovery_strategy(&self) -> CycleRecoveryStrategy;

    /// Has the value for `input` in this ingredient changed after `revision`?
    fn maybe_changed_after(&self, db: &DB, input: DependencyIndex, revision: Revision) -> bool;

    /// What were the inputs (if any) that were used to create the value at `key_index`.
    fn origin(&self, key_index: Id) -> Option<QueryOrigin>;

    /// Invoked when the value `output_key` should be marked as valid in the current revision.
    /// This occurs because the value for `executor`, which generated it, was marked as valid in the current revision.
    fn mark_validated_output(&self, db: &DB, executor: DatabaseKeyIndex, output_key: Id);

    /// Invoked when the value `stale_output` was output by `executor` in a previous
    /// revision, but was NOT output in the current revision.
    ///
    /// This hook is used to clear out the stale value so others cannot read it.
    fn remove_stale_output(&self, db: &DB, executor: DatabaseKeyIndex, stale_output_key: Id);
}

/// Optional trait for ingredients that wish to be notified when new revisions are
/// about to occur. If ingredients wish to receive these method calls,
/// they need to indicate that by invoking [`Routes::push_mut`] during initialization.
pub trait MutIngredient<DB: ?Sized>: Ingredient<DB> {
    /// Invoked when a new revision is about to start. This gives ingredients
    /// a chance to flush data and so forth.
    fn reset_for_new_revision(&mut self);
}
