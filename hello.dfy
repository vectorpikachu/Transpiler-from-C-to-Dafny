
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
  method main(a1: array<int>, a2: array<int>, a3: array<int>, a4: array<int>, a5: array<int>, a6: array<int>, a7: array<int>, a8: array<int>, a9: array<int>, N: int) returns (ret: int)
    requires true
    modifies a1, a2, a3, a4, a5, a6, a7, a8, a9, this
  {
    var a : int := *;
    a := 0;
    while (a < N) 
    {
      a1[a] := *;
      a := (a + 1);
    }
    var i : int := *;
    i := 0;
    while (i < N) 
    {
      a2[i] := a1[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a3[i] := a2[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a4[i] := a3[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a5[i] := a4[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a6[i] := a5[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a7[i] := a6[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a8[i] := a7[i];
      i := (i + 1);
    }
    i := 0;
    while (i < N) 
    {
      a9[i] := a8[i];
      i := (i + 1);
    }
    var x : int := *;
    x := 0;
    while (x < N) 
    {
      assert((a1[x] == a9[x]));
      x := (x + 1);
    }
    return 0;
  }

}

