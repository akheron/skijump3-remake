// Adapted from https://github.com/YuzuJS/setImmediate/blob/aa63d9174ea8551892ec6db3a5f01fe7cd5e8c3e/setImmediate.js
//
// Copyright (c) 2012 Barnesandnoble.com, llc, Donavon West, and Domenic Denicola
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

const messagePrefix = "setImmediate$" + Math.random() + "$";

const onwindowMessage = function(event) {
    if (event.source === window && typeof event.data === "string" && event.data.indexOf(messagePrefix) === 0) {
        runIfPresent(+event.data.slice(messagePrefix.length));
    }
};
window.addEventListener("message", onwindowMessage, false);

let nextHandle = 1; // Spec says greater than zero
const tasksByHandle = {};
let currentlyRunningATask = false;

export function setImmediate(callback, ...args) {
    tasksByHandle[nextHandle] = { callback, args };
    window.postMessage(messagePrefix + nextHandle, "*");
    return nextHandle++;
}

export function clearImmediate(handle) {
    delete tasksByHandle[handle];
}

function runIfPresent(handle) {
    // From the spec: "Wait until any invocations of this algorithm started before this one have completed."
    // So if we're currently running a task, we'll need to delay this invocation.
    if (currentlyRunningATask) {
        // Delay by doing a setTimeout. setImmediate was tried instead, but in Firefox 7 it generated a
        // "too much recursion" error.
        setTimeout(runIfPresent, 0, handle);
    } else {
        const task = tasksByHandle[handle];
        if (task) {
            currentlyRunningATask = true;
            try {
                const callback = task.callback;
                callback(...task.args);
            } finally {
                clearImmediate(handle);
                currentlyRunningATask = false;
            }
        }
    }
}
