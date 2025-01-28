class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    requires true
    decreases *
    modifies this
  {
    var x : real := *;
    assume {:axiom} (x > -1.0);
    assume {:axiom} (x < 1.0);
    var exp : real := 1.0;
    var term : real := 1.0;
    var count : bv32 := 1;
    var result : real := (2.0 * (1.0 / (1.0 - x)));
    var temp : int := *;
    while 1 != 0
      decreases *
      invariant {:axiom} count >= 1
    {
      term := (term * (x / count as real));
      exp := (exp + term);
      count := (count + 1);
      temp := *;
      if (temp == 0) {
        break;
      }
    }
    assert((result >= exp));
    return 0;
  }

}

