import { createCube } from '@cubearc/cube';

const cube = await createCube();
const result = cube.execute({
  operation: 'image.qr.encode',
  version: 1,
  inputs: { data: Uint8Array.of(0, 255) },
  parameters: { ecc: 'L', version: 1, mask: 0, raster: { scale: 4 } },
});
if (result.modules.length !== 441 || result.raster?.width !== 116) {
  throw new Error('Unexpected QR geometry');
}
