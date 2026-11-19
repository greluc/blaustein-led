# Research brief for a session with internet access

This is the task for a coding agent that runs on the owner's computer, can open web pages and can
ask the owner questions. The session that wrote the lighting plan could reach package registries
only: it opened no web page, saw no datasheet and never saw the model. Your job is to verify what
it wrote, research what it could not, put the whole plan under the microscope and turn it into
something that can be ordered and built.

## The project

Blaustein LED lights the BlueBrixx model Burg Blaustein, the full-size version with every expansion
and the side buildings, with flames at the fireplaces, torches, candles and a faint light in the
rooms. Fixed constraints: the firmware is written in Rust and runs on a Raspberry Pi Pico 2. The
Pico hides in the rock base, cables run from there to the LEDs with as little damage to the model
as possible, and the installation is powered through a USB lead whose quick-release coupling is
the only thing that shows outside. It is switched by cutting the power.

Read these first, in this order:

1. [`CLAUDE.md`](../CLAUDE.md) and [`CONTRIBUTING.md`](../CONTRIBUTING.md): the rules of this
   repository. They bind you.
2. [`lighting-plan.md`](lighting-plan.md): the analysis, the owner's decisions, the lighting
   concept, the draft channel plan, and the lists "Unverified", "Sources not read" and "Statements
   from memory".
3. [`hardware.md`](hardware.md): the electrical design, the power supply, the installation rules
   and the generic bill of materials.
4. [`README.md`](../README.md), `src/config.rs` and `fire-effect/`: what the firmware does today.

## Where the work stands

- Pull request 12 (`claude/burg-blaustein-led-lighting-0ip7t4`) carries the plan and the firmware
  rework in stages; its description lists which stages are done. Read it and `git log` before you
  start. If it is merged, branch from `main`. If it is still open, branch from its head and open a
  stacked pull request; do not push to its branch while one of its stages is unfinished.
- Nothing is installed in the model and nothing of the reworked firmware has been flashed to a
  board.

## How to work

- **Be honest and critical.** Do not tell the owner what they want to hear. If a decision of the
  plan is wrong or second best, say so plainly and show the evidence. If you do not know something
  or find no reliable source, say exactly that.
- **Open the page.** A search result snippet is not a source. Take facts from pages you opened,
  prefer the manufacturer's product page, building instructions and datasheets over blogs and
  shops, and note where sources contradict each other.
- **Invent nothing.** No part numbers, prices, dimensions, ratings or quotes from memory. Label
  every assumption as one.
- **Record what you verified.** For every source: the link, the date you opened it, and what it
  confirmed or corrected. For every price and availability: the date.
- **Do not guess when coding.** Read the documentation or the source of what you use.
- **Ask the owner** when a decision is theirs: budget, tools and soldering skills, preferred
  shops, how much of the model may be rebuilt, anything about taste. Ask with a question dialog if
  you have one, with your recommendation first.
- **Language.** Talk to the owner in German. Everything in the repository is English.
- **Hardware.** If a Pico 2 is connected, ask before you flash it. Never claim a result on
  hardware you did not see.

## Tasks

### 1. Verify every source and every statement

- Open every page under "Sources not read" in [`lighting-plan.md`](lighting-plan.md) and confirm or
  correct the statement taken from it. Find the pages listed without a link.
- Confirm or correct every item under "Statements from memory" against a primary source.
- Recheck every calculation in [`hardware.md`](hardware.md) with the verified values.
- Replace the three lists by one list of verified sources with dates. Whatever cannot be verified
  stays listed as unverified, with what you tried.

### 2. Research the model

- For every set, establish from the product pages, the building instructions and reviews with
  photos or video: rooms, fireplaces, ovens, forges, torch holders, candles, lanterns, windows
  that would show a light, and cavities that can carry cables. Sets: the base castle, Bergfried,
  Vorburg, Saalbau, Hurde, Münzturm, Torhaus, Schildwerk, Festungsspitzen, Rüstmeister,
  Stallungen, Zehntscheune. Confirm the item numbers and whether the list is complete.
- Establish which floors, roofs and modules separate and how the modules join, because every
  separation needs a connector.
- Establish what the rock base looks like inside. The owner says it is hollow and reachable.
- Find out whether BlueBrixx or a third party sells a light kit for this set, and what it lights.
- Result: an inventory of light points per module and a corrected channel plan. Decide with the
  owner whether 16 channels are enough, and update the channel table in the firmware to match.

### 3. Research how brick models are lit

Find what the community and the makers of light kits regard as best practice for lighting LEGO and
other brick sets, and judge each point against this project:

- Which light sources are usual: pre-wired SMD LEDs and their sizes, LED strips, light bricks,
  filament LEDs, fibre optics, addressable LEDs, LEDs with a built-in flicker circuit. Which of
  them suit a fireplace, a torch, a candle, embers and room light, and which are best for each?
- Which wires, connectors, distribution boards and power supplies the kits use, how they route
  cables between bricks, how they hide and diffuse LEDs and how they stop light bleeding through
  walls.
- Would any of it be an advantage over the plan? The plan drives single LEDs directly from the
  3.3 V PWM pins with a resistor on the board. Kit LEDs often carry a resistor for 5 V; check what
  that means. Re-examine honestly whether a 5 V rail with low-side switches, or addressable LEDs
  for some of the lights, would be better, and what each costs in wiring, resolution at low
  brightness, PWM frequency and robustness.

### 4. Make it modular

- The castle comes apart into floors and modules, and expansions are added over time. Design the
  wiring so that a module can be lifted off, added or serviced without soldering.
- Evaluate concrete connector systems for the three places that need them: at the control board,
  between modules and between floors. Consider pin and socket headers in several pitches, small
  wire-to-board and wire-to-wire families, spring-loaded and magnetic contacts that separate when
  a floor is lifted, flat flexible cables, and the connector systems of the light kits.
- Consider a small distribution board per module, and for the control board compare a perfboard, a
  ready-made breakout board for the Pico and a custom circuit board ordered from a manufacturer.
- Recommend one coherent system with part numbers and a pin assignment, and say what it costs.

### 5. Bill of materials with real parts

- Replace the generic bill of materials in [`hardware.md`](hardware.md) by one made of parts that
  can be bought in Germany today, mainly from Conrad and Amazon and from specialist electronics
  retailers such as Reichelt, Pollin, Völkner, Berrybase, Mouser or DigiKey, and from model-making
  shops for pre-wired LEDs.
- For every line: retailer, article number, link, price, date checked, and a second source where
  one exists. For every LED: forward voltage at the planned current, colour temperature or
  wavelength, viewing angle, wire gauge and length, and whether it carries a resistor.
- Then plan with exactly these parts: resistor values from the real forward voltages, the current
  budget against the verified limits of the RP2350, connector pin assignments, cable lengths from
  the real dimensions of the model, the lead, the coupling and the power supply. Check that a
  magnetic coupling that also carries data exists, or choose another quick-release solution.
- Give a total price and say which parts are worth buying as a sample first.

### 6. Printed parts

The owner has a Prusa CORE One L (filament) and an Elegoo Saturn Ultra with a 16K screen (resin);
confirm the exact model names with the owner. Find out whether bricks compatible with the set can
be printed well enough to clamp, and which printer suits which part:

- Dimensions and tolerances of the bricks, how well each process holds them, which materials clamp
  without cracking or wearing the original bricks, and how others calibrate printed bricks. Look
  for existing parametric brick generators and check their licences.
- Parts worth designing: bricks and plates with cable ducts, wall feed-throughs, holders for an
  LED inside a fireplace or behind a torch with a diffuser, connector bricks between modules and
  floors, a mount that sets the magnetic coupling flush into the rock, a carrier for the control
  board with studs, a strain relief for the lead, and inserts that block stray light.
- Colour: where a printed part is visible, how to match the colours of the set; otherwise keep
  printed parts out of sight.
- Propose a test plan, starting with a small calibration print that is tried on bricks of the set.
  If printed parts pay off, add their design files to the repository as text-based sources.
- Every printed part that replaces a drilled or modified original brick is a gain: the goal is an
  installation that can be removed without a trace.

### 7. Review the whole plan

With all of the above, go through the plan again from the start and challenge every decision: the
direct drive, the number of channels, the lighting concept and its roles, the placement of the
LEDs, the power supply and its coupling, the location of the control board, the installation
steps, and the firmware parameters. Keep what holds, change what does not, and tell the owner
where you disagree with the earlier session and why.

### 8. Verify the firmware where you can

- Check the `usb-update` feature against the header of the Pico SDK that defines the reset
  interface and against the source of picotool: the interface class, subclass and protocol, the
  request number and the layout of `wValue` in `fire-effect/src/picotool_reset.rs`; the vendor and
  product ID and the Microsoft OS 2.0 descriptors of a device with a single interface in
  `src/usb_update.rs`; and the picotool commands in the README.
- If the owner connects a Pico 2 and agrees: build and flash, run the identify build, cut and
  restore the power repeatedly, try the USB update, and measure the current. Report exactly what
  you saw.

## Results

- [`lighting-plan.md`](lighting-plan.md) and [`hardware.md`](hardware.md) corrected and completed,
  with verified sources and no statement left that only rests on a snippet or on memory unless it
  is marked as unverified.
- The bill of materials with real articles and prices, the connector system, the corrected channel
  plan, and the findings on best practice and on printed parts, each in the document where it
  belongs or in a new one under `docs/`.
- An entry in `CHANGELOG.md`, commits and a pull request as the repository rules require.
- For the owner, in German: a short honest summary of what changed in the plan and why, what
  remains uncertain, the total cost, and the decisions that are theirs to take.
