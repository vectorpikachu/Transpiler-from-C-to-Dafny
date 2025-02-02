
function to_bv32(n: int): bv32
  requires -0x80000000 <= n < 0x80000000
{
  if n >= 0 then
    n as bv32
  else
    (n + 0x100000000) as bv32  // 转换为补码形式
}

class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    requires true
    decreases *
    modifies this
  {
    var c1 : bv8 := *;
    var c2 : int := *;
    var ac : bv8 := *;
    if ((* - 3) >= 5) {
    }
    ac := c1;
    while (ac != c2 as bv8) 
      decreases *
    {
      ac := (ac + 1);
    }
    return 0;
  }

}

