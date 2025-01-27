class CProgram {
  var t: int
  var t0: int
  constructor(){
    t := 0;
    t0 := 0;
  }
  method f(x: int) returns (ret: int)
    decreases *
    modifies this
  {
    return x + 1;
  }

  method main()
    decreases *
    modifies this
  {
    var i : int;
    var n : int;
    var a : int;
    var b : int;
    n := 0;
    i := 0;
    a := 0;
    b := 0;
    t := 0;
    t0 := 0;
    while (i < n)
      decreases *
    {
      i := 0;
    }
    return ;
  }

}

