
function to_bv32(n: int): bv32
  requires -0x80000000 <= n < 0x80000000
{
  if n >= 0 then
    n as bv32
  else
    (n + 0x100000000) as bv32  // 转换为补码形式
}

class CProgram {
  var SIZE: int
  constructor(){
    SIZE := 40000;
  }
  method main() returns (ret: int)
    requires true
    decreases *
    modifies this
  {
    var i : int := *;
    var sum : bv64 := *;
    i := 0;
    sum := 0;
    while (i < SIZE) 
      decreases *
    {
      i := (i + 1);
      sum := to_bv32(sum as int + i) as bv64;
    }
    assert((sum == ((SIZE * (SIZE + 1)) / 2) as bv64));
    return 0;
  }

}

