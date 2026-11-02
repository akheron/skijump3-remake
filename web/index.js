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
  loadKeyboardLayout(state)

  const canvas = document.getElementById("screen")
  canvas.width = 320
  canvas.height = 200
  const context = canvas.getContext("2d")

  let waitingForKeyPress = false
  let previousTime
  let unplayedTime

  const textInput = document.getElementById("text-input")

  document.addEventListener("keydown", (event) => {
    // A soft keyboard on Android gives "Unidentified" for most keys, so the
    // input event of the text input gives the character instead
    if (event.target === textInput && (event.key === "Unidentified" || event.isComposing)) {
      return
    }
    const used = pressKey(event.key, event.code, {
      shift: event.shiftKey,
      ctrl: event.ctrlKey,
      alt: event.altKey,
      altGraph: event.getModifierState("AltGraph"),
      meta: event.metaKey,
      capsLock: event.getModifierState("CapsLock"),
    })
    if (used || event.target === textInput) {
      event.preventDefault()
    }
  })

  document.addEventListener("keyup", (event) => {
    state.keyup(event.code)
  })

  const showButtons = setUpTouchControls(pressKey, pressTouchKey, textInput)

  // Returns whether the game uses the key
  function pressKey(key, code, modifiers = {}) {
    const used = state.keydown(
      key,
      code,
      !!modifiers.shift,
      !!modifiers.ctrl,
      !!modifiers.alt,
      !!modifiers.altGraph,
      !!modifiers.meta,
      !!modifiers.capsLock,
    )
    resumeAfterKeyPress()
    return used
  }

  function pressTouchKey(ch, ch2) {
    state.press(ch, ch2)
    resumeAfterKeyPress()
  }

  function resumeAfterKeyPress() {
    if (waitingForKeyPress) {
      waitingForKeyPress = false
      state.resume()
      startLoop()
    }
  }

  // The game saves its files here, as when the window of the desktop game is
  // closed, and starts again in case the page is shown again.
  window.addEventListener("pagehide", () => {
    state.close()
    if (waitingForKeyPress) {
      waitingForKeyPress = false
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
    const buttons = state.take_changed_buttons()
    if (buttons !== undefined) {
      showButtons(JSON.parse(buttons))
    }

    if (!waitingForKeyPress) {
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
      }
    }
    return false
  }
}

// The delays of a typical keyboard auto-repeat
const REPEAT_DELAY_MS = 400
const REPEAT_INTERVAL_MS = 33

// The text input always holds this, so that a soft keyboard has a character
// to delete when Backspace is pressed
const TEXT_INPUT_FILLER = "  "

function setUpTouchControls(pressKey, pressTouchKey, textInput) {
  const pad = document.getElementById("pad")
  const canvas = document.getElementById("screen")

  // iOS can leave a button partly drawn when the button next to it is removed,
  // so every slot keeps its button and only hides it
  const buttons = new Map()
  for (const column of [pad, document.getElementById("actions")]) {
    for (const slot of column.dataset.slots.split(" ")) {
      const button = slotButton(slot)
      column.append(button.element)
      buttons.set(slot, button)
    }
  }
  let screenButton = null
  let canvasTouched = false

  followCameraSide()
  screen.orientation?.addEventListener("change", followCameraSide)

  canvas.addEventListener("pointerdown", (event) => {
    canvasTouched = event.pointerType === "touch"
    if (canvasTouched) requestFullscreen()
  })
  canvas.addEventListener("click", () => {
    if (!canvasTouched || !screenButton) return
    if (screenButton.keyboard) {
      openKeyboard()
    } else {
      pressTouchKey(...screenButton.key)
    }
  })

  textInput.addEventListener("input", (event) => {
    if (event.inputType.startsWith("delete")) {
      pressKey("Backspace", "Backspace")
    } else {
      for (const c of event.data ?? "") {
        pressKey(c, "", { shift: c !== c.toLowerCase() })
      }
    }
    textInput.value = TEXT_INPUT_FILLER
  })

  return function showButtons(specs) {
    const bySlot = new Map(specs.map((spec) => [spec.slot, spec]))
    for (const [slot, button] of buttons) {
      button.show(bySlot.get(slot) ?? null)
    }
    screenButton = bySlot.get("screen") ?? null
    pad.dataset.arrows = arrowLayout(bySlot)
    if (!specs.some((spec) => spec.keyboard)) {
      textInput.blur()
    }
  }

  function slotButton(slot) {
    const element = document.createElement("button")
    element.style.gridArea = slot
    element.style.visibility = "hidden"
    element.classList.toggle("main", slot === "main")
    let spec = null
    let repeatTimer = null

    const release = () => {
      clearTimeout(repeatTimer)
      repeatTimer = null
      element.classList.remove("pressed")
    }
    const repeat = () => {
      pressTouchKey(...spec.key)
      repeatTimer = setTimeout(repeat, REPEAT_INTERVAL_MS)
    }

    element.addEventListener("pointerdown", (event) => {
      event.preventDefault()
      if (!spec?.key) return
      element.setPointerCapture(event.pointerId)
      element.classList.add("pressed")
      requestFullscreen()
      pressTouchKey(...spec.key)
      if (spec.repeat) {
        clearTimeout(repeatTimer)
        repeatTimer = setTimeout(repeat, REPEAT_DELAY_MS)
      }
    })
    element.addEventListener("pointerup", release)
    element.addEventListener("pointercancel", release)
    element.addEventListener("lostpointercapture", release)
    element.addEventListener("contextmenu", (event) => event.preventDefault())
    // iOS opens the soft keyboard only on focus() from a click
    element.addEventListener("click", () => {
      if (!spec?.keyboard) return
      if (document.activeElement === textInput) {
        textInput.blur()
      } else {
        openKeyboard()
      }
    })

    const show = (newSpec) => {
      if (JSON.stringify(newSpec) === JSON.stringify(spec)) return
      release()
      spec = newSpec
      if (spec) element.textContent = spec.label
      element.style.visibility = spec ? "visible" : "hidden"
    }
    return { element, show }
  }

  function openKeyboard() {
    textInput.value = TEXT_INPUT_FILLER
    textInput.focus()
  }
}

// The names must stay in sync with the data-arrows rules in index.html
function arrowLayout(specs) {
  const upDown = specs.has("up") || specs.has("down")
  const leftRight = specs.has("left") || specs.has("right")
  return upDown && !leftRight ? "up-down" : "all"
}

// The camera is at the top edge of a phone in portrait orientation
function followCameraSide() {
  const side = {
    "landscape-primary": "left",
    "landscape-secondary": "right",
  }[screen.orientation?.type]
  if (side) {
    document.documentElement.dataset.cameraSide = side
  } else {
    delete document.documentElement.dataset.cameraSide
  }
}

// Only Android browsers can go fullscreen and lock the orientation. On an
// iPhone the page fills the screen when it is added to the home screen.
function requestFullscreen() {
  if (!document.fullscreenEnabled || document.fullscreenElement) return
  document.documentElement
    .requestFullscreen({ navigationUI: "hide" })
    .then(() => screen.orientation?.lock?.("landscape"))
    .catch(() => {})
}

// Only Chromium has getLayoutMap
async function loadKeyboardLayout(state) {
  if (!navigator.keyboard?.getLayoutMap) return
  try {
    const layoutMap = await navigator.keyboard.getLayoutMap()
    for (const [code, key] of layoutMap) {
      state.set_unshifted_key(code, key)
    }
  } catch (err) {
    console.warn("Unable to read the keyboard layout", err)
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
