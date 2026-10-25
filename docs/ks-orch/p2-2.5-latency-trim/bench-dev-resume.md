
### P (62c9fd8) + D-R16d probe, run 1, developer resume session, CPU load before 66%

### unlabelled

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.143 | 0.100 | 0.500 | 108.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.236 | 0.200 | 0.700 | 99.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.113 | 0.100 | 0.200 | 24.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.507 | 5.600 | 70.600 | 1158.800 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 19.069 | 7.900 | 187.700 | 349.400 |
| R6/5 | Hold(5) lateness | 10 | 446.710 | 358.500 | 872.000 | 872.000 |
| R6/20 | Hold(20) lateness | 10 | 412.460 | 331.900 | 650.100 | 650.100 |
| R6/50 | Hold(50) lateness | 10 | 386.440 | 325.700 | 753.200 | 753.200 |
| R6/200 | Hold(200) lateness | 10 | 404.730 | 353.000 | 621.500 | 621.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.202 | 13.600 | 94.900 | 289.300 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 6.925 | 4.900 | 57.700 | 579.000 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 8.566 | 5.700 | 69.400 | 305.300 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 8.240 | 5.700 | 95.600 | 342.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 6.737 | 4.900 | 63.200 | 252.100 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 7.023 | 4.900 | 62.500 | 337.500 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 8.018 | 5.100 | 82.100 | 429.300 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 17.724 | 11.900 | 106.900 | 296.300 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 19.555 | 7.600 | 195.900 | 477.400 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 17.949 | 6.000 | 180.900 | 790.500 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 5.708 | 4.400 | 31.500 | 188.600 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 96.735 | 66.800 | 480.400 | 2309.000 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 60.693 | 43.600 | 314.700 | 1622.300 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.002 | 0.010 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 0.654 | 0.602 | 1.029 | 3.369 |

### commit 11 (aa86922) + D-R16d probe, run 1, developer resume session, CPU load before 79%

### unlabelled

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.113 | 0.100 | 0.400 | 556.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.144 | 0.100 | 0.200 | 6.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 0.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 0.533 | 0.300 | 7.100 | 37.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.556 | 0.400 | 2.200 | 17.200 |
| R6/5 | Hold(5) lateness | 10 | 397.230 | 350.100 | 761.600 | 761.600 |
| R6/20 | Hold(20) lateness | 10 | 399.650 | 333.000 | 657.700 | 657.700 |
| R6/50 | Hold(50) lateness | 10 | 443.980 | 433.100 | 765.400 | 765.400 |
| R6/200 | Hold(200) lateness | 10 | 407.550 | 341.100 | 1007.800 | 1007.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.963 | 11.100 | 97.700 | 218.700 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.200 | 0.200 | 0.200 | 9.300 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 1.095 | 1.100 | 1.200 | 14.900 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 1.088 | 0.800 | 9.600 | 23.700 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.182 | 0.200 | 0.200 | 0.500 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 0.871 | 0.800 | 4.100 | 18.100 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 1.922 | 1.300 | 20.400 | 68.000 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 16.430 | 10.900 | 87.900 | 158.500 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 0.618 | 0.200 | 5.000 | 32.400 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 2.254 | 1.200 | 11.100 | 64.600 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 5.741 | 4.100 | 38.600 | 160.000 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 6.641 | 4.400 | 30.400 | 89.700 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 21.435 | 18.900 | 75.300 | 512.100 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.002 | 0.014 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 1.602 | 1.082 | 8.850 | 29.005 |
| R18 | the main thread's read of the modifiers between key events (MainState::resync_if_due) | 2000 | 7.193 | 5.900 | 40.100 | 1017.200 |

### P (62c9fd8) + D-R16d probe, run 2, developer resume session, CPU load before 90%

### unlabelled

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.177 | 0.100 | 0.600 | 260.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.339 | 0.300 | 1.500 | 12.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.127 | 0.100 | 0.300 | 12.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 9.750 | 7.100 | 95.000 | 634.800 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 8.033 | 6.200 | 47.900 | 126.400 |
| R6/5 | Hold(5) lateness | 10 | 383.430 | 257.100 | 901.700 | 901.700 |
| R6/20 | Hold(20) lateness | 10 | 404.620 | 293.900 | 1171.800 | 1171.800 |
| R6/50 | Hold(50) lateness | 10 | 547.410 | 511.600 | 942.000 | 942.000 |
| R6/200 | Hold(200) lateness | 10 | 755.240 | 729.900 | 1222.500 | 1222.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 73.262 | 24.500 | 814.600 | 3732.600 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 18.341 | 7.000 | 219.300 | 1973.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 17.296 | 8.100 | 167.400 | 518.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 13.092 | 6.900 | 147.100 | 426.600 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 12.485 | 6.600 | 130.500 | 555.900 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 9.748 | 5.000 | 108.500 | 443.900 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 7.355 | 6.000 | 48.900 | 200.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 15.957 | 13.100 | 61.000 | 305.700 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 28.848 | 11.300 | 332.800 | 1817.300 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 32.118 | 10.800 | 611.700 | 2943.500 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 8.982 | 6.200 | 62.100 | 1078.500 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 121.682 | 68.400 | 808.900 | 5508.600 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 295.574 | 99.000 | 2576.400 | 8312.200 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.002 | 0.001 | 0.003 | 0.402 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 1.648 | 1.098 | 9.366 | 17.789 |

### commit 11 (aa86922) + D-R16d probe, run 2, developer resume session, CPU load before 100%

### unlabelled

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.172 | 0.100 | 0.500 | 267.500 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.259 | 0.300 | 0.400 | 7.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.160 | 0.100 | 0.400 | 87.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 1.051 | 0.400 | 1.900 | 1049.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.696 | 0.500 | 1.200 | 29.900 |
| R6/5 | Hold(5) lateness | 10 | 536.050 | 631.600 | 702.000 | 702.000 |
| R6/20 | Hold(20) lateness | 10 | 537.260 | 377.800 | 1278.800 | 1278.800 |
| R6/50 | Hold(50) lateness | 10 | 379.430 | 308.300 | 569.300 | 569.300 |
| R6/200 | Hold(200) lateness | 10 | 475.260 | 436.900 | 758.300 | 758.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 35.973 | 21.300 | 188.100 | 497.200 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.370 | 0.200 | 0.400 | 238.900 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 1.165 | 0.800 | 1.400 | 583.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 1.320 | 1.100 | 2.200 | 103.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.215 | 0.200 | 0.300 | 2.500 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 2.009 | 1.100 | 24.300 | 94.200 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 3.294 | 1.900 | 37.600 | 103.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 39.436 | 20.500 | 321.200 | 2022.000 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 11.065 | 4.800 | 25.700 | 4960.100 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 11.592 | 4.900 | 45.800 | 4571.300 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 11.493 | 6.700 | 115.700 | 1915.900 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 33.373 | 21.700 | 239.500 | 4154.400 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 31.466 | 19.300 | 277.000 | 1500.400 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.002 | 0.133 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 2.366 | 1.116 | 20.483 | 39.603 |
| R18 | the main thread's read of the modifiers between key events (MainState::resync_if_due) | 2000 | 16.222 | 6.400 | 186.400 | 4947.700 |

### HEAD (commit 13 code) run 1, developer resume session, CPU load before 100%

### HEAD (commit 13 code) run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.224 | 0.100 | 0.600 | 1205.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.266 | 0.200 | 0.600 | 132.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.434 | 0.100 | 0.400 | 2243.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 2.753 | 0.400 | 2.600 | 5663.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.643 | 0.500 | 1.600 | 18.600 |
| R6/5 | Hold(5) lateness | 10 | 432.540 | 418.300 | 678.800 | 678.800 |
| R6/20 | Hold(20) lateness | 10 | 943.700 | 708.500 | 2734.700 | 2734.700 |
| R6/50 | Hold(50) lateness | 10 | 3031.330 | 895.700 | 16050.400 | 16050.400 |
| R6/200 | Hold(200) lateness | 10 | 601.050 | 416.600 | 2234.200 | 2234.200 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 31.724 | 19.900 | 316.400 | 763.300 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.304 | 0.300 | 0.900 | 24.100 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 4.189 | 1.700 | 7.400 | 2658.100 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 11.248 | 1.300 | 65.300 | 7189.500 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.952 | 0.200 | 0.500 | 1149.300 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 71.010 | 1.400 | 144.700 | 75574.900 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 26.580 | 3.000 | 160.600 | 11989.100 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 43.673 | 25.000 | 405.300 | 767.300 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 11.549 | 6.100 | 46.400 | 4028.800 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 9.983 | 6.200 | 55.500 | 866.500 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 16.034 | 6.700 | 172.000 | 2071.500 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 44.624 | 24.000 | 425.900 | 3011.000 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.001 | 0.004 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 0.871 | 0.726 | 2.560 | 14.471 |
| R18 | the main thread's read of the modifiers between key events (MainState::resync_if_due) | 2000 | 8.590 | 5.700 | 76.700 | 340.700 |

### P (62c9fd8) + D-R16d probe, run 3, developer resume session, CPU load before 47%

### P (62c9fd8) + D-R16d probe, run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.107 | 0.100 | 0.300 | 32.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.165 | 0.200 | 0.300 | 3.500 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.082 | 0.100 | 0.100 | 2.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.687 | 5.000 | 43.300 | 304.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.433 | 4.900 | 21.600 | 139.400 |
| R6/5 | Hold(5) lateness | 10 | 443.530 | 450.000 | 644.100 | 644.100 |
| R6/20 | Hold(20) lateness | 10 | 455.160 | 375.600 | 703.300 | 703.300 |
| R6/50 | Hold(50) lateness | 10 | 413.910 | 404.300 | 639.800 | 639.800 |
| R6/200 | Hold(200) lateness | 10 | 379.510 | 418.100 | 605.500 | 605.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.613 | 12.600 | 96.200 | 367.600 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.786 | 4.800 | 19.700 | 226.800 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 8.026 | 5.900 | 35.500 | 699.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 6.474 | 5.400 | 11.200 | 323.300 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.278 | 4.800 | 7.100 | 48.800 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 5.876 | 4.800 | 15.500 | 282.000 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 5.400 | 4.800 | 7.200 | 384.200 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 13.466 | 10.800 | 65.100 | 145.900 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 16.194 | 6.700 | 169.000 | 493.300 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 24.299 | 7.900 | 252.000 | 1170.300 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 5.575 | 4.100 | 26.800 | 171.700 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 94.182 | 66.800 | 384.600 | 7400.200 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 51.748 | 37.400 | 245.400 | 2409.600 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.002 | 0.003 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 0.626 | 0.584 | 1.175 | 1.825 |

### HEAD (commit 13 code) run 2, developer resume session, CPU load before 39%

### HEAD (commit 13 code) run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.080 | 0.100 | 0.200 | 31.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.176 | 0.200 | 0.400 | 5.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 2.000 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 0.486 | 0.200 | 5.900 | 26.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.345 | 0.300 | 0.500 | 8.700 |
| R6/5 | Hold(5) lateness | 10 | 281.410 | 179.000 | 495.300 | 495.300 |
| R6/20 | Hold(20) lateness | 10 | 384.560 | 352.600 | 672.500 | 672.500 |
| R6/50 | Hold(50) lateness | 10 | 259.290 | 202.100 | 569.700 | 569.700 |
| R6/200 | Hold(200) lateness | 10 | 345.960 | 222.500 | 642.400 | 642.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.712 | 11.500 | 82.600 | 251.100 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.197 | 0.200 | 0.200 | 5.400 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 1.121 | 0.900 | 1.700 | 46.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 1.177 | 0.800 | 11.100 | 54.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.230 | 0.200 | 0.400 | 2.900 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 1.050 | 0.800 | 8.700 | 58.500 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 1.823 | 1.600 | 13.900 | 99.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 53.756 | 12.800 | 419.300 | 9975.000 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 0.616 | 0.200 | 5.000 | 23.300 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 2.137 | 0.900 | 11.900 | 43.800 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.311 | 3.900 | 14.700 | 38.000 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 7.226 | 5.100 | 29.200 | 44.000 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.001 | 0.196 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 0.705 | 0.606 | 1.647 | 7.921 |
| R18 | the main thread's read of the modifiers between key events (MainState::resync_if_due) | 2000 | 4.732 | 4.600 | 6.900 | 36.600 |

### HEAD (commit 13 code) run 3, developer resume session, CPU load before 39%

### HEAD (commit 13 code) run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.092 | 0.100 | 0.300 | 79.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.173 | 0.200 | 0.400 | 23.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.084 | 0.100 | 0.100 | 1.800 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 0.389 | 0.200 | 1.100 | 17.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.395 | 0.300 | 1.400 | 11.400 |
| R6/5 | Hold(5) lateness | 10 | 316.550 | 189.300 | 584.100 | 584.100 |
| R6/20 | Hold(20) lateness | 10 | 386.870 | 360.900 | 626.100 | 626.100 |
| R6/50 | Hold(50) lateness | 10 | 335.510 | 307.300 | 618.500 | 618.500 |
| R6/200 | Hold(200) lateness | 10 | 435.650 | 430.900 | 718.300 | 718.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.463 | 10.700 | 24.000 | 84.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.199 | 0.200 | 0.200 | 5.600 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 0.720 | 0.700 | 0.800 | 14.500 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 0.991 | 0.800 | 1.100 | 36.700 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.194 | 0.200 | 0.200 | 3.100 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 0.772 | 0.800 | 0.800 | 2.700 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 1.334 | 1.300 | 1.400 | 2.300 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 12.760 | 11.000 | 30.600 | 131.200 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 0.623 | 0.200 | 4.900 | 11.000 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 1.183 | 0.300 | 6.900 | 432.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 7.467 | 4.800 | 58.000 | 307.600 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 10.623 | 5.600 | 37.500 | 1311.000 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.003 | 0.025 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 1.026 | 0.891 | 2.785 | 5.789 |
| R18 | the main thread's read of the modifiers between key events (MainState::resync_if_due) | 2000 | 9.193 | 6.700 | 27.900 | 1312.300 |
