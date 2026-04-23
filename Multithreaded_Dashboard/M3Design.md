# Design System Specification: The Monolithic Precision System

## 1. Overview & Creative North Star
**Creative North Star: "The Architectural Monolith"**

This design system rejects the ephemeral fluff of modern web trends in favor of grounded, architectural permanence. The aesthetic is defined by **Tonal Brutalism**: a high-utility, high-sophistication approach that uses solid masses of color to define space. 

By stripping away blurs, glassmorphism, and traditional drop shadows, we rely on the purity of the Material 3 "Expressive" logic. We create depth through "Carved Surfaces"—where the UI feels like a single block of obsidian with functional areas precisely milled into the surface. The result is an interface that feels authoritative, secure, and hyper-legible.

---

## 2. Colors & Surface Logic
The palette is rooted in deep minerals and high-contrast accents. We prioritize functional clarity over decorative gradients.

### The "No-Line" Rule
**Explicit Instruction:** 1px solid borders are strictly prohibited for sectioning or containment. 
Structure must be achieved through **Tonal Stepping**. To separate a sidebar from a main feed, or a header from a body, transition between `surface-container-low` (#1D2024) and `surface-container-highest` (#2B2D31). This creates a "milled" look where components appear to be physically inset or embossed within the interface.

### Surface Hierarchy (Dark Mode Default)
| Token | Hex | Role |
| :--- | :--- | :--- |
| **background** | #0b0e12 | The foundational "base" layer. |
| **surface-container-low** | #1D2024 | Primary background for main content areas and secondary sections. |
| **surface-container-highest**| #2B2D31 | Elevated surfaces: Cards, active modals, and high-priority containers. |
| **tertiary-container** | #d1f3dc | **Visited States:** A muted, sophisticated dark green to denote historical navigation. |
| **primary** | #bbdaff | Actionable elements and brand highlights. |

---

## 3. Typography: The Editorial Voice
We utilize a high-contrast scale to ensure the "Expressive" nature of the system is felt immediately. 

*   **Display & Headlines (Epilogue):** These are your "Statement" styles. Use `display-lg` (3.5rem) and `headline-lg` (2rem) with tight letter-spacing (-0.02em) to create a bold, editorial feel. These should feel like headlines in a premium architectural magazine.
*   **Body & Labels (Inter):** Reserved for high-density data. While the headers are expressive, the body remains a workhorse—clean, legible, and utilitarian.

**The Hierarchy Rule:** Never pair two "Display" sizes together. Use a bold `headline-md` for titles and immediately drop to `body-md` for descriptions to maximize the dynamic range of the layout.

---

## 4. Elevation & Depth: Tonal Stacking
Since shadows and blurs are forbidden, we use **The Stacking Principle** to communicate importance.

1.  **Level 0 (The Void):** `surface-container-low` (#1D2024). Use this for the largest background areas.
2.  **Level 1 (The Object):** `surface-container-highest` (#2B2D31). Use this for cards and list items. 
3.  **Level 2 (The Focus):** `primary` (#bbdaff). Used for the most critical interactive state.

**Ghost Borders (The Exception):** If high-density data requires a container but a background shift is too heavy, use `outline-variant` (#424850) at **15% opacity**. This creates a "perceived" edge that assists eye-tracking without introducing visual noise.

---

## 5. Components

### High-Contrast Touch Targets
Every interactive list element or tile must maintain a **minimum height of 56px**. This ensures the "Expressive" system remains accessible and feels premium under-thumb.

### Buttons & Chips
*   **Shape:** `rounded-full` (Pill shape).
*   **Primary:** Solid `primary` background with `on-primary` text. No shadows.
*   **Secondary:** `surface-container-highest` background.
*   **Interaction:** On press, shift the tonal value one step higher (e.g., from `surface-container-low` to `surface-container-highest`).

### Cards & Lists
*   **Rounding:** `rounded-[16px]`.
*   **The Divider Rule:** Forbid 1px dividers. Use a `1.4rem` (Spacing 4) vertical gap to separate list items. If separation is visually required, use a 1-step tonal shift between the list item and the background.
*   **Visited State:** Items that have been viewed or "planned" should transition their container or a secondary indicator to `tertiary-container` (Muted Green).

### The Bottom Sheet (Signature Component)
*   **Rounding:** `rounded-t-[32px]`.
*   **Style:** Must use `surface-container-highest` (#2B2D31) to contrast sharply against the lower-level background.
*   **Context:** Used for branch filtering and appointment confirmation.

### Input Fields
*   **Style:** Filled (not outlined).
*   **Background:** `surface-container-highest`.
*   **Active State:** A bottom-heavy `2px` border using the `primary` token. No glow/blur.

---

## 6. Do’s and Don’ts

### Do
*   **Do** use massive "Display" typography for branch names or empty states.
*   **Do** use the full spacing scale (up to `spacing-24`) to create "Breathing Room" around monolithic blocks.
*   **Do** rely on `surface-container` tiers to group related information.
*   **Do** ensure all primary actions use the `primary` (#bbdaff) color to pop against the dark mode.

### Don't
*   **Don't** use `drop-shadow`. If an element needs to stand out, make it a lighter tonal hex.
*   **Don't** use `backdrop-blur`. Backgrounds must remain solid and opaque.
*   **Don't** use 1px lines to separate content. Use whitespace or color shifts.
*   **Don't** cram data. If the touch target is less than 56px, the design is a failure of this system.
