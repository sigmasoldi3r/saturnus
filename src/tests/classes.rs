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
Hello.__index = Hello;
Hello.__meta__.__call = function(self, struct)
  return setmetatable(struct, Hello);
end;
setmetatable(Hello, Hello.__meta__);
Hello.value = 0;
Hello.tick = function(self)
  return 10 + self.value;
end;"
            .to_string(),
    );
}
