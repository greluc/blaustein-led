# Printed parts

Whether parts that clamp on the bricks of Burg Blaustein can be printed on the owner's two printers,
which printer suits which part, and how to find out before anything goes into the model. Nothing
was printed or measured for this document. Every statement rests on a page listed under
[Printed parts](sources.md#printed-parts) in the list of sources; where a step size or a criterion
is a proposal of this document, it says so.

## Verdict

Printed parts that clamp on the bricks are realistic, but not at the quality of moulded bricks, and
only after a calibration on the owner's own printers and bricks.

- The fit of two bricks lives on a few hundredths of a millimetre of interference: studs are drawn
  at 4.8 mm and reported at 4.87 mm to 4.90 mm on real bricks. Prusa states for its printers an
  accuracy of "at least 0.2 mm", several times coarser. A printed part therefore fits only when its
  dimensions were found by trial on one printer, nozzle and material, and it has to be found again
  when one of them changes.
- Every documented attempt confirms this. Brickset (2026) printed 2×4 bricks from precise models
  without tuning and found the clutch "nowhere near as good as that of real bricks". Prusa (2020)
  got a good fit only with its own redesigned models, best in ASA. Air and Wodehouse (2024) needed
  49 iterations before a printed 2×4 interlocked reliably.
- Nobody published a pull-force or cycle-life measurement of printed bricks.

The consequence is a design rule: a printed part is a service part. It sits captured between
original bricks, clamps slightly lighter than an original so that it cannot wear them, carries no
load through its clutch alone, and is reprinted when it loosens.

### Where a print is used

The owner's rule, see [hardware.md](hardware.md#working-on-the-bricks): a part is printed only
where it cannot be seen and a print is really worth it. Both have to hold.

- **Worth it: a part that does not exist as a brick.** The carrier for a controller board, the
  mount of the socket of the power lead, a holder for an LED and an insert that blocks stray
  light.
- **Worth it: a shape that cannot be made in an original without cutting away what clamps.** The
  brick with a pocket for a floor connector, and a brick with a duct for several wires.
- **Not worth it: a brick that only needs a hole** for a wire or an LED. The original is drilled,
  with a replacement brick at hand. That takes minutes, while a print needs a model, the
  calibration and a test first.
- **Never: a part that can be seen.** The owner's printers do not reach the colours of the set,
  see [Colour](#colour).

Plain standard bricks are not printed. The list of printed parts stays short on purpose: every
printed part that clamps brings the weaknesses below into the model.

### What speaks against a print

- **Colour and gloss.** They do not match the set, and layer lines show. This limits printing to
  hidden places.
- **Clutch.** It is weaker and less even than that of a moulded brick, and no source measured how
  it holds over time.
- **Calibration.** Every fit is found by trial for one printer, nozzle and material and again when
  one of them changes. The set mixes brick makers, so a fit found on one module may not hold on
  another.
- **Small shapes.** A 1×1 plate that still clamps is beyond the filament printer.
- **Structure.** A printed brick weakens a wall slightly, so it goes where originals interlock
  above and below it.
- **Unknowns.** Wear on the original studs, staining of light bricks and creep over months are
  documented nowhere; the test plan below answers them before a print is built in.
- **Effort.** No design file exists yet. A hole of 1 mm in an original is made in minutes; a
  printed brick needs a model, the calibration and a test first.

## The printers

| Printer | Facts from the manufacturer | Role |
| --- | --- | --- |
| Prusa CORE One L, filament. Prusa's product page now names it "CORE One L+" | Build volume 300 mm × 300 mm × 330 mm, layers of 0.05 mm to 0.30 mm, a 0.4 mm nozzle supplied, 0.25 mm selectable, a heated chamber. No accuracy figure beyond "CT-scan-verified" | Everything structural or larger than a few studs |
| Elegoo Saturn 4 Ultra 16K, resin | Pixels of 14 µm × 19 µm, build volume 211.68 mm × 118.37 mm × 220 mm, layers of 0.01 mm to 0.2 mm | Tiny parts where detail matters and clamping does not |

The resin printer resolves a stud with about 250 to 340 pixels, so resolution is not its limit. Its
limit is the shrinkage of the resin while it cures: a resin maker states 1.5 % to 2 % per axis, and
one documented measurement on another brand found 0.4 % to 1.5 %. On a 2×4 brick that is 0.13 mm to
0.48 mm, enough to turn a fit into no fit. The same user got a brick to fit by scaling it by 1.009.
A resin part is therefore judged only after its final wash and cure, and that routine stays fixed.

## The bricks

No official dimension or tolerance sheet of the bricks is public; every value is reverse-engineered
by third parties. The reference values for a first model:

| Feature | Value |
| --- | --- |
| Stud pitch | 8.0 mm |
| Brick height without stud, plate height | 9.6 mm, 3.2 mm |
| Outer size of a 2×4 brick | 31.8 mm × 15.8 mm, which leaves 0.1 mm of play per side |
| Stud diameter | 4.8 mm drawn, 4.87 mm to 4.90 mm reported on real bricks |
| Stud height | 1.6 mm to 1.8 mm, depending on the source |
| Outer wall | 1.2 mm with ribs on current bricks, 1.5 mm to 1.6 mm on older ones |
| Tube under a brick two studs wide | 6.4 mm to 6.5 mm outside, 4.8 mm to 4.9 mm inside |
| Pin under a brick one stud wide | about 3.1 mm to 3.2 mm |

These are values of LEGO bricks. No measurement of the bricks of this set was found, and the set
mixes makers: the earlier editions used bricks of GoBricks, and a report says the current edition
uses bricks of Xingbao. A fit calibrated on one module may not hold on another. The dimensions that
matter are therefore measured on spare bricks of every module before a part is designed.

## Materials

| Material | What is documented | Use |
| --- | --- | --- |
| PLA | Stiff and brittle, softens above 60 °C. Prusa warns that it can damage original bricks by abrasion | Trial prints only, never a part that stays in the model |
| PETG | Tough, does not shrink or warp. The maker of one brick generator recommends it for bricks | First choice for hidden parts that clamp |
| ASA | Best results for bricks at Prusa, resistant to light and heat, warps and needs the enclosure with filtration | Second choice, first for the mount of the power lead |
| TPU | No documented brick | Strain relief and soft pads only |
| Standard resin | Cracked when pushed onto bricks in two forum reports | Diffusers, masks and holders that do not clamp |
| ABS-like and tough resin | Suggested by users for bricks, no result reported. Prusa reports high friction between resin parts and joints that wear loose | Small parts that clamp once, after they passed the test below |

Not covered by any source that could be opened: whether PETG or ASA wears the studs of the original
bricks over many cycles, whether a filament or a resin stains light bricks, how well a printed clamp
holds over months, and how thin black prints can be before light shows through. The test plan
answers these on spare bricks.

## Which printer for which part

| Part | Printer and material | Notes |
| --- | --- | --- |
| Brick or plate with a duct for several wires | Filament, PETG | Only where no face of it shows and a drilled hole does not do, such as a trunk that runs along a row of bricks inside a floor, in the rock or in the base. It is loaded in compression between originals. The duct runs in the roof or as a notch in a side wall, so the tubes that clamp stay whole. A single wire gets a hole in an original instead |
| Wall feed-through | Not printed: an original Technic brick with a hole, or a drilled original brick | A wall shows on both sides, and a hole is quicker than a print |
| Holder for a pre-wired LED, with a diffuser | Resin; the diffuser as a separate part | Features of a few tenths of a millimetre. It needs no real clutch and sits over one stud with light friction. A separate diffuser can be replaced if it yellows |
| Connector brick between modules and floors | Filament, PETG or ASA | Only where it is hidden; where it shows, the pocket is cut into an original. It has to hold on several studs. The connector in it must separate with less force than the brick clamps, or lifting a floor pulls the brick off |
| Mount that sets the socket of the power lead flush into the edge of the base | Filament, ASA in black | The part that takes a push and a pull at every plugging. It anchors on a wide field of studs behind original bricks and holds the flange of the socket, whose screw holes are 30 mm apart. Original bricks frame the socket, so that only its face shows |
| Carrier for the board of a zone | Filament, PETG | Up to 12 × 10 studs: a perfboard of 90 mm × 70 mm measures 11.25 × 8.75 studs. The end of the controller with the antenna stays free of metal |
| Strain relief for the lead | Filament, PETG, optionally with a TPU pad | Never resin |
| Inserts that block stray light | Filament in black; resin for very thin or shaped masks | Check against the real LED whether the thin wall is opaque. Aluminium tape remains the fallback |

A 1×1 plate with a duct that still clamps is beyond the filament printer; there the wire runs
between the studs or through a drilled original, as described in [hardware.md](hardware.md). Every
printed part that replaces an original weakens the wall slightly, so it goes where originals
interlock above and below it.

## Design sources

The parts are modelled in OpenSCAD: all of them are prisms with studs, tubes, pockets and slots, a
`.scad` file is short text with named parameters, and one tool renders it from the command line.

| Generator | Licence | Assessment |
| --- | --- | --- |
| [cfinke/LEGO.scad](https://github.com/cfinke/LEGO.scad) | MIT | The only brick library whose licence fits this repository. It hides the play of studs and walls as constants and offers a single rescaling factor |
| [pks5/machineblocks](https://github.com/pks5/machineblocks) | CC BY-NC-SA 4.0 | The best calibration workflow, with a printed tool and one row per dimension. Nothing derived from it may enter this repository; it serves as a private tool only |
| [cwesson/openBrick](https://github.com/cwesson/openBrick) | CC BY-NC-SA 4.0 | Not usable |
| [paulcobbaut/FreeCAD-Brick](https://github.com/paulcobbaut/FreeCAD-Brick) | GPL-3.0 | Not usable |
| CadQuery and build123d | Apache-2.0 | No brick library. The alternative if a housing ever needs STEP import or fillets |

The plan is a small module of this repository under `MIT OR Apache-2.0` that models only what these
parts need and exposes every fitting dimension as a parameter per printer and material, with
`LEGO.scad` as the reference. No design file is part of the repository yet: no tool that renders one
was at hand in the session that wrote this document, and the parameters come from the calibration
below.

Files are named after their function, carry no logo on the studs and copy no existing element. The
patent on the basic brick expired decades ago and its shape cannot be a trademark in the European
Union, but single elements can be protected as designs.

No printable brick with a cable duct, connector brick or contact brick between floors was found. The
two largest collections of printable models could not be read, so this is not proof that none
exists.

## Colour

No source documents the colours of this set, and the bricks of the two makers are not guaranteed to
match those of LEGO, for which the only matching guide found was written. Even a matched colour
differs in gloss and shows layer lines, and the owner judges the colours as hard to reach on the
printer. Printed parts therefore stay out of sight, without exception; their colour is free, and
black is chosen where a part must not pass light.

The mount of the power lead sits at the edge of the base, where the socket has to be reached. It is
recessed behind original bricks that frame the socket, so that only the face of the bought socket
shows. Whether the frame hides the mount completely is seen on the model.

## Test plan

Tests never touch a brick that is built in.

### Before printing

1. Find out which edition each set is, and set aside spare bricks of each: 2×4 and 1×4 bricks, a 2×4
   and a 1×2 plate and a large plate.
2. Measure on them with a calliper, better a micrometer: the diameter of several studs in two
   directions, the outer width and length, the wall, the outer diameter of a tube, the height of a
   brick, a plate and a stud. These values replace the reference values above.

### Calibration print

One small print per printer, material and nozzle, every feature marked with an embossed index. The
step sizes are a proposal.

| Row | Feature | Filament | Resin | Judged by |
| --- | --- | --- | --- | --- |
| A | Single studs on a strip | 4.70 mm to 4.95 mm in steps of 0.05 mm | The measured diameter −0.06 mm to +0.06 mm in steps of 0.02 mm | Pressing an original brick on each stud, the smallest first |
| B | Sockets one stud wide | Pin and wall in steps of 0.05 mm | In steps of 0.02 mm | Pressing the socket on original studs |
| C | Sockets two studs wide | Tube of 6.30 mm to 6.60 mm in steps of 0.05 mm | In steps of 0.02 mm to 0.03 mm | The same |
| D | Outer width of a 1×2 and a 2×2 | −0.1 mm, 0, +0.1 mm | Scale 1.000, 1.005, 1.010, 1.015 | It drops in between two originals on a plate without spreading them |
| E | Height of a brick and a plate | | | Three plates equal one brick next to originals |
| F | First layer | Elephant foot compensation of 0.2 mm and 0.4 mm | The side that faced the build plate | The socket starts cleanly |
| G | Slots and holes for the wires | 0.4 mm to 1.2 mm | 0.2 mm to 1.0 mm | The wire lies in without being pinched and the brick seats fully |

Filament parts print with the studs up and the open side on the bed, without supports. An original
brick is never forced onto the print.

### Judging the fit

- **Feel.** The force to join and to separate is the same as, or slightly below, that of two
  originals of the same set.
- **Number.** The pull-off force of a 2×2 or 2×4 joint with a luggage scale or hanging weights,
  original on original as the reference.
- **Cycles.** Thirty times on and off the same spare brick, then the force again and the original
  under a magnifier for scratches, whitening, shavings and colour.
- **Dwell.** A joint left together for 14 days, one sample in a warm place, then the force again. A
  clear drop means creep: the next tighter step or another material.
- **Contact.** A printed part pressed against a light spare brick for the same 14 days, then a look
  for stains, tackiness and dulling. Resin parts go into the model only fully washed and cured.
- **Editions.** The same check on a spare brick of every module of another edition.

Printer, nozzle, material with batch and colour, slicer profile, offsets and results are recorded
next to the design sources. Any change of them repeats the print.

### Order of development

1. The calibration print and the parameter file, PETG first, then resin.
2. A plain 1×2 plate and a 2×2 brick with the chosen offsets, as proof that whole parts behave like
   the calibration print.
3. The plate and the brick with a duct, and a check that a wall built with them closes without gaps.
4. The inserts against stray light and the LED holder with its diffuser, tested on the bench with a
   real LED.
5. The carrier for the board of a zone and the strain relief.
6. The mount of the power lead.
7. The connector bricks between floors and modules, the part with the highest risk.

## Open points

- Which of the places that need a pocket or a duct for several wires are hidden. Only there a
  print is considered, and the model shows it.
- The dimensions, the clutch and the colours of the bricks of this set.
- The nozzles at hand and whether the filtration for ASA is fitted.
- Shrinkage and compensation values of the resins on this printer.
- Wear, staining and creep of every printed material against the bricks.
- The size of the finished board of a zone, for which no layout has been drawn.
