import { Plugin } from "obsidian";
import * as rustTypst from "./pkg/obsidian_rust_plugin.js";
import * as wasmbin from './pkg/obsidian_rust_plugin_bg.wasm';
import { Compiler } from "./pkg/obsidian_rust_plugin.js";
import { sign } from "crypto";

export class typstCompiler {
    compiler: Compiler;
    math_preamble: string = "#set page(margin: 0pt)\n#set align(horizon)";
    general_preamble: string = "#set text(fill: white, size: SIZE)\n#set page(width: WIDTH, height: HEIGHT)";

    async init(root: string) {
        
        // @ts-expect-error
        await rustTypst.default(Promise.resolve(wasmbin.default));

        this.compiler = new Compiler(root, this.requestData);

        customElements.define("typst-renderer", TypstRenderElement)
        TypstRenderElement.compile = (path: string, source: string, size: number, display: boolean, fontSize: number) => {
            console.log("raara");
            const dpr = window.devicePixelRatio;
            // * (72 / 96)
            const pxToPt = (px: number) => px.toString() + "pt"
            const sizing = `#let (WIDTH, HEIGHT, SIZE, THEME) = (${display ? pxToPt(size) : "auto"}, ${!display ? pxToPt(size) : "auto"}, ${pxToPt(fontSize)}, "${document.body.getCssPropertyValue("color-scheme")}")`
            return this.compileToTypst(
                path,
                `${sizing}\n${this.general_preamble}\n${source}`,
                size,
                display
            )
        }
    }

    compileToTypst(path: string, source: string, size: number, display: boolean): string {
        console.log(source);
        return this.compiler.compile_svg(source, path);
    }

    decoder = new TextDecoder()

    requestData(path: string): string {
        try {
            // @ts-expect-error
            let buffer = new Int32Array(new SharedArrayBuffer(4, { maxByteLength: 1e8 }))
            buffer[0] = 0;
            postMessage({ buffer, path })
            const res = Atomics.wait(buffer, 0, 0);
            if (buffer[0] == 0) {
                return this.decoder.decode(Uint8Array.from(buffer.slice(1)))
            }
            throw buffer[0]
        } catch (e) {
            if (typeof e != "number") {
                console.error(e)
                throw 1
            }
            throw e
        }
    }

    createTypstRenderElement(path: string, source: string, display: boolean, math: boolean) {

        let renderer = new TypstRenderElement();
        renderer.format = "svg"
        renderer.source = source
        renderer.path = path
        renderer.display = display
        renderer.math = math

        console.log(renderer)
        return renderer
    }

    createTypstMath(source: string, r: { display: boolean }) {
        const display = r.display;
        source = `${this.math_preamble}\n${display ? `$ ${source} $` : `$${source}$`}`

        // let svg = this.compiler.compile_svg(source, "/586f8912-f3a8-4455-8a4a-3729469c2cc1.typ");
        // debugger;
        // let el = document.createElement("span");
        // let b = this.createTypstRenderElement("/586f8912-f3a8-4455-8a4a-3729469c2cc1.typ", source, display, true);
        // el.innerHTML = svg;
        // console.log("svddsf");
        // return el;
        return this.createTypstRenderElement("/586f8912-f3a8-4455-8a4a-3729469c2cc1.typ", source, display, true)
    }

}

class TypstRenderElement extends HTMLElement {
    static compile: (path: string, source: string, size: number, display: boolean, fontSize: number) => string
    static nextId = 0;
    static prevHeight = 0;

    // The Element's Id
    id: string
    // The number in the element's id.
    num: string

    abortController: AbortController
    format: string
    source: string
    path: string
    display: boolean
    resizeObserver: ResizeObserver
    size: number
    math: boolean

    canvas: HTMLCanvasElement

    async connectedCallback() {
        if (!this.isConnected) {
            console.warn("Typst Renderer: Canvas element has been called before connection");
            return;
        }

        if (this.format == "image" && this.canvas == undefined) {
            this.canvas = this.appendChild(createEl("canvas", { attr: { height: TypstRenderElement.prevHeight }, cls: "typst-doc" }))
        }

        this.num = TypstRenderElement.nextId.toString()
        TypstRenderElement.nextId += 1
        this.id = "TypstRenderElement-" + this.num
        this.abortController = new AbortController()

        if (this.display) {
            this.style.display = "block"
            this.resizeObserver = new ResizeObserver((entries) => {
                if (entries[0]?.contentBoxSize[0].inlineSize !== this.size) {
                    this.draw()
                }
            })
            this.resizeObserver.observe(this)
        }
        await this.draw()
    }

    disconnectedCallback() {
        if (this.format == "image") {
            TypstRenderElement.prevHeight = this.canvas.height
        }
        if (this.display && this.resizeObserver != undefined) {
            this.resizeObserver.disconnect()
        }
    }

    async draw() {
        this.abortController.abort()
        this.abortController = new AbortController()
        try {
            await navigator.locks.request(this.id, { signal: this.abortController.signal }, async () => {
                let fontSize = parseFloat(getComputedStyle(this).fontSize)
                this.size = this.display ? this.clientWidth : parseFloat(getComputedStyle(this).lineHeight)

                // resizeObserver can trigger before the element gets disconnected which can cause the size to be 0
                // which causes a NaN. size can also sometimes be -ve so wait for resize to draw it again
                if (!(this.size > 0)) {
                    return;
                }

                try {
                    let result = await TypstRenderElement.compile(this.path, this.source, this.size, this.display, fontSize)

                    this.innerHTML = result;
                    let svg = (this.firstElementChild as SVGElement);

                    let childToBeRemoved = svg.querySelector("path");
                    
                    if(childToBeRemoved !== null)
                        svg.removeChild(childToBeRemoved);

                    svg.setAttribute("width", svg.getAttribute("width")!.replace("pt", ""))
                    svg.setAttribute("height", svg.getAttribute("height")!.replace("pt", ""))
                    svg.setAttribute("width", `${this.firstElementChild!.clientWidth / fontSize}em`);
                    svg.setAttribute("height", `${this.firstElementChild!.clientHeight / fontSize}em`);

                } catch (error) {
                    // For some reason it is uncaught so remove "Uncaught "
                    error = error.slice(9)
                    let pre = createEl("pre", {
                        attr: {
                            style: "white-space: pre;"
                        }
                    })//"<pre> </pre>"
                    pre.textContent = error
                    this.outerHTML = pre.outerHTML
                    return
                }


            })

        } catch (error) {
            return
        }
    }
}