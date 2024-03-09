import { greet, decode_image } from './pkg';

function onImageLoad (data) {
  const text = decode_image(new Uint8Array(data));
  console.log({ text });
  return text;
} 

function toBlob(canvas, type = "image/png", quality = 1) {
  return new Promise((resolve) => canvas.toBlob(blob => resolve(blob)))
}

window.__iload = (text) => {
  return new Promise((resolve, reject) => {
    const img = new Image();

    img.src = 'http://localhost:3001/auth/image?text=' + encodeURIComponent(text);

    img.crossOrigin = "anonymous";

    img.addEventListener('load', () => {
      const canvas = document.createElement('canvas');

      canvas.style['display'] = 'none';

      document.body.appendChild(canvas);

      const context = canvas.getContext('2d');
      canvas.width = img.width;
      canvas.height = img.height;

      context.drawImage(img, 0, 0);

      toBlob(canvas, "image/webp", 1)
        .then(blob => blob.arrayBuffer())
        .then(buffer => {
          const u8 = new Uint8Array(buffer);
          console.log(u8);
          resolve(u8);
          onImageLoad(buffer);
        })
        .catch(reject);

      // const u8 = new Uint8Array(context.getImageData(0, 0, img.width, img.height).data.buffer);
      // console.log(u8);
    });

    img.addEventListener('error', (e) => {
      reject(e);
    });
  });
}

