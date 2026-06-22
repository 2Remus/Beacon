#!/usr/bin/env node
import fs from "fs";
import path from "path";

if (process.argv.length !== 4) {
    console.error("Usage: node generateHtmlHeader.js <input.html> <output.h>");
    process.exit(1);
}

const inputPath = path.resolve(process.argv[2]);
const outputPath = path.resolve(process.argv[3]);

if (!fs.existsSync(inputPath)) {
    console.error(`Input file does not exist: ${inputPath}`);
    process.exit(1);
}

// Read the Vite-emitted dist/index.html precisely as it is
const fileBuffer = fs.readFileSync(inputPath);

const hexBytes = [];
for (const byte of fileBuffer) {
    hexBytes.push(`0x${byte.toString(16).padStart(2, '0')}`);
}

const lines = [];
for (let i = 0; i < hexBytes.length; i += 12) {
    lines.push(hexBytes.slice(i, i + 12).join(", "));
}

const headerContent = `#pragma once
// Auto-generated binary asset file from ${path.basename(inputPath)}
// Do not edit manually.

inline const unsigned char INDEX_HTML_BYTES[] = {
    ${lines.join(",\n    ")}
};

inline const unsigned int INDEX_HTML_SIZE = ${fileBuffer.length};
`;

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, headerContent, "utf-8");

console.log(`Generated binary safe header: ${outputPath} (${fileBuffer.length} bytes)`);