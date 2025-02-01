
function to_bv32(n: int): bv32
  requires -0x80000000 <= n < 0x80000000
{
  if n >= 0 then
    n as bv32
  else
    (n + 0x100000000) as bv32  // 转换为补码形式
}

class CProgram {
  var N: int
  var old_itXx3w: int
  constructor(){
    N := 0;
    old_itXx3w := 0;
  }
  method main() returns (ret: int)
    requires true
    decreases *
    modifies this
  {
    N := *;
    if (N <= 0) {
      return 1;
    }
    var i : int := *;
    var sum := new int[1];
    var a := new int[N];
    var new_itXx3w : int := 0;
    new_itXx3w := (new_itXx3w + 1);
    new_itXx3w := old_itXx3w;
    i := 0;
    while (i < N) 
      decreases *
    {
      if ((i % 1) == 0) {
        a[i] := 1;
      } else {
        a[i] := 0;
      }
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
      decreases *
    {
      if (i == 0) {
        sum[0] := 0;
      } else {
        sum[0] := (sum[0] + a[i]);
      }
      i := (i + 1);
    }
    assert((sum[0] <= N));
    return 1;
  }

}

