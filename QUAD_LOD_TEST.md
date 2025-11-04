# Quad LOD Test Instructions

## What Was Built

A flat 2D quad (2000m × 2000m) at y=0 with LOD splitting/collapsing based on camera distance.

## Current Setup

- **Quad position**: Origin (0, 0, 0)
- **Quad size**: -1000 to +1000 on X and Z axes
- **Camera position**: (0, 500, 0) - 500 meters above, looking down
- **Camera target**: (0, 0, 0) - center of quad

## Expected Behavior

1. **At startup**: You should see a single white quad (root chunk) below the camera
2. **Move camera closer**: Chunks subdivide into 4 children when distance < threshold
3. **Move camera away**: Children merge back into parent when distance > threshold
4. **LOD levels**: 10 levels total (0-9), smallest chunks are ~2m at max depth

## Split Distances (per depth level)

- Depth 0: 1000m
- Depth 1: 500m
- Depth 2: 250m
- Depth 3: 125m
- Depth 4: 62.5m
- Depth 5: 31.25m
- Depth 6: 15.6m
- Depth 7: 7.8m
- Depth 8: 3.9m
- Depth 9: 2.0m

## Testing Steps

1. **Run**: `cargo run`
2. **Initial view**: You should see a white flat quad from above
3. **Use editor**: Move camera up/down (Y axis) to test LOD
4. **Watch logs**: Check terminal for "Initializing quad LOD", "Generating meshes", "Splitting chunk", "Collapsing chunk"
5. **Expected at camera height 500m**: Root chunk only (distance ~500m > split threshold of 1000m)
6. **Lower to ~300m**: Should split into 4 chunks
7. **Lower to ~150m**: Should split again (more detail)
8. **Raise back up**: Should collapse (less detail)

## Debug Logging

The following logs should appear:
- "Initializing quad LOD test for entity"
- "Generating meshes for N quad chunks"
- "Splitting chunk at depth X"
- "Collapsing chunk"

## Troubleshooting

**Can't see anything?**
- Check camera is at (0, 500, 0) facing (0, 0, 0)
- Check terminal for initialization logs
- Chunks are white - make sure background is dark

**No LOD changes?**
- Move camera closer (Y < 300m) to trigger first split
- Check logs for "Splitting chunk" messages

**Chunks disappearing?**
- This is normal - parent meshes are removed when split
- Children should appear in their place

## Next Phase

Once this works:
1. Apply same logic to PlanetChunk
2. Scale planet to 63,710m radius (1:100 Earth)
3. Add more LOD levels for walking detail
