# Chess BR

# World

## Flypath

- [x] Generate
- [x] Step
- [ ] Autodrop

## Zones

### Shape
- [x] Shape could be rect in rects
- [ ] Shape could be random growths of determined sizes

### Fog and Zones
Doesn't depend on shapes but given game length of N ticks.

- [x] Game should expose next zone.
- [x] Should be able to contract fog, this is based on 8 neighbour distance.

## Actions

- [ ] Record history as text

### Drop

- [ ] Release from path position at altitude

### Fly

- [ ] Translate and decrease altitude

### Move

- [x] Basic valid piece moves
- [ ] En passant (requires piece knowing last move)
- [ ] Limit pawn to one direction (requires piece knowing last move)
- [ ] Allow pawn first move two steps (requires piece knowing last move)
- [ ] Allow promotion (requires pawn know traversed distance)
- [ ] Gain piece by proximity
- [ ] Castling pre 1972 rules (req know traversed distance)
