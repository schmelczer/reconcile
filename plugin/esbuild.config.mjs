import esbuild from "esbuild";
import process from "process";
import builtins from "builtin-modules";
import path from "node:path";
import fs from "node:fs";
import { wasmPack } from "esbuild-plugin-wasm-pack";

const prod = process.argv[2] === "production";

async function copyFiles(sourceDir, destinationDir) {
	try {
		await fs.promises.mkdir(destinationDir, { recursive: true });

		const paths = Array.isArray(sourceDir)
			? sourceDir
			: (await fs.promises.readdir(sourceDir)).map((file) =>
					path.join(sourceDir, file)
			  );

		await Promise.all(
			paths.map(async (sourcePath) => {
				const stat = await fs.promises.stat(sourcePath);

				if (stat.isFile()) {
					const destinationFile = path.join(
						destinationDir,
						path.basename(sourcePath)
					);
					await fs.promises.copyFile(sourcePath, destinationFile);
					console.debug(`Copied ${sourcePath} to ${destinationFile}`);
				} else {
					console.info(`Skipping directory ${sourcePath}`);
				}
			})
		);

		console.info("All files copied successfully.");
	} catch (err) {
		console.error("Error copying files:", err);
	}
}

let wasmPlugin = {
	name: "wasm",
	setup(build) {
		// Resolve ".wasm" files to a path with a namespace
		build.onResolve({ filter: /\.wasm$/ }, (args) => {
			if (args.resolveDir === "") {
				return; // Ignore unresolvable paths
			}
			return {
				path: path.isAbsolute(args.path)
					? args.path
					: path.join(args.resolveDir, args.path),
				namespace: "wasm-binary",
			};
		});

		// Virtual modules in the "wasm-binary" namespace contain the
		// actual bytes of the WebAssembly file. This uses esbuild's
		// built-in "binary" loader instead of manually embedding the
		// binary data inside JavaScript code ourselves.
		build.onLoad(
			{ filter: /.*/, namespace: "wasm-binary" },
			async (args) => ({
				contents: await fs.promises.readFile(args.path),
				loader: "binary",
			})
		);
	},
};

const copyBundle = {
	name: "post-compile",
	setup(build) {
		build.onEnd(async (result) => {
			if (prod) {
				await fs.promises.copyFile(
					"manifest.json",
					"build/manifest.json"
				);
				return;
			}

			if (result.errors.length === 0) {
				await copyFiles(
					["manifest.json", ".hotreload"],
					"/mnt/c/Users/Andras/Desktop/test/test/.obsidian/plugins/my-plugin"
				);

				await copyFiles(
					"build",
					"/mnt/c/Users/Andras/Desktop/test/test/.obsidian/plugins/my-plugin"
				);

				await copyFiles(
					["manifest.json", ".hotreload"],
					"/mnt/c/Users/Andras/Desktop/test/test2/.obsidian/plugins/my-plugin"
				);

				await copyFiles(
					"build",
					"/mnt/c/Users/Andras/Desktop/test/test2/.obsidian/plugins/my-plugin"
				);
			}
		});
	},
};

const cssContext = await esbuild.context({
	entryPoints: ["src/styles.css"],
	bundle: true,
	outfile: "build/styles.css",
	plugins: [copyBundle],
});

const jsContext = await esbuild.context({
	entryPoints: ["src/plugin.ts"],
	bundle: true,
	external: [
		"obsidian",
		"electron",
		"@codemirror/autocomplete",
		"@codemirror/collab",
		"@codemirror/commands",
		"@codemirror/language",
		"@codemirror/lint",
		"@codemirror/search",
		"@codemirror/state",
		"@codemirror/view",
		"@lezer/common",
		"@lezer/highlight",
		"@lezer/lr",
		...builtins,
	],
	format: "cjs",
	target: "es2020",
	logLevel: "info",
	resolveExtensions: [".ts"],
	sourcemap: prod ? false : "inline",
	treeShaking: false,
	outfile: "build/main.js",
	minify: prod,
	plugins: [
		wasmPlugin,
		prod
			? null
			: wasmPack({
					target: "web",
					path: "../backend/sync_lib",
			  }),
		copyBundle,
	].filter(Boolean),
});

if (prod) {
	await Promise.all([cssContext.rebuild(), jsContext.rebuild()]);
	process.exit(0);
} else {
	await Promise.all([cssContext.watch(), jsContext.watch()]);
}
