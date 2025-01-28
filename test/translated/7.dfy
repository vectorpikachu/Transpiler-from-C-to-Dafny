class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    decreases *
    modifies this
  {
    var x : int := *;
    var y : int := *;
    assume {:axiom} (x >= 0);
    assume {:axiom} (x <= 10);
    assume {:axiom} (y <= 10);
    assume {:axiom} (y >= 0);
    while *
      decreases *
      invariant exists n :: n * 10 <= x <= n * 10 + 10
    {
      x := (x + 10);
      y := (y + 10);
    }
    if (x == 20) {
      assert((y != 0));
    }
    return 0;
  }

}

