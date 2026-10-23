// ███████╗██╗██╗  ██╗████████╗██╗   ██╗██████╗ ███████╗███████╗
// ██╔════╝██║╚██╗██╔╝╚══██╔══╝██║   ██║██╔══██╗██╔════╝██╔════╝
// █████╗  ██║ ╚███╔╝    ██║   ██║   ██║██████╔╝█████╗  ███████╗
// ██╔══╝  ██║ ██╔██╗    ██║   ██║   ██║██╔══██╗██╔══╝  ╚════██║
// ██║     ██║██╔╝ ██╗   ██║   ╚██████╔╝██║  ██║███████╗███████║
// ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝
// FILE: fixtures

//! The fixture data the show holds — see the crate's README.md (decisions §34 25.10): fixture
//! types with their revisions and modes, fixtures (their MVR part), the MVR metadata, the
//! attribute definitions, universes. Data
//! only; the patch's rules are `lumacore-patch`, the ops over `ShowData` stay in the tree.

pub mod attribute_def;
pub mod fixture;
pub mod fixture_type;
mod identity;
pub mod devices_meta;
pub mod fixture_mvr;
pub mod universe;

pub use attribute_def::{
    ActivationGroup, AttributeDefinition, AttributeRule, EncoderBarLayout, EncoderButton, EncoderResolution, FeatureGroup,
    NaturalReadout, PhysicalUnit, PlaceHow, PlacedAttribute, Placement, ProfileAttributes, RuleKind,
};
pub use fixture::{Fixture, FixtureBreak, LumaFixture, MasterReact, MultipatchCopy, Slot};
pub use devices_meta::{MvrClass, MvrFile, MvrGroup, MvrLayer, MvrMappingDefinition, MvrMetadata, MvrPosition};
pub use fixture_mvr::{IdType, Matrix, MvrFixture};
pub use fixture_type::{AttributeType, ChannelFunction, FileAttribute, FixtureCategory, FixtureType, Mode, Revision};
pub use universe::Universe;
