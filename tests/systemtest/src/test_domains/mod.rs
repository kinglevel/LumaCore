mod artnet;
mod engines;
mod fixtures;
mod groups;
mod health;
mod keybindings;
mod label;
mod navigation;
mod osc;
mod patch;
mod programmer;
mod runtime;
mod sacn;
mod syntax;
mod treeshell;
mod undo;
mod users;

use crate::defs::*;
use crate::harness::work_dir;

pub use health::run_command_batch;

pub(super) fn gate_demo_show_exists() -> Result<(), String> {
    let demo = work_dir().join("library/shows/demo.luma");
    if demo.exists() {
        Ok(())
    } else {
        Err(format!("missing demo show at {}", demo.display()))
    }
}

pub fn curated_groups() -> Vec<GroupDef> {
    vec![
        GroupDef {
            name: "Commandline",
            tests: {
                let mut t = fixtures::tests();
                t.extend(navigation::tests());
                t.extend(patch::tests());
                t.extend(groups::tests());
                t.extend(programmer::tests());
                t.extend(label::tests());
                t
            },
        },
        GroupDef {
            name: "Runtime",
            tests: runtime::tests(),
        },
        GroupDef {
            name: "Users",
            tests: users::tests(),
        },
        GroupDef {
            name: "Engines",
            tests: {
                let mut t = engines::tests();
                t.extend(artnet::tests());
                t.extend(sacn::tests());
                t.extend(osc::tests());
                t
            },
        },
        GroupDef {
            name: "Syntax",
            tests: syntax::tests(),
        },
        GroupDef {
            name: "Undo",
            tests: undo::tests(),
        },
        GroupDef {
            name: "TreeShell",
            tests: treeshell::tests(),
        },
        GroupDef {
            name: "Keybindings",
            tests: keybindings::tests(),
        },
        GroupDef {
            name: "Health",
            tests: health::tests(),
        },
    ]
}
