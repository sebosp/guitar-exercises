# Color palettes for a long-session music-practice app (desktop, Android, web, dim rooms)

Scope of these notes: color preference research, fatigue/anxiety, dark-mode evidence, WCAG 2.2 / APCA / Material / Apple contrast guidance, color-vision deficiency (CVD), concrete palettes of learning and music apps, paper-like notation backgrounds, and how many hues a learner can track in a tab. Every hex value is marked as **official**, **third-party extraction**, or **computed here**. Contrast numbers marked "computed here" were produced with a small script (WCAG 2 relative-luminance formula and the APCA-W3 0.1.9 "SAPC-4g" constants; CVD simulation with the Machado 2009 severity-1.0 matrices in linear sRGB, distances as CIELAB ΔE76) and are reproducible but not from a published table.

---

## Key question 1: What does research say about color preference (Palmer & Schloss, cross-cultural) and about colors that reduce fatigue or anxiety during sustained attention?

### Takeaway
Average color preference is well predicted (80% of variance) by how much people like the objects a color evokes: blues peak, dark yellow/olive and brown trough, and saturated colors beat light/muted ones on average, but this is a US-sample finding and a Himba study shows the pattern is not universal. Claims that specific hues reduce anxiety or fatigue during sustained work are weakly supported: the red-impairs-performance literature does not survive publication-bias correction, and the robust ergonomic findings are about luminance polarity and brightness, not hue.

### Cited Findings
- Palmer & Schloss (2010, PNAS) tested 32 chromatic colors (8 hues x 4 "cuts": saturated, light, muted, dark) with 48 raters. Averages "produced approximately parallel functions with a broad peak at blue and a narrow trough at chartreuse"; saturated colors were preferred to light and muted ones (F(1,47)=9.20, P<0.01). Dark orange (brown) and dark yellow (olive) were significantly less preferred than other oranges and yellows, while dark red and dark green were more preferred than other reds/greens. — [Palmer & Schloss 2010, Palmer Lab PDF](https://palmerlab.berkeley.edu/pdf/Palmer%26Schloss%282010%29.pdf)
- The Ecological Valence Theory predictor (weighted affective valence estimates of color-associated objects, "WAVEs") accounted for 80% of the variance in average preference with a single predictor (r=+0.893), beating a cone-contrast model (37%), a color-appearance model (60%) and a color-emotion model (55%). — [Palmer & Schloss 2010](https://palmerlab.berkeley.edu/pdf/Palmer%26Schloss%282010%29.pdf); abstract: [PNAS/PMC2889342](https://pmc.ncbi.nlm.nih.gov/articles/PMC2889342)
- Palmer & Schloss hypothesise "browns and olives may be universally disliked because feces..." and report preliminary cross-cultural data: American WAVEs predicted American preferences (r=0.89) better than Japanese (r=0.74); Japanese WAVEs predicted Japanese preferences (r=0.66) better than American (r=0.55). — [Palmer & Schloss 2010](https://palmerlab.berkeley.edu/pdf/Palmer%26Schloss%282010%29.pdf)
- Taylor, Clifford & Franklin (2013, JEP:General), "Color preferences are not universal": British and Himba (rural Namibia) preferences "share few characteristics", Himba preferences "display none of the so-called 'universal' patterns or sex differences", and "no one model of color preference is able to account for both". — [PubMed 23148465](https://pubmed.ncbi.nlm.nih.gov/23148465/)
- Later commentary: chroma was the main predictor of Himba preferences (they leaned toward saturated colors regardless of hue); Sorokowski et al. argue the screen-based presentation may explain that. — [Frontiers in Psychology 2015 commentary](https://www.frontiersin.org/articles/10.3389/fpsyg.2015.01840)
- Follow-up EVT work: university "school spirit" raises preference for the school's colors (sociocultural effects on preference within a lifetime). — [Schloss, Poggesi & Palmer 2011](https://palmerlab.berkeley.edu/pdf/SchlossPoggesi&Palmer-2011.pdf)
- Red and achievement: Elliot et al. (2007) reported that a small red stimulus in an achievement context lowered test performance vs green/gray with d between -0.51 and -1.14, explained as avoidance motivation. — [ScienceDaily summary of Elliot 2007](https://sciencedaily.com/releases/2007/02/070228170240.htm)
- But Gnambs (2020) meta-analysed 67 effect sizes from 38 samples and found "substantially larger effects in initial studies"; after publication-bias correction "no evidential value for an effect of red color on intellectual performance was available". — [Gnambs 2020, Psychonomic Bulletin & Review](https://link.springer.com/article/10.3758/s13423-020-01772-1); replication failures: [Steele et al. posters](https://www.appstate.edu/~steelekm/documents/Psychonomic2017Poster_Steele.pdf)
- Arousal literature (older, small): Spiegel & Keith-Spiegel (1971) found "significantly less physiological arousal under blue than red or white illumination"; Valdez & Mehrabian (1994) found higher state anxiety with red and yellow; Jacobs & Suess (1975): warm colors produce higher arousal. These are cited second-hand in a thesis and a student journal, not verified at the primary source. — [Hefling 2001 thesis](https://esirc.emporia.edu/bitstream/handle/123456789/1106/Hefling%202001.pdf?sequence=1); [Zilka 2014, Red River Psychology Journal](https://leia.mnstate.edu/contentassets/cc6ee9f5358147a1883a0c5f822c6767/red-river-psychology-journal-2014-tiffany-zilka.pdf)
- Light vs dark mode (NN/g literature review of Piepenbrock et al. 2013 Ergonomics and Human Factors; Dobres et al. 2017): light mode (dark text on light background) gave better visual acuity and proofreading for young (18-33) and old (60-85) adults; the advantage grew as font size shrank; participants reported no subjective readability difference; fatigue measures (eyestrain, headache, etc.) showed no polarity effect. Dobres: no polarity effect in simulated daylight, but at night light mode was faster and small text was much harder in dark mode. Explanation offered: more light constricts the pupil, reducing optical aberrations and increasing depth of field. — [NN/g, Dark Mode vs. Light Mode](https://www.nngroup.com/articles/dark-mode/)
- Aleman et al. 2018 (n=7): one hour of light-mode reading thinned the choroid, dark mode thickened it (a possible myopia link); NN/g flags the tiny sample. Legge et al. 1985 (n=7): readers with cloudy ocular media (cataract) read faster in dark mode. — [NN/g](https://www.nngroup.com/articles/dark-mode/)
- NN/g recommendations: do not default to dark mode for the general population; offer a switch; prioritise dark mode in long-form reading apps; follow the OS dark-mode API. — [NN/g](https://www.nngroup.com/articles/dark-mode/)
- A 2025 eye-tracking study reported higher relative pupil dilation but lower perceived workload in dark mode; a small controlled experiment found no significant reading-speed or comprehension difference ("the comfort advantage is real; the performance advantage isn't"). Secondary coverage. — [Gadget Hacks summary](https://android.gadgethacks.com/news/dark-mode-vs-light-mode-what-research-says-about-when-each-works-best/)
- Evening screens: Lighting Research Center iPad study: melatonin suppression 23% with no Night Shift, 19% with cool Night Shift, 12% with warm; authors conclude changing screen color alone is insufficient and brightness must be reduced. — [Edison Report on LRC study](https://edisonreport.com/?p=8678); [LEDs Magazine](https://www.ledsmagazine.com/articles/2018/02/changing-a-gadget-s-nighttime-screen-hue-makes-no-difference-to-sleep.html)
- Rello & Bigham 2017 note the common design lore "warm colors are used to stimulate observers... cool colors are generally used to calm and relax the viewer" and explicitly say their warm-background result "do[es] not necessarily mean that warm colors are recommended" since a reader "might prefer a cool color... if that color relaxes the reader". — [Rello & Bigham 2017, ASSETS](https://www.changedyslexia.org/publications/pdfs/2017-ASSETS-Good%20Background%20Colors.pdf)

### Inferences
- Studied: blue-family hues are the safest "liked by default" choice for a Western audience; avoid olive/brown/chartreuse as large areas or as the "success" colour. Saturated accents are liked more than muted ones on average, which conflicts with dark-theme contrast advice (Q3) — resolve by using saturated hues only as small marks on dark surfaces and desaturating large fills.
- Studied (ergonomics): what matters for a long practice session in a dim room is luminance, not hue: keep the dark theme's text contrast high (Q3), let the user pick light vs dark, and offer a brightness/dim control rather than relying on a warm tint. Dobres' night result argues for making the light theme available even for night use and not forcing dark.
- Designer convention, weakly studied: "calming blue/green, alarming red". Use it as a convention (users expect red = wrong), not as a fatigue-reduction mechanism. The red-impairs-performance effect should not drive a decision to avoid red error marks.
- Cross-cultural: EVT fits within culture; the Himba data warn against treating "blue is universally preferred" as fact for a global audience. For a global app, the safest move is neutral surfaces with a user-selectable accent.

### Gaps
- No study found that measured hue effects on fatigue or anxiety specifically during multi-hour sustained attention tasks on screens; the arousal studies are 1970s-1990s room-illumination experiments with small samples.
- PNAS full text was read from the lab PDF; the PMC HTML was blocked by a browser check, so quoted phrases come from the PDF text extraction.
- No peer-reviewed source found on whether dark mode reduces fatigue over sessions longer than ~1 hour.

---

## Key question 2: What palettes do Duolingo, Yousician, Simply Guitar, Rocksmith+, Soundslice, Ultimate Guitar, Flat.io, MuseScore, Headspace and Calm use (dark themes, success/warning/error)?

### Takeaway
Only Duolingo publishes an official palette (Feather Green #58CC02 core; logo neutrals Eel #4B4B4B and Snow #FFFFFF); its error red #FF4B4B, streak orange #FF9600, gold #FFC800 and blue #1CB0F6 come from third-party CSS extractions. The music apps mostly do not document colors: Rocksmith+ and Soundslice document *mechanisms* (colorblind presets, chroma-normalised palette, a Colors-off toggle, Boomwhackers pitch colors) rather than hex values, MuseScore 4 documents light/dark/high-contrast themes plus accent choice, and Headspace/Calm hexes are unofficial extractions.

### Cited Findings

**Duolingo**
- Official: Duolingo's design site calls Feather Green "the core color of the brand"; the page lists Wing Overlay #43C000 (RGB 67,192,0; PMS 369 C). (The site now redirects to blog.duolingo.com/hub/design.) — [design.duolingo.com/identity/color](https://design.duolingo.com/identity/color)
- Logo palette (brand-palette aggregator, consistent across sources): Feather Green #58CC02 (PMS 361 C), Mask Green #89E219, Eel Black #4B4B4B, Snow White #FFFFFF, beak/feet Orange #F49000, Golden Yellow #FFC200, Light Yellow #FFDE00, Brown #B66E28. — [brandpalettes.com Duolingo](https://brandpalettes.com/duolingo-colors/)
- Third-party CSS-variable extraction (unofficial, "352 :root variables", only some listed): `--color-owl` #58CC02 (primary), `--color-tree-frog` #58A700 (pressed), `--color-sea-sponge` #A5ED6E (source inconsistent, also #D7FFB8), `--color-macaw` #1CB0F6 (secondary accent), `--color-eclipse` #000437 (deep navy), `--color-cardinal` #FF4B4B (error), `--color-fire-ant` #EA2B2B, `--color-fox` #FF9600 (streak), `--color-bee` #FFC800 (gold), `--color-beetle` #CE82FF (Super/diamond), `--color-kiwi` #7AC70C, neutrals `--color-black-text` #3C3C3C, `--color-eel` #4B4B4B, `--color-wolf` #777777 (secondary text), `--color-hare` #AFAFAF (disabled), `--color-koala` #D7D7D7. No separate success/warning tokens; dark mode tokens (`--color-*-always-dark`) exist but are not documented. — [shadcn.io Duolingo DESIGN.md (unofficial)](https://www.shadcn.io/design/duolingo/raw)
- Sources disagree on the blue (#1CB0F6 in most, #14D4F4 in one). — [designpieces Duolingo palette](https://www.designpieces.com/palette/duolingo-color-palette-hex-and-rgb/)
- A design-system writeup calls #58CC02 the signature color representing "growth, energy, and success" (designer convention). — [open-design.ai Duolingo](https://open-design.ai/systems/duolingo/)

**Yousician**
- Note colors encode the recommended finger: for guitar, "yellow suggests to use the pointer finger". — [Yousician support: How to read guitar tablature](https://support.yousician.com/hc/en-us/articles/206932099-How-to-read-guitar-tablature) (page returned 403 on direct fetch; quote from search snippet)
- Piano falling-notes view: index = yellow, middle = purple, ring = blue, little finger = red, each note shows its name; an "enhanced sheet" view uses a different scheme in which the thumb is green. — [Yousician support: Notation options in piano](https://support.yousician.com/hc/en-us/articles/208678605-Notation-options-in-piano)
- Reviewers describe the visual style as "bright, primary colors and bold bouncy graphics" that make it look like "a child's toy". No dark-theme documentation found. — [Guitar World Yousician review](https://guitarworld.com/reviews/yousician-review)

**Simply Guitar (Simply, formerly JoyTunes)**
- Reviews describe "lively, vibrant colors" and a "fun, colorful look"; no palette or dark theme documented. Simply Piano's "white background" option is iOS-only and plain white. — [GuitarChalk review](https://www.guitarchalk.com/simply-guitar-review/); [Simply Piano help: enable white background](https://piano-help.hellosimply.com/en/articles/16035829-how-to-enable-white-background-ios-only)

**Rocksmith+ (Ubisoft)**
- Official accessibility spotlight: "Color Customization provides several colorblind presets for each type of colorblindness"; "Balanced Colors Preset provides an alternative color palette offering normalized chroma values"; a "Color Boost" option increases saturation of any non-default preset; an "RS Tab" alternative UI "relies more on numbers and less on color to show which note to play". — [Ubisoft News, Rocksmith accessibility spotlight](https://news.ubisoft.com/en-us/article/5XfINjoRmVJgBKR8fGt4EM/rocksmith-accessibility-spotlight)
- Patch note: custom string colors are available under "My Instrument" after selecting the "Custom Colors" preset. — [The Riff Repeater](https://theriffrepeater.com/?p=5817)
- Rocksmith 2014 string-color convention (forum evidence only): low E is red, A is yellow, B is green-ish; colorblind mode uses teal/green for the high B. Full six-string mapping not confirmed by an official page. — [Steam discussions](https://steamcommunity.com/app/221680/discussions/0/3198119849661572945); [Rocksmith 2014 colorblind mode, Wikipedia](https://en.wikipedia.org/wiki/Rocksmith_2014)

**Soundslice**
- Player dark mode is "light notes on a dark background", off by default because it is "just nontraditional enough" that users should opt in; Light/Dark/Auto (follows OS); embeds use a `dark` URL parameter. No hex values published. — [Soundslice blog: player dark mode](https://www.soundslice.com/blog/239/introducing-our-players-dark-mode); [Soundslice help: Theme](https://www.soundslice.com/help/en/player/advanced/301/theme/)
- Note colors: only noteheads (and attached accidentals) can be colored; coloring a tab note also colors its tab number and the two cannot differ; viewers have a Colors toggle to show everything in black; purpose stated as helping beginners read and highlighting phrases in analysis. — [Soundslice help: Colors](https://soundslice.com/help/en/creating/notations/249/colors); [blog: disable colors](https://www.soundslice.com/blog/247/new-disable-colors-in-any-slice-youre-viewing)
- Boomwhackers pitch convention offered as an automatic palette: C red, D orange, E yellow, F green, G teal, A indigo, B violet (no hex given). — [Soundslice help: Boomwhackers colors](https://www.soundslice.com/help/en/player/advanced/336/boomwhackers-colors/)

**Ultimate Guitar**
- A dark mode "for playing in low-light settings" appears in release notes for v7.3.17; no palette published. — [APKMirror UG 7.3.17 notes](https://www.apkmirror.com/?p=14759840)
- Design critique notes a dark splash screen; a redesign case study kept "the color palette, fonts, and general aesthetic". — [Pratt design critique](https://ixd.prattsi.org/2024/09/design-critique-ultimate-guitar-ios-app/); [UX Collective redesign](https://uxdesign.cc/redesigning-the-ultimate-guitar-app-dd74915ed9c9)

**MuseScore 4**
- Appearance preferences offer light, dark and high-contrast themes and a choice of accent colors (one source says 7). Hex values of the accents are not in the handbook text found. — [MuseScore handbook: Appearance](https://handbook.musescore.org/en_gb/customization/appearance); [UbuntuHandbook on MuseScore 4](https://ubuntuhandbook.org/index.php/2022/12/musescore-4-released-new-mixer-accent-colors-muse-sounds-plugin/)
- Users repeatedly request that the score itself invert in dark mode (i.e. the score page stays white by default). — [MuseScore issue: Dark Mode improvement](https://musescore.org/en/node/340122)

**Headspace**
- No official hex published ("No official brand hex is published"); orange is the core brand color, reportedly inspired by Buddhist robes. Unofficial extractions disagree: #FFAC1D (wordmark/primary button) vs #FFCD00 yellow + #0061EF blue vs #F47D31 logo dot. — [madegooddesigns Headspace](https://madegooddesigns.com/?p=112985); [webdesignhot Headspace DESIGN.md](https://www.webdesignhot.com/design.md/headspace/); [Kombai Headspace](https://ww2.kombai.com/gallery/inspirations/headspace/)

**Calm**
- Unofficial token extraction from calm.com ("editorial extraction of publicly observable design tokens", not endorsed): bg #0A1F3A (twilight navy), bg-strong #040C1A, surface #13294A, text #FAFAFA, headings #FFFFFF, text-muted #A8B3C5, text-soft #7D8A9E, text-faint #5A6678, brand #0066FF (hover #0052D6, active #003FA3, pale #A8C8FF), accents twilight-purple #7C5FB8 and dusk-pink #E88FA8, borders #243D65 / #365182 / #4A648A, success #42D68A, warning #FFBA43, danger #FF6B6B. — [webdesignhot Calm DESIGN.md](https://www.webdesignhot.com/design.md/calm-app/)
- Other aggregators give different Calm blues (#1D4763 deep blue + #2A4A3E green; or #60B0E7 + #6463E0), so none is authoritative. — [brandcolorcode Calm](https://www.brandcolorcode.com/calm)

**Flat.io**
- No documentation of a dark theme or score colors was found (searches returned unrelated "flat" themes).

**Material (for Android baseline)**
- Material dark theme codelab: base surface #121212 ("typically a dark gray"); text on dark = white at 87% (high emphasis), 60% (medium), 38% (disabled); elevation shown by white overlays (e.g. 12% on the bottom app bar, 2% on cards), "the more elevated the surface is, the stronger and brighter the overlay becomes"; "More saturated colors tend to visually 'vibrate' against darker backgrounds, making them harder to read"; pure white text "appears to bleed or blur against the dark background". — [Google codelab: Design a dark theme](https://codelabs.developers.google.com/codelabs/design-material-darktheme)
- Material (M2) dark theme recommends #121212 and a 15.8:1 white-text-to-surface ratio so that body text still reaches 4.5:1 on the lightest (24dp) elevated surface; desaturate accents (baseline uses the 200 tone). — [UX Planet summary of Material dark theme](https://uxplanet.org/8-tips-for-dark-theme-design-8dfc2f8f7ab6); [Android Wear styles page](https://developer.android.com/design/ui/wear/guides/m2-5/styles/color) (official M2 page body did not load)
- Material 3 dark scheme tokens seen in references: error #F2B8B5 (tone 80), error-container #8C1D18 (tone 30), surface-container #211F26. — [abacktools Material color reference](https://abacktools.com/tools/data/dev-utilities/material-design-color-reference); [Compose Material3 ColorScheme docs](https://developer.android.com/reference/kotlin/androidx/compose/material3/ColorScheme)

### Inferences
- Computed here, WCAG / APCA on #121212: Duolingo Feather Green #58CC02 = 8.97:1 / Lc -61; Cardinal #FF4B4B = 5.67:1 / Lc -42; Macaw #1CB0F6 = 7.66:1 / Lc -54; Bee #FFC800 = 12.06:1 / Lc -77. So Duolingo's light-theme brand colors are usable as text on a Material-dark surface except Cardinal red, which only clears APCA's Lc 45 floor for large/sub-fluent text, not body text. On white, Feather Green (2.09:1 / Lc 40), Macaw (2.44:1) and Fox (2.18:1) fail WCAG AA for text; Duolingo uses them as fills with white text or as big shapes, not as small text.
- Computed here for the Calm extraction: #FAFAFA on #0A1F3A = 15.8:1 / Lc -102; muted #A8B3C5 = 7.8:1 / Lc -59; soft #7D8A9E = 4.7:1 / Lc -37 (below APCA body-text floor); brand #0066FF on the navy = 3.4:1 / Lc -27 (fails as text, fine as a filled button with white label); success #42D68A 8.8:1; warning #FFBA43 9.7:1; danger #FF6B6B 6.0:1 / Lc -47. A navy rather than neutral-grey dark surface is a Calm-style "pleasant dark" that still keeps text contrast.
- Pattern across the music apps: they do not publish hex palettes but converge on four mechanisms worth copying: (1) a global Colors on/off toggle (Soundslice), (2) CVD presets plus a chroma-normalised "balanced" preset plus a saturation boost (Rocksmith+), (3) a number-first alternative view (RS Tab), (4) user-chosen accent colors on a fixed neutral dark/light/high-contrast base (MuseScore 4).
- Success/warning/error in these apps are the conventional green/orange-gold/red (Duolingo, Calm extraction, GitHub-style dark semantics). That convention is widely recognised, but Q4 shows green-vs-red collapses under deutan simulation, so treat the convention as needing redundant encoding.

### Gaps
- No official hex palettes exist for Yousician, Simply Guitar, Rocksmith+, Soundslice, Ultimate Guitar, Flat.io, MuseScore 4 accents, Headspace or Calm; everything beyond Duolingo's Feather Green and the logo colors is third-party extraction.
- Duolingo's dark-mode tokens are not documented anywhere found.
- Rocksmith+ default and colorblind-preset string colors were not found in any official source.
- Material M2 dark-theme page and Apple HIG Dark Mode page returned no body text (JS-rendered), so the 15.8:1 figure and the 1dp-24dp overlay percentages are from secondary summaries; Apple's guidance (pure-black base on iOS, semantic colors) is not cited here.

---

## Key question 3: WCAG 2.2 and APCA contrast guidance for text and non-text UI (tab notes, highlighted current note) and how it applies to dark backgrounds

### Takeaway
WCAG 2.2 requires 4.5:1 for normal text (AA), 3:1 for large text and for every "visual information required to identify user interface components and states" and "parts of graphics required to understand the content" (SC 1.4.11), measured against adjacent colors; a highlighted current note and the tab numbers both fall under 1.4.11 at 3:1, but a state change expressed only by color is a 1.4.1 problem, not a 1.4.11 one. APCA is polarity-aware (light-on-dark is reported as negative Lc) and uses use-case thresholds (Lc 90 preferred body, 75 minimum body, 60 for 24px+, 45 for 36px+ or sub-fluent, 30 spot/placeholder floor, 15 invisibility); it reports much lower contrast for dark colors than WCAG 2 does, so dark-theme palettes that pass WCAG by ratio can fail APCA.

### Cited Findings
- SC 1.4.11 (AA): "The visual presentation of the following have a contrast ratio of at least 3:1 against adjacent color(s)": user-interface components ("Visual information required to identify user interface components and states, except for inactive components") and graphical objects ("Parts of graphics required to understand the content", unless essential). The threshold has no rounding (2.999:1 fails). Measured against the colors in the stylesheet, not the anti-aliased pixels. Selected and focused states are covered (the glossary lists "select" as a state); hover effects "can be considered supplemental" and need not meet 3:1 but must not reduce the component's or the focus indicator's contrast. Where a state changes only the color of an internal area (e.g. a darker fill), 1.4.11 sets no requirement for the difference between states; such a change "would likely fail 1.4.1 Use of Color instead". Each line in a graph is an example of a required graphical part. — [W3C Understanding SC 1.4.11](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast)
- Focus indicator: the 3:1 for the indicator comes from 1.4.11; 2.4.13 Focus Appearance (AAA) adds a minimum area (a 2-CSS-pixel perimeter) and a 3:1 change between focused and unfocused. — [Adrian Roselli CSUN 2024 slides](https://blog.adrianroselli.com/wp-content/uploads/2024/03/CSUNATC-2024_WCAGmire_Adrian-Roselli.pdf); [TPGi/BOIA summary of 1.4.11](https://www.boia.org/WCAG2/CP/1.4.11)
- APCA "Bronze" levels: Lc 90 "Preferred for body text"; Lc 75 "Minimum for body text (>18px)"; Lc 60 "Medium (>24px) fluent text"; Lc 45 "Large (>36px) fluent text, or subfluent text minimum"; Lc 30 "Minimum for any sub-fluent or spot text of any use case"; Lc 15 "Invisibility point for some individuals". Light text on dark gives negative values (max currently Lc -90); use the absolute value. WCAG 2's 3:1, 4.5:1, 7:1 correspond only roughly to Lc 45, 60, 75; "Compared to WCAG 2, APCA reports much lower contrast for dark colors and slightly higher contrast for light colors"; WCAG 2 is described as "relatively meaningless" for dark-mode colors. Non-text items are mentioned but not specified. — [Myndex APCA introduction](https://github.com/Myndex/apca-introduction)
- Secondary APCA summaries: Lc 30 as the floor for placeholder/disabled text, Lc 45 for headings; sources disagree on heading thresholds. — [sass-apca docs](https://npmjs.com/package/sass-apca); [University of Bath blog](https://blogs.bath.ac.uk/digital-content-and-development/?p=430)
- Material dark theme: surface #121212, 87/60/38% white text opacities, white elevation overlays, desaturated accents, warning that saturated colors "vibrate" and pure white "bleeds" on dark. — [Google dark-theme codelab](https://codelabs.developers.google.com/codelabs/design-material-darktheme)
- Material (M2) 15.8:1 benchmark between white text and the base surface so that text on the 24dp surface still passes 4.5:1. — [UX Planet summary](https://uxplanet.org/8-tips-for-dark-theme-design-8dfc2f8f7ab6)
- Material 3 dark scheme: error #F2B8B5, error-container #8C1D18, surface-container #211F26. — [Material color reference](https://abacktools.com/tools/data/dev-utilities/material-design-color-reference)

### Inferences
Computed here (WCAG 2 ratio / APCA Lc, text on background). Use these directly:

| Pair | WCAG | APCA Lc |
|---|---|---|
| #FFFFFF on #121212 | 18.7 | -107 |
| #DEDEDE (87% white) on #121212 | 13.9 | -86 |
| #9E9E9E (60% white) on #121212 | 7.0 | -49 |
| #6C6C6C (38% white) on #121212 | 3.6 | -25 |
| #FFFFFF on #000000 | 21.0 | -108 |
| #E6EDF3 on #0D1117 (GitHub-dark style) | 16.0 | -95 |
| #8B949E on #0D1117 | 6.2 | -44 |
| #E7E5E4 on warm dark #1C1917 | 13.9 | -90 |
| #A8A29E on #1C1917 | 6.9 | -51 |
| M3 error #F2B8B5 on #121212 | 11.0 | -72 |
| Okabe-Ito yellow #F0E442 on #121212 | 14.2 | -87 |
| Okabe-Ito orange #E69F00 on #121212 | 8.3 | -58 |
| Okabe-Ito sky blue #56B4E9 on #121212 | 8.1 | -56 |
| Okabe-Ito reddish purple #CC79A7 on #121212 | 6.1 | -44 |
| Okabe-Ito bluish green #009E73 on #121212 | 5.5 | -40 |
| Okabe-Ito vermillion #D55E00 on #121212 | 4.8 | -36 |
| Okabe-Ito blue #0072B2 on #121212 | 3.6 | -26 |
| Okabe-Ito blue #0072B2 on #FFFFFF | 5.2 | 75 |
| Okabe-Ito vermillion on #FFFFFF | 3.9 | 65 |
| Okabe-Ito bluish green on #FFFFFF | 3.4 | 61 |
| Okabe-Ito orange on #FFFFFF | 2.3 | 44 |
| Okabe-Ito sky blue on #FFFFFF | 2.3 | 45 |
| Okabe-Ito yellow on #FFFFFF | 1.3 | 16 |
| #000000 on cream #FFFFF0 | 20.8 | 105 |
| #333333 on cream #FDF6E3 | 11.7 | 93 |
| #1A1A1A on off-white #FAF3E0 | 15.7 | 97 |

- The Material "60% white" secondary text (#9E9E9E on #121212) passes WCAG AA (7.0:1) but is only Lc -49: below APCA's Lc 60/75 body-text levels. On dark surfaces, use at least ~#B0B0B0-#BDBDBD for any secondary text the learner must read repeatedly (tempo, section names); keep 60% white for captions only. This is the concrete instance of "WCAG overstates contrast for dark colors".
- For tab digits and the current-note highlight on a dark surface: a filled highlight behind a digit must (a) itself reach 3:1 against the page surface (1.4.11, component/state), and (b) keep the digit at 4.5:1 against the highlight fill (1.4.3). With #121212 pages, Okabe-Ito yellow #F0E442 (14:1) or sky blue #56B4E9 (8:1) as a fill with #121212 digits both satisfy this; Okabe-Ito blue #0072B2 (3.6:1 on #121212) is too dark for a highlight fill on dark pages but excellent on light pages (5.2:1 on white).
- Okabe-Ito and Tol palettes were designed for light backgrounds; on #121212 the dark members (blue #0072B2, vermillion #D55E00, bluish green #009E73) fall to 3.6-5.5:1, so for a dark theme lighten them (e.g. toward Tol "light" scheme #77AADD, #EE8866, #44BB99, which are "reasonably distinct in both normal and colour-blind vision" per [Tol](https://sronpersonalpages.nl/~pault/)) and keep the originals for the light theme.
- The guidance that saturated accents "vibrate" on dark surfaces (Material) is designer convention backed by Material's own user research, not an independent study; the Lc numbers above are the measurable proxy.
- Apply APCA and WCAG both: WCAG 2.2 AA is what Android/Play, web audits and EU/US regulations check; APCA is better calibrated for dark mode but not yet normative.

### Gaps
- APCA has no published non-text thresholds yet (the intro page only says "and non-text items"); the WCAG 1.4.11 3:1 remains the only hard non-text number.
- The official M2 dark-theme page and M3 token pages could not be fetched, so the 24dp overlay table (1dp 5% ... 24dp 16%) is not cited.
- Apple HIG dark-mode page returned no body text.

---

## Key question 4: Which hue pairs remain distinguishable for deuteranopia/protanopia/tritanopia when encoding 'correct', 'early/late', 'wrong', 'current target', and what redundant encodings are recommended?

### Takeaway
Roughly 8% of men and 0.4-0.5% of women have a red-green deficiency (deuteranomaly most common), so the classic green=correct / red=wrong pair is the one that collapses: under deutan simulation (computed here) green #22C55E vs red #EF4444 drops to ΔE 13 and GitHub's #3FB950 vs #F85149 to ΔE 8. The pairs that survive all three deficiencies are those that differ in lightness and on the blue-yellow axis: blue/sky-blue vs orange/vermillion, and anything vs a neutral (white/grey). Encode the four states with lightness + hue + a non-color channel (shape, outline weight, motion), and treat Okabe-Ito or Tol palettes as the starting point.

### Cited Findings
- Prevalence: about 8% of men and 0.5% (0.4% in a peer-reviewed source) of women; deuteranomaly is "by far the most common"; lower in Asian (~5% of men) and African (~4%) populations. — [color-blindness.com facts](https://www.color-blindness.com/50-facts-about-color-blindness/); [JCDR 2022](https://doaj.org/article/054cdcb82c8244988387b6bd782667c8)
- Okabe-Ito palette (Okabe & Ito 2008, popularised by Wong 2011 Nature Methods): orange #E69F00, sky blue #56B4E9, bluish green #009E73, yellow #F0E442, blue #0072B2, vermillion #D55E00, reddish purple #CC79A7, black #000000. — [NYU Siegal lab palette page](https://siegal.bio.nyu.edu/color-palette/); [Concept Viz Okabe-Ito reference](https://conceptviz.app/blog/okabe-ito-palette-hex-codes-complete-reference); [R `see` docs](https://search.r-project.org/CRAN/refmans/see/html/okabeito_colors.html)
- Paul Tol qualitative schemes: Bright #4477AA #EE6677 #228833 #CCBB44 #66CCEE #AA3377 #BBBBBB ("colour-blind safe"); Vibrant #EE7733 #0077BB #33BBEE #EE3377 #CC3311 #009988 #BBBBBB ("equally colour-blind safe"); Muted #CC6677 #332288 #DDCC77 #117733 #88CCEE #882255 #44AA99 #999933 #AA4499 ("lacking a clear red or medium blue"); Light #77AADD #EE8866 #EEDD88 #FFAABB #99DDFF #44BB99 #BBCC33 #AAAA00 #DDDDDD ("reasonably distinct in both normal and colour-blind vision", meant for filled cells not lines). Tol's general rule: "pure red and green can be distinguished. Instead, yellow/green, blue/violet and ... cyan/pink/grey combinations can be problematic." Tritanopia is not addressed. — [Paul Tol's colour schemes](https://sronpersonalpages.nl/~pault/)
- Design guidance: brown/red, brown/green, red/green, orange/red are hard for deutans and protans; make status colors differ in brightness, not only hue; add text and icons to status colors; verify with simulators; UI components need 3:1. — [WebAbility: colors to avoid](https://www.webability.io/blog/colors-to-avoid-for-color-blindness)
- ISO 24505-2:2025 ("Ergonomics - Accessible design - Part 2: Colour combinations for people with colour deficiency and low vision") exists as a standard for deutan/protan combinations (contents not accessible here). — [BSI listing](https://knowledge.bsigroup.com/products/ergonomics-accessible-design-part-2-colour-combinations-for-people-with-colour-deficiency-and-low-vision); [ISO 24505-2:2025](https://genorma.com/en/standards/iso-24505-2-2025)
- Rocksmith+ (a direct precedent for a guitar app): colorblind presets per deficiency type, a chroma-normalised "Balanced Colors" preset, a saturation "Color Boost", and a number-based "RS Tab" view. — [Ubisoft accessibility spotlight](https://news.ubisoft.com/en-us/article/5XfINjoRmVJgBKR8fGt4EM/rocksmith-accessibility-spotlight)
- Redundant encoding research: a 2026 visualization paper (CatPAW) on unifying color and shape encodings notes viewers can efficiently process categories through a single channel only while the count stays small, "as the number of categories increases, the discriminability ... diminishes", motivating redundant color+shape. — [arXiv 2602.06792](https://arxiv.org/pdf/2602.06792)
- Visual search: color pops out unconditionally (Enns & Rensink), and search for a color-defined target is not slowed by set size, whereas shape search is; pop-out depends on target-distractor dissimilarity (Duncan & Humphreys 1989). — [Rensink: conditional vs unconditional pop-out](https://www2.psych.ubc.ca/~rensink/publications/abs.93.1.html); [arXiv 2103.06084 on colors/shapes and outlier detection](https://arxiv.org/pdf/2103.06084)
- WCAG 1.4.1 Use of Color: color must not be the only visual means of conveying information (the Understanding 1.4.11 page defers state-only-by-color cases to it). — [W3C Understanding 1.4.11](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast)

### Inferences
Computed here (Machado 2009 severity-1.0 simulation, CIELAB ΔE76; a ΔE below ~20 is risky for small marks, below ~10 is effectively indistinguishable):

| Candidate 4-state set | normal min ΔE | protan | deutan | tritan | weakest pair |
|---|---|---|---|---|---|
| Okabe-Ito: correct #009E73, early/late #E69F00, wrong #D55E00, target #56B4E9 | 33 | 25 | 18 | 16 | orange vs vermillion (deutan); green vs sky-blue (tritan) |
| Naive traffic light: #22C55E / #EAB308 / #EF4444 / #3B82F6 | 72 | 25 | **13** | 27 | green vs red (deutan), green vs yellow (protan) |
| Tol vibrant: teal #009988 / orange #EE7733 / magenta #EE3377 / blue #0077BB | 56 | 20 | 20 | **13** | teal vs magenta (protan/deutan); teal vs blue (tritan) |
| Two-tone + neutrals: correct #FFFFFF, early/late #9E9E9E, wrong #E69F00, target #56B4E9 | 35 | 32 | 35 | 35 | white vs grey (all), still ≥32 |
| GitHub-dark semantics: #3FB950 / #D29922 / #F85149 / #58A6FF | 57 | 16 | **8** | 18 | green vs red (deutan) |

- The robust pattern: a set whose states differ in *lightness* survives every simulation at ΔE ≥ 32, while any set that relies on green-vs-red or green-vs-yellow hue differences collapses for deutans/protans. Recommended encoding for a dark tab: **current target** = sky blue (#56B4E9, or Tol #77AADD) with an outline ring and a pulse/scale animation; **correct** = the note simply turns neutral white/off-white and stays (no hue needed, lightness carries it); **early/late** = same hue as correct but at reduced lightness/opacity plus a left/right arrow or offset tick; **wrong** = orange/vermillion (#E69F00 on dark; #D55E00 on light) plus an X glyph or a thicker stroke. Orange vs blue is the pair that stays apart on both the red-green and blue-yellow axes.
- If a red is wanted for "wrong" by convention, pair it with blue (never green) as the opposing state and keep red lighter than the surface by a wide margin (M3's #F2B8B5 at 11:1 on #121212 is the "dark-mode red" pattern); the green=correct convention should carry a checkmark or the "turn neutral" treatment described above.
- Offer the Rocksmith+ mechanisms: per-deficiency presets, a chroma-normalised preset, a saturation boost, and a numbers-only mode. A single "colorblind mode" is weaker than per-type presets because protan and deutan confusion lines differ (orange/vermillion closes most under deutan, green/yellow under protan in the table above).
- Shape/weight/motion redundancy is both required (WCAG 1.4.1) and supported by the pop-out literature: motion and luminance are preattentive channels that do not depend on hue discrimination.

### Gaps
- The ΔE table is a simulation, not a user study; Machado-matrix simulations approximate dichromacy and over-state the deficit for the much larger anomalous-trichromat group.
- ISO 24505-2 contents (the standard's actual pair recommendations) were not accessible.
- Wong 2011 (Nature Methods) full text was behind a login; its prevalence and pair guidance are therefore cited through the palette pages above.
- No study found that tested CVD readability of guitar-tab state colors specifically.

---

## Key question 5: What is known about 'paper-like' / 'parchment' backgrounds in sheet-music apps and why they feel pleasant for notation reading?

### Takeaway
The largest study on background colors (341 readers, 89 dyslexic; Rello & Bigham 2017) found warm, low-chroma light backgrounds (Peach #EDD1B0, Orange #EDDD6E, Yellow #F8FD89) gave the shortest reading times and cool pale blues (Blue Grey #DBE1F1, Blue #96ADFC) the longest, for both groups; dyslexia guidance (BDA, Dyslexia Scotland) independently recommends dark-not-black text on cream/off-white rather than pure white, citing glare. "Parchment" is therefore an evidence-backed warm off-white, but the specific sheet-music apps found only offer sepia/tint filters or user-set near-whites (#FFFFF0 suggested on the MobileSheets forum), and the "why it feels pleasant" explanations (glare, Meares-Irlen sensitivity, paper familiarity) are only partly studied.

### Cited Findings
- Rello & Bigham 2017 (ASSETS), 341 participants (89 with dyslexia) read 10 texts (18pt Arial, black #000000 font) on 10 backgrounds; hex and black-text contrast: Blue #96ADFC 9.68:1; Blue Grey #DBE1F1 16.05:1; Green #A8F29A 15.83:1; Grey #D8D3D6 14.21:1; Orange #EDDD6E 15.17:1; Peach #EDD1B0 14.35:1; Purple #B987DC 7.56:1; Red #E0A6AA 10.2:1; Turquoise #A5F7E1 16.99:1; Yellow #F8FD89 19.4:1. Results (both groups): Peach shortest reading time, then Orange, then Yellow; Blue Grey longest ("significantly longer than the rest", p<0.001), Blue second longest; Blue Grey also had the longest mouse distance. Mean reading times (control): Peach 14.85 s, Orange 15.33 s, Yellow 16.30 s. Authors: warm backgrounds "are beneficial for readability", but the result "do[es] not necessarily mean that warm colors are recommended", cool colors may relax some readers, and a comparison with white/off-white was left for future work. — [Rello & Bigham 2017 PDF](https://www.changedyslexia.org/publications/pdfs/2017-ASSETS-Good%20Background%20Colors.pdf)
- Earlier Rello et al. eye-tracking (22 dyslexic readers): black on cream gave the shortest fixation durations (0.214 s mean), the pair recommended by the BDA; people with dyslexia read faster with lower-contrast pairs while non-dyslexic readers preferred higher brightness contrast. — [Rello & Baeza-Yates, UPF repository](https://repositori.upf.edu/items/e0ca6f40-a0fa-490f-9d03-53d043270deb); [W3C text-customization research page](https://w3.org/WAI/RD/2012/text-customization/r11)
- BDA style guide (as quoted by Dyslexia Scotland): "avoid using contrast colours. Use dark (not black) text on a light (not white) background", "use less contrasted colours, such as cream-black", avoid yellow-black, white-blue and grey combinations; Dyslexia Scotland recommends "dark blue text on cream, pastel or off-white backgrounds". The article admits neither recommendation cites research and notes the tension with WCAG 4.5:1. — [Dyslexia Scotland: Contrasting advice](https://dyslexiascotland.org.uk/contrasting-advice-what-colours-are-best-for-accessibility/)
- Secondary summaries of the BDA guide attribute the advice to glare from pure white and recommend "soft pastel shades, off-white, cream, or light grey". — [QMUL inclusive formatting guidelines](https://www.qmul.ac.uk/disability-and-dyslexia-service/media/disability-and-dyslexia-service-/documents/Guidelines-for-inclusive-writing-and-formatting-of-written-assessments-v9-07-11-2019.pdf)
- Light mode performance advantage grows for small text and at night; subjective readability is not different (Piepenbrock; Dobres). — [NN/g](https://www.nngroup.com/articles/dark-mode/)
- Sheet-music apps: ScorePDF offers sepia, green or inverted filters "if the page white looks too bright"; OnSong offers tint colors to "reduce visual stress on stage"; the MobileSheets developer planned background-color options and a user suggested "#FFFFF0 to take the edge off the white"; some MobileSheets users prefer inverted/night mode for charts. — [ScorePDF App Store](https://apps.apple.com/app/id1606085433); [OnSong tint docs](https://new.onsongapp.com/docs/tint); [MobileSheets forum thread](https://zubersoft.com/mobilesheets/forum/showthread.php?pid=4022)
- Soundslice keeps black-on-white as the default notation and treats dark mode as "nontraditional"; MuseScore keeps the score page white in dark mode and users request inversion. — [Soundslice dark mode](https://www.soundslice.com/blog/239/introducing-our-players-dark-mode); [MuseScore issue 340122](https://musescore.org/en/node/340122)

### Inferences
- Studied: a warm, low-chroma, high-luminance page (Peach #EDD1B0 region, or paler analogues) reads faster than cool pale blues and is preferred by dyslexia guidance; computed here, black on #FFFFF0 is 20.8:1 / Lc 105 and #1A1A1A on #FAF3E0 is 15.7:1 / Lc 97, so a cream page costs nothing in contrast. A reasonable light-theme page: #FAF3E0-#F7F1E3 (cream) with near-black #1A1A1A-#2A2A2A ink, i.e. "dark not black" per BDA while still far above WCAG AAA.
- Designer convention, plausibly studied via glare: the pleasantness of parchment likely comes from (a) lower peak luminance than #FFFFFF, which reduces glare in dim rooms where the display is the brightest object, (b) familiarity with printed paper, and (c) the warm-hue reading-time advantage in Rello & Bigham. (b) is not studied in the sources found.
- Do not over-tint: Rello & Bigham's best colors are far more saturated than typical "parchment" UI; a UI parchment should stay near-neutral so that the colored note states of Q4 keep their hue contrast against it (orange "wrong" marks on a strongly orange page lose ΔE).
- For dark mode, the analogue is a warm near-black (#1C1917-style, computed 13.9:1 with #E7E5E4 ink) rather than pure #000000, matching Material's #121212 advice and Material's warning about pure white bleeding; but the notation itself should flip to light-on-dark only as an opt-in, following Soundslice's and MuseScore users' practice, because the ergonomics literature still favours dark-on-light for small glyphs.

### Gaps
- No study found that compares cream vs pure white backgrounds for musical notation specifically; Rello & Bigham explicitly left the white/off-white comparison for future work.
- No source explains the "paper feel" preference mechanistically beyond glare and the dyslexia guidance; the familiarity explanation is an inference.
- No sheet-music app was found that ships a documented parchment hex.

---

## Key question 6: How many simultaneous hues can a learner track in a tablature display without confusion (e.g. per-finger colors), and when should an app drop per-finger coloring?

### Takeaway
Visualization practice caps categorical hues at about 6-12 (Ware), with practitioners advising ~8 and a single-channel comfort zone below ~6; visual working memory holds ~4 items and people fall back on coarse color categories under load. The one controlled music study (Rogers 1991, n=92) found pitch-colored notation helped only while the colors were present and *hurt* sight-reading of plain notation, so per-finger coloring (4 hues, as Yousician does) is within the perceptual limit but should be treated as scaffolding that fades.

### Cited Findings
- Ware: the number of colors that work as effective category codes is "between 6 and 12"; Andy Kirk, citing Ware's *Visual Thinking for Design*, says schemes beyond 12 are hard to interpret reliably and suggests a practical maximum of about 8; UBC lecture notes give a rule of thumb of no more than ~12 hues for nominal data. — [arXiv 1405.5263 citing Ware](https://arxiv.org/pdf/1405.5263); [Visualising Data: multiple-colour legends](https://visualisingdata.com/2012/07/problems-of-interpreting-multiple-colour-legends/); [UBC Munzner slides citing Ware](https://www.cs.ubc.ca/~tmm/talks/sig03.multimodal/mgp00025.txt)
- Visual working memory capacity is about four items (Cowan 2010; Luck & Vogel 1997); with higher load, people rely more on categorical color representations, and recalled colors are biased toward category centers even without a delay. — [Pupillometry study, PMC9539610](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9539610/); [Bae et al., color categories in working memory](https://durham-repository.worktribe.com/output/1418625/why-some-colors-appear-more-memorable-than-others-a-model-combining-categories-and-particulars-in-color-working-memory)
- A visualization paper states viewers can process categories through a single channel such as color without overload only for a small number, with discriminability falling as categories grow (heuristic, not a measured limit). — [arXiv 2602.06792](https://arxiv.org/pdf/2602.06792)
- Rogers 1991 (J. Research in Music Education 39:64-73): 92 fifth/sixth-grade beginning wind players; each pitch had its own color. Memory performance equal; the color group sight-read better *with* color but "scored significantly lower when sight-reading plain notation"; students may have "memorized the colors instead of the notation"; 65% of all subjects favored colored notation as easier; students with learning disabilities depended heavily on color. Rogers 1996 (134 first/second graders) used arbitrary weekly-changing colors for rhythm and found a small advantage for the colored group. — [Wikipedia: Colored music notation](https://en.wikipedia.org/wiki/Colored_music_notation); [Rogers 1991, SAGE](https://journals.sagepub.com/doi/10.2307/3345410)
- A 2003 patent application summarises Rogers: his color-coded notation experiment "did not result in higher student achievement". — [Justia patent 20030000364](https://patents.justia.com/patent/20030000364)
- Yousician uses four finger colors (guitar: yellow = index; piano: yellow/purple/blue/red for index/middle/ring/little; thumb green in another view), always with the note name shown. — [Yousician guitar tab help](https://support.yousician.com/hc/en-us/articles/206932099-How-to-read-guitar-tablature); [Yousician piano notation options](https://support.yousician.com/hc/en-us/articles/208678605-Notation-options-in-piano)
- Boomwhackers/Chroma-Notes pitch convention uses seven hues (C red ... B violet). — [Soundslice Boomwhackers help](https://www.soundslice.com/help/en/player/advanced/336/boomwhackers-colors/)
- Rocksmith+ provides a numbers-first "RS Tab" view for players who do not want to rely on color. — [Ubisoft](https://news.ubisoft.com/en-us/article/5XfINjoRmVJgBKR8fGt4EM/rocksmith-accessibility-spotlight)
- Berklee: fingerings are usually shown only in pedagogical materials "unless a specific string or finger color is desired". — [Berklee Take Note](https://online.berklee.edu/takenote/?p=10016)
- Color pop-out is not slowed by set size, but only when the target is dissimilar from distractors (Duncan & Humphreys). — [Rensink pop-out abstract](https://www2.psych.ubc.ca/~rensink/publications/abs.93.1.html); [arXiv 2103.06084](https://arxiv.org/pdf/2103.06084)

### Inferences
- Budget: four finger hues + one "current target" + one "wrong" = 6, already at the low end of Ware's 6-12 and at the single-channel comfort limit; adding six string colors (Rocksmith style, 6 more) on top of finger colors would exceed every heuristic. Pick one categorical dimension per view: per-finger colors *or* per-string colors, never both, and keep state colors (Q4) on a different channel (fill vs outline, or the note turning neutral).
- Dropping rule (studied, from Rogers 1991): colored scaffolding produces dependence; fade finger colors once a learner's accuracy on a section is stable (e.g. after the section is passed at full speed), and show plain tab by default in review/"sight-read" modes so that the learned skill transfers. Keep the per-finger colors available as a toggle (65% of Rogers' students liked them).
- Perceptual rule: with 4 finger hues, choose them to be maximally dissimilar in both hue and lightness (Okabe-Ito sky blue, orange, reddish purple, bluish green, or Tol light #77AADD #EE8866 #FFAABB #44BB99 on dark) so the current-target pop-out survives (Duncan & Humphreys), and verify with the deutan/protan simulation in Q4.
- Working-memory inference: because people drop to category labels under load, finger colors must be nameable and conventional ("blue", "orange", "pink", "green"), not two shades of one hue.

### Gaps
- No controlled study found on per-finger (as opposed to per-pitch) coloring for guitar, nor on string coloring; the evidence is transferred from wind-instrument pitch coloring (Rogers) and from visualization heuristics.
- Ware's exact wording and the 6-12 range were only available through secondary citations.
- No study found that measured the maximum number of simultaneous hues in a scrolling real-time display (note highway) specifically.
