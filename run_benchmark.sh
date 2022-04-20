#!/usr/bin/bash

N=10000
nsteps=1000
nsteps=300
L=10000

for rho in 500 # 1000 10000
do
    echo "dynamic -> $rho"
    /usr/bin/time -f "%e %M" ./target/release/benchmark --popsize $N -r $rho -s $L --seed1 101 --seed2 202 --nsteps $nsteps -d 1.0 dynamic

    echo "tskit -> $rho"
    /usr/bin/time -f "%e %M" ./target/release/benchmark --popsize $N -r $rho -s $L --seed1 101 --seed2 202 --nsteps $nsteps -d 1.0 tskit -s 1
done
