#!

# NOTE: sudo apt install google-perftools

# WARNING: only works on Ubuntu/Pop_OS! (probably...)

LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libprofiler.so.0 CPUPROFILE=bmark.prof ./target/release/benchmark -N 10000 -r 500 \
-L 10000 -S 101 -n 300 dynamic
google-pprof -pdf ./target/release/benchmark bmark.prof > bmark.pdf

