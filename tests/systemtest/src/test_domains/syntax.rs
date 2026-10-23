use lumacore_objects::commands::registry::all_words;

use crate::defs::*;

pub fn tests() -> Vec<TestDef> {
    let skip_verbs = [
        "exit",
        "logout",
        "shutdown",
        "hissme",
        "rebuild",
        "import",
        "export",
        "save",
        "load",
        "login",
        "user",
        "shell",
        "cmdline",
        "sysmon",
        "sysnow",
        "disconnect",
    ];
    let mut tests = Vec::new();
    // Every command and object documents examples; run them all.
    for word in all_words() {
        let name = word.names()[0].to_lowercase();
        if word.doc().hidden || skip_verbs.contains(&name.as_str()) {
            continue;
        }
        let mut steps = Vec::new();
        for example in word.doc().examples {
            let trimmed = example
                .split("--")
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            if trimmed.is_empty() {
                continue;
            }
            let leaked_name: &'static str = Box::leak(trimmed.clone().into_boxed_str());
            let leaked_cmd: &'static str = leaked_name;
            steps.push(StepDef {
                name: leaked_name,
                kind: StepKind::SyntaxCommand(leaked_cmd),
            });
        }
        if !steps.is_empty() {
            tests.push(TestDef {
                name: word.label(),
                user: "Admin",
                gates: vec![GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                }],
                steps,
            });
        }
    }
    tests
}
