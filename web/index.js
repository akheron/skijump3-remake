import init, { SJ3, AsyncState } from "./skijump3.js"

// The original game runs in VGA mode 13h and waits for the vertical retrace
// before every frame. Mode 13h refreshes at 25.175 MHz / (800 * 449) ≈ 70.086 Hz,
// and the game logic advances exactly one step per retrace.
const TICK_MS = (800 * 449) / 25175.0

// If the browser stalls (background tab, debugger, ...), don't try to catch up
// more than this. The original game simply slowed down when it couldn't keep up.
const MAX_CATCH_UP_MS = 100

// Safety valve for loops that poll the keyboard without rendering. Key events
// can't arrive while JavaScript is running, so such a loop would never end.
// When the limit is hit, we yield to the browser and continue on the next
// animation frame. A normal tick polls only a few times.
const MAX_POLLS_PER_TICK = 100

export async function run() {
  const wasm = await init()
  const packData = await loadFile("assets.pack")
  const state = SJ3.new(packData)

  const canvas = document.getElementById("screen")
  canvas.width = 320
  canvas.height = 200
  const context = canvas.getContext("2d")

  let waitingForKeyPress = false
  let finished = false
  let lastTime = null
  let accumulator = 0

  // Runs the game until it has produced one frame. Returns false if the game
  // is waiting for a key press (or stuck polling) and can't advance right now.
  const runOneTick = () => {
    for (let polls = 0; polls < MAX_POLLS_PER_TICK; polls++) {
      const op = state.tick()
      if (op === AsyncState.Render) {
        state.resume()
        return true
      } else if (op === AsyncState.KeyPressed) {
        // Key events are queued between animation frames, so the queue is
        // already up to date. No need to yield to the browser here.
        state.resume()
      } else if (op === AsyncState.WaitForKeyPress) {
        // Resumed when state.keydown() is called in response to a key press
        waitingForKeyPress = true
        return false
      } else {
        // The game has exited. Polling it again would panic.
        finished = true
        return false
      }
    }
    console.warn("Game polled the keyboard too many times without rendering, yielding")
    return false
  }

  const onAnimationFrame = (now) => {
    if (finished) return
    requestAnimationFrame(onAnimationFrame)

    if (lastTime === null || waitingForKeyPress) {
      lastTime = now
      return
    }
    accumulator += Math.min(now - lastTime, MAX_CATCH_UP_MS)
    lastTime = now

    let rendered = false
    while (accumulator >= TICK_MS) {
      if (!runOneTick()) {
        accumulator = 0
        break
      }
      accumulator -= TICK_MS
      rendered = true
    }
    if (rendered) {
      // The game rendered its latest frame into the screen buffer. Showing it
      // here, inside the rAF callback, ties it to the display's refresh.
      renderToCanvas(wasm.memory.buffer, state.screen(), context)
    }
  }

  // Run the first tick right away so that the first frame is visible as soon
  // as possible
  if (runOneTick()) renderToCanvas(wasm.memory.buffer, state.screen(), context)
  requestAnimationFrame(onAnimationFrame)

  document.addEventListener("keydown", (event) => {
    if (finished) return
    state.keydown(event.key)
    if (waitingForKeyPress) {
      waitingForKeyPress = false
      state.resume()
      // Run until the next frame immediately to respond as fast as possible.
      // Wait states don't accumulate time, so start counting from now.
      if (runOneTick()) renderToCanvas(wasm.memory.buffer, state.screen(), context)
      lastTime = performance.now()
      accumulator = 0
    }
  })
}

function renderToCanvas(buffer, screen, context) {
  const width = 320
  const height = 200
  const data = new Uint8ClampedArray(buffer, screen, width * height * 4)
  context.putImageData(new ImageData(data, width, height), 0, 0)
}

async function loadFile(url) {
  const response = await fetch(url)
  if (!response.ok) throw new Error(`Unable to load file ${url}`)
  const arrayBuffer = await response.arrayBuffer()
  return new Uint8Array(arrayBuffer)
}
