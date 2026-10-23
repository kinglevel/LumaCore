//! lumacore-base: Foundation types for the LumaCore object tree.
//!
//! Provides the base traits and types used by all domain crates:
//! - TreeNode trait and NodeMeta for identity and addressing
//! - IdPool for per-type sequential ID management
//! - TreeEvent for reactive observation
//! - KeyBinding for key binding management
//! - UndoStack for undo/redo
//! - VariableValue/VariableStore for runtime variables

pub mod clock;
pub mod embedded_json;
pub mod error_code;
pub mod events;
pub mod fingerprint;
pub mod id_pool;
pub mod identity;
pub mod inbound;
pub mod keybindings;
pub mod node;
pub mod paint;
pub mod replication;
pub mod threads;
pub mod undo;
pub mod value;
pub mod variables;

// Re-export all public types
pub use error_code::{Coded, ErrorInfo, with_code};
pub use events::{TreeEvent, TreeObserver};
pub use id_pool::IdPool;
pub use identity::{Guid, Identified, IdentityReport, LinkEntry, ObjectEntry, Walk, Walker};
pub use inbound::{InboundEvent, TransportAction};
pub use keybindings::{
    BindingSource, KeyBinding, KeyBindingMap, Layer, LayerKey, PRG_KEYS, debug_keybindings, normalize_key, resolve_bindings,
    resolve_bindings_with_source, slot, vanilla_keybindings,
};
pub use node::{Annotation, IconRef, Identity, Meta, NodeMeta, TreeNode};
pub use paint::Paint;
pub use replication::{PendingReplicationOp, ReplicationJournal, ReplicationOp};
pub use undo::{
    DEFAULT_MAX_BYTES, DEFAULT_MAX_FRAMES, LocalSnapshot, LocalSubtree, OopsCategory, OopsKind, ShowSubtree, SubtreeSnapshot, UndoFrame,
    UndoStack, UndoStep, UserUndoStep,
};
pub use value::{Absolute, Step, Value, next_stamp};
pub use variables::{VariableStore, VariableValue};

