use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::display::get_bar;

use super::{CompilationInfo, CompilationTarget};

pub struct FilePipeline;
impl FilePipeline {
    /// Collects all the source objects into a single file, if supported by the target.
    pub fn collect_file(
        &self,
        info: &CompilationInfo,
        objects: &Vec<PathBuf>,
        objects_base_path: &PathBuf,
        target_base_path: &PathBuf,
        output: Option<PathBuf>,
        exclude: &HashSet<PathBuf>,
    ) {
        let pb = get_bar(glob::glob("./dist/cache/objects/**/*.lua").unwrap().count() as u64);
        let mut mains: Vec<PathBuf> = vec![];
        let mut file_out = match info.target {
            CompilationTarget::Lua => {
                let out_path = info.output.join("target").join("main.lua");
                File::create(output.unwrap_or(out_path)).unwrap()
            }
        };
        let mut main_paths = match &info.main {
            crate::janusfile::PathBufOrPathBufList::PathBuf(main) => {
                vec![objects_base_path.join(main.strip_prefix(&info.source).unwrap())]
            }
            crate::janusfile::PathBufOrPathBufList::PathBufList(mains) => mains
                .iter()
                .map(|main| objects_base_path.join(main.strip_prefix(&info.source).unwrap()))
                .collect(),
        };
        for path in &mut main_paths {
            path.set_extension("lua");
        }
        for entry in glob::glob("./dist/cache/objects/**/*.lua").unwrap() {
            let entry = entry.expect("Could not unwrap an entry path");
            if main_paths.contains(&entry) {
                mains.push(entry.clone());
                continue;
            }
            if exclude.contains(&entry) {
                continue;
            }
            let base_target = entry.strip_prefix(&objects_base_path).unwrap();
            let target = target_base_path.join(base_target);
            pb.set_message(format!("Linking {:?}...", &target));
            let src = fs::read_to_string(&entry).unwrap();
            let mut path_name = entry.clone();
            path_name.set_extension("");
            let mut path_name = path_name.strip_prefix(&objects_base_path).unwrap();
            if path_name.file_name().unwrap() == "init" {
                path_name = path_name.parent().unwrap();
            }
            let path_name = path_name
                .to_string_lossy()
                .to_string()
                .replace("\\", ".")
                .replace("/", ".");
            file_out
                .write_fmt(format_args!(
                    "\npackage.preload[\"{}\"] = function()\n",
                    path_name
                ))
                .unwrap();
            file_out.write_all(&src.as_bytes()).unwrap();
            file_out.write(b"\nend;").unwrap();
            pb.inc(1);
        }
        for entry in mains {
            pb.set_message("Collecting entry files...");
            let src = fs::read_to_string(entry).unwrap();
            file_out.write(b"\n").unwrap();
            file_out.write_all(&src.as_bytes()).unwrap();
            pb.inc(1);
        }
        pb.finish_with_message("Done");
    }
}
