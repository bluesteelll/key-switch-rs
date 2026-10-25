
### master alt 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.123 | 0.100 | 0.300 | 142.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.124 | 0.100 | 0.200 | 11.200 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.087 | 0.100 | 0.100 | 76.100 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.039 | 5.500 | 47.900 | 1011.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 60.268 | 40.900 | 264.400 | 609.300 |
| R6/5 | Hold(5) lateness | 10 | 720.470 | 554.700 | 1299.900 | 1299.900 |
| R6/20 | Hold(20) lateness | 10 | 476.610 | 399.000 | 1228.800 | 1228.800 |
| R6/50 | Hold(50) lateness | 10 | 776.390 | 555.000 | 2010.700 | 2010.700 |
| R6/200 | Hold(200) lateness | 10 | 669.880 | 575.700 | 1198.700 | 1198.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 23.896 | 14.200 | 202.800 | 599.200 |

### HEAD alt 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.141 | 0.100 | 0.500 | 196.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.175 | 0.200 | 0.300 | 15.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.094 | 0.100 | 0.200 | 7.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.689 | 5.300 | 24.200 | 792.800 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.169 | 5.400 | 9.800 | 33.700 |
| R6/5 | Hold(5) lateness | 10 | 376.100 | 381.100 | 642.500 | 642.500 |
| R6/20 | Hold(20) lateness | 10 | 399.130 | 329.400 | 783.100 | 783.100 |
| R6/50 | Hold(50) lateness | 10 | 677.180 | 493.700 | 1909.000 | 1909.000 |
| R6/200 | Hold(200) lateness | 10 | 447.740 | 443.900 | 718.500 | 718.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 22.980 | 13.800 | 215.300 | 541.600 |

### master alt 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.115 | 0.100 | 0.300 | 75.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.135 | 0.100 | 0.300 | 11.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.064 | 0.100 | 0.100 | 0.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.450 | 5.300 | 24.000 | 244.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 44.547 | 37.000 | 150.200 | 397.000 |
| R6/5 | Hold(5) lateness | 10 | 660.280 | 663.800 | 960.100 | 960.100 |
| R6/20 | Hold(20) lateness | 10 | 777.380 | 711.300 | 1865.200 | 1865.200 |
| R6/50 | Hold(50) lateness | 10 | 655.650 | 513.800 | 1855.000 | 1855.000 |
| R6/200 | Hold(200) lateness | 10 | 534.370 | 395.800 | 1044.200 | 1044.200 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.932 | 13.600 | 111.200 | 570.900 |

### HEAD alt 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.114 | 0.100 | 0.300 | 31.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.262 | 0.200 | 0.500 | 89.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.120 | 0.100 | 0.200 | 109.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 9.320 | 5.500 | 85.400 | 1801.800 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 7.301 | 5.700 | 30.800 | 71.500 |
| R6/5 | Hold(5) lateness | 10 | 399.930 | 351.500 | 687.700 | 687.700 |
| R6/20 | Hold(20) lateness | 10 | 438.210 | 419.500 | 676.600 | 676.600 |
| R6/50 | Hold(50) lateness | 10 | 542.930 | 370.800 | 1131.900 | 1131.900 |
| R6/200 | Hold(200) lateness | 10 | 462.220 | 360.200 | 818.500 | 818.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 29.069 | 17.200 | 255.100 | 773.900 |

### master alt 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.118 | 0.100 | 0.300 | 109.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.215 | 0.200 | 0.500 | 67.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.096 | 0.100 | 0.500 | 3.700 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.397 | 5.500 | 38.500 | 468.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 47.821 | 39.300 | 236.100 | 284.000 |
| R6/5 | Hold(5) lateness | 10 | 855.490 | 687.400 | 1298.000 | 1298.000 |
| R6/20 | Hold(20) lateness | 10 | 735.060 | 632.800 | 1235.300 | 1235.300 |
| R6/50 | Hold(50) lateness | 10 | 712.360 | 577.500 | 1234.000 | 1234.000 |
| R6/200 | Hold(200) lateness | 10 | 596.980 | 491.600 | 931.800 | 931.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.641 | 11.200 | 140.000 | 563.000 |

### HEAD alt 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.125 | 0.100 | 0.400 | 42.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.207 | 0.200 | 0.400 | 54.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.124 | 0.100 | 0.200 | 189.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.801 | 5.500 | 41.500 | 881.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 16.008 | 7.500 | 184.800 | 406.300 |
| R6/5 | Hold(5) lateness | 10 | 545.350 | 542.500 | 1031.400 | 1031.400 |
| R6/20 | Hold(20) lateness | 10 | 482.720 | 407.500 | 934.300 | 934.300 |
| R6/50 | Hold(50) lateness | 10 | 391.810 | 415.800 | 644.300 | 644.300 |
| R6/200 | Hold(200) lateness | 10 | 580.000 | 502.700 | 1339.000 | 1339.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 23.983 | 16.000 | 184.600 | 646.800 |

### master alt 4

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.111 | 0.100 | 0.300 | 38.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.301 | 0.200 | 0.600 | 438.200 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.083 | 0.100 | 0.200 | 2.200 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.070 | 5.700 | 45.700 | 1130.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 52.855 | 43.000 | 172.100 | 746.100 |
| R6/5 | Hold(5) lateness | 10 | 632.640 | 540.700 | 1148.500 | 1148.500 |
| R6/20 | Hold(20) lateness | 10 | 676.150 | 487.100 | 1605.300 | 1605.300 |
| R6/50 | Hold(50) lateness | 10 | 874.480 | 671.500 | 2202.700 | 2202.700 |
| R6/200 | Hold(200) lateness | 10 | 689.490 | 516.400 | 1655.000 | 1655.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 20.871 | 14.200 | 148.500 | 1016.700 |

### HEAD alt 4

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.123 | 0.100 | 0.400 | 52.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.188 | 0.200 | 0.400 | 23.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.103 | 0.100 | 0.200 | 20.100 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.656 | 5.500 | 39.000 | 640.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 7.171 | 5.800 | 23.200 | 72.200 |
| R6/5 | Hold(5) lateness | 10 | 638.050 | 575.200 | 1061.300 | 1061.300 |
| R6/20 | Hold(20) lateness | 10 | 527.140 | 420.500 | 862.700 | 862.700 |
| R6/50 | Hold(50) lateness | 10 | 455.940 | 410.900 | 684.900 | 684.900 |
| R6/200 | Hold(200) lateness | 10 | 566.690 | 413.600 | 1165.100 | 1165.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 24.613 | 18.400 | 156.200 | 649.400 |
