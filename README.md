# Chess BR

# World

## Zones

### Shape
* Shape could be rect in rects
* Shape could be random growths of determined sizes

### Fog progress
Doesn't depend on shapes but given game length of N ticks.

1. Game should expose next zone.
2. Wait a number of ticks depending of which zone
3. Then it should transition to next zone (different speeds per zone and different per direction depending on direction)
   This is based on 8 neighbour distance to zone

