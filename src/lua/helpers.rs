/// Generates the given source as a virtual module, used by Lua target.
pub fn generate_module_chunk(name: &String, source: &String) -> String {
    let tmp = format!("__saturnus_module_{name}");
    format!(
        "local {tmp} = function()
  {source}
end;
if jit then
  package.loaded[\"{name}\"] = {tmp}();
else
  package.preload[\"{name}\"] = {tmp};
end"
    )
}
