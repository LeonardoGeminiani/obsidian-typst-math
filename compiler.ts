import { Plugin } from "obsidian";
import * as rustTypst from "./pkg/obsidian_rust_plugin.js";
import * as wasmbin from './pkg/obsidian_rust_plugin_bg.wasm';

export class typstCompiler {
    init(plugin: Plugin) {

		
		// @ts-expect-error
		await rustTypst.default(Promise.resolve(wasmbin.default));
		// @ts-expect-error
		rustTypst.onload(plugin);
    }
}