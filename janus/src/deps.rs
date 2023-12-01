use std::time::Duration;

use crate::{compilation::CompilationInfo, display::get_bar, janusfile::DependencyList};

pub fn resolve_deps(info: &CompilationInfo, dependencies: DependencyList) {
    let _ = info; // TODO!
    let pb = get_bar(dependencies.len() as u64);
    println!("Resolving dependencies...");
    for (name, _dep) in dependencies.into_iter() {
        pb.set_message(format!("Linking {}...", name));
        std::thread::sleep(Duration::from_millis(20));
        pb.inc(1);
    }
    pb.finish_with_message("Done");
}
