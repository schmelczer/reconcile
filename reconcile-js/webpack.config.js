const path = require("path");
const webpack = require("webpack");
const packageJson = require("./package.json");


module.exports = {
	entry: "./src/index.ts",
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: ["ts-loader"]
			},
			{
				test: /\.wasm$/,
				type: "asset/inline"
			}
		]
	},
	resolve: {
		extensions: [".ts"],
		alias: {
			root: __dirname,
			src: path.resolve(__dirname, "src")
		}
	},
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
        libraryTarget: "commonjs2"
    },
};
 