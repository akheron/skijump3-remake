import init, { SJ3, AsyncState, Files } from "./skijump3.js"
import { setImmediate } from "./setImmediate.js";

export async function run() {
  const wasm = await init()

  const files = await Promise.all([
    "LANGBASE.SKI",
    "ANIM.SKI",
    "HISCORE.SKI",
    "CONFIG.SKI",
    "PLAYERS.SKI",
    "NAMES0.SKI",
    "MOREHILL.SKI",
    "HILLBASE.SKI",
    "MAIN.PCX",
    "LOAD.PCX",
    "FRONT1.PCX",
    "BACK1.PCX",
    "GOALS.SKI",
  ].map(loadFile))

  const state = SJ3.new(Files.new(...files))

  const canvas = document.getElementById("screen")
  canvas.width = 320
  canvas.height = 200
  const context = canvas.getContext("2d")

  const targetFps = 70
  const frameTime = 1000 / targetFps

  let lastFrame = performance.now()
  let waitingForKeyPress = false

  const mainLoop = () => {
    const op = state.tick()
    if (op === AsyncState.Render) {
      const render = () => {
        const now = performance.now()
        if (now - lastFrame < frameTime) {
          setImmediate(render)
          return
        }
        lastFrame = now
        renderToCanvas(wasm.memory.buffer, state.screen(), context)
        state.resume()
        mainLoop()
      }
      setImmediate(render)
    } else if (op === AsyncState.KeyPressed) {
      // Just to give the browser a chance to feed key presses
      setImmediate(() => {
        state.resume()
        mainLoop()
      })
    } else if (op === AsyncState.WaitForKeyPress) {
      // Resumed when state.keydown() is called in response to a key press
      waitingForKeyPress = true
    }
  }
  mainLoop()

  document.addEventListener("keydown", (event) => {
    console.log('feeding key press', event.key, '(waiting? ', waitingForKeyPress, ')')
    state.keydown(event.key)
    if (waitingForKeyPress) {
      waitingForKeyPress = false
      state.resume()
      mainLoop()
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
