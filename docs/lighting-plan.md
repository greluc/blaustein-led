# Lighting plan

The goal is to make the BlueBrixx model Burg Blaustein (the full-size version, with every expansion
and the side buildings) look alive: flames at the fireplaces, torches inside and outside, candles
and a faint light in the rooms. The Raspberry Pi Pico 2 hides in the rock base, cables run from
there to the LEDs without drilling or removing bricks where it can be avoided, and the installation
is switched by cutting its power. Rust and the Pico 2 are fixed.

This document holds the analysis that led to the plan, the lighting concept, the channel plan, the
firmware work and the steps to build it. The wiring, the power supply, the installation rules and
the bill of materials are in [hardware.md](hardware.md).

## Decisions

Taken by the owner on 2026-11-19:

| Question | Decision |
| --- | --- |
| Scope | The full build: base castle, all expansions up to the Schildwerk, and the side buildings Rüstmeister, Stallungen and Zehntscheune |
| Installed today | Nothing; the plan is implemented from scratch |
| Place for the Pico | The rock base of the base castle; it is hollow and reachable |
| Kinds of light | Fire and torches, candles, and a faint ambient light in rooms without a fire |
| Firmware update | Through USB with the Pico in place, as an optional Cargo feature; the Pico is socketed as well |
| Light points per module | Open. To be researched by a session that can reach the manufacturer's pages, see [Open research](#open-research) |

## Analysis of version 2.0.0

Version 2.0.0 was clean, tested and tiny. Its weaknesses were the look of the flame, the single
flame type and the missing documentation of the build. The figures come from a release build and a
host simulation of the installed configuration over 200,000 ticks (50 minutes).

| Topic | Finding | Assessment |
| --- | --- | --- |
| Memory | 5.5 KB of flash (0.13 % of 4 MB) and 880 bytes of static RAM with 16 channels | Nothing to do |
| CPU load | About 0.1 %, the core sleeps otherwise. An estimate from the code, not measured | Nothing to do |
| Tests | 26 host tests passed, the firmware built without warnings | Good |
| Cutting the power as a switch | The firmware never writes to flash and keeps no state. Watchdog of 1 s, a panic resets | Good, a hard power-off is uncritical |
| Lockstep | The breathing envelope was identical and in phase on all channels. Correlation between any two channels 0.65, with the baseline profile 0.99. The pattern repeated after exactly 5 s (autocorrelation 0.65) | The largest defect: the whole castle breathed together |
| README claim | "the flames never move in lockstep" held for the noise only, not for the envelope | Wrong |
| Parameters | `max_intensity: 128` bounded the envelope only. The frequent flares added about 72 on average, so the output ran from 18 to 250 with a mean of 140 | Misleading, brightness hard to set |
| Flame types | Every channel used the same profile | Hearth, torch, candle, ember and ambient light need characters of their own |
| Resolution in the dark | The intensity had 8 bits before the gamma curve. From step 20 to 21 the brightness jumped by 11 % (36 to 40 counts) | Irrelevant for a bright flame, visibly steppy for embers and ambient light |
| PWM phases | The eight slices started one right after the other, a few microseconds apart by estimate, so their on-phases overlapped almost completely | Needless current peaks on long thin wires |
| Resistors | The README recommended 330 Ω to 1 kΩ. 330 Ω on an orange LED are about 3.9 mA, 63 mA at peak with 16 channels | Too tight for 16 channels, 680 Ω to 1 kΩ is better |
| Hardware documentation | The README named "GPIO, resistor, LED, ground" only. Channel plan, schematic, bill of materials and location were missing | The real build could not be assessed |
| Maintenance | Flashing needed the BOOTSEL button on the Pico | Impractical once buried in the rock |

More work on size or speed brings nothing measurable. The effort belongs to the look of the flame,
the electrical design and the documentation.

## Lighting concept

Five light roles replace the single flame. Each has its own LED colour, placement and motion. The
times are starting values to be tuned on the model, not measurements.

| Role | Where | LED | Placement | Motion |
| --- | --- | --- | --- | --- |
| Hearth | Hall, kitchen, chambers, keep | Amber or orange for the flame, red for the embers | Flame LED under the logs behind transparent orange parts, ember LED sideways in the mantel | Irregular swell over 1 s to 3 s plus lively flicker, embers dragged along sluggishly |
| Torch | Gate, courtyard, wall walk, outer walls | Amber or orange, size 0402 | Under or behind the transparent flame part | Fast and restless, small swing, occasional gusts |
| Candle | Bedroom, chapel, archive, guard room, table | Warm white, 2000 K to 2700 K if available | On a table or wall bracket, very faint | Almost steady, a rare short twitch |
| Ember | Forge, oven | Red, at the forge orange as well | Deep in the firebox, indirect | Very slow glow |
| Ambient | Rooms without a fire of their own, half-timbered house | Warm white, one or two per room | Hidden behind a lintel or beam, lighting a wall or the ceiling | Constant and dim with a few percent of unrest |

Large fireplaces get two LEDs. The bright flame and the red, sluggish embers together give the
colour shift of a real fire.

## Channel plan

Sixteen channels are enough for the full build if every real fire gets a channel of its own and
candles, ambient light and torches far apart share channels. The table is a draft: the rooms come
from shop and blog descriptions that were available as search results only, and the interiors of
Hurde, Münzturm and Schildwerk are unknown. It is confirmed against the model and the building
instructions before anything is wired.

| Channel | GPIO | Place | Role | LEDs |
| --- | --- | --- | --- | --- |
| 0 | 16 | Base castle, hall: fireplace flame | Hearth | 1 amber |
| 1 | 17 | Base castle, hall: fireplace embers | Ember, following channel 0 | 1 red |
| 2 | 18 | Base castle, kitchen: cooking fire | Hearth | 1 amber, 1 red |
| 3 | 19 | Base castle, bedroom | Candle | 1 warm white |
| 4 | 20 | Base castle, courtyard | Torch | 1 or 2 amber |
| 5 | 21 | Keep, living room: fireplace | Hearth | 1 amber, 1 red |
| 6 | 22 | Keep, guard platform | Torch | 1 amber |
| 7 | 7 | Saalbau, chambers: fireplace | Hearth | 1 amber, 1 red |
| 8 | 8 | Saalbau, knights' hall: table and chandelier | Candle | 2 warm white |
| 9 | 9 | Saalbau, chapel | Candle | 1 or 2 warm white |
| 10 | 10 | Gatehouse, left gate torch, and wall walk of the Hurde | Torch | 2 amber |
| 11 | 11 | Gatehouse, guard room and archive | Candle | 2 warm white |
| 12 | 12 | Forge | Ember | 1 red, 1 orange |
| 13 | 13 | Vorburg, half-timbered house | Ambient | 2 warm white |
| 14 | 14 | Gatehouse, right gate torch, and wall walk of the Schildwerk | Torch | 2 amber |
| 15 | 15 | Münzturm | Candle or ambient | 1 or 2 warm white |

- The order follows the firmware: GPIO 16 to 22, then GPIO 7 to 15.
- Two LEDs on one channel flicker identically. Only LEDs that are never seen side by side share a
  channel; the two gate torches are on different channels on purpose.
- The sources do not say clearly whether the forge belongs to the Saalbau or to the Rüstmeister.
- The side buildings Stallungen and Zehntscheune have no channel yet; a lantern each would share
  an ambient channel.
- If 16 channels are not enough, up to 26 are possible: the ten free pins (GPIO 0 to 6 and 26 to
  28) can run as PWM through PIO, for which Embassy ships a program. Effort and resolution have
  not been examined.

## Not worth doing

- **Shrinking flash and RAM further.** 0.13 % of the flash was in use.
- **Lowering the clock.** The PWM resolution hangs on the system clock, and nothing noticeable is
  saved at the power supply.
- **Addressable LEDs (WS2812).** Only 8 linear bits per colour, steppy in the dark, a larger
  package, and the chain fails behind a broken spot. For warm fire, single LEDs on the hardware
  PWM are better.
- **External PWM chips.** The built-in PWM with 25 kHz and about 12.5 bits beats common I2C
  drivers.

## Implementation steps

Each step has a checkpoint. First the bench, then one floor, then the whole castle, so faults show
before wires are built in.

1. **Bench setup.** Pico 2 on a breadboard, one LED of every colour with the starting resistors.
   - Checkpoint: the brightness fits in a darkened room and the resistor values are fixed.
2. **Firmware.** Tune the roles on the bench setup.
   - Checkpoint: the look of every role is approved on the bench.
3. **Board.** Perfboard with the socketed Pico, resistors and pin headers.
   - Checkpoint: all 16 channels are verified in the identify build. The measured total LED current
     is below 40 mA.
4. **Power supply.** Lead, strain relief, coupling and power supply.
   - Checkpoint: separate and reconnect twenty times, the firmware starts every time. Then a run of
     24 hours.
5. **Pilot in the base castle.** Hall, kitchen, bedroom and courtyard, channels 0 to 4, with the
   floor connectors.
   - Checkpoint: the floors come off, no brick stands proud, no wire is visible. The picture is
     tuned in the dark.
   - Decision: are 16 channels enough for the remaining modules?
6. **Expansions.** Module by module along the channel plan, each with a connector at its joint.
   - Checkpoint per module as in step 5.
7. **Wrap-up.** Channel plan, photos and bill of materials in this repository, then a release.

## Unverified

The session that wrote this plan could reach package registries only. No web page could be opened:
bluebrixx.com, brickfact.com and the Raspberry Pi datasheet hosts did not resolve, and after two
failures no further host was tried. Every statement below therefore rests on a search result
snippet, on memory or on a calculation, and is confirmed against the source before it is relied on.

- **Set.** Rooms, fireplaces, part counts, dimensions, item numbers and the order of the expansions
  come from search result snippets, see [Sources not read](#sources-not-read).
- **Datasheet values.** The permitted total GPIO current of the RP2350 and the power consumption of
  the Pico 2 have not been looked up.
- **LED data.** The forward voltages are typical values, not manufacturer data.
- **Building technique.** Cable routes, clamping force and translucency are experience values of
  lighting brick models, not tested on this set.
- **Effect.** The simulation measures lockstep, not beauty. The look of the flame can only be
  judged on the model.
- **Hardware.** Nothing of the firmware described here has been flashed to a board by the session
  that wrote it.

## Open research

For a session that can reach the manufacturer's pages and the datasheets:

- [ ] Open every source under [Sources not read](#sources-not-read) and confirm or correct the
      statement taken from it, in this document and in [hardware.md](hardware.md).
- [ ] Confirm or correct every statement under [Statements from memory](#statements-from-memory).
- [ ] For every set, list the real fireplaces, ovens, forges, torch holders, candles and lanterns
      per room from the product pages and the building instructions, and correct the channel plan.
- [ ] Find out whether the forge belongs to the Saalbau or to the Rüstmeister.
- [ ] Find out which floors and modules separate, and where connectors are needed.
- [ ] Look up the permitted total GPIO current in the RP2350 datasheet and the power consumption in
      the Pico 2 datasheet, and correct the current budget in [hardware.md](hardware.md).
- [ ] Check the switching times of the ULN2803A, or drop the remark about it.
- [ ] Pick concrete articles for the bill of materials.

## Sources

### Read and measured

- This repository at version 2.0.0: a release build with `llvm-size -A`, the 26 host tests and a
  host simulation of the installed configuration over 200,000 ticks.
- The source of embassy-rp 0.10.0 from the Cargo registry: `pwm.rs`, `watchdog.rs`, `clocks.rs`,
  `gpio.rs`, `trng.rs`, `rom_data/rp235x.rs` and `pio_programs/pwm.rs`.
- The crates.io index, which lists `usbd-picotool-reset`, `rp-usb-reset` and `drooling` as
  implementations of the picotool reset interface.

### Sources not read

None of these pages was opened. The middle column is what this plan takes from the search result
snippet; the right column is what to check when the page is read.

#### The set

| Source | Statement taken from the snippet | To check |
| --- | --- | --- |
| [BlueBrixx, Burg Blaustein 109778](https://www.bluebrixx.com/de/prod/109778/burg-blaustein/) | The base castle has a walled courtyard, a kitchen in the basement, a hall with a fireplace on the first floor and a bedroom in the attic | Rooms and fireplaces, part count, dimensions, which floors come off |
| [BlueBrixx, Bergfried 109780](https://www.bluebrixx.com/en/prod/109780/blaustein-castle-keep-expansion/) | Tower with a secured entrance, a cellar, a living room with a fireplace and a bed, and a guard platform; the back comes out | Rooms, fireplace, whether the cellar offers room for electronics |
| [BlueBrixx, Saalbau 109080](https://www.bluebrixx.com/en/prod/109080/blaustein-castle-hall-expansion/) | Vaulted cellar, chambers with a canopy bed and a fireplace, a knights' hall with a shield gallery and a banquet table, a passage to the chapel. A snippet also names a forge with a workbench, a grinding wheel and a glowing fire pit | Rooms and fireplaces; whether the forge is part of this set |
| [BlueBrixx, Torhaus 109714](https://www.bluebrixx.com/en/prod/109714/blaustein-castle-gatehouse-expansion/) | Guard room with the winches for portcullis and drawbridge, tables and benches, an archive and an inner courtyard | Rooms, torch holders at the gate |
| [BlueBrixx, Schildwerk 109715](https://www.bluebrixx.com/en/prod/109715/blaustein-castle-ramp-shield-expansion/) | Nothing beyond the name | Interior and light points |
| [BlueBrixx, Blaustein category](https://www.bluebrixx.com/en/all-themes/middle-ages/blaustein-castle/) | The series also holds an Armourer (Rüstmeister), a Tithe Barn (Zehntscheune), stables and a figure pack | The complete list of sets and their item numbers |
| BlueBrixx product pages of Vorburg, Hurde, Münzturm, Festungsspitzen, Rüstmeister, Stallungen and Zehntscheune (links not known) | Vorburg: a half-timbered house and wooden palisades. Stallungen: half-timbered, with a paddock. The others: nothing | Interiors and light points of each |
| [mydealz, deal post](https://www.mydealz.de/deals/verfugbarkeitsdeal-bluebrixx-burg-blaustein-vorburg-2808492) | Item numbers: Bergfried 109780, Vorburg 109955, Saalbau 109080, Hurde 109711, Münzturm 109713, Torhaus 109714, Schildwerk 109715, Festungsspitzen 108932 | The item numbers; other pages name 104185, 108714 and 110451 for a Vorburg |
| [Brickfact, dimensions](https://brickfact.com/de-de/blog/klemmbausteine/burg-blaustein-masse) | Base castle: 5,562 parts, 40.6 cm × 39.8 cm × 29.5 cm. The full build measures about 100 cm × 72 cm × 73.5 cm. Every expansion requires the ones before it | Dimensions and part counts; another snippet gives 5,747 parts |
| [Steine-Kanal, overview](https://www.steine-kanal.de/burg-blaustein-von-bluebrixx-die-ultimative-uebersicht/) | The rooms of the base castle as above | Rooms of all sets |
| [Steine-Kanal, order](https://www.steine-kanal.de/bluebrixx-burg-blaustein-reihenfolge-alle-erweiterungen-richtig-aufgebaut/) | Order: base castle, Bergfried, Vorburg, Saalbau, Hurde, Münzturm, Torhaus, Schildwerk; optional sets furnish the interior | The order; which sets are optional |
| [CosmoShiva, order](https://cosmoshiva.de/burg-blaustein-alle-erweiterungen-die-richtige-reihenfolge-und-was-noch-geplant-ist/) | A different order for Hurde and Schildwerk | Which order is right |
| [Klemmbaustein.com, expansions and releases](https://klemmbaustein.com/bluebrixx-burg-blaustein/) | A list of all expansions | Completeness of the list of sets |
| [Das Klemmbrett, forum thread](https://klemmbrett.info/viewtopic.php?t=724) | Not used; listed as an overview of all expansions | Photos of interiors |
| [Verklemmt und Zugenoppt, review](https://verklemmtundzugenoppt.de/burg-blaustein/) | The modular build lets the three floors be taken off | Which floors separate and how |
| [PC Games Hardware, new edition](https://www.pcgameshardware.de/Klemmbausteine-Hardware-280122/News/Neuauflage-von-Burg-Blaustein-bietet-Drucke-und-Figuren-1484950/) | The new edition has a print on the mantelpiece | That the hall has a fireplace |

#### Pico 2 and RP2350

| Source | Statement taken from the snippet | To check |
| --- | --- | --- |
| [RP2350 datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) | Nothing; only the title came back | Permitted total GPIO current, pad drive strength at reset, state of the pads at reset, double buffering of the PWM `TOP` and `CC` registers, the wording and the affected revisions of erratum E9 |
| [Pico 2 datasheet](https://pip-assets.raspberrypi.com/categories/1005-raspberry-pi-pico-2/documents/RP-008299-DS-2-pico-2-datasheet.pdf) | Nothing; only the table of contents came back | VSYS range, the diode from VBUS to VSYS, load limit of the 3V3 pin, power consumption, board dimensions, the BOOTSEL test point |
| [Hardware design with RP2350](https://datasheets.raspberrypi.com/rp2350/hardware-design-with-rp2350.pdf) | Not used | Recommendations for powering the board |
| [SunFounder, Pico 2](https://docs.sunfounder.com/projects/newton-lab-kit/en/latest/raspberry_pi_pico2.html) | VSYS takes about 1.8 V to 5.5 V; the 3V3 pin should carry less than 300 mA | Both values against the Pico 2 datasheet |
| [Hackaday on erratum E9](https://hackaday.com/2024/09/20/raspberry-pi-rp2350-e9-erratum-redefined-as-input-mode-leakage-current) | E9 is a leakage current of input pads of up to 120 µA at 3.3 V; another snippet says revision A4 fixes it | What E9 means for buttons and sensors |
| ULN2803A datasheet of Texas Instruments (link not known) | Nothing | Turn-on and turn-off times, against PWM pulses of 240 ns |
| `usb_reset_interface.h` in the repository `raspberrypi/pico-sdk` and the README of `raspberrypi/picotool` (paths not known) | Nothing | The interface class, subclass and protocol, the request numbers and how picotool finds a device, against the `usb-update` feature |

#### Power lead and installation

| Source | Statement taken from the snippet | To check |
| --- | --- | --- |
| [mikrocontroller.net, USB-C without power](https://www.mikrocontroller.net/topic/578354) | A USB-C power supply gives no voltage over a C-to-C cable unless the sink has 5.1 kΩ on CC1 and CC2 | The statement, for variant B of the power supply |
| [Joy-IT K-1483, datasheet](https://joy-it.net/files/files/Produkte/K-1483/Datenblatt%20K-1483.pdf) | A Micro-USB to USB-C adapter specified for data and power up to 3 A | Whether it carries the CC resistors |
| [techtest.org, magnetic cables](https://techtest.org/die-guenstigen-magnetischen-usb-ladekabel-von-essager-im-test) | Cheap magnetic cables charge some devices slowly or not at all | Whether a magnetic adapter with data lines exists |
| [Light My Bricks, cable management](https://www.lightmybricks.com.au/blogs/learning-centre/expert-tips-managing-lego-lighting-cables-power-docks) | Thin cables fit between and under bricks, plates and tiles if they lie between the studs; pieces must not be forced together around a cable | Wire gauges that fit; photos of the technique |
| [Fechtner Modellbau, pre-wired LED](https://en.fechtner-modellbau.de/detail/index/sArticle/14130) | One offer of pre-wired 0402 LEDs names a 1.5 kΩ resistor on the anode for 7.2 V to 16 V; which offer of the result list said so is not known | That LEDs without a resistor are on sale, wire gauge and length |

### Statements from memory

These rest on no source at all and are checked like the snippets above:

- The RP2040 permits 50 mA in total over all GPIOs.
- The pads of the RP2350 drive 4 mA by default and are high impedance until the firmware
  configures them.
- A Raspberry Pi Pico 2 measures 51 mm × 21 mm and has eight ground pins.
- Typical forward voltages at about 1 mA: 2.0 V for amber and orange, 1.9 V for red, 2.8 V for
  warm white.
- WS2812 LEDs have 8 linear bits per colour, and a broken LED cuts off the rest of the chain.
- Common I2C PWM drivers such as the PCA9685 have 12 bits and about 1.5 kHz.
- Many power banks switch off at a load of a few tens of milliamperes.
- Round 1×1 bricks have a hollow stud, light bricks are slightly translucent, and superglue leaves
  a white haze on plastic.

Calculated, not looked up: 2.2 Ω per metre for copper wire of 0.1 mm, the resistor currents in
[hardware.md](hardware.md), and 240 ns for 36 counts at 150 MHz.
