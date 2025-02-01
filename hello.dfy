
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
    var j : int := 1;
    var i : int := 10000;
    while ((i - j) >= 1) 
      decreases *
    {
      j := (j + 1);
      i := (i - 1);
    }
    return 0;
  }

}

