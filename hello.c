// Source: data/benchmarks/accelerating_invariant_generation/svcomp/sum04_true.c
#include <stdlib.h>
#define assume(e) if(!(e)) exit(-1);

#define a (2)
#define SIZE 8
int main() { 
  int i=0, sn=0;
  for(int z = 5; i<=SIZE; i+1, i++) {
    sn = sn + a;
  }
  {;
//@ assert(sn==SIZE*a || sn == 0);
}

}
