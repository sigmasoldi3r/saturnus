use crate::runtime::RuntimeHost;

#[test]
fn test_basic_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/basic.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_class_decorators_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/class_decorators.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_collections_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/collections.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_destructuring_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/destructuring.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_extra_op_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/extra_op.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_function_decorators_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/function_decorators.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_hello_world_oop_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/hello_world_oop.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_js_style_programs_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/js_style_programs.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_loops_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/loops.saturn").to_owned();
    rt.run(&src).unwrap();
}

#[test]
fn test_oop_example() {
    let rt = RuntimeHost::new("  ".into());
    let src = include_str!("../../examples/oop.saturn").to_owned();
    rt.run(&src).unwrap();
}
