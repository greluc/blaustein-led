# How brick models are lit

What the makers of light kits, model makers and builders do to light brick models, and what follows
for this project. Every fact comes from a page listed under
[Lighting practice](sources.md#lighting-practice) in the list of sources; judgements are marked as
such.

## Summary

- The makers of light kits for brick models work differently from this project. They run a 5 V rail
  through the model on two-wire cables and put the resistor and any effect into the light itself.
  Their fire is a fixed flicker pattern.
- Model makers and builders who want control do what the plan does: single small LEDs, each on a
  channel of its own.
- No kit offers what the plan offers: a flame per fire that the firmware shapes, a character per
  role, embers that follow their flame, 25 kHz PWM and wiring that a short cannot harm.
- The earlier plan overlooked one cost. A kit crosses a removable floor with two contacts; a
  central controller needs a contact for every channel that crosses the joint, plus ground. The
  design answers with one controller per zone and two contacts between zones.
- The other weak point was the supply of the warm white LEDs. It is solved by a buffer stage on
  5 V, see [hardware.md](hardware.md).

## Light sources

| Light source | What it is | Use here |
| --- | --- | --- |
| Pre-wired LED of size 0402 without resistor | A light of 1 mm × 0.5 mm on two thin wires | The source for every role: the only size that hides under a flame part or in a candle |
| Pre-wired LED of size 0603 or 0805 | 1.6 mm × 0.8 mm or 2 mm × 1.25 mm | Where there is room: hearths, embers, room light. Easier to handle |
| Kit lights for 5 V | An LED with its resistor on a cable with a plug. No maker states a forward voltage, a current or a resistor value, except one current of 15 mA | Not on this board: on a 3.3 V output their brightness is undefined, and they are made for 15 mA to 20 mA |
| LED with a built-in flicker circuit | A fixed pattern: one analysed type jumps 13 to 14 times a second and stays at full brightness half of the time | No. The firmware cannot shape it, and it is too restless for a candle and wrong for embers |
| Effect boards of the kit makers | A handful of fixed effects on a 5 V output | No |
| LED strips and filaments | 5 V, made for long bright lines; one filament draws about 150 mA | No. At most as hidden room light in a large hall |
| Light bricks | A self-contained light in a brick, not made to be wired | No |
| Fibre optics | One maker mentions them; no data found | Unverified. An idea for a candle tip fed from a hidden LED |
| Addressable LEDs with 8 bits | 256 levels at 0.4 kHz to 2 kHz; the smallest packages cannot be soldered by hand | No, see below |
| Addressable LEDs and driver chips with 16 bits | 65,536 levels on paper at 2 kHz to 4 kHz | Unproven, see below |

### Which source for which role

| Role | Source | Open question for the bench |
| --- | --- | --- |
| Hearth | An amber LED for the flame and a red one for the embers, both 0402 or 0603 | One kit maker lights a forge with white light under transparent orange tiles, where the colour comes from the part. Compare an amber LED with a warm white one behind the transparent orange parts of the set. A model railway product uses a red and a yellow LED that flash independently: a yellow LED as a second flame is worth a try |
| Torch | One amber 0402 under or behind the flame part | Whether the flame part of the set is transparent |
| Candle | One warm white 0402 at a very low current | A candle flame has about 1850 K; no pre-wired LED below 2600 K was found. Compare warm white with a yellow LED, and with warm white behind a transparent yellow part |
| Ember | Red, at the forge with orange | |
| Ambient | Warm white, hidden, lighting a wall or the ceiling | |

A warm white LED looks slightly warmer at a low current than at a high one. Setting the brightness
with a large resistor and a moderate duty cycle should therefore look warmer than a small resistor
with a tiny duty cycle. This supports the division of labour in the plan: the resistor sets the
brightness coarsely, the firmware finely.

## What the kit makers use

| Maker | Cables and plugs | Supply | Effects |
| --- | --- | --- | --- |
| Light My Bricks | Flat cables with reversible plugs; distribution docks of 8 mm × 6.4 mm in section. No thickness stated | 5 V, up to 2.4 A | Flicker built into the lights; an effect module |
| Lightailing and Briksmax | Reversible plugs; boards with 4 to 12 ports | 5 V from USB | None for fire found |
| Game of Bricks | Cables with two or four wires; boards with 2 to 12 ports | USB or a battery box | A board with four effects |
| BrickBling | Boards with plugs of 0.8 mm pitch | 5 V, less than 1 A | Lights that flash by themselves |
| Brickstuff | Stranded wire of 32 gauge, about 0.4 mm per conductor; small plugs; magnetic connectors with two contacts | 5 V | A flicker chip in the light |
| Evan Designs | Very thin enamelled wire, bare ends | 3 V or 5 V to 12 V, depending on the built-in resistor | A flicker circuit on the wire |
| Fechtner Modellbau | Stranded micro wire of 180 mm to 220 mm | 7.2 V to 16 V through the resistor in the lead | None |

- A third party identifies the plugs of most kits as an insulation-displacement family of 0.8 mm
  pitch. It is a wire-to-board family, so every joint needs a small board, which is why the kits
  are built around distribution boards. No kit maker publishes a specification of its plugs.
- An open hardware project, BrickPico, does with kit lights what this project does with single
  LEDs: a Pico dims them by PWM, on eight outputs fused at 200 mA each.

## Components of Light My Bricks

The range of this maker was examined part by part in its shop for the European Union, as building
blocks for this design and not as a kit. The shop lists 249 components, ships from the Netherlands
and charges 9.99 € for an order below 80 €.

### The contact plates for removable floors

The part the maker used to call a wireless connector is sold as "Flux Touch Plate Pair". It is not
wireless: two spring pins of one half touch two pads of the other.

| Property | Finding |
| --- | --- |
| Halves | Two black parts with a footprint of 1×2 studs, about one plate high by the drawings of the maker's guides. No dimension is printed anywhere |
| Contacts | Two per pair, so one pair carries two conductors. A joint with three circuits needs three pairs by the maker's own advice |
| Mounting | Clamped on studs, nothing glued. The lower half replaces a tile on top of a wall, the upper half is let into the plates under the floor above |
| Orientation | The halves must touch and must face the same way; turned round, plus meets minus |
| Leads | A flat cable of 5 cm, 15 cm or 30 cm that the maker gives as 0.2 mm thick, ending in a plug of the maker's own shape |
| Ratings | None for the part; only the line for the whole system, 5 V and 2.4 A |
| Price | 10.99 €, 11.99 € and 12.99 € a pair |

The plug is the obstacle. The maker sells no cable with open ends and no adapter to a pin header,
and nobody documents soldering to its flat cable. The one bridge is the maker's "Converter Dock"
to the small two-wire plug of its older system, at 4.50 € each. A joint for one channel and ground
then costs about 20 €, and one for three channels and ground about 40 €.

Contact plates with ordinary twisted wires, which can be cut and soldered to any wiring:

| Maker | Part | Price | Notes |
| --- | --- | --- | --- |
| Light My Bricks, older system | A tile with two pads and a plate with two spring contacts, 15 cm of twisted pair | 12.95 € for two pairs at a Dutch reseller | The last items in stock; gone from the maker's own shops |
| Game of Bricks | A board with two pads and a plate-shaped shell with two spring contacts | 24.99 € for a pack of three | One buyer reports adhesive strips on the back of the pad half |
| BrickBling | A part with two pads and a shell of 1×2 studs | 5.95 € | Shipped by air into the European Union |
| Lightailing | A board with two pads and a 1×2 plate | 13.99 USD | Long delivery to Europe, duties possible |
| Brickstuff | A round magnetic pair with solder tabs | 7.99 USD | From the United States; needs a housing |

None of these states a rating, and none is sold on Amazon.de.

### The rest of the range

| Family | What it is | Use here |
| --- | --- | --- |
| Flat cables | Two conductors, sold only with the maker's plugs at both ends, in fixed lengths | No. A trunk for three channels would need two cables and an adapter at each end |
| Docks | Distribution parts whose ports take only the maker's plugs; what they contain is not stated | No |
| Single lights | Three sizes in warm white, cool white and some colours; orange only in the two larger sizes. No forward voltage, current, resistor or colour temperature is stated. The maker's figure of 400 LEDs per wall adapter brackets one LED at 6 mA to 13.5 mA | Only after a test. The resistor in the light cannot be changed, so each light would be as bright as its maker made it |
| Flicker lights and effect modules | A fixed pattern in the light | No |
| Power parts, dimmer, remote control | Made for the maker's 5 V rail | No |
| Mounting aids | Clips that hold a part or spare cable on studs without glue | Optional |

### What it means for this design

- **The mounting principle is worth adopting whatever the connector:** a contact let into the tile
  course below and into the plate layer above, with one obvious orientation.
- **A two-contact plate fits a joint that carries two conductors.** In this design that is a 5 V
  branch between two zones, or a single channel such as the light of the bedroom. The joint under
  the hall of the base castle carries four.
- **Untested at these currents.** The makers design for kits that draw tens of milliamperes. Whether
  such a contact is reliable at 0.3 mA, and whether a floor that lies loose on tiles presses the
  spring pins down, has to be tried.

## Techniques

### Cables between bricks

- All makers lay their cables between the studs and put bricks, plates and tiles on top. One maker
  warns: "Do NOT forcefully join LEGO together around cables".
- No source gives a measured limit for the thickness of a wire that still lets a brick close.
  Reported to work: enamelled wire of 0.1 mm in a documented build with photos, enamelled wire of
  0.15 mm, stranded wire of about 0.4 mm and wire-wrap wire of 30 gauge by their makers.
- Neighbouring bricks leave a gap of 0.2 mm. A wire that passes under the wall of a brick lifts
  that wall by its own thickness. Enamelled wire of 0.1 mm to 0.15 mm is therefore the only class
  that can pass under a wall without a visible gap; everything thicker needs a path between studs
  or through an opening. This is a deduction from the geometry and is checked on spare bricks.
- The documented build with enamelled wire rebuilt the model from the ground up and laid the wires
  while building, because they hide better that way. The kits are installed after the build.

### What fails

- Plugs that are not fully seated or were pulled out while building on.
- Pinched cables.
- Enamelled wire copes with bending and pinching but not with a pull: a tug breaks the wire or its
  solder joint. Pre-wired LEDs have no strain relief at the LED.
- A short between two channel wires harms nothing but makes each channel light the other's LED.

### Hiding, diffusing and blocking light

- LEDs go under transparent parts: under transparent orange tiles for a forge, under a transparent
  round plate for a lamp, above a ceiling as soft room light.
- Builders often use white LEDs under coloured transparent bricks instead of coloured LEDs.
- Cotton wool or an opaque cylinder over the LED softens a camp fire.
- No source on stray light through brick walls could be read. Model railway practice is an opaque
  layer on the inside and a seal at the joints. Paint is not reversible; the black plate, the
  aluminium tape and the removable putty of the plan are the reversible equivalents. Whether they
  are enough on the light bricks of this set is unverified.
- Less light bleeds when there is less light: the plan runs its LEDs at about a tenth of the
  current of a kit light.

### Fire

- The kit maker with a medieval set uses warm white flicker lights for fireplaces and candles and a
  pulsing white light under orange tiles for the forge.
- A model railway maker uses a red and a yellow LED per fire that flash independently.
- A forum thread on a camp fire LED found random steps every 100 ms too abrupt and recommends
  gliding to each new level, keeping the level in the upper half so that the flame never goes out,
  two independent channels and a mix of colours, because a single red LED looks too red.
- A real candle is made not to flicker; flames flicker the slower the larger they are.

The firmware already does what these sources recommend: it glides between levels, never goes dark
and gives every channel its own motion. Its presets match the behaviour of real flames as starting
points.

## The three architectures

| | Direct drive from the 3.3 V pins, the earlier plan | 5 V rail with low-side switches | Addressable LEDs |
| --- | --- | --- | --- |
| Resolution | About 12.5 bits at 25 kHz | The same, limited by the switch | 8 bits; 16 bits claimed for two newer types |
| PWM frequency | 25 kHz | 25 kHz if the switch follows | 0.4 kHz to 4 kHz |
| Headroom for warm white | About 0.5 V | About 2.2 V, less the drop of the switch | Constant current |
| Wires into the castle | One per channel and ground | One per channel and 5 V | Three in total |
| Contacts at a joint | The channels beyond it and ground | The channels beyond it and 5 V | Three |
| A short in the model | Harmless, the resistor on the board limits it | The full supply current | The full supply current |
| A failed LED | That LED | That LED | Everything behind it, on the common types |
| Smallest light | 1 mm × 0.5 mm | 1 mm × 0.5 mm | 1.3 mm × 1.3 mm, not solderable by hand |

### Verdict

This is a judgement, not a finding.

- **Addressable LEDs with 8 bits are out, and the plan was right about them.** A common type
  drives 12 mA at its highest code. A flame of 3 mA is code 64 of 255, a torch of 2 mA code 43 and
  a candle of 0.3 mA code 6, so a candle would live in six steps. The plan's PWM has 6000 steps
  for the same span. Builders report exactly this: about twenty usable steps and a brightness that falls off a
  cliff at the bottom. The claim of the plan that these LEDs are linear holds for the clones only;
  the claim that a broken LED cuts off the chain holds for the common types and not for the ones
  with a second data line.
- **Driver chips with 16 bits are an interesting option that nobody has proven.** One such chip
  would drive three ordinary LEDs from 0.57 mA full scale, and only three wires would cross every
  joint. Against it stand a 5 V wire in the castle, 2 kHz instead of 25 kHz, no report of how it
  behaves at the lowest levels, a manufacturer whose product page and datasheet disagree, unknown
  availability and new firmware. It is not adopted. It is the option to examine if the number of
  contacts at the joints ever becomes the limit.
- **A Darlington array as a low-side switch is out.** It switches off about 110 ns later than it
  switches on, which stretches the shortest pulses by half, and it drops about 0.9 V.
- **The direct drive holds for amber and red and is second best for warm white.** Warm white LEDs
  need typically 2.7 V and at most 3.3 V at 5 mA, and they are about five times as bright as orange
  ones at the same current. A candle therefore needs a fraction of a milliampere from a headroom of
  about half a volt, where the spread of the forward voltage and the ripple of the regulator both
  show.
- **The remedy is a buffer, not a rail.** A logic buffer on 5 V between the pins and the resistors
  gives every LED 5 V of drive and keeps what the plan is built on: one wire per channel and
  ground, a protective resistor on the board, only current-limited lines in the castle, LEDs dark
  at power-on. It is described in [hardware.md](hardware.md).
- **The external PWM chip stays out.** The common I2C chip has 4096 steps at no more than 1526 Hz,
  all outputs at one frequency. A constant-current chip with 16 bits exists; it would add a rail
  and a second control path for no gain at these currents.

## Controllers and where they sit

The verdict above concerns how an LED is driven. A second question is how many controllers there
are and where. Six arrangements were compared on what the owner asked for: few contacts at each
joint, growth in stages, nothing bulky to hide, smooth dimming at the lowest brightness, a safe
reaction to a short circuit, and parts on sale in Germany. Reusing the existing firmware was
explicitly no criterion.

| Arrangement | Contacts at a joint | Dimming | Verdict |
| --- | --- | --- | --- |
| One central controller with buffers | The channels beyond the joint and ground, up to seven | 6000 counts at 25 kHz | Electrically the safest and the easiest to update, but it needs a trunk from every set and the only bulky board. Not chosen |
| A central controller of another family | The same | 6000 to 7200 counts on boards with 32 or more channels; an ESP32 about 2000 | Changes nothing the owner asked for. Not chosen |
| A small controller in every module, on a 5 V bus | Two | 5000 to 6000 counts | The best fit for joints and stages. Its costs are 5 V in the model and an update at every controller |
| A central controller with driver chips in the modules | Four | 4096 steps at 1.5 kHz on the common chip; 65,536 on a finer one whose board no shop of the owner sells | Not chosen |
| Addressable driver chips on a chain, as model railway projects use them | Three | 256 steps; a candle is code 5 or 6 | Fails at the lowest brightness. Not chosen |
| Decoders of model railways | Two | Not stated; fixed effects | Up to 22 V in the model and the highest price. Not chosen |

The design that was chosen sits between the first and the third: **one controller per zone of
neighbouring sets**, as few as possible. Inside a zone it is the central design in small, with
buffers, a protective resistor per channel and only limited lines beyond the board. Between zones
it is the distributed one, with two contacts for 5 V and ground behind a fuse. Three controllers
carry the castle.

- **The controller is a Raspberry Pi Pico 2 W.** It has the hardware PWM of the Pico 2 and a radio
  for updates. A board with the RP2350B and 24 channels was compared with it zone by zone: it is
  76 mm long instead of 51 mm, costs about three times as much, and no zone needs more than 16
  channels.
- **The smallest boards,** 21 mm × 18 mm with 11 channels, were the alternative for a controller in
  every module. With one controller per zone their size is not needed, and they have no radio.
- **Ready firmware for wireless controllers** updates over the air without any work, but dims in
  about 2000 steps on six channels and offers general flicker effects only.
- **No ready way exists to update several controllers over one shared wire.** The paths are a USB
  socket per controller, the radio, or an update task written for the purpose.

## Open points

- The forward voltage, current and resistor of any kit light.
- A measured limit for the wire thickness under a brick.
- Stray light through the bricks of this set.
- Whether a pre-wired LED warmer than 2600 K exists.
- How the 16-bit driver chips behave at their lowest levels.
