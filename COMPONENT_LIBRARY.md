# Clion-FE-Agent Component Pattern Library

## Overview

This document defines reusable component patterns for the Clion-FE-Agent design system. All components are built on the design principles, tokens, and guardrails defined in DESIGN_SYSTEM.md.

---

## Button Patterns

### Primary Button
**Purpose:** Primary action component for CTAs, navigation, and key interactions.

```tsx
<Button 
  className="bg-primary text-white hover:bg-primary-hover active:bg-primary-active"
  size="lg"
>
  Button Label
</Button>
```

**Props Contract:**
- `label`: string - Button text
- `size`: "sm" | "md" | "lg" | "xl" - Button size
- `variant`: "solid" | "outline" - Button style
- `disabled`: boolean - Disabled state
- `loading`: boolean - Loading state
- `icon`: ReactNode - Optional icon

**Usage Guidelines:**
- Use primary style for main CTAs and positive actions
- Use secondary style for supportive actions
- Use outline style for secondary or tertiary actions
- Apply focus ring on keyboard navigation
- Scale to 0.95 on active press for tactile feedback

---

### Secondary Button
**Purpose:** Supporting button for less prominent actions.

```tsx
<Button 
  variant="outline"
  className="border border-primary text-primary hover:bg-surface active:bg-surface-card"
  size="md"
>
  Button Label
</Button>
```

**Usage Guidelines:**
- Use for cancel, back, or tertiary actions
- Pair with primary buttons for visual hierarchy
- Slightly reduced size to indicate secondary importance

---

### Input Field
**Purpose:** Text input with label and helper text.

```tsx
<FormField label="Field Label">
  <Input type="text" placeholder="Value" />
</FormField>
```

**Props Contract:**
- `label`: string - Field label
- `placeholder`: string - Placeholder text
- `disabled`: boolean - Disabled state
- `error`: boolean - Error state
- `helperText`: string - Helper text below input
- `required`: boolean - Required field indicator

**Usage Guidelines:**
- Use clear, concise labels
- Provide helpful validation messages (not just "Invalid")
- Group related fields logically with `<FormField>`

---

### Card Component
**Purpose:** Content container with optional header and footer.

```tsx
// Standard Card
<Card className="bg-surface-card border border-subtle shadow-medium">
  <CardHeader>Card Title</CardHeader>
  <CardContent>Card content...</CardContent>
  <CardFooter>Optional footer actions</CardFooter>
</Card>

// Elevated Card
<Card className="bg-white shadow-elevated">
  Premium content
</Card>
```

**Props Contract:**
- `className`: string - Optional class override
- `elevated`: boolean - Use elevated shadow
- `title`: string - Card title
- `header`: ReactNode - Optional header content
- `footer`: ReactNode - Optional footer content

**Usage Guidelines:**
- Use standard cards for primary content
- Use elevated cards for featured content
- Ensure consistent spacing and typography
- Apply hover elevation on interactive cards

---

### Form Field
**Purpose:** Labeled input wrapper with optional error state.

```tsx
<FormField label="Field Label">
  <Input type="text" placeholder="Value" />
</FormField>
```

**Props Contract:**
- `label`: string - Field label
- `placeholder`: string - Placeholder text
- `disabled`: boolean - Disabled state
- `error`: boolean - Error state
- `helperText`: string - Helper text below input

**Usage Guidelines:**
- Wrap inputs with `<FormField>` for consistent styling
- Use helper text for validation guidance
- Show error state with red border or text color

---

### Navigation Pattern
**Purpose:** Breadcrumb navigation showing current location.

```tsx
<Breadcrumb items={[
  {label: 'Home', href: '/'}, 
  {label: 'About', href: '/about'}
]} />
```

**Props Contract:**
- `items`: array of objects - Navigation items with `label` and `href`

**Usage Guidelines:**
- Use on pages with 3+ levels
- Keep labels short and clear
- Use chevron separators or other visual indicators
- Ensure keyboard accessibility

---

## Modal Pattern
**Purpose:** Overlay dialog for focused interactions or critical messages.

```tsx
<Modal 
  open={isOpen}
  onClose={() => setIsOpen(false)}
>
  <ModalHeader>Modal Title</ModalHeader>
  <ModalBody>Modal content...</ModalBody>
  <ModalFooter>
    <Button onClick={() => setIsOpen(false)}>Close</Button>
  </ModalFooter>
</Modal>
```

**Props Contract:**
- `open`: boolean - Modal open state
- `onClose`: function - Close handler
- `title`: string - Modal title
- `size`: "sm" | "md" | "lg" | "xl" - Modal size

**Usage Guidelines:**
- Always include close button
- Use backdrop blur or dimming
- Focus first focusable element when opened
- Ensure keyboard escape (ESC) closes modal

---

## Hero Section Pattern
**Purpose:** Full-screen or prominent section with clear value proposition.

```tsx
<section className="relative h-screen flex items-center overflow-hidden">
  <motion.div className="absolute inset-0 bg-gradient-to-br from-blue-900 via-purple-900 opacity-90" />
  <div className="container mx-auto px-6 z-10 grid md:grid-cols-2 gap-12">
    <h1 className="text-7xl font-bold text-white leading-tight">
      {brandName} <span className="text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-blue-500">Innovation</span>
    </h1>
    <p className="text-xl text-gray-300 mt-6">{tagline}</p>
    <Button size="lg" className="mt-8 bg-gradient-to-r from-cyan-500 to-blue-600 hover:shadow-2xl">
      Get Started <ArrowRight className="ml-2" />
    </Button>
  </div>
</section>
```

**Props Contract:**
No specific contract - Composite pattern with flexible content.

**Usage Guidelines:**
- Clear title and primary CTA above fold
- Use 60/40 content/layout split for desktop
- Strong typography hierarchy (display, h1, body)
- Add supporting visual (image, gradient, or pattern)

---

## Loading State Pattern
**Purpose:** Visual feedback during async operations.

```tsx
<Spinner 
  className="animate-spin h-8 w-8 border-4 border-t-primary-30 rounded-full"
/>
```

**Props Contract:**
- `size`: "sm" | "md" | "lg" - Spinner size
- `variant`: "pulse" | "dots" - Animation variant

**Usage Guidelines:**
- Use for data fetching, form submission, or route transitions
- Include with full-screen or section-level loaders
- Center in parent container
- Use brand accent color (color_accent) for consistency

---

## Toast Notification Pattern
**Purpose:** Non-intrusive feedback messages.

```tsx
<Toast 
  message="Success"
  type="success"
  onDismiss={() => setIsOpen(false)}
/>
```

**Props Contract:**
- `message`: string - Toast message
- `type`: "success" | "error" | "warning" | "info" - Toast type
- `onDismiss`: function - Dismiss handler
- `duration`: number - Auto-dismiss duration in ms

**Usage Guidelines:**
- Position top-right or top-center
- Auto-dismiss after configurable duration (default 3000ms)
- Use appropriate type color (success/warning/error)
- Include close button for manual dismissal
- Stack multiple toasts properly

---

## Progress Indicator Pattern
**Purpose:** Visual progress indication for multi-step processes.

```tsx
<ProgressBar 
  value={0.75}
  max={1}
  label="Uploading..."
/>
```

**Props Contract:**
- `value`: number - Current progress (0-1)
- `max`: number - Maximum value (1)
- `label`: string - Optional label
- `variant`: "determinate" | "indeterminate" - Progress variant

**Usage Guidelines:**
- Use determinate for known progress
- Use indeterminate for unknown duration
- Position consistently (top of card, bottom of hero section)
- Show percentage label when determinate

---

## Dropdown Pattern
**Purpose:** Selectable list with optional search.

```tsx
<Dropdown>
  <DropdownTrigger>Trigger</DropdownTrigger>
  <DropdownContent align="start">
    <DropdownItem value="option1">Option 1</DropdownItem>
    <DropdownItem value="option2">Option 2</DropdownItem>
  </DropdownContent>
</Dropdown>
```

**Props Contract:**
- `value`: string - Selected value
- `options`: array - Available options
- `placeholder`: string - Trigger text
- `searchable`: boolean - Enable search functionality

**Usage Guidelines:**
- Use for selecting from list of 5+ items
- Align dropdown content with trigger (left/start/right)
- Ensure keyboard accessibility (arrow keys, type to open)
- Use consistent spacing between items

---

## Badge Pattern
**Purpose:** Small indicator or status label.

```tsx
<Badge variant="outline">Label</Badge>
```

**Props Contract:**
- `variant`: "solid" | "outline" - Badge style
- `size`: "sm" | "md" - Badge size

**Usage Guidelines:**
- Use for status indicators (beta, new, popular)
- Use solid variant for emphasis
- Use outline for subtle indicators
- Pair with text or icon

---

## Avatar Pattern
**Purpose:** User profile image or icon.

```tsx
<Avatar 
  src="/avatar.jpg"
  alt="User Name"
  size="md"
/>
```

**Props Contract:**
- `src`: string - Image URL
- `alt`: string - Alt text for accessibility
- `size`: "xs" | "sm" | "md" | "lg" | "xl" - Avatar size
- `fallback`: ReactNode - Fallback initial

**Usage Guidelines:**
- Always provide alt text for accessibility
- Use appropriate size for context (xs for comments, md for user cards)
- Use brand accent color for consistency
- Support loading states with spinner

---

## Divider Pattern
**Purpose:** Visual separator between content sections.

```tsx
<Divider className="w-full h-px bg-border-subtle" />
```

**Props Contract:**
- `className`: string - Optional class override
- `variant`: "solid" | "dashed" - Divider style
- `orientation`: "horizontal" | "vertical" - Divider orientation

**Usage Guidelines:**
- Use to create visual sections without extra spacing
- Use subtle border color (border_subtle)
- Ensure consistent height (1px) for horizontal dividers
- Use full width for horizontal dividers

---

## Alert/Callout Pattern
**Purpose:** Important message with optional action.

```tsx
<Alert>
  <AlertTitle>Important Update</AlertTitle>
  <AlertBody>We've made changes to your account.</AlertBody>
  <AlertActions>
    <Button variant="outline" size="sm">Learn More</Button>
    <Button variant="solid" size="sm">Dismiss</Button>
  </AlertActions>
</Alert>
```

**Props Contract:**
- `variant`: "info" | "warning" | "error" - Alert variant
- `title`: string - Alert title
- `children`: ReactNode - Alert content
- `actions`: ReactNode - Action buttons
- `dismissible`: boolean - Allow dismissal

**Usage Guidelines:**
- Use for system notifications, updates, or warnings
- Use variant for visual hierarchy (info=blue, warning=amber, error=red)
- Include dismiss action for non-critical alerts
- Position prominently but don't block content

---

## Empty State Pattern
**Purpose:** Placeholder when no data is available.

```tsx
<EmptyState
  icon={Folder}
  title="No projects yet"
  description="Create your first project to get started"
/>
```

**Props Contract:**
- `icon`: ReactNode - Icon component
- `title`: string - Empty state title
- `description`: string - Empty state description
- `actionLabel`: string - Optional action button label
- `actionHref`: string - Optional action button href

**Usage Guidelines:**
- Use helpful, encouraging tone
- Provide clear action to resolve empty state
- Use brand accent color for icon
- Center content with flex layout

---

## Skeleton Pattern
**Purpose:** Loading placeholder during data fetch.

```tsx
<Skeleton className="animate-pulse w-full h-32 bg-surface" />
```

**Props Contract:**
- `className`: string - Optional class override
- `variant`: "text" | "circular" - Skeleton variant
- `width`: string | "full" | "sm" | "md" - Skeleton width

**Usage Guidelines:**
- Use during initial page load or data fetching
- Match expected content structure (lines of text vs circular avatar)
- Use surface color (surface or surface_card)
- Add subtle pulse animation for visual interest

---

## Tooltip Pattern
**Purpose:** Contextual information on hover/focus.

```tsx
<Tooltip content="Additional info">
  <TooltipTrigger>Hover me</TooltipTrigger>
</Tooltip>
```

**Props Contract:**
- `content`: ReactNode - Tooltip content
- `position`: "top" | "bottom" | "left" | "right" - Tooltip position
- `delay`: number - Show delay in ms (default 0)
- `trigger`: "hover" | "focus" - Trigger behavior

**Usage Guidelines:**
- Keep tooltips brief and concise
- Use for additional context or help text
- Position away from interactive elements to avoid occlusion
- Respect prefers-reduced-motion (disable if set)

---

## Tabs Pattern
**Purpose:** Tabbed navigation for related content.

```tsx
<Tabs defaultValue="tab1">
  <TabsList>
    <TabsTrigger value="tab1">Overview</TabsTrigger>
    <TabsTrigger value="tab2">Details</TabsTrigger>
    <TabsTrigger value="tab3">Settings</TabsTrigger>
  </TabsList>
</tabs>
```

**Props Contract:**
- `defaultValue`: string - Initial tab value
- `children`: ReactNode - Tab content and triggers

**Usage Guidelines:**
- Use for organizing related content (overview, details, settings)
- Keep labels short and clear
- Ensure keyboard navigation (arrow keys between tabs)
- Use active indicator (underline or background change)

---

## Pagination Pattern
**Purpose:** Navigation through large content sets.

```tsx
<Pagination 
  currentPage={1}
  totalPages={10}
  onPageChange={(page) => setCurrentPage(page)}
>
  <PreviousButton disabled={currentPage === 1}>Previous</PreviousButton>
  <PageIndicator>Page {currentPage} of {totalPages}</PageIndicator>
  <NextButton disabled={currentPage === totalPages}>Next</NextButton>
</Pagination>
```

**Props Contract:**
- `currentPage`: number - Current page number
- `totalPages`: number - Total pages
- `onPageChange`: function - Page change handler

**Usage Guidelines:**
- Use for large datasets, tables, or content lists
- Disable previous/first buttons appropriately
- Show page numbers or range (1-5 of 10)
- Ensure keyboard accessibility (arrow keys, page selection)

---

## Data Table Pattern
**Purpose:** Structured data display with headers.

```tsx
<Table>
  <TableHeader>
    <TableColumn key="name">Name</TableColumn>
    <TableColumn key="status">Status</TableColumn>
    <TableColumn key="role">Role</TableColumn>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell>Name</TableCell>
      <TableCell>Active</TableCell>
      <TableCell>Admin</TableCell>
    </TableRow>
  </TableBody>
</Table>
```

**Props Contract:**
- `data`: array of objects - Table data
- `columns`: array - Column definitions with `key` and `label`
- `striped`: boolean - Remove alternating row colors
- `sortable`: boolean - Enable column sorting

**Usage Guidelines:**
- Use for admin dashboards, user tables, or metrics displays
- Keep headers concise and aligned
- Use status badges (active/inactive/admin)
- Ensure sortable columns have clear sort indicators
- Use consistent alignment (left for text, center for status)

---

## Chart/Graph Pattern
**Purpose:** Data visualization for metrics and trends.

```tsx
<BarChart data={monthlyData} />
<LineChart data={revenueData} />
<PieChart data={categoryData} />
```

**Props Contract:**
- `data`: array - Chart data points
- `color`: string - Data series color
- `height`: string | number - Chart height
- `showGrid`: boolean - Show grid lines
- `showLegend`: boolean - Show legend

**Usage Guidelines:**
- Use consistent colors based on data series
- Ensure accessibility with aria labels and alt text
- Provide tooltips for data points on hover
- Use brand accent color for key data points
- Maintain responsive sizing (charts shrink on mobile)

---

## Search/Filter Pattern
**Purpose:** Filter and search functionality for content lists.

```tsx
<SearchBar onSearch={(query) => setQuery(query)} />
<FilterBar onFilterChange={(filters) => setFilters(filters)} />
```

**Props Contract:**
- `placeholder`: string - Search placeholder
- `value`: string - Current search value
- `filters`: object - Active filters
- `onSearch`: function - Search handler
- `onFilterChange`: function - Filter change handler

**Usage Guidelines:**
- Use helpful placeholder text
- Debounce search input (300ms delay)
- Show active filter count badge
- Clear filters button to reset all filters
- Use brand accent color for active filter indicators

---

## Utility Pattern
**Purpose:** Helper components for common UI patterns.

```tsx
<Container className="max-w-7xl mx-auto">
  Content here
</Container>

<Wrapper className="space-y-4">
  <div>Content with wrapper</div>
</Wrapper>
```

**Props Contract:**
- `className`: string - Optional class override
- `maxWidth`: string - Maximum width constraint

**Usage Guidelines:**
- Use Container for max-width constraints (max-w-7xl)
- Use Wrapper for consistent spacing (space-y-4)
- Ensure responsive behavior at all breakpoints
- Center content horizontally when appropriate

---

## Icons Pattern
**Purpose:** Consistent iconography for navigation and actions.

```tsx
import { Activity, Bot, FileText, FolderTree, Settings, Terminal, ArrowRight } from 'lucide-react'

<Activity className="w-5 h-5" />
<Bot className="w-5 h-5" />
<FolderTree className="w-5 h-5" />
<Settings className="w-5 h-5" />
<Terminal className="w-5 h-5" />
<ArrowRight className="w-5 h-5" />
```

**Icon Guidelines:**
- Use lucide-react for consistency
- Maintain consistent size (w-5 h-5 for primary icons, w-4 for secondary)
- Use appropriate icon for semantic meaning
- Apply brand text color for consistency
- Use subtle hover states (lift effect or color shift)

---

## Accordion Pattern
**Purpose:** Expandable content sections with smooth animations.

```tsx
<Accordion>
  <AccordionItem value="item1">
    <AccordionTrigger>Section 1</AccordionTrigger>
    <AccordionContent>Content for section 1</AccordionContent>
  </AccordionItem>
  <AccordionItem value="item2">
    <AccordionTrigger>Section 2</AccordionTrigger>
    <AccordionContent>Content for section 2</AccordionContent>
  </AccordionItem>
</Accordion>
```

**Props Contract:**
- `value`: string | Controlled accordion value
- `multiple`: boolean - Allow multiple items open
- `type`: "single" | "multiple" - Accordion type
- `collapsible`: boolean - Allow collapse all items

**Usage Guidelines:**
- Use for FAQs or feature descriptions
- Ensure smooth expand/collapse animations (300ms)
- Use keyboard accessibility (arrow keys, Enter/Space to toggle)
- Include clear visual indicators (chevron rotation)
- Maintain consistent spacing between items

---

## Theme Context
**Implementation Note:**

All components in this library should:
- Consume design tokens from the Clion-FE-Agent design system
- Use semantic color roles (--color-primary, --color-secondary, etc.)
- Apply spacing from the 8pt scale (--space-4, --space-8, etc.)
- Follow accessibility guidelines (WCAG AA contrast, ARIA roles)
- Respect prefers-reduced-motion setting
- Use appropriate border radius from scale (--radius-sm, --radius-md, etc.)
- Apply shadows from shadow system (--shadow-medium, --shadow-strong, etc.)
- Use typography from type scale (--font-h1, --font-h2, --font-body)
- Apply motion principles (300ms entrance, 200ms hover effects)

**Token Mapping Examples:**

```tsx
<Button className="bg-primary text-white">
  // Uses --color-primary, --text-primary, hover:bg-primary-hover
  Button
</Button>

<Card className="bg-surface-card border border-subtle shadow-medium">
  // Uses --color-surface-card, --border-subtle, --shadow-medium
  Card
</Card>

<Input className="border border-subtle">
  // Uses --border-subtle
  Input
</Input>
```

---

## Accessibility Compliance

All components must include:

### ARIA Attributes
- `aria-label` on all interactive elements (buttons, inputs)
- `aria-expanded` for dropdowns, accordions, modals
- `aria-live="polite"` for toasts, alerts
- `aria-describedby` for error/helpful text
- `role="dialog"` for modals
- Navigation landmarks for complex components

### Keyboard Navigation
- All interactive elements must be keyboard accessible
- Tab order follows DOM order
- Focus indicators visible on all interactive elements
- Escape key support for modals

### Focus States
- Focus ring on keyboard-navigable elements (4px expansion from 0)
- Outline color change or shadow elevation on focus
- Reduced opacity for non-focused states

### Contrast Requirements
- WCAG AA compliance (4.5:1 contrast for --text-primary on --color-surface)
- Minimum 16px font size for body text
- Minimum 44px touch target for interactive elements

---

## Responsive Behavior

All components must respond to breakpoints:
- **Mobile:** 640px - Single column, full-width elements
- **Tablet:** 768px - Two column grid
- **Desktop:** 1024px - Multi-column grid
- **Large Desktop:** 1280px - Expanded layouts

---

## Usage Examples

### Dashboard Layout
```tsx
<Sidebar>
  <NavGroup title="Main">
    <NavItem href="/dashboard" icon={Activity}>Dashboard</NavItem>
    <NavItem href="/projects" icon={FolderTree}>Projects</NavItem>
  </NavGroup>
</sidebar>
<div className="p-6">
  <Card className="bg-surface-card">
    <CardContent>
      <h2 className="text-h2">Total Revenue</h2>
      <div className="text-4xl font-bold">${totalRevenue}</div>
    </CardContent>
  </Card>
  <Card className="bg-surface-card">
    <CardContent>
      <h2 className="text-h2">Active Projects</h2>
      <div className="text-4xl font-bold">${activeProjects}</div>
    </CardContent>
  </Card>
</div>
```

### Form Layout
```tsx
<FormField label="Email Address">
  <Input type="email" placeholder="user@example.com" />
</FormField>
<FormField label="Password">
  <Input type="password" placeholder="••••••••" />
</FormField>
<Button type="submit" size="lg">Sign In</Button>
```

### Landing Page
```tsx
<HeroSection>
  <HeroContent>
    <h1 className="text-7xl">Welcome to {brandName}</h1>
    <p className="text-xl">Build something amazing today.</p>
    <Button size="lg" className="mt-8">Get Started</Button>
  </HeroContent>
</HeroSection>
```

The Clion-FE-Agent Component Library provides reusable, accessible, and design-system-compliant patterns for all front-end development tasks.