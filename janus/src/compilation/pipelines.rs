use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::display::get_bar;

use super::{CompilationInfo, CompilationTarget};

pub struct FilePipeline;
impl FilePipeline {
    pub const LUA_GLUE_HEADER: &'static [u8] = b"local __modules__ = {};
    do
      local __native_require__ = require;
      require = function(fp)
        if __modules__[fp] ~= nil then
          if package.loaded[fp] == nil then
            package.loaded[fp] = __modules__[fp]();
          end
          return package.loaded[fp];
        end
        return __native_require__(fp);
      end;
    end";

    /// Collects all the source objects into a single file, if supported by the target.
    pub fn collect_file(
        &self,
        info: &CompilationInfo,
        objects: &Vec<PathBuf>,
        objects_base_path: &PathBuf,
        target_base_path: &PathBuf,
        output: Option<PathBuf>,
    ) {
        let pb = get_bar(objects.len() as u64);
        let mut main: Option<PathBuf> = None;
        let mut file_out = match info.target {
            CompilationTarget::Lua => {
                let out_path = info.output.join("target").join("main.lua");
                let mut file_out = File::create(output.unwrap_or(out_path)).unwrap();
                file_out.write_all(Self::LUA_GLUE_HEADER).unwrap();
                file_out
            }
        };
        let mut main_path = objects_base_path.join(info.main.strip_prefix(&info.source).unwrap());
        main_path.set_extension("lua");
        for entry in objects.iter() {
            if entry == &main_path {
                main = Some(entry.clone());
                continue;
            }
            let base_target = entry.strip_prefix(&objects_base_path).unwrap();
            let target = target_base_path.join(base_target);
            pb.set_message(format!("Linking {:?}...", &target));
            let src = fs::read_to_string(entry).unwrap();
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
                    "\n__modules__[\"{}\"] = function()\n",
                    path_name
                ))
                .unwrap();
            file_out.write_all(&src.as_bytes()).unwrap();
            file_out.write(b"\nend;").unwrap();
            pb.inc(1);
        }
        if let Some(entry) = main {
            pb.set_message("Linking standard library...");
            file_out
                .write(b"\n__modules__[\"std\"] = function()\n")
                .unwrap();
            file_out
                .write_all(
                    fs::read_to_string(objects_base_path.join("std.lua"))
                        .unwrap()
                        .as_bytes(),
                )
                .unwrap();
            file_out.write(b"\nend;").unwrap();
            pb.set_message("Collecting main file...");
            let src = fs::read_to_string(entry).unwrap();
            file_out.write(b"\n").unwrap();
            file_out.write_all(&src.as_bytes()).unwrap();
            pb.inc(1);
        }
        pb.finish_with_message("Done");
    }
}
