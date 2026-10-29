<!-- Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 -->
# nvmr brand guide

![nvmr — Switch Node. Keep your flow.](assets/brand/readme-banner.svg)

nvmr is a native Node.js version manager for busy developers. Its identity should feel precise, familiar, and quick to understand. Useful commands and clear documentation carry the design.

## Name and positioning

| Context | Use |
| --- | --- |
| Project and product | **nvmr**, lowercase, with a hyphen. |
| Short description | A native Node.js version manager, written in Rust. |
| Headline | Switch Node. Keep your flow. |
| Supporting line | Native speed. Familiar commands. |
| Compatible shell command | `nvm`. |
| Optional branded command | `nvmr`, enabled with `alias nvmr='nvm'` after loading nvm. |
| Upstream project | nvm-sh/nvm, linked and credited when discussing compatibility. |

Use `nvmr` in prose, page titles, and release announcements. Avoid “nvmr”, “NvmRs”, or “nvm.rs”. Preserve real code identifiers: `$NVM_DIR`, `NVMR_BUNDLE`, `nvmr`, `nvmr-fetch`, and `nvmr-linux-*` are current implementation interfaces, not alternate product names.

Describe nvmr as an independent Rust implementation targeting nvm v0.40.8 behavior. An alias changes the command spelling; compatibility comes from the implementation and parity checks. Do not imply that an alias alone provides compatibility or that upstream endorses the project.

## Mark and wordmark

<img src="assets/brand/mark.svg" alt="nvmr switch mark" width="72" height="72">

The switch mark is one continuous path crossing between two levels. It suggests a version change and the lowercase “n”. The cobalt tile anchors the identity; the pale stroke keeps it legible at small sizes.

- Use the [original SVG](assets/brand/mark.svg). It also serves as the website favicon.
- Pair the mark with **nvmr** in Space Grotesk, weight 700, for the primary horizontal lockup.
- Set the mark approximately as tall as the wordmark’s line box. Leave a gap of one quarter of the mark’s height.
- Keep clear space around the lockup equal to at least one quarter of the mark’s width.
- Use the mark at 24 px or larger in product navigation; 16 px is reserved for favicons.
- On a dark surface, keep the cobalt tile and use the pale text color for the wordmark.
- Preserve proportions, path, corner shape, and contrast. Do not add shadows, gradients, rotation, or extra outlines.

The [README banner](assets/brand/readme-banner.svg) is a separate, editable asset. It uses a system sans fallback so GitHub can render the SVG without network font requests. Its wording remains selectable in the source; on the website the wordmark uses the locally hosted display face.

## Color

The palette uses cool paper, blue-tinted ink, and one cobalt accent. [tokens.css](tokens.css) is the source of truth; reference its named custom properties in UI styles.

| Token | OKLCH value | Role |
| --- | --- | --- |
| `--color-paper` | `oklch(98.5% 0.004 250)` | Main page ground. |
| `--color-surface` | `oklch(96% 0.008 250)` | Code recipes and quiet supporting surfaces. |
| `--color-ink` | `oklch(24% 0.022 258)` | Headlines and primary text. |
| `--color-body` | `oklch(39% 0.022 258)` | Reading text. |
| `--color-muted` | `oklch(49% 0.025 258)` | Notes and secondary labels. |
| `--color-rule` | `oklch(86% 0.014 250)` | Dividers and boundaries. |
| `--color-accent` | `oklch(53% 0.22 260)` | Primary action, links, focus, brand mark. |
| `--color-accent-ink` | `oklch(99% 0.003 250)` | Text on cobalt fills. |
| `--color-code` | `oklch(21% 0.022 258)` | Command examples and installation band. |
| `--color-code-ink` | `oklch(94% 0.008 250)` | Primary text on dark surfaces. |
| `--color-code-muted` | `oklch(75% 0.022 250)` | Comments and supporting copy on dark surfaces. |
| `--color-code-accent` | `oklch(79% 0.11 255)` | Prompts, links, and focus on dark surfaces. |

Keep cobalt concentrated in the mark, one primary action, and small points of emphasis. Use solid colors. The dark installation band supplies a change of rhythm. Error states pair a readable message with the error color; color alone never carries meaning.

## Typography

| Role | Family | Weight and treatment |
| --- | --- | --- |
| Display and wordmark | Space Grotesk | 600 for headings, 700 for wordmark; roman, tight tracking. |
| Body and UI | Inter | 400 for prose, 500–600 for emphasis and controls. |
| Commands and data | JetBrains Mono | 400–500; preserve literal syntax and whitespace. |

The website self-hosts Latin WOFF2 subsets in [assets/fonts](assets/fonts). Each family’s SIL Open Font License is included there. There are no runtime font-service requests.

Headlines use short, concrete language, with a display ceiling of 4.8 rem. Body text is 1 rem with a 1.65 line height. Keep paragraphs around 45–75 characters wide. Tables use tabular numerals. All headings stay upright; weight and color provide emphasis.

## Layout and interaction

The website follows a command-led Workbench structure: introduction and commands, installation, quick reference, architecture, compatibility. Information should be easy to scan on a short break between tasks.

- Use a 4 px spacing rhythm; tokens range from 4 px to 96 px. Separate sections generously and keep related instructions close.
- Use a maximum content width of 74 rem, with fluid page gutters.
- Prefer plain rows, short paragraphs, definition lists, and tables. Let code be code; omit fake terminal title bars and traffic-light buttons.
- Use 6 px control corners and 10 px code surfaces, with thin borders and no decorative shadows.
- Stack multi-column sections below 960 px. Support 320 px screens without horizontal page scrolling.
- Keep controls at least 44 px tall. Labels stay on one line; their parent rows may wrap.
- Give keyboard focus a visible, immediate outline. Copy controls announce success or failure without a toast.
- Motion is limited to a 120 ms button press. Reduced motion removes it.
- Keep content, navigation, and disclosures usable without JavaScript. Clipboard buttons appear only when the Clipboard API is available in a secure context.

## Voice

Write for someone who wants to get back to their code. Lead with the command or the next action, then provide the detail that prevents a mistake.

| Prefer | Avoid |
| --- | --- |
| “Install and use the latest LTS.” | “Unlock the power of seamless version management.” |
| “Your versions and aliases share `$NVM_DIR`.” | “Zero friction. Zero compromises.” |
| “Linux verified; macOS not yet tested.” | “Works everywhere.” |
| “Recorded medians from a Linux arm64 VM.” | An unqualified speed multiplier. |
| “Clipboard access was blocked. Select and copy the commands.” | “Oops! Something went wrong.” |

Use the supported compatibility scope from the repository. Every performance figure must cite its saved `bench/results/<run>/` report, environment, and limitations. There are no publishable saved results included with this page yet. Preserve exact shell syntax inside code blocks. Never invent adoption metrics, testimonials, release URLs, badges, supported platforms, or licensing claims.

## Asset and file map

| File | Purpose |
| --- | --- |
| [assets/brand/mark.svg](assets/brand/mark.svg) | Standalone switch mark and favicon. |
| [assets/brand/readme-banner.svg](assets/brand/readme-banner.svg) | GitHub README masthead. |
| [tokens.css](tokens.css) | Color, type, spacing, radius, and motion tokens. |
| [site.css](site.css) | Responsive page styles and local font declarations. |
| [index.html](index.html) | Marketing, install, and quick docs page. |
| [site.js](site.js) | Optional clipboard enhancement. |
| [README.md](../README.md) | Repository introduction and practical documentation. |

Paths in this guide are relative to `website/`. For local preview, serve the repository root and visit `/website/`. For static hosting, copy `website/` and its linked project documentation into a public directory with the same relative structure. The brand guide records the current identity; future runtime or packaging renames should update code and documentation together.
