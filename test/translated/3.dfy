class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    decreases *
    modifies this
  {
    var x : int := 0;
    var y : int := *;
    var z : int := *;
    while (x < 5)
      decreases *
    {
      x := 1;
      if (z <= y) {
        y := z;
      }
    }
    assert((z >= y));
    return 0;
  }

}

