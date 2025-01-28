class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    decreases *
    modifies this
  {
    var x : int := 0;
    var size : int := *;
    var y : int := *;
    var z : int := *;
    while (x < size)
      decreases *
    {
      x := 1;
      if (z <= y) {
        y := z;
      }
    }
    if (size > 0) {
      assert((z >= y));
    }
    return 0;
  }

}

