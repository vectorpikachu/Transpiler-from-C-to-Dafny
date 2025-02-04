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
    if * {
      assume {:axiom} (c1 as int != c2);
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

