# Bill of materials

The parts for the design in [hardware.md](hardware.md), as on sale in Germany on 2026-11-19. Every
price is the one the shop showed on that day, with tax and without shipping, unless a row says
otherwise. The pages are listed in [sources.md](sources.md).

- **Page** in the last column means the product page was opened. **List** means the article, its
  price and its stock were read from the shop's own result list and the product page was not
  opened: check it before ordering.
- The quantities follow the channel plan in [lighting-plan.md](lighting-plan.md): three controllers
  and 43 LEDs for the castle up to the Schildwerk. The side buildings with a fourth controller are
  a later step and not included.
- The installation is built in stages, and so are the orders: the last section splits the parts
  into the first order for stage 1, the base castle with the Bergfried, and what follows with each
  later stage.
- Nothing has been bought or tried. The first order doubles as the samples: its parts are tested on
  the bench before anything is built in or ordered for a later stage.

## Total

| Group | Sum |
| --- | --- |
| Controllers and their boards | 68.27 € |
| LEDs | 77.94 € |
| Wire | 36.85 € |
| Connectors | 36.26 € |
| Power lead and supply | 25.78 € |
| Small parts | 12.42 € |
| **Electronics** | **257.52 €** |
| Replacement bricks, an estimate | about 127 € |

Not included: shipping, the optional parts marked below, a pin vice with drills, tools, and the
circuit boards that may later replace the perfboards, which start at 14.20 € for three boards at
Aisler.

## Controllers and their boards

For three controllers.

| Part | Retailer and article | Price | Quantity | Sum | Read |
| --- | --- | --- | --- | --- | --- |
| Controller, Raspberry Pi Pico 2 W with headers | [Reichelt RASP PI PICO 2WH](https://www.reichelt.de/de/de/shop/produkt/raspberry_pi_pico_2_wh_rp235x_cortex-m33_wifi_microusb-398576) | 8.50 € | 3 | 25.50 € | List; in stock |
| Socket strip 1×20, divisible | [Reichelt FIS BL1 20 Z](https://www.reichelt.de/de/de/shop/produkt/buchsenleiste_2_54mm_1x20_trennbar_verzinnt-283794) | 1.55 € | 6 | 9.30 € | Page |
| Perfboard 90 mm × 70 mm, double-sided | [Reichelt LR-DS-79](https://www.reichelt.de/de/de/shop/produkt/lochrasterplatine_doppelseitig_90_x_70_mm-319109) | 2.60 € | 3 | 7.80 € | List |
| Perfboard 80 mm × 20 mm for the distribution strips | [Reichelt LR-DS-28](https://www.reichelt.de/de/de/shop/produkt/lochrasterplatine_doppelseitig_80_x_20_mm-319114) | 1.15 € | 4 | 4.60 € | List |
| Buffer 74HCT541, DIP-20, Texas Instruments SN74HCT541N | [Reichelt 74HCT 541](https://www.reichelt.de/de/de/shop/produkt/buffer_8-kanal_ttl-kompatibel_dip-20-3420) | 0.93 € | 7 | 6.51 € | Page |
| Socket for the buffer, 20 pins | [Reichelt GS 20](https://www.reichelt.de/de/de/shop/produkt/ic-sockel_20-polig_doppelter_federkontakt-8212) | 0.08 € | 6 | 0.48 € | List |
| Resistor network, eight times 100 kΩ, nine pins | [Reichelt SIL 9-8 100K](https://www.reichelt.de/de/de/shop/produkt/widerstandsnetzwerk_100_kohm_sternschaltung_8wid_9pins-18018) | 0.19 € | 6 | 1.14 € | List |
| Capacitor 100 nF, 2.5 mm pitch | [Reichelt X7R-2,5 100N KOM](https://www.reichelt.de/de/de/shop/produkt/vielschicht-kerko_100_nf_100_v_x7r_10_rm_2_5_ammo-329838) | 0.05 € | 8 | 0.40 € | List |
| Resettable fuse, holds 0.10 A, trips at 0.20 A | Reichelt PFRA 010 | 0.37 € | 2 | 0.74 € | Page |
| Schottky diode 1N5817 | [Reichelt 1N 5817](https://www.reichelt.de/de/de/shop/produkt/schottkydiode_20_v_1_a_do-41-41848) | 0.05 € | 4 | 0.20 € | List |
| Resistors, metal film, 1 %, see below | Reichelt METALL | | 220 | 11.60 € | Page and list |

- One buffer, two capacitors and two diodes are spares.
- The Pico 2 W without headers costs 7.50 € and was not deliverable at Reichelt or BerryBase.
- The fuse of the next size, `PFRA 020`, holds 0.20 A and costs 0.35 €.
- The Raspberry Pi Pico 2 that serves as the test device is at hand.

| Resistor | Article | Price | Quantity | Sum | Use |
| --- | --- | --- | --- | --- | --- |
| 330 Ω | [METALL 330](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_330_ohm_0207_0_6_w_1_-11733) | 0.035 € from ten | 60 | 2.10 € | Protective resistor of every channel |
| 680 Ω | [METALL 680](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_680_ohm_0207_0_6_w_1_-11942) | 0.035 € from ten | 20 | 0.70 € | Flame of a hearth |
| 1 kΩ | [METALL 1,00K](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_1_00_kohm_0207_0_6_w_1_-11403) | 0.040 € from ten | 30 | 1.20 € | Torch |
| 1.5 kΩ | [METALL 1,50K](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_1_50_kohm_0207_0_6_w_1_-11429) | 0.035 € from ten | 20 | 0.70 € | Embers |
| 2.2 kΩ | [METALL 2,20K](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_2_20_kohm_0207_0_6_w_1_-11584) | 0.070 € from ten | 10 | 0.70 € | Trimming |
| 4.7 kΩ | [METALL 4,70K](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_4_70_kohm_0207_0_6_w_1_-11784) | 0.10 € | 20 | 2.00 € | Room light |
| 6.8 kΩ | [METALL 6,80K](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_6_80_kohm_0207_0_6_w_1_-11908) | 0.07 € | 40 | 2.80 € | Candle |
| 10 kΩ | [METALL 10,0K](https://www.reichelt.de/de/de/shop/produkt/widerstand_metallschicht_10_0_kohm_0207_0_6_w_1_-11449) | 0.07 € | 20 | 1.40 € | Trimming |

The last three rows are calculated with the price of a single piece; the price from ten was not
read. These resistors have a body of size 0207, which is large for a distribution strip in a small
room. A smaller series was not researched.

## LEDs

Pre-wired LEDs of size 0402 from Modellbau Schönwitz, on enamelled wire of 0.15 mm and about
150 mm, in packs of ten, with free shipping within Germany.

| Part | Article | Price | Quantity | Sum | Data |
| --- | --- | --- | --- | --- | --- |
| Orange, for flames and torches | [Schönwitz 50206](https://modellbau-schoenwitz.de/de/modelleisenbahn/elektronik/leds-mit-kabel/led-smd-0402-kupferlackdraht-draht-kabel-orange) | 12.99 € | 2 | 25.98 € | Typically 1.7 V and at most 2.3 V at 5 mA, 605 nm, typically 30 mcd, 120° |
| Red, for embers | [Schönwitz 50200](https://modellbau-schoenwitz.de/de/modelleisenbahn/elektronik/leds-mit-kabel/led-smd-0402-kupferlackdraht-draht-kabel-rot) | 12.99 € | 1 | 12.99 € | Typically 1.7 V and at most 2.3 V at 5 mA, 625 nm, typically 40 mcd, 120° |
| Warm white, for candles and room light | [Schönwitz 50201](https://modellbau-schoenwitz.de/de/modelleisenbahn/elektronik/leds-mit-kabel/led-smd-0402-kupferlackdraht-draht-kabel-warmwei) | 12.99 € | 3 | 38.97 € | Typically 2.7 V and at most 3.3 V at 5 mA, 2600 K to 3200 K, typically 150 mcd, 120° |
| Optional: yellow, to try as a second flame and as a candle | [Schönwitz 50205](https://modellbau-schoenwitz.de/de/modelleisenbahn/elektronik/leds-mit-kabel/led-smd-0402-kupferlackdraht-draht-kabel-gelb) | 12.99 € | 1 | not in the sum | Typically 1.7 V at 5 mA, 590 nm, typically 20 mcd |

- The plan needs 16 orange, 5 red and 22 warm white LEDs; the packs hold 20, 10 and 30. Stage 1
  needs 5 orange, 3 red and 5 warm white, one pack of each.
- The data are those of the bare LEDs the shop sells in the same size with a datasheet. That the
  pre-wired articles are these LEDs, and that they carry no resistor, is inferred from the shop
  pages. No source states a forward voltage at the planned currents of 0.3 mA to 3 mA.
- A second source that sells singly is LEDs-and-more.de: [orange](https://leds-and-more.de/SMD-LED-0402-Orange-/-Amber-mit-Anschluss-Draht-45-mcd-120)
  and [red](https://leds-and-more.de/SMD-LED-mit-Anschluss-Draht-0402-Rot-40-mcd-120) at 1.29 €. Its
  warm white states typically 3.0 V and about 3200 K.
- Not usable as delivered: the pre-wired LEDs of Fechtner Modellbau and Conrad's Sol Expert 0603,
  which carry a resistor in the lead.
- No pre-wired LED warmer than 2600 K was found.

## Wire

| Part | Retailer and article | Price | Quantity | Sum | Read |
| --- | --- | --- | --- | --- | --- |
| Stranded micro cable 0.014 mm², outer diameter about 0.28 mm, 10 m | [Schönwitz 50946](https://modellbau-schoenwitz.de/de/kabel-und-litzen/mikrokabel-0-014mm/10-meter-spule-mikrokabel-litze-flexibel-fep-0-014mm-schwarz) | 4.99 € | 2 | 9.98 € | Page |
| Enamelled wire 0.15 mm, 10 m, to lengthen the leads of the LEDs | [Schönwitz 50324](https://modellbau-schoenwitz.de/de/kabel-und-litzen/kupferlackdraht/10m-spule-kupferlackdraht-lackdraht-schwarz-0-15mm) | 2.99 € | 2 | 5.98 € | Page |
| Silicone wire of 30 gauge, outer diameter 0.8 mm, eight colours of 5 m, for the trunks | [Amazon B0DH2FBWH7](https://www.amazon.de/dp/B0DH2FBWH7) | 15.49 € | 1 | 15.49 € | Page; a seller's data |
| Heat-shrink tube 1.2 mm to 0.6 mm, 15 m | [Reichelt WKK 211114348](https://www.reichelt.de/de/de/shop/produkt/schrumpfschlauch-box_1_2_0_6_gelb_15m-423864) | 5.40 € | 1 | 5.40 € | List |

A thinner trunk wire is the stranded miniature cable of 0.04 mm² with about 0.5 mm of outer
diameter, [Schönwitz 50337](https://modellbau-schoenwitz.de/de/kabel-und-litzen/miniaturkabel-0-04mm/10-meter-ring-miniaturkabel-litze-hochflexibel-lify-0-04mm-schwarz),
at 2.99 € for 10 m per colour.

## Connectors

| Part | Retailer and article | Price | Quantity | Sum | Read |
| --- | --- | --- | --- | --- | --- |
| Five pairs with two contacts, 1.25 mm, leads of 28 gauge | [Amazon B0BNF8949T](https://www.amazon.de/dp/B0BNF8949T) | 5.99 € | 1 | 5.99 € | Page |
| Five pairs with three contacts | The same listing | 6.69 € | 1 | 6.69 € | Page |
| Five pairs with four contacts | The same listing | 6.99 € | 1 | 6.99 € | Page |
| Five pairs with five contacts | The same listing | 7.59 € | 1 | 7.59 € | Page |
| Magnetic connector, four contacts, without cable | [BerryBase 157032](https://www.berrybase.de/berrybase-pogo-pin-stecker-und-sockel-magnetisch-2a-dc-ohne-kabel/stecker-buchse-ohne-oesen/typ-4-polig) | 2.90 € | 1 | 2.90 € | Page |
| Magnetic connector, two contacts, with 150 mm of cable | [BerryBase 157022](https://www.berrybase.de/berrybase-pogo-pin-stecker-und-sockel-magnetisch-2a-dc-mit-kabel/stecker-buchse-ohne-oesen/typ-4-polig) | 2.90 € | 1 | 2.90 € | Page |
| Magnetic connector, three contacts, with cable | The same page, article 157023 | 3.20 € | 1 | 3.20 € | Page; 30 in stock |

- The 1.25 mm pairs have no datasheet. Their dimensions, ratings and polarisation are unknown until
  the samples are measured. The branded original costs about 1 € per pre-crimped wire at DigiKey.
- BerryBase states neither the dimensions nor the holding force nor the polarity of the magnets of
  its connectors. The version with four contacts and a cable, 157024, was not available.
- Contact plates of the makers of light kits, to try on a loose floor: a pair of Light My Bricks
  costs 10.99 € to 12.99 € in its shop for the European Union and needs an adapter of 4.50 € per
  half to reach other wiring; the older ones with ordinary wires cost 12.95 € for two pairs at a
  Dutch reseller, as last items.

## Power lead and supply

| Part | Retailer and article | Price | Quantity | Sum | Read |
| --- | --- | --- | --- | --- | --- |
| Power supply 5 V, 1.5 A, USB-A socket | [Reichelt HNP 06-USBL6](https://www.reichelt.de/de/de/shop/produkt/usb-ladegeraet_5_v_1500_ma-176978) | 5.45 € | 1 | 5.45 € | Page and datasheet |
| Magnetic USB cable, USB-A, 1.1 m | [Reichelt DELOCK 85724](https://www.reichelt.de/de/de/shop/produkt/usb_daten-_und_ladekabel_magnetisch_schwarz_1_10_m-248526) | 9.11 € | 1 | 9.11 € | Page, twice |
| Magnetic tip with a Micro-USB plug | [Reichelt DELOCK 65932](https://www.reichelt.de/de/de/shop/produkt/magnetischer_adapter_usb_micro-b_stecker-398103) | 5.02 € | 1 | 5.02 € | Page, twice |
| Panel-mount cable, Micro-USB socket to plug, 25 cm | [Reichelt DELOCK 85245](https://www.reichelt.de/de/de/shop/produkt/usb_2_0_kabel_micro_b_buchse_auf_micro_b_stecker_0_25_m-212024) | 6.20 € | 1 | 6.20 € | Page, twice |
| Optional: USB cable with a switch, power only | [BerryBase 125534](https://www.berrybase.de/en/usb-2.0-dc-kabel-mit-schalter-a-stecker-a-buchse-0-25m-schwarz) | 2.30 € | 1 | not in the sum | Page |

- Second sources: the panel-mount cable at BerryBase as `D85245` for 6.90 €, the magnetic cable at
  Alternate for 9.29 €.
- The fallback without a magnet is an ordinary cable from USB-A to Micro-USB in the same socket,
  for example Reichelt `ST USBAUB1MBK` for 4.80 €.
- A timer or a smart plug in front of the supply switches the installation as well: Reichelt
  `GOOBAY 59038`, whose page shows 4.64 € and 5.75 €, with 1 W of standby, or `SHELLY PLUGSG3B` for
  23.68 €.
- The same panel-mount cable with 0.5 m, `DELOCK 35108` for 4.95 €, reaches the first controller
  once the socket sits at the edge of the display base.

## Small parts

| Part | Retailer and article | Price | Read |
| --- | --- | --- | --- |
| Removable adhesive putty, white, 80 pads | [Amazon B000GPI7S6](https://www.amazon.de/dp/B000GPI7S6) | 2.45 € | Page |
| Aluminium tape, 10 m × 50 mm | Conrad 541274 | 9.48 €, calculated from the net price | List |
| Cable ties 100 mm, 100 pieces | Conrad 3048263 | 0.49 €, calculated from the net price | List |

No black putty was found. Currents are measured with a multimeter in the supply line; the
resolution of the cheap USB meters at a few tens of milliamperes is not stated. A pin vice and
drills of 0.6 mm to 1.5 mm were not researched to an article.

## Replacement bricks

Bricks are worked on only after their replacements have arrived, see
[hardware.md](hardware.md#working-on-the-bricks). BlueBrixx sells no single bricks; the source is
its Part Packs, bags of 25 to 200 pieces of one shape and colour.

| Purpose | Bags | Sum |
| --- | --- | --- |
| Wall bricks to drill or notch: 1×2 in five colours of the walls, 1×4 in four | 9 at 4.95 € | 44.55 € |
| Floor layers: plates 1×2 in four colours, plates 2×2 in two, tiles 1×2, 2×2 and 1×4 | 9 at 4.95 € to 5.95 € | about 48.55 € |
| Fires: the 1×1 slope in transparent red and in two transparent oranges, the flame piece, round 1×1 plates in transparent red and orange | 6 at 4.95 € to 5.95 € | 33.70 € |
| **Together** | **24** | **about 127 €** |

- A bag is ordered only for a shape that is really worked on; the model shows which. The
  replacement of a brick that cannot be seen need not match in colour, so a bag that fails the
  comparison with the set still serves there.
- Nothing is printed as long as these bricks can be bought. A second copy of a set supplies what
  no bag does, as long as the set is on sale.
- The bricks that hold a board and the socket of the lead, such as plates and Technic bricks, are
  chosen on the model and are not listed.
- Shipping costs 5.95 € and is free from 99 €.
- Not priced: a slope in the orange of the set, if neither orange of the packs matches; the parts
  of the candles and of the hoods, whose shapes are identified on the model.
- No bag is known to match the sets in colour and clutch. One grey brick, one tile and the red
  slope are ordered first and compared with the set, about 16.50 €.
- The only guaranteed match is a second copy of a set; a second Bergfried costs 79.95 €.

## Orders by stage

### First order: stage 1

The base castle with the Bergfried: the first controller and ten channels. Every part is also a
sample for the later stages.

| Parts | Sum | What the bench settles before anything is built in |
| --- | --- | --- |
| One controller with its board, three buffers, and all resistors | 31.58 € | An update over the radio; the driver stage; whether the board fits the chamber under the terrace |
| One pack each of the orange, red and warm white LEDs | 38.97 € | The forward voltages at 0.3 mA, 1 mA and 3 mA, the spread within a pack, that no resistor sits in the lead, brightness and colour behind the transparent parts of the set |
| The whole group "Power lead and supply" | 25.78 € | Whether the plug has an LED, whether data passes the chain, whether the magnet lets go before the tip leaves the socket |
| Connectors: the pairs with five and with two contacts, the three magnetic connectors | 22.58 € | The real size of the connectors, their fit in the model, thirty matings, the polarity of the magnets |
| The whole group "Wire" | 36.85 € | Which wire still lets a brick close, and which hole it needs |
| The whole group "Small parts" | 12.42 € | |
| **First order** | **168.18 €** | |
| Three bags of replacement bricks to compare | about 16.50 € | Whether colour and clutch match the set |

### Later stages

| Stage | Set | LEDs it adds | To order |
| --- | --- | --- | --- |
| 2 | Vorburg | 3 orange, 5 warm white | A second pack of warm white LEDs, 12.99 € |
| 3 | Saalbau | 2 orange, 2 red, 5 warm white | The second controller with its board, two buffers, a fuse and a diode, 18.32 €; a second pack of orange LEDs, 12.99 € |
| 4 | Hurde | 2 warm white | Nothing |
| 5 | Münzturm | 1 orange, 1 warm white | The pack of pairs with three contacts, 6.69 € |
| 6 | Torhaus | 3 orange, 3 warm white | The third controller with its board, 18.37 €; a third pack of warm white LEDs, 12.99 € |
| 7 | Schildwerk | 2 orange, 1 warm white | The pack of pairs with four contacts, 6.99 € |
| | **Later stages together** | | **89.34 €** |

- A pack of LEDs is ordered one stage before the last LED of the previous pack is used, so a
  damaged LED does not stop a stage. At the end four orange, five red and eight warm white LEDs
  are left.
- The replacement bricks for a stage are ordered with it, once the first three bags have shown
  whether the Part Packs match.
- The connectors at every unit that the instructions of the expansions lift off are counted when
  the removal steps of each instruction have been read.
