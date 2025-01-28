class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    decreases *
    modifies this
  {
    var x : int := *;
    var y : int := *;
    x := 1;
    y := 0;
    while (y < 100000)
      decreases *
    {
      x := (x + y);
      y := (y + 1);
    }
    assert((x >= y));
    return 0;
  }

}

