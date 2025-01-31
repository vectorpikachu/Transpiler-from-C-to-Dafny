
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
  constructor(){
    N := 0;
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
    assert a.Length == N;
    i := 0;
    while (i < N) 
      decreases *
      invariant 0 <= i <= N
      invariant a.Length == N
      invariant forall k : int :: 0 <= k < i ==> a[k] <= 1
    {
      assert 0 <= i < N;
      if ((i % 1) == 0) {
        assert 0 <= i < N;
        a[i] := 1;
        assert a[i] <= 1;
      } else {
        a[i] := 0;
        assert a[i] <= 1;
      }
      i := (i + 1);
    }
    i := 0;
    assert forall k : int :: 0 <= k < N ==> a[k] <= 1;
    while (i < N) 
      decreases *
      invariant 0 <= i <= N
      invariant a.Length == N
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

