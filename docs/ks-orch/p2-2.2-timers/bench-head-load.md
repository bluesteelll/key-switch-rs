
### HEAD at C14 with KS_BENCH_LOAD=1, run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.138 | 0.100 | 0.400 | 52.200 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.183 | 0.200 | 0.400 | 17.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.093 | 0.100 | 0.200 | 4.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.857 | 5.200 | 16.200 | 311.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.289 | 5.300 | 35.200 | 51.900 |
| R6/5 | Hold(5) lateness | 10 | 280.110 | 201.800 | 511.400 | 511.400 |
| R6/20 | Hold(20) lateness | 10 | 479.000 | 500.300 | 653.700 | 653.700 |
| R6/50 | Hold(50) lateness | 10 | 843.540 | 566.900 | 1553.400 | 1553.400 |
| R6/200 | Hold(200) lateness | 10 | 498.170 | 540.800 | 1012.000 | 1012.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.901 | 15.000 | 77.800 | 497.700 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 27.788 | 16.400 | 252.100 | 545.400 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 28.721 | 18.800 | 234.100 | 566.000 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 39.109 | 26.500 | 266.600 | 557.600 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 40.496 | 29.100 | 255.600 | 600.100 |

### HEAD at C14 with KS_BENCH_LOAD=1, run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.133 | 0.100 | 0.400 | 46.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.229 | 0.200 | 0.700 | 6.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.097 | 0.100 | 0.200 | 8.200 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.994 | 5.700 | 44.100 | 944.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 9.891 | 6.500 | 63.100 | 118.700 |
| R6/5 | Hold(5) lateness | 10 | 528.510 | 329.400 | 1078.700 | 1078.700 |
| R6/20 | Hold(20) lateness | 10 | 456.650 | 471.800 | 539.800 | 539.800 |
| R6/50 | Hold(50) lateness | 10 | 434.340 | 482.100 | 620.500 | 620.500 |
| R6/200 | Hold(200) lateness | 10 | 443.970 | 454.100 | 684.700 | 684.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 15.597 | 12.300 | 58.900 | 205.100 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 53.442 | 18.600 | 561.000 | 2949.700 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 22.489 | 15.100 | 204.500 | 350.900 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 71.326 | 29.800 | 918.600 | 2974.200 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 33.540 | 24.000 | 214.400 | 384.200 |

### HEAD at C14 with KS_BENCH_LOAD=1, run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.114 | 0.100 | 0.300 | 65.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.253 | 0.200 | 0.500 | 126.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.111 | 0.100 | 0.200 | 72.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.697 | 5.600 | 85.800 | 773.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.065 | 5.800 | 8.900 | 19.800 |
| R6/5 | Hold(5) lateness | 10 | 378.110 | 405.300 | 594.500 | 594.500 |
| R6/20 | Hold(20) lateness | 10 | 429.720 | 400.800 | 686.300 | 686.300 |
| R6/50 | Hold(50) lateness | 10 | 427.370 | 394.700 | 972.100 | 972.100 |
| R6/200 | Hold(200) lateness | 10 | 471.470 | 432.800 | 714.700 | 714.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 23.328 | 16.500 | 113.200 | 544.400 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 131.039 | 24.000 | 2632.300 | 3222.800 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 44.686 | 23.500 | 393.700 | 502.000 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 161.891 | 38.100 | 2812.300 | 4228.400 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 61.012 | 36.200 | 443.000 | 523.800 |
