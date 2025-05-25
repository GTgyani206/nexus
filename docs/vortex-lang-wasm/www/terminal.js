import { Terminal } from "https://cdn.jsdelivr.net/npm/xterm@5.3.0/+esm";
import init, { run_vortex } from "../pkg/vortex_lang_wasm.js";

await init();

const term = new Terminal({
  theme: { background: "#181818", foreground: "#eee" },
  cursorBlink: true,
  fontFamily: "monospace",
  fontSize: 16,
});
term.open(document.getElementById("terminal"));

const PROMPT = "vortex> ";
let input = "";

function prompt() {
  term.write(`\r\n${PROMPT}`);
  input = "";
}

prompt();

term.onData(e => {
  switch (e) {
    case "\r": // Enter
      term.write("\r\n");
      if (input.trim() !== "") {
        try {
          const result = run_vortex(input);
          term.write(result.replace(/\n/g, "\r\n"));
        } catch (err) {
          term.write(`❌ JS Error: ${err}\r\n`);
        }
      }
      prompt();
      break;
    case "\u007F": // Backspace
      if (input.length > 0) {
        term.write("\b \b");
        input = input.slice(0, -1);
      }
      break;
    default:
      if (e >= String.fromCharCode(0x20)) {
        term.write(e);
        input += e;
      }
  }
});