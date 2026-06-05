import * as Automerge from "@automerge/automerge";

let doc = Automerge.init();
doc = Automerge.change(doc, (d) => {
  d.blobs = {};
  d.blobs["test-sha"] = new Uint8Array([1, 2, 3, 4]);
});

const raw = doc.blobs["test-sha"];
console.log("Is Uint8Array?", raw instanceof Uint8Array);
console.log("Is ArrayBufferView?", ArrayBuffer.isView(raw));
console.log("Constructor name:", raw?.constructor?.name);
console.log("Length:", raw?.length);
console.log("Keys:", Object.keys(raw || {}));
console.log("Typeof:", typeof raw);
console.log("Is array?", Array.isArray(raw));

try {
  let bytes;
  if (raw instanceof Uint8Array) {
    bytes = raw;
  } else if (ArrayBuffer.isView(raw)) {
    bytes = new Uint8Array((raw).buffer, (raw).byteOffset, (raw).byteLength);
  } else {
    const len = raw.length ?? Object.keys(raw).length;
    bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) bytes[i] = raw[i] ?? 0;
  }
  console.log("Converted bytes:", bytes);
  const blob = new Blob([bytes], { type: "image/png" });
  console.log("Blob size:", blob.size);
} catch (e) {
  console.error("Failed", e);
}
