---
name: motion-law
description: Animation and motion design standards
platforms: [cli, telegram, discord, slack, api]
---

# Motion Law

## Purpose
Ensures animations and transitions enhance UX without distracting.

## Rules

1. **Duration**: Micro-interactions 100-200ms. Transitions 200-400ms. Complex animations 400-800ms. Never exceed 1s.
2. **Easing**: Use ease-out for entrances, ease-in for exits, ease-in-out for state changes. Never linear for UI.
3. **Choreography**: Elements enter in reading order (top-left to bottom-right). Stagger 50-100ms between items.
4. **Reduce motion**: Always respect `prefers-reduced-motion`. Provide instant alternatives.
5. **Purpose**: Every animation must serve a purpose — guide attention, show state change, or provide feedback. Decorative animation is banned.
6. **Performance**: Use transform and opacity only for animations. Never animate layout properties (width, height, top, left).

## Checklist
- [ ] Durations within specified ranges
- [ ] Easing curves match interaction type
- [ ] prefers-reduced-motion respected
- [ ] Only transform/opacity animated
- [ ] Animation serves functional purpose
