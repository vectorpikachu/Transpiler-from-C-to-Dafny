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
    return (x + 1);
  }

  method main()
    decreases *
    modifies this
  {
    var i : int;
    var n : int;
    var a : int;
    var b : int;
    n := *;
    i := 0;
    a := 0;
    b := 0;
    t := 1;
    t0 := 1;
    var z : int := f(t);
    while (i < n)
      decreases *
    {
      i := (i + 1);
    }
    while (i < n)
      decreases *
    {
      if (*) {
        a := (a + 1);
        b := (b + 2);
        break;
      } else {
        a := (a + 2);
        b := (b + 1);
        continue;
      }
      i := (i + 1);
    }
    var c : int;
    c := (- a);
    c := (c + 1);
    if (((a + b) != (3 * n))) {
      assert false;
    }
    return ;
  }

}

