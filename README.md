# Chess BR

# World

## Flypath

- [x] Generate
- [x] Step
- [x] Autodrop

## Zones

### Shape
- [x] Shape could be rect in rects
- [ ] Shape could be random growths of determined sizes

### Fog and Zones
Doesn't depend on shapes but given game length of N ticks.

- [x] Game should expose next zone.
- [x] Should be able to contract fog, this is based on 8 neighbour distance.

## Actions

- [x] Record history as text

### Drop

- [x] Release from path position at altitude

### Fly

- [x] Translate and decrease altitude
- [ ] Resolve legal / safe landing
- [ ] Push apart while in air?

## Move
- [x] Take
- [x] Update piece status on take?
- [x] Basic valid piece moves
- ~~En passant (requires piece knowing last move, orthogonal en passant?)~~
- [x] Limit pawn to one direction (requires piece knowing last move)
- [x] Allow pawn first move two steps (requires piece knowing last move)
- [x] Fix first pawn move can take too
- [ ] Allow promotion (requires pawn know traversed distance)
- [ ] Gain piece by proximity
- [x] Castling pre 1972 rules (req know traversed distance)
- [ ] Find all opposing pieces that threaten ( watch out for king special rules to no stack overflow)

## Scoring

- [x] Add take value to score
