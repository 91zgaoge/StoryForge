# Frontend Design Skill for CINEMA-AI

## Design System: Literary Salon Style (Claude-inspired)

You are a frontend design expert for CINEMA-AI. All generated React + TypeScript code must follow this design system.

### Core Philosophy
Claude interface is a digital reimagining of a literary salon - warm, unhurried, intellectual.
The experience is built on a parchment-toned canvas that evokes high-quality paper.

### Color System

**Primary Colors**
- anthropic-black: #141413 (primary text, dark surfaces)
- terracotta: #c96442 (brand color, primary CTAs)  
- coral: #d97757 (lighter accent, links)

**Background Colors**
- parchment: #f5f4ed (page background - warm cream paper)
- ivory: #faf9f5 (card surfaces)
- warm-sand: #e8e6dc (button backgrounds)
- dark-surface: #30302e (dark containers)

**Text Colors**
- charcoal: #4d4c48 (button text on light)
- olive-gray: #5e5d59 (secondary body text)
- stone-gray: #87867f (tertiary text)
- warm-silver: #b0aea5 (text on dark surfaces)

**Border Colors**
- border-cream: #f0eee6 (light theme borders)
- border-warm: #e8e6dc (prominent borders)
- ring-warm: #d1cfc5 (button hover/focus rings)
- focus-blue: #3898ec (input focus - only cool color)

### Typography

**Font Families**
- Headlines: Georgia, serif
- Body/UI: system-ui, sans-serif
- Code: JetBrains Mono, monospace

**Type Scale**
- Display/Hero: 64px (4rem), weight 500, line-height 1.10
- Section Heading: 52px (3.25rem), weight 500, line-height 1.20
- Sub-heading: 32px (2rem), weight 500, line-height 1.10
- Body: 17px (1.06rem), line-height 1.60
- Caption: 14px (0.88rem), line-height 1.43

### Component Patterns

**Primary Button**
```
bg-[#c96442] text-[#faf9f5] px-4 py-2 rounded-lg shadow-[0px_0px_0px_1px_#c96442] hover:brightness-110
```

**Secondary Button**
```
bg-[#e8e6dc] text-[#4d4c48] px-3 py-2 rounded-lg shadow-[0px_0px_0px_1px_#d1cfc5]
```

**Card**
```
bg-[#faf9f5] border border-[#f0eee6] rounded-2xl shadow-[rgba(0,0,0,0.05)_0px_4px_24px] p-6
```

**Input**
```
bg-white text-[#141413] px-3 py-1.5 rounded-xl border border-[#e8e6dc] focus:border-[#3898ec] focus:ring-1 focus:ring-[#3898ec]
```

### Layout Principles

**Spacing**: Base unit 8px, section padding 80-120px
**Border Radius**: Buttons 8-12px, Cards 8-16px, Hero 32px
**Shadows**: Ring-based 0px 0px 0px 1px pattern

### Page Templates

**Light Theme**
```html
<div class="min-h-screen bg-[#f5f4ed] text-[#141413]">
  <nav class="sticky top-0 bg-[#f5f4ed] border-b border-[#f0eee6]">...</nav>
  <main>
    <section class="py-20 px-8">
      <h1 class="font-serif text-6xl font-medium text-[#141413]">Title</h1>
      <p class="text-xl text-[#5e5d59] leading-relaxed mt-4">Description</p>
    </section>
  </main>
</div>
```

**Dark Section**
```html
<section class="bg-[#141413] text-[#faf9f5] py-20 px-8">
  <h2 class="font-serif text-5xl font-medium">Dark Section</h2>
  <p class="text-[#b0aea5]">Content</p>
</section>
```

### Design Checklist

- [ ] Background uses parchment (#f5f4ed) or dark surface
- [ ] All grays have warm undertone
- [ ] Headlines use serif font, weight 500
- [ ] Buttons use ring shadows
- [ ] Body line-height at least 1.60
- [ ] Border radius >= 6px
- [ ] Interactive elements have hover states

### DON'Ts

- NO cool blue-grays
- NO bold (700+) on serif fonts
- NO saturated colors beyond terracotta
- NO sharp corners (< 6px radius)
- NO heavy drop shadows
- NO pure white as page background
- NO monospace for non-code content
