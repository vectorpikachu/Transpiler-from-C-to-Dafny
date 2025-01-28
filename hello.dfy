class CProgram {
  constructor(){
  }
  method main() returns (ret: int)
    decreases *
    modifies this
  {
    var i : int := 0;
    var sn : int := 0;
    var z : int := 5;
    while (i <= 8)
      decreases *
    {
      sn := (sn + 2);
      i := (i + 1);
    }
    assert(((sn == (8 * 2)) || (sn == 0)));
    return 0;
  }

}

