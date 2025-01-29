class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    requires true
    decreases *
    modifies this
  {
    var n : bv8 := *;
    if (n == 0) {
      return 0;
    }
    var v : bv8 := 0;
    var s : bv32 := 0;
    var i : bv32 := 0;
    while (i < n as bv32) 
      decreases *
    {
      v := *;
      s := (s + v as bv32);
      i := (i + 1);
    }
    if (s < v as bv32) {
      assert(false);
      return 1;
    }
    if (s > 65025) {
      assert(false);
      return 1;
    }
    return 0;
  }

}

