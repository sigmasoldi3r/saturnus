use super::compile;
use spectral::prelude::*;

#[test]
fn simple_class_test() {
    let out = compile(
        "
    class Hello
      let value = 0;
      fn tick(self)
        return 10 + self.value;
      end
    end
  ",
    );
    assert_that!(out).is_equal_to(
        "
local Hello = {};
Hello.__meta__ = {};
Hello.__meta__.__call = function(self, struct)
  return setmetatable(struct, self.prototype.__meta__);
end;
Hello.prototype = {};
Hello.prototype.__meta__ = {};
Hello.prototype.__meta__.__index = Hello.prototype;
setmetatable(Hello, Hello.__meta__);
Hello.prototype.value = 0;
Hello.prototype.tick = function(self)
  return 10 + self.value;
end;"
            .to_string(),
    );
}

#[test]
fn struct_construction_syntax() {
    let out = compile("let hello = Hello { name: \"World\" };");
    assert_that!(out).is_equal_to(
        "
local hello = Hello({name = \"World\"});"
            .to_string(),
    );
}
