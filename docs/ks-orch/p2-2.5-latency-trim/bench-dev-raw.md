
### P (62c9fd8) + D-R16d probe, developer run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.192 | 0.200 | 0.500 | 79.000 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.321 | 0.300 | 0.500 | 23.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.143 | 0.100 | 0.200 | 22.700 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 10.467 | 8.400 | 60.600 | 822.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 15.193 | 8.400 | 169.500 | 620.800 |
| R6/5 | Hold(5) lateness | 10 | 394.760 | 383.800 | 649.200 | 649.200 |
| R6/20 | Hold(20) lateness | 10 | 373.800 | 390.600 | 564.600 | 564.600 |
| R6/50 | Hold(50) lateness | 10 | 422.140 | 395.300 | 608.800 | 608.800 |
| R6/200 | Hold(200) lateness | 10 | 415.840 | 433.800 | 654.200 | 654.200 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 23.616 | 20.000 | 93.700 | 123.900 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 15.552 | 8.800 | 160.900 | 357.300 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 16.414 | 10.700 | 150.900 | 413.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 14.256 | 10.000 | 156.400 | 647.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 10.886 | 8.800 | 94.300 | 368.900 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 10.809 | 8.800 | 70.400 | 387.300 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 12.966 | 8.600 | 135.800 | 2167.200 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 20.420 | 19.200 | 61.900 | 151.500 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 34.457 | 11.900 | 363.700 | 1150.900 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 23.349 | 11.400 | 214.300 | 721.500 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 8.572 | 7.200 | 45.400 | 210.400 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 122.625 | 85.800 | 712.000 | 3432.500 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 387.884 | 131.200 | 2642.700 | 7679.500 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.009 | 0.022 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 1.177 | 0.968 | 3.595 | 6.673 |

### N (62c9fd8 with the per-event sync line removed) + D-R16d probe, developer run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.111 | 0.100 | 0.400 | 60.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.254 | 0.300 | 0.500 | 58.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.091 | 0.100 | 0.200 | 4.800 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 0.643 | 0.300 | 9.200 | 102.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.601 | 0.300 | 2.000 | 48.800 |
| R6/5 | Hold(5) lateness | 10 | 372.820 | 311.900 | 661.200 | 661.200 |
| R6/20 | Hold(20) lateness | 10 | 424.660 | 409.000 | 703.700 | 703.700 |
| R6/50 | Hold(50) lateness | 10 | 508.440 | 425.800 | 1053.700 | 1053.700 |
| R6/200 | Hold(200) lateness | 10 | 418.690 | 420.800 | 684.200 | 684.200 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.206 | 12.300 | 83.400 | 268.200 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.244 | 0.200 | 0.500 | 22.600 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 1.102 | 0.800 | 2.500 | 23.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 1.217 | 0.900 | 13.800 | 25.000 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.264 | 0.200 | 0.500 | 0.700 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 0.232 | 0.200 | 0.400 | 7.900 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 0.217 | 0.200 | 0.300 | 0.700 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 17.400 | 13.300 | 101.700 | 229.200 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 2.458 | 1.600 | 11.700 | 21.300 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 2.811 | 1.900 | 12.200 | 55.700 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 7.909 | 4.800 | 55.100 | 356.800 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 15.348 | 12.800 | 36.900 | 251.700 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 15.090 | 10.900 | 43.700 | 1575.400 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.002 | 0.263 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 1.432 | 1.017 | 5.004 | 11.590 |

### P (62c9fd8) + D-R16d probe, developer run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.099 | 0.100 | 0.200 | 44.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.154 | 0.200 | 0.200 | 7.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.082 | 0.100 | 0.100 | 4.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.089 | 4.900 | 6.500 | 155.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.168 | 4.900 | 10.200 | 28.300 |
| R6/5 | Hold(5) lateness | 10 | 282.220 | 231.900 | 550.600 | 550.600 |
| R6/20 | Hold(20) lateness | 10 | 385.880 | 351.700 | 590.900 | 590.900 |
| R6/50 | Hold(50) lateness | 10 | 289.300 | 209.100 | 516.300 | 516.300 |
| R6/200 | Hold(200) lateness | 10 | 333.530 | 275.300 | 566.100 | 566.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.469 | 10.700 | 25.600 | 134.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.011 | 4.800 | 7.200 | 67.000 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.357 | 5.000 | 6.600 | 229.700 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.406 | 5.400 | 5.500 | 22.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.945 | 4.800 | 6.000 | 104.000 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 4.869 | 4.800 | 6.000 | 15.500 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 4.812 | 4.800 | 4.900 | 39.000 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.118 | 10.500 | 25.300 | 50.200 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 6.169 | 4.900 | 27.900 | 419.900 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.410 | 4.900 | 12.500 | 108.000 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.268 | 4.000 | 8.500 | 196.900 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 61.880 | 58.300 | 170.600 | 663.700 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 30.856 | 28.700 | 111.300 | 322.900 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.001 | 0.006 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 0.629 | 0.583 | 1.016 | 1.367 |

### N (62c9fd8 with the per-event sync line removed) + D-R16d probe, developer run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.081 | 0.100 | 0.200 | 44.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.139 | 0.100 | 0.200 | 7.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.081 | 0.100 | 0.100 | 4.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 0.356 | 0.200 | 0.900 | 10.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.364 | 0.300 | 0.400 | 9.400 |
| R6/5 | Hold(5) lateness | 10 | 372.070 | 337.300 | 517.800 | 517.800 |
| R6/20 | Hold(20) lateness | 10 | 373.290 | 328.700 | 567.700 | 567.700 |
| R6/50 | Hold(50) lateness | 10 | 351.900 | 236.100 | 752.500 | 752.500 |
| R6/200 | Hold(200) lateness | 10 | 396.240 | 395.200 | 663.300 | 663.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 13.201 | 10.900 | 38.800 | 310.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.191 | 0.200 | 0.200 | 15.000 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 0.356 | 0.300 | 0.600 | 11.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 0.812 | 0.800 | 1.200 | 5.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.184 | 0.200 | 0.200 | 3.700 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 0.174 | 0.200 | 0.200 | 0.300 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 0.175 | 0.200 | 0.200 | 2.800 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 12.681 | 11.000 | 35.500 | 61.900 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 1.081 | 0.300 | 7.500 | 14.500 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 0.757 | 0.400 | 5.400 | 35.600 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 7.807 | 6.500 | 52.600 | 266.000 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 1.752 | 1.200 | 8.500 | 25.200 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 4.199 | 2.500 | 23.300 | 35.000 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.001 | 0.004 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 1.145 | 0.805 | 7.266 | 36.194 |

### commit 11 (aa86922, the lane without D12) + D-R16d probe, developer run 1, CPU load before 65%

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.093 | 0.100 | 0.300 | 85.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.233 | 0.200 | 0.900 | 10.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.096 | 0.100 | 0.300 | 7.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 0.551 | 0.200 | 3.200 | 92.300 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 0.780 | 0.400 | 3.000 | 45.000 |
| R6/5 | Hold(5) lateness | 10 | 440.720 | 450.500 | 648.300 | 648.300 |
| R6/20 | Hold(20) lateness | 10 | 384.070 | 335.800 | 657.500 | 657.500 |
| R6/50 | Hold(50) lateness | 10 | 487.970 | 487.100 | 955.200 | 955.200 |
| R6/200 | Hold(200) lateness | 10 | 473.780 | 388.100 | 683.400 | 683.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.095 | 12.200 | 99.600 | 338.100 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 0.193 | 0.200 | 0.200 | 13.600 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 0.902 | 0.800 | 1.300 | 22.100 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 1.386 | 0.800 | 12.800 | 196.300 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 0.199 | 0.200 | 0.300 | 0.800 |
| R12m | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl down | 2000 | 2.112 | 1.000 | 17.600 | 406.200 |
| R12m2 | keyboard_hook_callback, unbound A key-down, the daemon's injected LCtrl and LShift down | 2000 | 1.707 | 1.600 | 2.700 | 33.400 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 16.445 | 12.900 | 66.900 | 309.900 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 2.765 | 0.700 | 11.500 | 1764.200 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 2.556 | 1.400 | 13.400 | 90.500 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 10.864 | 6.400 | 90.800 | 286.200 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 7.360 | 4.300 | 30.000 | 337.800 |
| D-R16d | probe: A key-down next to a deferred wake (wait return, SetWaitableTimer, wait) | 1000 | 5.426 | 2.400 | 27.100 | 739.300 |
| R17t | GetTickCount, per call (mean of 1000 per sample) | 2000 | 0.001 | 0.001 | 0.001 | 0.004 |
| R17a | GetAsyncKeyState(VK_F13), per call (mean of 1000 per sample) | 2000 | 0.930 | 0.777 | 2.747 | 4.983 |
| R18 | the main thread's read of the modifiers between key events (MainState::resync_if_due) | 2000 | 6.808 | 4.800 | 42.500 | 282.200 |
