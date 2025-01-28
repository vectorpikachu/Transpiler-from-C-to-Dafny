class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    requires true
    decreases *
    modifies this
  {
    var x : int := *;
    var y : int := *;
    var z : int := *;
    assume {:axiom} (x < 100);
    assume {:axiom} (x > -100);
    assume {:axiom} (z < 100);
    assume {:axiom} (z > -100);
    while ((x < 100) && (100 < z))
      decreases *
    {
      var tmp : int := *;
      if tmp != 0 {
        x := (x + 1);
      } else {
        x := (x - 1);
        z := (z - 1);
      }
    }
    assert(((x >= 100) || (z <= 100)));
    return 0;
  }

}

