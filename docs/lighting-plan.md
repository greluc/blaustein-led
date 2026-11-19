# Lighting plan

The goal is to make the BlueBrixx model Burg Blaustein look alive: flames at the fireplaces, torches
inside and outside, candles and a faint light in the rooms. The controllers hide inside the castle,
one power lead enters it, and the installation is switched by cutting its power. The lighting grows
with the model, one set after the other.

This document holds the decisions, the analysis that led to the plan, what the research of
2026-11-19 changed, the lighting concept, the channel plan, the stages and the steps to build it.
Its companions:

| Document | Content |
| --- | --- |
| [model.md](model.md) | The sets, their light points, joints and cavities |
| [hardware.md](hardware.md) | Controllers, driver stage, power, connectors, work on the bricks, installation |
| [bill-of-materials.md](bill-of-materials.md) | The parts with articles and prices, and the orders by stage |
| [lighting-practice.md](lighting-practice.md) | How brick models are lit, and the architectures that were compared |
| [printed-parts.md](printed-parts.md) | Printed parts and their test plan |
| [sources.md](sources.md) | Every source, and what stays unverified |

## Decisions

Taken by the owner on 2026-11-19.

| Question | Decision |
| --- | --- |
| Scope | The full build: base castle and all expansions up to the Schildwerk, finally on the display base. The side buildings Rüstmeister, Stallungen and Zehntscheune are an optional last step |
| Order | In stages. First only the base castle with the Bergfried is lit, which is what stands today; then each expansion is added one by one as it is built, see [Stages](#stages) |
| Kinds of light | Fire, candles and a faint light in rooms without a fire. Torches are added generously at gates, platforms and wall walks, although no set has a torch holder |
| Architecture | Controllers inside the castle, as many as necessary and as few as possible; not one per expansion |
| Controller | The Raspberry Pi Pico 2 W, whose radio serves for updates and settings. The Pico 2 at hand is the test device |
| Power | When all controllers are in place, one cable leaves the castle |
| Driver stage | A buffer on 5 V for every channel; one wire per channel into a room, the resistor of each LED in the room |
| Intervention | Bricks may be drilled, cut and glued where it serves the lighting, provided a replacement brick was ordered first. A part is printed only where a print is really worth it and the owner has approved that part. Approved are the carrier of a board, the mount of the socket of the power lead, the holder of an LED and the insert that blocks stray light. Nothing printed is visible |
| Firmware | Rust is preferred. Reusing the existing firmware is no argument for or against a design; a new one may be written |
| Skills, shops, budget | Through-hole soldering, no surface-mount work. Reichelt, BerryBase, Conrad, Völkner and Amazon; no orders shipped directly from China. No fixed budget |
| Printers | A Prusa CORE One L and an Elegoo Saturn 4 Ultra 16K |

## Analysis of version 2.0.0

Version 2.0.0 was clean, tested and tiny. Its weaknesses were the look of the flame, the single
flame type and the missing documentation of the build. The figures come from a release build and a
host simulation of the installed configuration over 200,000 ticks (50 minutes).

| Topic | Finding | Assessment |
| --- | --- | --- |
| Memory | 5.5 KB of flash and 880 bytes of static RAM with 16 channels | Nothing to do |
| CPU load | About 0.1 %, the core sleeps otherwise. An estimate from the code, not measured | Nothing to do |
| Tests | 26 host tests passed, the firmware built without warnings | Good |
| Cutting the power as a switch | The firmware never writes to flash and keeps no state. Watchdog of 1 s, a panic resets | Good, a hard power-off is uncritical |
| Lockstep | The breathing envelope was identical and in phase on all channels. Correlation between any two channels 0.65. The pattern repeated after exactly 5 s | The largest defect: the whole castle breathed together |
| Parameters | `max_intensity: 128` bounded the envelope only; the output ran from 18 to 250 | Misleading, brightness hard to set |
| Flame types | Every channel used the same profile | Hearth, torch, candle, ember and room light need characters of their own |
| Resolution in the dark | The intensity had 8 bits before the gamma curve; from step 20 to 21 the brightness jumped by 11 % | Visibly steppy for embers and room light |
| PWM phases | The eight slices started one right after the other, so their on-phases overlapped almost completely | Needless current peaks on long thin wires |
| Hardware documentation | Channel plan, schematic, bill of materials and location were missing | The real build could not be assessed |
| Maintenance | Flashing needed the BOOTSEL button | Impractical once the controller is built in |

Stage 1 of the rework removed the lockstep, bounded the brightness by `peak`, added presets for five
roles and raised the intensity to 16 bits. More work on size or speed brings nothing measurable.

## What the research changed

The first version of this plan was written without access to any web page. On 2026-11-19 every
source was opened, the building instructions of all sets were read, alternatives were researched,
and the plan was reviewed against them. This is the result, decision by decision.

| Earlier plan | Verdict | Why |
| --- | --- | --- |
| One controller with all channels in the rock of the base castle | **Changed** to one controller per zone of neighbouring sets | The rock has one chamber of useful height, and the expansions move the base castle into the middle of the model. A central board would have needed a trunk from every set and up to seven contacts at one joint. Controllers in the zones need two contacts between zones, hide in the model from the first stage, and let each stage be built and tested on its own |
| A Pico 2 with 16 channels, ten more through PIO if needed | **Changed** to several Pico 2 W | Three of them give 48 channels at the full resolution, so every fire and nearly every torch moves on its own. Their radio updates the controllers the power lead does not reach |
| A controller of another family | **Examined and dropped** | No board dims finer at 25 kHz: the candidates reach 6000 to 7200 counts, an ESP32 about 2000. Another family changes neither the contacts nor the space the design needs |
| LEDs hang directly on the 3.3 V pins | **Changed** to a buffer on 5 V | Warm white needs typically 2.7 V and at most 3.3 V at 5 mA and is five times as bright as orange, so a candle would take a fraction of a milliampere from half a volt of headroom. For amber and red the direct drive would have held |
| One resistor per LED on the control board, one wire per LED | **Changed**: one protective resistor per channel on the board, the resistor of each LED in its room | A wire per LED would have needed seven contacts at the joint under the hall of the base castle. Light kits cross such a joint with two |
| The 3.3 V rail is regulated, so ripple does not show | **Wrong** | The regulator of the Pico 2 is a switching regulator that works in bursts at light load. The LEDs now run from the 5 V of the supply |
| 40 mA in total, because the chip allows 50 mA | **Corrected** | 50 mA is the limit of the RP2040. The RP2350 allows 100 mA. With buffers the limit no longer matters |
| The Pico 2 has eight ground pins | **Corrected** | Seven, and a separate analogue ground |
| Nothing is drilled, cut or glued | **Changed by the owner** | Bricks may be worked on once replacement bricks are at hand. A drilled seat for each LED and drilled passages for the wires remove the largest open risk of the plan, the wire clamped under a wall |
| Magnetic Micro-USB coupling | **Kept**, as an add-on to a flush socket | One maker specifies a magnetic cable for data. No independent test exists, so the tip sits in a normal socket that also takes an ordinary cable |
| A Micro-USB to USB-C adapter of Joy-IT for a USB-C socket in the rock | **Dropped** | It has a USB-C plug and a Micro-USB socket, the wrong direction; the controller has Micro-USB, so no adapter is needed |
| The 5.1 kΩ rule of USB-C, cited from a forum thread | **Confirmed**, from another source | The thread does not state it; an application note of Microchip does |
| Pin headers of 2.54 mm as connectors | **Changed** to keyed wire-to-wire pairs and magnetic connectors | Headers are not keyed and a plug goes on shifted |
| Trunk of 0.14 mm² wire, enamelled wire of 0.1 mm | **Changed** | The 0.14 mm² wire on sale is 1.3 mm thick. The LEDs come on enamelled wire of 0.15 mm; a stranded micro cable of 0.28 mm serves the runs in a zone |
| Cables rise in chimneys and in stacked round bricks | **Corrected** | The hoods of the fireplaces are solid and no chimney is known to be hollow. The model offers a well shaft, tower shafts, joist bays and a pillar instead |
| A forge in the Saalbau, a chapel, a half-timbered house with a fire | **Corrected** | The forge stands in the Rüstmeister. The Saalbau has a fireplace in the hall, a tiled stove in the chambers and an altar niche. The Vorburg has no fire at all |
| Gate torches on channels of their own | **Kept**, as an addition | No set has a torch holder. Torches are added with a few original parts |
| Five light roles | **Kept** | The presets match how real flames behave: a candle burns almost steadily, and a flame flickers the slower the larger it is. Room light shares a candle channel, because steady roles can share without showing it |
| Amber LEDs for flames, warm white of 2000 K to 2700 K for candles | **Kept as the first choice, with two bench comparisons** | A kit maker lights fire with white light behind transparent orange parts. No pre-wired LED below 2600 K was found, so a candle is compared in warm white, in yellow and behind a transparent yellow part |
| Addressable LEDs are not worth it | **Kept**, with better reasons | A candle of 0.3 mA would be code 6 of 255. Newer types with 16 bits exist as bare chips only |
| External PWM chips are not worth it | **Kept** | The common chip has 4096 steps at no more than 1526 Hz. A finer one with 16 bits exists, but its board is sold by none of the owner's shops |
| The USB update needs only the reset interface | **Completed**, and the firmware confirmed | On Windows the device also needs a Microsoft OS 2.0 descriptor, and picotool finds it without options only under the vendor and product ID of the SDK. The `usb-update` feature has both, and its constants match the Pico SDK and picotool, see [Firmware](#firmware) |
| Printed parts | **New, each with the owner's approval** | Printed parts that clamp are realistic after a calibration on the owner's printers and bricks, but they match neither the colour nor the clutch of the set, see [printed-parts.md](printed-parts.md). Four parts that do not exist as a brick are approved. A hole, a notch or a pocket goes into an original brick instead |
| A light kit might exist | **Settled** | None exists for any of these sets. The parts of one kit maker were examined one by one; its contact plates for removable floors are the one part worth a test |

## Lighting concept

Each role has its own LED colour, placement and motion. The times are starting values to be tuned
on the model, not measurements.

| Role | Where | LED | Placement | Motion |
| --- | --- | --- | --- | --- |
| Hearth | Kitchen, halls, living room of the keep | Amber for the flame, red for the embers | Under or behind the transparent red and orange parts of the fire | Irregular swell over 1 s to 3 s plus lively flicker; embers dragged along sluggishly |
| Torch | Gates, courtyard, platforms, wall walks | Amber | Under or behind an added flame part | Fast and restless, small swing, occasional gusts |
| Candle | Candelabra, wall candles, flame bowls, lanterns | Warm white | At the candle, very faint | Almost steady, a rare short twitch |
| Ember | Tiled stove, forge | Red, with amber | Deep in the firebox, indirect | Very slow glow |
| Ambient | Rooms without a light of their own | Warm white | Hidden behind a lintel or beam, lighting a wall or the ceiling | The glow of a candle that is not modelled; it shares a candle channel |

Every fireplace gets two LEDs on two channels. The bright flame and the red embers that follow it
sluggishly give the colour shift of a real fire.

## Channel plan

Each controller has 16 channels, numbered in the order in which they are installed. The plan is a
draft from the building instructions. It is confirmed on the model, stage by stage, while the sets
are built.

### Controller 1: base castle, Bergfried, Vorburg

| Channel | Stage | Place | Role | LEDs |
| --- | --- | --- | --- | --- |
| 0 | 1 | Base castle, kitchen: flame of the cooking fire | Hearth | 1 amber |
| 1 | 1 | Base castle, kitchen: embers | Ember, following channel 0 | 1 red |
| 2 | 1 | Base castle, hall: flame of the fireplace | Hearth | 1 amber |
| 3 | 1 | Base castle, hall: embers | Ember, following channel 2 | 1 red |
| 4 | 1 | Base castle, hall: candelabrum and flame bowl; bedroom | Candle | 4 warm white |
| 5 | 1 | Bergfried, living room: flame of the fire | Hearth | 1 amber |
| 6 | 1 | Bergfried, living room: embers | Ember, following channel 5 | 1 red |
| 7 | 1 | Bergfried, entrance level: flame bowl | Candle | 1 warm white |
| 8 | 1 | Base castle: courtyard | Torch | 1 amber |
| 9 | 1 | Bergfried: guard platform | Torch | 1 amber |
| 10 | 2 | Vorburg: watchtower candles, guard house, workshop | Candle | 5 warm white |
| 11 | 2 | Vorburg: stone gate, left | Torch | 1 amber |
| 12 | 2 | Vorburg: stone gate, right | Torch | 1 amber |
| 13 | 2 | Vorburg: wall walk of the palisade | Torch | 1 amber |
| 14, 15 | | Reserve | | |

### Controller 2: Saalbau, Hurde, Münzturm

| Channel | Stage | Place | Role | LEDs |
| --- | --- | --- | --- | --- |
| 0 | 3 | Saalbau, knights' hall: flame of the fireplace | Hearth | 1 amber |
| 1 | 3 | Saalbau, knights' hall: embers | Ember, following channel 0 | 1 red |
| 2 | 3 | Saalbau, chambers: tiled stove | Ember | 1 red, 1 amber |
| 3 | 3, 4 | Saalbau, knights' hall: candelabrum, wall candle, altar; from stage 4 also the galleries of the Hurde | Candle | 3 warm white, later 5 |
| 4 | 3 | Saalbau: candlestick of the chambers, flame bowl of the cellar | Candle | 2 warm white |
| 5 | 5 | Münzturm: candle of the chamber | Candle | 1 warm white |
| 6 | 5 | Münzturm: platform with the gun | Torch | 1 amber |
| 7 to 15 | | Reserve | | |

### Controller 3: Torhaus, Schildwerk

| Channel | Stage | Place | Role | LEDs |
| --- | --- | --- | --- | --- |
| 0 | 6 | Torhaus: desk, sleeping floor, guard room | Candle | 3 warm white |
| 1 | 6 | Torhaus: main gate, left | Torch | 1 amber |
| 2 | 6 | Torhaus: main gate, right | Torch | 1 amber |
| 3 | 6 | Torhaus: platform with the ballista | Torch | 1 amber |
| 4 | 7 | Schildwerk: passage | Torch | 1 amber |
| 5 | 7 | Schildwerk: terrace | Torch | 1 amber |
| 6 | 7 | Schildwerk: lantern of the terrace | Candle | 1 warm white |
| 7 to 15 | | Reserve | | |

- The castle up to the Schildwerk uses 43 LEDs on 28 channels: 16 amber, 5 red and 22 warm white.
  Stage 1 uses 13 LEDs on ten channels: 5 amber, 3 red and 5 warm white.
- Every fire and every torch moves on its own. Only candles and room light share channels, which
  does not show because they are nearly steady.
- A fourth controller serves the side buildings in the last stage: the forge of the Rüstmeister,
  the fire bowls of the Stallungen and the Hilfsschmiede, lanterns and candles.
- The reserve takes what the model still hides. Two points of the instructions may be fires: a
  block with a red part in the chamber of the Münzturm and coloured tiles in the yard of the
  Torhaus.
- The channels map to the pins as in the firmware: GPIO 16 to 22, then GPIO 7 to 15. The table of
  controller 1 is installed in `src/config.rs`; the tables of controllers 2 and 3 follow with the
  firmware support for several controllers.

## Stages

The lighting grows with the model. Nothing that was installed in an earlier stage is soldered
again.

| Stage | Set | What its instruction takes apart first | Lights it adds | Controller |
| --- | --- | --- | --- | --- |
| 1 | Base castle with Bergfried | Nothing; both stand | Kitchen fire, fireplace of the hall, candles of the hall and the bedroom, fire and flame bowl of the keep, two torches | 1 arrives, in the chamber under the terrace |
| 2 | Vorburg | The palisade, the log gatehouse, the keep as one unit and the upper floors. The rest of the base castle is set onto the new base in one piece | Candles of the watchtower and the guard house, two gate torches, a torch on the wall walk | 1 |
| 3 | Saalbau | Trees, rock pieces, a lean-to roof, a door and wall sections on the side of the keep | Fireplace of the knights' hall, tiled stove, candles | 2 arrives |
| 4 | Hurde | The roof and battlements of the keep, roof and wall-walk parts of the Saalbau. The castle is set onto the plinth | Light in the galleries; the torch of the keep moves to its new wooden top storey | 2, and 1 |
| 5 | Münzturm | 76 steps of removal, not yet read in detail | Candle of the chamber, torch on the platform | 2 |
| 6 | Torhaus | 52 steps of removal, among them the wooden ramp and a palisade section. The half-timbered storey of the guard house moves onto a new yard building and takes its light along | Candles of the gatehouse, two torches at the main gate, a torch on the platform | 3 arrives |
| 7 | Schildwerk | The whole wooden yard, lifted off as one assembly | Torches of the passage and the terrace, a lantern | 3 |
| 8 | Display base with the side buildings | Not published | Forge, fire bowls, lanterns and candles of the side buildings | 4 arrives |

What makes this work:

- **A zone is complete in itself.** Its controller, its wiring and its LEDs are built and tested on
  the bench and in the model before the next zone exists. A later zone adds a 5 V branch and
  nothing else.
- **As few controllers as possible.** A set joins the controller of its neighbour as long as that
  has free channels and the trunk does not cross a boundary the next instruction takes apart.
- **Every unit that an instruction lifts off has a plug at its foot:** the upper floors of the base
  castle, the keep, the wooden yard, the storey of the guard house. The next instruction can take
  the model apart without touching a solder joint.
- **One lead from the first day.** It enters at the first controller. Stage 3 and stage 6 each add
  a fused branch inside the model; the display base moves the socket to its edge.
- **Before a stage is wired, the removal steps of the next instruction are read.** A light goes
  only where the next instruction leaves it standing, or it gets a plug. The torch on the guard
  platform of the keep is the known case: the Hurde replaces the battlements under it. The gate
  torches of the Vorburg are the suspected one, because the Münzturm and the Torhaus rebuild that
  corner.

## Not worth doing

- **Shrinking flash and RAM further, or lowering the clock.** A fraction of a percent of the flash
  is in use, and the PWM resolution hangs on the system clock.
- **One central controller.** It needs the most contacts at every joint, a trunk from every set
  and the only bulky board of all designs.
- **A controller per expansion.** Three controllers carry the castle; more would add update work
  and 5 V branches for nothing.
- **Addressable LEDs with 8 bits, and the model railway systems built on them.** A candle of
  0.3 mA is code 6 of 255.
- **Remote driver chips on a bus.** The common one has 4096 steps at no more than 1526 Hz; the
  finer one is not on sale at the owner's shops, and four contacts would cross every joint.
- **Decoders of model railways.** Fixed effects, up to 22 V in the model and the highest price.
- **Kit lights and lights with a built-in flicker.** Their data are not published, and their fire
  is a fixed pattern.
- **Ready firmware for wireless controllers.** It updates over the air out of the box, but it dims
  in about 2000 steps on six channels and offers general flicker effects only.

The comparison behind these verdicts is in [lighting-practice.md](lighting-practice.md).

## Firmware

Nine changes answer the analysis. All of them are implemented; none has run on a board yet.

| No. | Change | Result |
| --- | --- | --- |
| 1 | Decouple the envelope | Three layers of smoothly interpolated value noise (swell, sway, flicker) and occasional gusts replace the triangular envelope, the jitter, the flares and the smoothing. Over 50 simulated minutes two hearth channels correlate by at most 0.04, and no role has a dominant period |
| 2 | Light roles | Presets for hearth, torch, candle, ember and ambient light, and a channel table in `src/config.rs` with one line per channel: a role and a brightness trim. The installed table is the plan of controller 1 |
| 3 | 16-bit brightness | The simulation returns a 16-bit intensity and the gamma curve works on it, so dim roles fade without steps. `peak` bounds the output of a role by construction |
| 4 | Coupled channels | A follower drives the ember LED of a fireplace from its flame, heavily smoothed |
| 5 | Installation aids | The `identify` feature builds a firmware in which every LED blinks its channel number. After power-on the channels ignite 400 ms apart and fade in over 2 s |
| 6 | PWM phases | The slices start in one register write with their counters spread evenly over the period |
| 7 | Random start | 32 bits of the ring oscillator are mixed into the seeds at boot. The hardware random generator is not used: its driver needs an interrupt binding and can wait without bound |
| 8 | Update through USB | The `usb-update` feature adds a USB device with the reset interface of picotool, so picotool reboots a buried controller into its bootloader. The default build contains no USB stack |
| 9 | Documentation | This plan, its companions and the README |

Release image, in bytes. Version 2.0.0 had sixteen channels; the other rows are the installed table
of controller 1 with fourteen:

| Build | `.text` | `.rodata` | `.bss` |
| --- | --- | --- | --- |
| Version 2.0.0 | 4616 | 508 | 880 |
| Default | 6236 | 356 | 1032 |
| `identify` | 4468 | 176 | 528 |
| `usb-update` | 15516 | 540 | 2424 |
| `identify` with `usb-update` | 13952 | 360 | 1920 |

Checked on 2026-11-19 on this state: 97 host tests pass, and formatting, Clippy in every feature
combination, `cargo deny` and the four release builds pass.

### The USB update against the Pico SDK and picotool

The feature was written from three crates on crates.io, without access to the Pico SDK or to
picotool. On 2026-11-19 it was compared with the sources of pico-sdk 2.3.1 and picotool 2.3.1,
listed under [Updating through USB](sources.md#updating-through-usb):

| What | Firmware | Source | Verdict |
| --- | --- | --- | --- |
| Interface class, subclass, protocol | `0xFF`, `0x00`, `0x01` | The same in the header of the SDK; picotool searches for exactly these | **Matches** |
| Request for the bootloader | `bRequest` `0x01`, a class request to the interface, accepted when `wIndex` is the interface number | The same in the SDK and in picotool | **Matches** |
| Request for a reboot into the flashed program | `0x02` is rejected | The SDK reboots through the watchdog | **Deliberately not offered.** A `picotool reboot -f` without `-u` asks for it and is refused; the README uses `-f -u` |
| `wValue` | Bits 0 and 1 go to the boot ROM as the interfaces to switch off; all other bits are ignored | The SDK passes `wValue & 0x3`; bit 8 says an activity LED is named, bit 7 that it is active low, bits 9 to 15 hold its pin | **Matches.** Ignoring the LED is a choice |
| Vendor and product ID | `2e8a:0009` | The IDs of a running RP2350 program in the SDK; picotool looks only at known IDs unless told otherwise | **Matches** |
| USB version | `bcdUSB` `0x0210`, the default of embassy-usb 0.6.0 | The SDK reports `0x0210` when it carries the Microsoft descriptor | **Matches** |
| Microsoft OS 2.0 descriptor | Windows 8.1 as the lowest version, the compatible ID `WINUSB` and the registry property `DeviceInterfaceGUIDs`, a list with one GUID, as features of the whole device | The SDK states the same Windows version and compatible ID, inside a function subset because its device has several interfaces, and registers `DeviceInterfaceGUID`, a single string. libusb 1.0.29, through which picotool reaches the device, reads `DeviceInterfaceGUIDs` first and accepts both forms. Microsoft's page on WinUSB devices confirms the compatible ID and the GUID for a device with a single interface, in the older descriptor format | **Differs in form, expected to work, not proven.** The specification of the descriptors is a download behind a licence agreement and was not read; Windows decides on the board |
| Commands in the README | `picotool reboot -f -u`, `picotool load -x -t elf`, `picotool load -f -x -t elf` | The README of picotool describes `-f`, `-u`, `-x` and `-t` this way | **Matches** |

One correction came out of it: on Linux a udev rule has to cover the product ID `000f` of the
bootloader as well, or `picotool load` needs `sudo` after the reboot. The README says so now.

The device reports Raspberry Pi's vendor ID because picotool looks for it; that suits a private
installation and is not a licence to distribute devices under that ID.

### What only a board can show

- The look of every role and of the ignition. The presets were tuned by statistics in a host
  simulation and are starting points.
- Whether the ring oscillator gives different bits at every power-on.
- On an oscilloscope, that the phases are staggered and stay so.
- The total current of the identify build, in which all LEDs are fully on at once at the start of
  every cycle. With the buffers it loads the 5 V supply, not the pins.
- For `usb-update`: that Windows, Linux and macOS enumerate the device, that Windows binds WinUSB
  to a device with a single interface, that picotool finds, reboots and flashes it, and that the
  reboot works with the watchdog running.
- For `usb-update` on a plain charger: some chargers short the data lines or hold them on a
  voltage divider. The USB driver is not expected to starve the light task then, but until that
  is tried on the supply that will be used, the default build is the safe one to install.

### What the installation still needs

The research and the decisions add requirements the firmware does not meet yet:

- **Several controllers.** The same firmware runs on every controller with a channel table of its
  own. How a controller knows which table is its own is to be designed; a build per controller is
  the simplest way, but an update over the radio is easier with one image for all.
- **The Pico 2 W.** It has the pins of the Pico 2, so the channels stay on GPIO 7 to 22. Its
  regulator is switched to steady operation through a pin of the radio chip, if the bench shows
  that it matters.
- **Update over the radio.** The driver `cyw43` of the Embassy project supports the radio chip. The
  bootloader library `embassy-boot-rp` describes itself as made for the RP2040; whether it serves
  the RP2350 is to be established first. The radio is on in a service window only.
- **Update over USB** stays the way for the first controller, which the lead reaches, and the
  fallback for the others.

## Implementation steps

Each step has a checkpoint. First the bench, then one zone, then one stage after the other, so
faults show before wires are built in.

1. **First order.** The parts of [bill-of-materials.md](bill-of-materials.md) for stage 1, and two
   or three bags of replacement bricks to compare with the set.
   - Checkpoint: the forward voltage of every LED is measured at 0.3 mA, 1 mA and 3 mA. The
     replacement bricks match the set in colour and clutch, or serve for hidden places and for
     practice only.
2. **Bench.** One buffer on a breadboard, one LED of every colour behind its resistor, driven by
   the Pico 2 at hand.
   - Checkpoint: the brightness fits in a darkened room and the resistor values are fixed. Amber
     is compared with warm white behind the transparent orange parts of the set, and a candle in
     warm white with one in yellow.
3. **Radio.** An update of a Pico 2 W over its radio.
   - Checkpoint: the update works twenty times in a row and survives a power cut in the middle. If
     it does not, the controllers are updated through their USB sockets, and each needs a place
     where its socket can be reached.
4. **Firmware.** Flash the installed table of controller 1 and tune the roles on the bench.
   - Checkpoint: the look of every role is approved on the bench.
5. **Board of zone 1.** Perfboard with the socketed controller, two buffers and the outlets of
   stage 1.
   - Checkpoint: every channel of the table is verified in the identify build. No LED lights while
     the firmware is not running, and none flashes at power-on. The board fits the chamber under
     the terrace with its plug.
6. **Power lead.** Supply, magnetic lead, socket.
   - Checkpoint: detach and attach twenty times, the firmware starts every time. Then a run of
     24 hours.
7. **Trial on spare bricks and printed parts.** Every operation on the bricks is tried on a spare
   brick first. The calibration print of [printed-parts.md](printed-parts.md) comes before the
   first approved printed part.
   - Checkpoint: a drilled spare still clamps between two originals. A printed part clamps
     slightly lighter than an original and sits flush; the mount of the socket stays in place
     while the lead is attached and detached twenty times.
8. **Stage 1 in the model.** Base castle and Bergfried.
   - Checkpoint: the floors and the keep come off, no brick stands proud, no wire is visible. The
     picture is tuned in the dark.
9. **Stages 2 to 7.** For each expansion: read its removal steps, order its parts, wire it while it
   is built, test it on its own, then join it.
   - Checkpoint per stage: everything of the earlier stages still works after the rebuild.
10. **Stage 8.** The socket of the lead moves to the edge of the display base; the side buildings
    follow if wanted.
11. **Wrap-up.** Circuit boards replace the perfboards; channel plan, photos and bill of materials
    are brought up to date in this repository, then a release.

## Open points

What no source could settle is listed with what was tried under
[Unverified](sources.md#unverified). The points that decide the next steps:

- The forward voltage of the LEDs at the planned currents, and their colour on the model.
- Whether an update over the radio works on the Pico 2 W, and what its radio draws.
- Whether the board of zone 1 fits the chamber under the terrace.
- Whether the replacement bricks match the set.
- Whether the approved printed parts clamp well enough on the bricks of this set.
- How a controller knows its channel table, once there are several.
- Whether data passes the magnetic coupling.
- The size and quality of the 1.25 mm connectors, which have no datasheet.
- Every dimension of the model that was read from a drawing.
- The look of the flame, which only the model can show.
