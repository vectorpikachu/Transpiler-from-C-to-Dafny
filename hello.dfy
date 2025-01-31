
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
    var x := new int[5];
    x[0] := 0;
    x[1] := 1;
    x[2] := 2;
    x[3] := 3;
    x[4] := 4;
    x[0] := 2;
    while (i < SIZE) 
      decreases *
    {
      i := (i + 1);
      sum := (sum as int + i) as bv64;
    }
    assert((sum == ((SIZE * (SIZE + 1)) / 2)));
    return 0;
  }

}

