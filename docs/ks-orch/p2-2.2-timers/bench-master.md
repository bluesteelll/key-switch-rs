
### master + bench (C1), run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.115 | 0.100 | 0.400 | 60.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.202 | 0.200 | 0.300 | 30.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.081 | 0.100 | 0.100 | 18.700 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.105 | 5.100 | 24.200 | 407.500 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 45.116 | 37.800 | 220.400 | 367.500 |
| R6/5 | Hold(5) lateness | 10 | 504.110 | 500.300 | 748.200 | 748.200 |
| R6/20 | Hold(20) lateness | 10 | 772.590 | 615.900 | 1952.900 | 1952.900 |
| R6/50 | Hold(50) lateness | 10 | 534.270 | 468.400 | 913.100 | 913.100 |
| R6/200 | Hold(200) lateness | 10 | 627.990 | 593.400 | 924.300 | 924.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 14.168 | 10.200 | 55.000 | 265.900 |

### master + bench (C1), run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.118 | 0.100 | 0.300 | 122.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.174 | 0.200 | 0.500 | 14.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.068 | 0.100 | 0.100 | 2.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.360 | 5.600 | 58.100 | 498.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 83.890 | 46.400 | 403.500 | 552.200 |
| R6/5 | Hold(5) lateness | 10 | 659.040 | 639.000 | 894.400 | 894.400 |
| R6/20 | Hold(20) lateness | 10 | 716.030 | 643.800 | 1528.200 | 1528.200 |
| R6/50 | Hold(50) lateness | 10 | 700.190 | 582.800 | 1118.900 | 1118.900 |
| R6/200 | Hold(200) lateness | 10 | 823.320 | 724.800 | 1615.900 | 1615.900 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 24.071 | 15.300 | 171.800 | 1270.900 |

### master + bench (C1), run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.128 | 0.100 | 0.400 | 44.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.151 | 0.100 | 0.300 | 8.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.064 | 0.100 | 0.100 | 0.200 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.255 | 5.300 | 50.400 | 1212.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 31.713 | 24.600 | 115.700 | 191.500 |
| R6/5 | Hold(5) lateness | 10 | 501.850 | 461.800 | 802.500 | 802.500 |
| R6/20 | Hold(20) lateness | 10 | 943.600 | 552.900 | 3145.700 | 3145.700 |
| R6/50 | Hold(50) lateness | 10 | 763.580 | 656.200 | 1660.700 | 1660.700 |
| R6/200 | Hold(200) lateness | 10 | 579.840 | 562.900 | 1034.000 | 1034.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.848 | 8.600 | 38.000 | 109.500 |

### master + bench (C1 611c83f), rerun 1 after HEAD

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 43.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.157 | 0.100 | 0.400 | 65.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.079 | 0.100 | 0.100 | 10.700 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.627 | 5.300 | 28.100 | 526.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 38.179 | 26.900 | 145.500 | 501.300 |
| R6/5 | Hold(5) lateness | 10 | 611.690 | 476.100 | 1183.700 | 1183.700 |
| R6/20 | Hold(20) lateness | 10 | 466.310 | 459.400 | 818.600 | 818.600 |
| R6/50 | Hold(50) lateness | 10 | 604.350 | 561.500 | 1216.200 | 1216.200 |
| R6/200 | Hold(200) lateness | 10 | 595.510 | 638.500 | 833.400 | 833.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 16.497 | 11.000 | 113.800 | 367.900 |

### master + bench (C1 611c83f), rerun 2 after HEAD

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.109 | 0.100 | 0.300 | 40.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.154 | 0.100 | 0.500 | 15.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.067 | 0.100 | 0.100 | 0.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.329 | 5.600 | 47.700 | 1392.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 38.175 | 34.100 | 125.300 | 143.900 |
| R6/5 | Hold(5) lateness | 10 | 606.920 | 472.900 | 1152.700 | 1152.700 |
| R6/20 | Hold(20) lateness | 10 | 810.210 | 657.700 | 2090.500 | 2090.500 |
| R6/50 | Hold(50) lateness | 10 | 579.350 | 490.300 | 1170.000 | 1170.000 |
| R6/200 | Hold(200) lateness | 10 | 493.050 | 442.600 | 850.100 | 850.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 15.429 | 12.900 | 58.300 | 170.000 |

### master + bench (C1 611c83f), rerun 3 after HEAD

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.115 | 0.100 | 0.300 | 37.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.137 | 0.100 | 0.300 | 8.200 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.066 | 0.100 | 0.100 | 0.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.855 | 5.400 | 30.600 | 543.300 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 41.730 | 34.500 | 148.800 | 248.300 |
| R6/5 | Hold(5) lateness | 10 | 676.750 | 448.800 | 1919.500 | 1919.500 |
| R6/20 | Hold(20) lateness | 10 | 710.750 | 605.600 | 1303.700 | 1303.700 |
| R6/50 | Hold(50) lateness | 10 | 569.220 | 479.800 | 740.400 | 740.400 |
| R6/200 | Hold(200) lateness | 10 | 781.860 | 592.200 | 1687.000 | 1687.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 16.529 | 13.200 | 82.700 | 237.900 |
