---
name: design-law
description: Design system constraints for UI/UX work
platforms: [cli, telegram, discord, slack, api]
---

# Design Law

## Purpose
Enforces design system consistency across all UI outputs.

## Rules

1. **Spacing**: Use 4px base grid. Spacing scale: 4, 8, 12, 16, 24, 32, 48, 64, 96.
2. **Typography scale**: 12, 14, 16, 18, 20, 24, 30, 36, 48, 60px. No arbitrary sizes.
3. **Border radius**: 0, 2, 4, 6, 8, 12px. Never exceed 12px unless circular (50%).
4. **Colors**: Use semantic tokens (--color-text-primary, --color-bg-surface) not raw hex.
5. **Components**: Reuse existing components before creating new ones. Check the component library.
6. **Accessibility**: WCAG 2.1 AA minimum. 4.5:1 contrast for body text, 3:1 for large text.
7. **Responsive**: Mobile-first. Breakpoints: 640, 768, 1024, 1280, 1536px.
8. **No anti-patterns**: No glassmorphism, no gradient text, no oversized shadows, no card grid dashboards.

## Checklist
- [ ] Spacing follows 4px grid
- [ ] Typography uses defined scale
- [ ] Colors use semantic tokens
- [ ] Contrast ratios pass WCAG AA
- [ ] Mobile layout works at 375px width
- [ ] Existing components reused where possible
