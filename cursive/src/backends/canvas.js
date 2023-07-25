const fontWidth = 12;
const fontHeight = fontWidth * 2;
const textColorPairSize = 12;

export function paint(buffer) {
    console.time('paint');
    const data = new Uint8Array(buffer);
    const canvas = document.getElementById('cursive-wasm-canvas');
    const context = canvas.getContext('2d');
    context.font = `${fontHeight}px monospace`;
    for (let x = 0; x < 1000; x++) {
        for (let y = 0; y < 1000; y++) {
            const n = 1000 * y + x;
            const textColorPair = data.slice(n * textColorPairSize, (n + 1) * textColorPairSize);
            const text = String.fromCharCode(textColorPair[0] + (2**8) *textColorPair[1] + (2**16)* textColorPair[2] + (2 ** 24) + textColorPair[3]);
            const front = byte_to_hex_string(textColorPair.slice(4, 7));
            const back = byte_to_hex_string(textColorPair.slice(7, 10));
            context.fillStyle = back;
            context.fillRect(x * fontWidth, y * fontHeight, fontWidth, fontHeight);
            if (text != ' ') {
                context.fillStyle = front;
                context.fillText(text, x * fontWidth, (y + 0.8) * fontHeight);
            }
        }
    }
    console.timeEnd('paint');
}

function byte_to_hex_string(bytes) {
    const red = bytes[0].toString(16).padStart(2, '0');
    const green = bytes[1].toString(16).padStart(2, '0');
    const blue = bytes[2].toString(16).padStart(2, '0');
    return  `#${red}${green}${blue}`;
}