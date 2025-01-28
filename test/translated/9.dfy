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
    assume {:axiom} (x <= 2);
    assume {:axiom} (y <= 2);
    assume {:axiom} (y >= 0);
    while *
      decreases *
    {
      x := (x + 2);
      y := (y + 2);
    }
    if (x == 4) {
      assert((y != 0));
    }
    return 0;
  }

}

