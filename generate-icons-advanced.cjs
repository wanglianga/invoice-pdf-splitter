const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

const iconsDir = path.join(__dirname, "src-tauri", "icons");

function createPng(width, height, r, g, b, alpha = 255) {
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  function crc32(data) {
    let crc = 0xffffffff;
    const table = [];
    for (let i = 0; i < 256; i++) {
      let c = i;
      for (let j = 0; j < 8; j++) {
        c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
      }
      table[i] = c;
    }
    for (let i = 0; i < data.length; i++) {
      crc = table[(crc ^ data[i]) & 0xff] ^ (crc >>> 8);
    }
    return (crc ^ 0xffffffff) >>> 0;
  }

  function createChunk(type, data) {
    const typeBuffer = Buffer.from(type, "ascii");
    const length = Buffer.alloc(4);
    length.writeUInt32BE(data.length, 0);
    const crcData = Buffer.concat([typeBuffer, data]);
    const crcValue = crc32(crcData);
    const crcBuffer = Buffer.alloc(4);
    crcBuffer.writeUInt32BE(crcValue, 0);
    return Buffer.concat([length, typeBuffer, data, crcBuffer]);
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const rawData = [];
  for (let y = 0; y < height; y++) {
    rawData.push(0);
    for (let x = 0; x < width; x++) {
      const cx = x - width / 2;
      const cy = y - height / 2;
      const dist = Math.sqrt(cx * cx + cy * cy);
      const radius = Math.min(width, height) / 2 - 2;
      
      if (dist < radius) {
        rawData.push(r, g, b, alpha);
      } else {
        rawData.push(0, 0, 0, 0);
      }
    }
  }

  const compressed = zlib.deflateSync(Buffer.from(rawData));
  const ihdrChunk = createChunk("IHDR", ihdr);
  const idatChunk = createChunk("IDAT", compressed);
  const iendChunk = createChunk("IEND", Buffer.alloc(0));

  return Buffer.concat([signature, ihdrChunk, idatChunk, iendChunk]);
}

function createIco(sizes) {
  const iconCount = sizes.length;
  const headerSize = 6;
  const entrySize = 16;
  let dataOffset = headerSize + entrySize * iconCount;
  
  const iconDatas = sizes.map(size => {
    const png = createPng(size, size, 102, 126, 234);
    return { size, data: png };
  });
  
  const header = Buffer.alloc(headerSize);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(iconCount, 4);
  
  const entries = [];
  let offset = dataOffset;
  
  for (const icon of iconDatas) {
    const entry = Buffer.alloc(entrySize);
    entry.writeUInt8(icon.size >= 256 ? 0 : icon.size, 0);
    entry.writeUInt8(icon.size >= 256 ? 0 : icon.size, 1);
    entry.writeUInt8(0, 2);
    entry.writeUInt8(0, 3);
    entry.writeUInt16LE(1, 4);
    entry.writeUInt16LE(32, 6);
    entry.writeUInt32LE(icon.data.length, 8);
    entry.writeUInt32LE(offset, 12);
    entries.push(entry);
    offset += icon.data.length;
  }
  
  const allData = iconDatas.map(i => i.data);
  return Buffer.concat([header, ...entries, ...allData]);
}

const icoSizes = [16, 32, 48, 64, 128, 256];
const icoBuffer = createIco(icoSizes);
fs.writeFileSync(path.join(iconsDir, "icon.ico"), icoBuffer);
console.log("Created icon.ico");

function createIcns(sizes) {
  const iconTypes = {
    16: "icp4",
    32: "icp5",
    64: "icp6",
    128: "ic07",
    256: "ic08",
    512: "ic09",
  };
  
  const icons = [];
  let totalSize = 8;
  
  for (const size of sizes) {
    const type = iconTypes[size];
    if (!type) continue;
    const png = createPng(size, size, 102, 126, 234);
    const entrySize = 8 + png.length;
    const entry = Buffer.alloc(entrySize);
    entry.write(type, 0, "ascii");
    entry.writeUInt32BE(entrySize, 4);
    png.copy(entry, 8);
    icons.push(entry);
    totalSize += entrySize;
  }
  
  const header = Buffer.alloc(8);
  header.write("icns", 0, "ascii");
  header.writeUInt32BE(totalSize, 4);
  
  return Buffer.concat([header, ...icons]);
}

const icnsSizes = [16, 32, 64, 128, 256, 512];
const icnsBuffer = createIcns(icnsSizes);
fs.writeFileSync(path.join(iconsDir, "icon.icns"), icnsBuffer);
console.log("Created icon.icns");

console.log("All icons generated successfully!");
