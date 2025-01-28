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
    {
      x := (x + 10);
      y := (y + 10);
    }
    if (y == 0) {
      assert((x != 20));
    }
    return 0;
  }

}

