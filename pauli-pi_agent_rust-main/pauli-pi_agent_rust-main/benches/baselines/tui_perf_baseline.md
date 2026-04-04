# TUI Performance Baselines (PERF-8)

Captured: 2026-02-13
Platform: Linux x86_64, 64 cores
Build: dev profile (unoptimized + debuginfo)
Criterion: 100 samples per benchmark

## build_conversation_content

| Messages | p50 (median) |
|----------|-------------|
| 10       | ~160 us     |
| 50       | ~713 us     |
| 100      | ~1.37 ms    |
| 500      | ~6.83 ms    |

## view (full render cycle)

| Messages | p50 (median) |
|----------|-------------|
| 0        | ~24 us      |
| 10       | ~203 us     |
| 50       | ~785 us     |
| 200      | ~3.17 ms    |

## message_generation (data creation overhead)

| Messages | p50 (median) |
|----------|-------------|
| 10       | ~924 ns     |
| 100      | ~12.1 us    |
| 1000     | ~128 us     |

## viewport_operations

| Operation           | p50 (median) |
|--------------------|-------------|
| set_content 5000L  | ~2.07 ms    |
| page_up 10000L     | ~2.8 ns     |
| page_down 10000L   | ~5.97 ns    |
| goto_bottom 10000L | ~7.36 ns    |

## markdown_rendering

| Input                       | p50 (median) |
|----------------------------|-------------|
| short (100 chars)          | ~17.9 us    |
| long (10KB + code + tables) | ~96 us     |

## Notes

- build_conversation_content re-renders all messages every frame (no caching yet)
- view() cost is dominated by build_conversation_content + markdown rendering
- Viewport navigation (page_up/down/goto_bottom) is sub-10ns (very fast)
- set_content is ~2ms due to line counting/splitting
- 200-message view() at 3.17ms is within 16.67ms frame budget but leaves limited headroom
- 500-message content build at 6.83ms would be problematic with full view() overhead
