import init, { SJ3, AsyncState } from "./skijump3.js"

// The game speed is tied to the frame rate. The original game draws a frame on
// each vertical retrace of VGA mode 13h, which has a refresh rate of
// 25.175 MHz / (800 * 449), about 70.086 Hz.
const FRAME_MS = (1000 * 800 * 449) / 25_175_000

// After a pause, such as a hidden tab, the game plays at most this much of the
// missed time and skips the rest.
const MAX_CATCH_UP_MS = 100

// The browser sends key events only between calls to the game. When a game loop
// polls the keyboard but does not draw, such as the jump of a computer jumper
// that is not shown, playFrame returns after polling for this long.
const MAX_POLLING_MS = 10

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
  let previousTime
  let unplayedTime

  document.addEventListener("keydown", (event) => {
    state.keydown(event.key)
    if (waitingForKeyPress) {
      waitingForKeyPress = false
      state.resume()
      startLoop()
    }
  })

  startLoop()

  function startLoop() {
    previousTime = null
    unplayedTime = FRAME_MS
    requestAnimationFrame(onAnimationFrame)
  }

  function onAnimationFrame(now) {
    if (previousTime !== null) {
      unplayedTime += Math.min(now - previousTime, MAX_CATCH_UP_MS)
    }
    previousTime = now

    let frameDrawn = false
    while (unplayedTime >= FRAME_MS) {
      if (!playFrame()) {
        unplayedTime = 0
        break
      }
      unplayedTime -= FRAME_MS
      frameDrawn = true
    }
    if (frameDrawn) {
      renderToCanvas(wasm.memory.buffer, state.screen(), context)
    }

    if (!waitingForKeyPress && !finished) {
      requestAnimationFrame(onAnimationFrame)
    }
  }

  // Returns whether the game drew a frame
  function playFrame() {
    const deadline = performance.now() + MAX_POLLING_MS
    while (performance.now() < deadline) {
      switch (state.tick()) {
        case AsyncState.Render:
          state.resume()
          return true
        case AsyncState.KeyPressed:
          state.resume()
          break
        case AsyncState.WaitForKeyPress:
          waitingForKeyPress = true
          return false
        case undefined:
          // The game has ended, and one more tick() would panic
          finished = true
          return false
      }
    }
    return false
  }
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
